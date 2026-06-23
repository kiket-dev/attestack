use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use attestack_core::{
    new_bundle_id, sha256_hex, sign_bundle_manifest, verify_bundle_manifest, verify_event_chain,
    AttestackError, BundleFileEntry, BundleManifest, EventKind, EventPayload, VerificationReport,
    VerificationTargetKind, BUNDLE_SCHEMA_V1,
};
use ed25519_dalek::VerifyingKey;
use thiserror::Error;
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::identity::{Identity, DEFAULT_IDENTITY_FILE};
use crate::Store;

pub const MAX_BUNDLE_FILE_BYTES: u64 = 50 * 1024 * 1024;
pub const MAX_BUNDLE_FILES: usize = 1000;
pub const MAX_BUNDLE_EVENTS: usize = 10_000;

#[cfg(test)]
pub(crate) const TEST_MAX_BUNDLE_FILE_BYTES: u64 = 4096;

pub(crate) fn max_bundle_file_bytes() -> u64 {
    #[cfg(test)]
    {
        TEST_MAX_BUNDLE_FILE_BYTES
    }
    #[cfg(not(test))]
    {
        MAX_BUNDLE_FILE_BYTES
    }
}

#[derive(Debug, Clone)]
pub struct BundleCreateOptions {
    pub output: Option<PathBuf>,
    pub include_diff: bool,
    pub include_command_output: bool,
    pub redact_paths: bool,
}

