use ed25519_dalek::{Signature as DalekSignature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

use crate::error::{AttestackError, Result};
use crate::hash::sha256_digest;
use crate::types::{BundleManifest, Signature};

pub const SIGNATURE_ALG_ED25519: &str = "Ed25519";

pub fn generate_signing_key() -> SigningKey {
    SigningKey::generate(&mut OsRng)
}

pub fn verifying_key_from_bytes(bytes: &[u8]) -> Result<VerifyingKey> {
    let array: [u8; 32] = bytes.try_into().map_err(|_| AttestackError::SignatureInvalid)?;
    VerifyingKey::from_bytes(&array).map_err(|_| AttestackError::SignatureInvalid)
}

pub fn signing_key_from_bytes(bytes: &[u8]) -> Result<SigningKey> {
    let array: [u8; 32] = bytes.try_into().map_err(|_| AttestackError::SignatureInvalid)?;
    Ok(SigningKey::from_bytes(&array))
}

pub fn canonical_signable_bytes(value: &serde_json::Value) -> Result<Vec<u8>> {
    let mut copy = value.clone();
    if let Some(object) = copy.as_object_mut() {
        object.remove("signature");
    }
    serde_jcs::to_vec(&copy).map_err(|err| AttestackError::Canonicalization(err.to_string()))
}

pub fn hash_signable_json(value: &serde_json::Value) -> Result<String> {
    Ok(sha256_digest(&canonical_signable_bytes(value)?))
}

pub fn sign_json_value(
    value: &serde_json::Value,
    signing_key: &SigningKey,
    key_id: &str,
) -> Result<Signature> {
    let bytes = canonical_signable_bytes(value)?;
    let signature = signing_key.sign(&bytes);
    Ok(Signature {
        alg: SIGNATURE_ALG_ED25519.into(),
        key_id: key_id.into(),
        value: base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            signature.to_bytes(),
        ),
    })
}

pub fn verify_json_signature(
    value: &serde_json::Value,
    signature: &Signature,
    verifying_key: &VerifyingKey,
) -> Result<()> {
    if signature.alg != SIGNATURE_ALG_ED25519 {
        return Err(AttestackError::UnsupportedSignatureAlgorithm(signature.alg.clone()));
    }

    let bytes = canonical_signable_bytes(value)?;
    let sig_bytes =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &signature.value)
            .map_err(|_| AttestackError::SignatureInvalid)?;
    let sig_array: [u8; 64] = sig_bytes.try_into().map_err(|_| AttestackError::SignatureInvalid)?;
    let sig = DalekSignature::from_bytes(&sig_array);
    verifying_key.verify(&bytes, &sig).map_err(|_| AttestackError::SignatureInvalid)
}

pub fn sign_bundle_manifest(
    manifest: &mut BundleManifest,
    signing_key: &SigningKey,
    key_id: &str,
) -> Result<()> {
    let value = serde_json::to_value(&*manifest)?;
    manifest.signature = Some(sign_json_value(&value, signing_key, key_id)?);
    Ok(())
}

pub fn verify_bundle_manifest(
    manifest: &BundleManifest,
    verifying_key: &VerifyingKey,
) -> Result<()> {
    let Some(signature) = &manifest.signature else {
        return Err(AttestackError::SignatureInvalid);
    };
    let value = serde_json::to_value(manifest)?;
    verify_json_signature(&value, signature, verifying_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BundleFileEntry;

    #[test]
    fn sign_and_verify_manifest() {
        let signing_key = generate_signing_key();
        let verifying_key = signing_key.verifying_key();
        let key_id = "id_test".to_string();
        let mut manifest = BundleManifest::new(
            "bun_test".into(),
            vec!["ses_test".into()],
            vec![BundleFileEntry {
                path: "sessions/ses_test/events.jsonl".into(),
                sha256: "abc".into(),
                size_bytes: 1,
                media_type: "application/jsonl".into(),
                purpose: None,
            }],
        );
        sign_bundle_manifest(&mut manifest, &signing_key, &key_id).unwrap();
        verify_bundle_manifest(&manifest, &verifying_key).unwrap();
    }

    #[test]
    fn modified_manifest_fails_verification() {
        let signing_key = generate_signing_key();
        let verifying_key = signing_key.verifying_key();
        let mut manifest = BundleManifest::new("bun_test".into(), vec!["ses_test".into()], vec![]);
        sign_bundle_manifest(&mut manifest, &signing_key, "id_test").unwrap();
        manifest.bundle_id = "bun_tampered".into();
        assert!(verify_bundle_manifest(&manifest, &verifying_key).is_err());
    }

    #[test]
    fn unknown_signature_algorithm_fails() {
        use crate::AttestackError;

        let signing_key = generate_signing_key();
        let verifying_key = signing_key.verifying_key();
        let mut manifest = BundleManifest::new("bun_test".into(), vec!["ses_test".into()], vec![]);
        sign_bundle_manifest(&mut manifest, &signing_key, "id_test").unwrap();
        manifest.signature.as_mut().unwrap().alg = "HS256".into();
        let err = verify_bundle_manifest(&manifest, &verifying_key).unwrap_err();
        assert!(matches!(err, AttestackError::UnsupportedSignatureAlgorithm(_)));
    }
}
