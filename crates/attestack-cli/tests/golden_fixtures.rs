mod common;

use std::path::PathBuf;

use attestack_core::verify_event_chain;
use attestack_store::{load_public_key_from_file, verify_bundle_file, verify_local_session};
use common::{attestack, init_and_start, latest_bundle_path, stop_session};
use tempfile::tempdir;

fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata").join(name)
}

#[test]
fn minimal_session_fixture_verifies() {
    let fixture_dir = fixture_dir("minimal-session");
    let events_raw = std::fs::read_to_string(fixture_dir.join("events.jsonl")).unwrap();
    let events: Vec<attestack_core::Event> = events_raw
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(events.len(), 3);
    verify_event_chain(&events).unwrap();

    let report = verify_local_session(&fixture_dir).unwrap();
    assert!(report.verified, "{:?}", report.errors);
}

#[test]
fn command_session_fixture_verifies() {
    let fixture_dir = fixture_dir("command-session");
    let events_raw = std::fs::read_to_string(fixture_dir.join("events.jsonl")).unwrap();
    assert!(events_raw.contains("command.started"));
    assert!(events_raw.contains("command.finished"));

    let report = verify_local_session(&fixture_dir).unwrap();
    assert!(report.verified, "{:?}", report.errors);
}

#[test]
fn valid_bundle_fixture_verifies() {
    let fixture_dir = fixture_dir("valid-bundle");
    let bundle = fixture_dir.join("demo.attestack.zip");
    let public_key = load_public_key_from_file(&fixture_dir.join("default.public.json")).unwrap();
    let report = verify_bundle_file(&bundle, Some(public_key)).unwrap();
    assert!(report.verified, "{:?}", report.errors);
    assert_eq!(report.signature_verified, Some(true));
}

#[test]
fn verify_strict_fails_on_unsigned_bundle_warning() {
    use std::io::{Read, Write};
    use zip::read::ZipArchive;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let fixture = fixture_dir("valid-bundle");
    let dir = tempdir().unwrap();
    let unsigned = dir.path().join("unsigned.attestack.zip");
    let source = std::fs::File::open(fixture.join("demo.attestack.zip")).unwrap();
    let mut archive = ZipArchive::new(source).unwrap();
    let dest_file = std::fs::File::create(&unsigned).unwrap();
    let mut writer = ZipWriter::new(dest_file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let name = entry.name().to_string();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if name == "bundle.json" {
            let mut manifest: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            manifest.as_object_mut().unwrap().remove("signature");
            bytes = serde_json::to_vec_pretty(&manifest).unwrap();
        }
        writer.start_file(name, options).unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap();

    attestack()
        .current_dir(dir.path())
        .args(["verify", &unsigned.display().to_string()])
        .assert()
        .success();
    attestack()
        .current_dir(dir.path())
        .args(["verify", &unsigned.display().to_string(), "--strict"])
        .assert()
        .code(1);
}

#[test]
fn agent_and_ci_commands_record_events() {
    let dir = tempdir().unwrap();
    init_and_start(dir.path(), "agent ci test");

    attestack()
        .current_dir(dir.path())
        .args([
            "agent",
            "tool-call",
            "--tool",
            "read_file",
            "--input-hash",
            "sha256:abc",
            "--summary",
            "Read auth module",
        ])
        .assert()
        .success();

    attestack()
        .current_dir(dir.path())
        .args(["agent", "decision", "--summary", "Reuse JWT middleware"])
        .assert()
        .success();

    stop_session(dir.path());

    let events_path = std::fs::read_dir(dir.path().join(".attestack/sessions"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path()
        .join("events.jsonl");
    let events = std::fs::read_to_string(events_path).unwrap();
    assert!(events.contains("ai.tool_call"));
    assert!(events.contains("ai.decision"));
}

#[test]
fn ci_flow_exports_redacted_bundle() {
    let dir = tempdir().unwrap();
    attestack().current_dir(dir.path()).args(["ci", "start", "--json"]).assert().success();
    attestack().current_dir(dir.path()).args(["ci", "run", "--", "echo", "ci"]).assert().success();
    attestack().current_dir(dir.path()).args(["ci", "finish", "--json"]).assert().success();

    let bundle = latest_bundle_path(dir.path());
    assert!(bundle.is_file());

    let report = attestack()
        .current_dir(dir.path())
        .args(["verify", &bundle.display().to_string(), "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(report).unwrap();
    assert!(stdout.contains("\"verified\":true"));
}