#[derive(Debug, Error)]
pub enum BundleError {
    #[error(transparent)]
    Core(#[from] AttestackError),

    #[error(transparent)]
    Store(#[from] crate::StoreError),

    #[error(transparent)]
    Identity(#[from] crate::identity::IdentityError),

    #[error("bundle error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("session not found: {0}")]
    SessionNotFound(String),

    #[error("session is still open: {0}")]
    SessionOpen(String),

    #[error("no report found for session {0}")]
    ReportMissing(String),
}

pub type Result<T> = std::result::Result<T, BundleError>;

pub struct BundleCreateResult {
    pub bundle_path: PathBuf,
    pub bundle_id: String,
    pub manifest: BundleManifest,
}

impl Store {
    pub fn create_bundle(
        &self,
        repo_root: &Path,
        session_id: Option<String>,
        options: BundleCreateOptions,
    ) -> Result<BundleCreateResult> {
        let session = match session_id {
            Some(id) => self.read_session(self.session_dir(&id))?,
            None => self.find_latest_closed_session()?.ok_or_else(|| {
                BundleError::SessionNotFound("no closed session found; pass a session id".into())
            })?,
        };

        if session.status == attestack_core::SessionStatus::Open {
            return Err(BundleError::SessionOpen(session.session_id.clone()));
        }

        let events = self.read_events(&session.session_id)?;
        verify_event_chain(&events).map_err(BundleError::Core)?;

        let identity = Identity::load(self.root())?;
        let bundle_id = new_bundle_id();
        let repo_root = repo_root.canonicalize().unwrap_or_else(|_| repo_root.to_path_buf());

        let mut files_to_add: Vec<(String, Vec<u8>, String, Option<String>)> = Vec::new();

        let session_json = redact_path_bytes(
            fs::read(self.session_dir(&session.session_id).join("session.json"))?,
            &repo_root,
            options.redact_paths,
        );
        files_to_add.push((
            format!("sessions/{}/session.json", session.session_id),
            session_json,
            "application/json".into(),
            Some("session.metadata".into()),
        ));

        // Preserve events.jsonl verbatim so exported bundles remain verifiable.
        let events_jsonl = fs::read(self.session_dir(&session.session_id).join("events.jsonl"))?;
        files_to_add.push((
            format!("sessions/{}/events.jsonl", session.session_id),
            events_jsonl,
            "application/jsonl".into(),
            Some("session.events".into()),
        ));

        if let Some(report_path) = find_report_path(self, &session.session_id)? {
            let report_bytes =
                redact_path_bytes(fs::read(report_path)?, &repo_root, options.redact_paths);
            files_to_add.push((
                format!("sessions/{}/report.md", session.session_id),
                report_bytes,
                "text/markdown".into(),
                Some("session.report".into()),
            ));
        }

        for artifact_id in
            collect_artifact_ids(&events, options.include_command_output, options.include_diff)
        {
            let artifact_path =
                self.session_dir(&session.session_id).join("artifacts").join(&artifact_id);
            if !artifact_path.is_file() {
                continue;
            }
            let bytes =
                redact_path_bytes(fs::read(artifact_path)?, &repo_root, options.redact_paths);
            files_to_add.push((
                format!("artifacts/{artifact_id}"),
                bytes,
                "application/octet-stream".into(),
                Some("session.artifact".into()),
            ));
        }

        let mut manifest =
            BundleManifest::new(bundle_id.clone(), vec![session.session_id.clone()], Vec::new());
        for (path, bytes, media_type, purpose) in &files_to_add {
            manifest.files.push(BundleFileEntry {
                path: path.clone(),
                sha256: sha256_hex(bytes),
                size_bytes: bytes.len() as u64,
                media_type: media_type.clone(),
                purpose: purpose.clone(),
            });
        }

        sign_bundle_manifest(&mut manifest, identity.signing_key(), &identity.identity_id)
            .map_err(BundleError::Core)?;
        let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;

        let output_path = options.output.unwrap_or_else(|| {
            let slug = slugify(&session.title);
            self.root().join(crate::BUNDLES_DIR).join(format!("{slug}.attestack.zip"))
        });
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = File::create(&output_path)?;
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

        zip.start_file("bundle.json", options)?;
        zip.write_all(&manifest_bytes)?;

        for (path, bytes, _, _) in files_to_add {
            zip.start_file(path, options)?;
            zip.write_all(&bytes)?;
        }
        zip.finish()?;

        self.append_typed_event(
            &session.session_id,
            EventKind::BundleCreated,
            EventPayload::BundleCreated(attestack_core::BundleCreatedPayload {
                bundle_id: bundle_id.clone(),
                bundle_path: output_path.display().to_string(),
            }),
        )?;

        Ok(BundleCreateResult { bundle_path: output_path, bundle_id, manifest })
    }
}

fn find_report_path(store: &Store, session_id: &str) -> Result<Option<PathBuf>> {
    let report = store.session_dir(session_id).join("reports").join("report.md");
    if report.is_file() {
        Ok(Some(report))
    } else {
        Ok(None)
    }
}

fn collect_artifact_ids(
    events: &[attestack_core::Event],
    include_command_output: bool,
    include_diff: bool,
) -> Vec<String> {
    let mut artifact_ids = HashSet::new();
    for event in events {
        match &event.payload {
            EventPayload::CommandFinished(payload) => {
                if include_command_output {
                    if let Some(id) = &payload.stdout_artifact {
                        artifact_ids.insert(id.clone());
                    }
                    if let Some(id) = &payload.stderr_artifact {
                        artifact_ids.insert(id.clone());
                    }
                }
            }
            EventPayload::GitSnapshot(payload) if include_diff => {
                if let Some(id) = &payload.diff_artifact {
                    artifact_ids.insert(id.clone());
                }
            }
            _ => {}
        }
    }
    artifact_ids.into_iter().collect()
}

pub(crate) fn redact_path_bytes(bytes: Vec<u8>, repo_root: &Path, redact: bool) -> Vec<u8> {
    if !redact {
        return bytes;
    }
    let mut text = String::from_utf8_lossy(&bytes).into_owned();
    let root = repo_root.display().to_string();
    text = text.replace(&root, ".");
    text.into_bytes()
}

fn slugify(title: &str) -> String {
    let mut slug = String::new();
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if (ch.is_whitespace() || ch == '-' || ch == '_')
            && !slug.ends_with('-')
            && !slug.is_empty()
        {
            slug.push('-');
        }
    }
    slug.trim_matches('-').chars().take(64).collect()
}

pub fn verify_local_session(session_dir: &Path) -> Result<VerificationReport> {
    let target = session_dir.display().to_string();
    if !session_dir.join("session.json").is_file() {
        return Ok(VerificationReport::failure(
            target,
            VerificationTargetKind::Session,
            vec!["missing session.json".into()],
        ));
    }

    let events_path = session_dir.join("events.jsonl");
    if !events_path.is_file() {
        return Ok(VerificationReport::failure(
            target,
            VerificationTargetKind::Session,
            vec!["missing events.jsonl".into()],
        ));
    }

    let raw = fs::read_to_string(&events_path)?;
    let events: Vec<attestack_core::Event> = raw
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(serde_json::from_str)
        .collect::<std::result::Result<Vec<_>, _>>()?;

    if events.len() > MAX_BUNDLE_EVENTS {
        return Ok(VerificationReport::failure(
            target,
            VerificationTargetKind::Session,
            vec!["event count exceeds limit".into()],
        ));
    }

    match verify_event_chain(&events) {
        Ok(()) => {
            let mut report = VerificationReport::success(target, VerificationTargetKind::Session);
            report.event_count = Some(events.len() as u64);
            Ok(report)
        }
        Err(err) => Ok(VerificationReport::failure(
            target,
            VerificationTargetKind::Session,
            vec![err.to_string()],
        )),
    }
}

pub fn verify_bundle_file(
    bundle_path: &Path,
    public_key: Option<VerifyingKey>,
) -> Result<VerificationReport> {
    let target = bundle_path.display().to_string();
    if !bundle_path.is_file() {
        return Ok(VerificationReport::failure(
            target,
            VerificationTargetKind::Bundle,
            vec!["bundle file not found".into()],
        ));
    }

    let file = File::open(bundle_path)?;
    let mut archive = ZipArchive::new(file).map_err(BundleError::Zip)?;
    if archive.len() > MAX_BUNDLE_FILES {
        return Ok(VerificationReport::failure(
            target,
            VerificationTargetKind::Bundle,
            vec!["bundle contains too many files".into()],
        ));
    }

    let mut manifest: Option<BundleManifest> = None;
    let mut extracted: HashMap<String, Vec<u8>> = HashMap::new();

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(BundleError::Zip)?;
        let name = entry.name().to_string();
        if !is_safe_zip_path(&name) {
            return Ok(VerificationReport::failure(
                target.clone(),
                VerificationTargetKind::Bundle,
                vec![format!("unsafe zip path: {name}")],
            ));
        }
        if entry.size() > max_bundle_file_bytes() {
            return Ok(VerificationReport::failure(
                target.clone(),
                VerificationTargetKind::Bundle,
                vec![format!("file too large: {name}")],
            ));
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        if name == "bundle.json" {
            manifest = match serde_json::from_slice(&bytes) {
                Ok(parsed) => Some(parsed),
                Err(err) => {
                    return Ok(VerificationReport::failure(
                        target,
                        VerificationTargetKind::Bundle,
                        vec![format!("invalid bundle.json: {err}")],
                    ));
                }
            };
        }
        extracted.insert(name, bytes);
    }

    let Some(manifest) = manifest else {
        return Ok(VerificationReport::failure(
            target,
            VerificationTargetKind::Bundle,
            vec!["missing bundle.json".into()],
        ));
    };

    if manifest.schema_version != BUNDLE_SCHEMA_V1 {
        return Ok(VerificationReport::failure(
            target,
            VerificationTargetKind::Bundle,
            vec![format!("unsupported schema version: {}", manifest.schema_version)],
        ));
    }

    let mut errors = Vec::new();
    for file in &manifest.files {
        let Some(bytes) = extracted.get(&file.path) else {
            errors.push(format!("missing file listed in manifest: {}", file.path));
            continue;
        };
        if sha256_hex(bytes) != file.sha256 {
            errors.push(format!("digest mismatch: {}", file.path));
        }
        if bytes.len() as u64 != file.size_bytes {
            errors.push(format!("size mismatch: {}", file.path));
        }
    }

    if let Some(events_path) = manifest
        .files
        .iter()
        .find(|file| file.path.ends_with("events.jsonl"))
        .map(|file| file.path.clone())
    {
        if let Some(bytes) = extracted.get(&events_path) {
            let raw = String::from_utf8_lossy(bytes);
            match raw
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(serde_json::from_str::<attestack_core::Event>)
                .collect::<std::result::Result<Vec<_>, _>>()
            {
                Ok(events) => {
                    if let Err(err) = verify_event_chain(&events) {
                        errors.push(err.to_string());
                    }
                }
                Err(err) => errors.push(format!("invalid events.jsonl: {err}")),
            }
        }
    }

    let mut signature_verified = None;
    if let Some(public_key) = public_key {
        match verify_bundle_manifest(&manifest, &public_key) {
            Ok(()) => signature_verified = Some(true),
            Err(AttestackError::UnsupportedSignatureAlgorithm(alg)) => {
                errors.push(format!("unsupported signature algorithm: {alg}"));
                signature_verified = Some(false);
            }
            Err(_) => {
                errors.push("bundle manifest signature invalid".into());
                signature_verified = Some(false);
            }
        }
    } else if manifest.signature.is_some() {
        errors.push("bundle is signed but no public key was provided".into());
    }

    if errors.is_empty() && manifest.signature.is_none() {
        let mut report = VerificationReport::success(target, VerificationTargetKind::Bundle);
        report.file_count = Some(manifest.files.len() as u64);
        report.signature_verified = signature_verified;
        report.warnings.push("bundle manifest is unsigned".into());
        return Ok(report);
    }

    if !errors.is_empty() {
        let mut report =
            VerificationReport::failure(target, VerificationTargetKind::Bundle, errors);
        report.file_count = Some(manifest.files.len() as u64);
        report.signature_verified = signature_verified;
        return Ok(report);
    }

    let mut report = VerificationReport::success(target, VerificationTargetKind::Bundle);
    report.file_count = Some(manifest.files.len() as u64);
    report.signature_verified = signature_verified;
    Ok(report)
}

pub(crate) fn is_safe_zip_path(path: &str) -> bool {
    if path.contains('\\') || path.starts_with('/') {
        return false;
    }
    if path.as_bytes().get(1) == Some(&b':') {
        return false;
    }
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        return false;
    }
    !candidate.components().any(|component| matches!(component, Component::ParentDir))
}

pub fn load_public_key_from_file(path: &Path) -> Result<VerifyingKey> {
    let raw = fs::read_to_string(path)?;
    let public: attestack_core::PublicIdentity = serde_json::from_str(&raw)?;
    let bytes =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &public.public_key)
            .map_err(|_| BundleError::Core(AttestackError::SignatureInvalid))?;
    attestack_core::verifying_key_from_bytes(&bytes).map_err(BundleError::Core)
}

pub fn load_public_key_from_store(store_root: &Path) -> Result<Option<VerifyingKey>> {
    let public_path = store_root.join(crate::IDENTITIES_DIR).join(DEFAULT_IDENTITY_FILE);
    if !public_path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(public_path)?;
    let public: attestack_core::PublicIdentity = serde_json::from_str(&raw)?;
    let bytes =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &public.public_key)
            .map_err(|_| BundleError::Core(AttestackError::SignatureInvalid))?;
    Ok(Some(attestack_core::verifying_key_from_bytes(&bytes).map_err(BundleError::Core)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{new_session, Store};
    use attestack_core::{EventKind, SessionStatus, SessionStoppedPayload};
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    #[test]
    fn create_and_verify_bundle() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path()).unwrap();
        let identity_id = store.config().unwrap().default_identity_id;

        let session = new_session("demo bundle".into(), identity_id);
        store.create_session(session.clone()).unwrap();
        store
            .append_typed_event(
                &session.session_id,
                EventKind::SessionStopped,
                EventPayload::SessionStopped(SessionStoppedPayload { report_path: None }),
            )
            .unwrap();
        let mut closed = session.clone();
        closed.status = SessionStatus::Closed;
        store.write_session(&closed).unwrap();

        let report_dir = store.session_dir(&session.session_id).join("reports");
        fs::create_dir_all(&report_dir).unwrap();
        fs::write(report_dir.join("report.md"), "# demo").unwrap();

        let result = store
            .create_bundle(
                dir.path(),
                Some(session.session_id.clone()),
                BundleCreateOptions {
                    output: None,
                    include_diff: false,
                    include_command_output: false,
                    redact_paths: false,
                },
            )
            .unwrap();

        let public_key = load_public_key_from_store(store.root()).unwrap().unwrap();
        let report = verify_bundle_file(&result.bundle_path, Some(public_key)).unwrap();
        assert!(report.verified, "{:?}", report.errors);
    }

    #[test]
    fn redact_path_bytes_replaces_repo_root() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let input = format!("log from {}", root.display());
        let redacted = redact_path_bytes(input.into_bytes(), &root, true);
        let text = String::from_utf8(redacted).unwrap();
        assert!(!text.contains(&root.display().to_string()));
        assert!(text.contains("log from ."));
    }

    #[test]
    fn redact_path_bytes_noop_when_disabled() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let input = format!("path {}", root.display());
        let bytes = input.clone().into_bytes();
        let output = redact_path_bytes(bytes.clone(), &root, false);
        assert_eq!(output, bytes);
    }

    #[test]
    fn is_safe_zip_path_rejects_traversal() {
        assert!(!is_safe_zip_path("../etc/passwd"));
        assert!(!is_safe_zip_path("/absolute/path"));
        assert!(!is_safe_zip_path("foo/../../bar"));
        assert!(is_safe_zip_path("sessions/ses_123/session.json"));
    }

    #[test]
    fn verify_rejects_zip_slip_path() {
        let dir = tempdir().unwrap();
        let bundle_path = dir.path().join("evil.attestack.zip");
        let file = File::create(&bundle_path).unwrap();
        let mut writer = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        writer.start_file("../evil.txt", options).unwrap();
        writer.write_all(b"bad").unwrap();
        writer.finish().unwrap();

        let report = verify_bundle_file(&bundle_path, None).unwrap();
        assert!(!report.verified);
        assert!(report.errors.iter().any(|e| e.contains("unsafe zip path")));
    }

    #[test]
    fn verify_rejects_unknown_signature_algorithm() {
        use attestack_core::{sign_bundle_manifest, AttestackError, BundleManifest};

        let signing_key = attestack_core::generate_signing_key();
        let verifying_key = signing_key.verifying_key();
        let mut manifest = BundleManifest::new("bnd_test".into(), vec!["ses_test".into()], vec![]);
        sign_bundle_manifest(&mut manifest, &signing_key, "id_test").unwrap();
        manifest.signature.as_mut().unwrap().alg = "RSA-SHA256".into();

        let err = verify_bundle_manifest(&manifest, &verifying_key).unwrap_err();
        assert!(matches!(err, AttestackError::UnsupportedSignatureAlgorithm(_)));

        let dir = tempdir().unwrap();
        let bundle_path = dir.path().join("bad-sig.attestack.zip");
        let file = File::create(&bundle_path).unwrap();
        let mut writer = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        writer.start_file("bundle.json", options).unwrap();
        writer.write_all(&serde_json::to_vec(&manifest).unwrap()).unwrap();
        writer.finish().unwrap();

        let report = verify_bundle_file(&bundle_path, Some(verifying_key)).unwrap();
        assert!(!report.verified);
        assert!(report.errors.iter().any(|e| e.contains("unsupported signature algorithm")));
    }

    #[test]
    fn verify_rejects_oversized_file() {
        let dir = tempdir().unwrap();
        let bundle_path = dir.path().join("big.attestack.zip");
        let file = File::create(&bundle_path).unwrap();
        let mut writer = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        writer.start_file("big.bin", options).unwrap();
        writer.write_all(&vec![b'x'; (TEST_MAX_BUNDLE_FILE_BYTES + 1) as usize]).unwrap();
        writer.finish().unwrap();

        let report = verify_bundle_file(&bundle_path, None).unwrap();
        assert!(!report.verified);
        assert!(report.errors.iter().any(|e| e.contains("file too large")));
    }

    #[test]
    fn verify_rejects_unsupported_schema() {
        use attestack_core::BundleManifest;

        let dir = tempdir().unwrap();
        let bundle_path = dir.path().join("bad-schema.attestack.zip");
        let file = File::create(&bundle_path).unwrap();
        let mut writer = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        writer.start_file("bundle.json", options).unwrap();
        let manifest = BundleManifest {
            schema_version: "attestack.bundle.v999".into(),
            bundle_id: "bnd_test".into(),
            created_at: time::OffsetDateTime::now_utc(),
            sessions: vec![],
            files: vec![],
            signature: None,
        };
        writer.write_all(&serde_json::to_vec(&manifest).unwrap()).unwrap();
        writer.finish().unwrap();

        let report = verify_bundle_file(&bundle_path, None).unwrap();
        assert!(!report.verified, "{:?}", report.errors);
        assert!(report.errors.iter().any(|e| e.contains("unsupported schema version")));
    }

    #[test]
    fn create_bundle_records_created_event() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path()).unwrap();
        let identity_id = store.config().unwrap().default_identity_id;

        let session = new_session("event test".into(), identity_id);
        store.create_session(session.clone()).unwrap();
        store
            .append_typed_event(
                &session.session_id,
                EventKind::SessionStopped,
                EventPayload::SessionStopped(SessionStoppedPayload { report_path: None }),
            )
            .unwrap();
        let mut closed = session.clone();
        closed.status = SessionStatus::Closed;
        store.write_session(&closed).unwrap();

        let report_dir = store.session_dir(&session.session_id).join("reports");
        fs::create_dir_all(&report_dir).unwrap();
        fs::write(report_dir.join("report.md"), "# demo").unwrap();

        store
            .create_bundle(
                dir.path(),
                Some(session.session_id.clone()),
                BundleCreateOptions {
                    output: None,
                    include_diff: false,
                    include_command_output: false,
                    redact_paths: false,
                },
            )
            .unwrap();

        let events = store.read_events(&session.session_id).unwrap();
        assert!(events.iter().any(|event| matches!(event.payload, EventPayload::BundleCreated(_))));
    }
}
