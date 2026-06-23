use std::fs;
use std::path::{Path, PathBuf};

use attestack_core::{generate_signing_key, identity_id_from_public_key, PublicIdentity};
use ed25519_dalek::SigningKey as DalekSigningKey;
use thiserror::Error;

pub const DEFAULT_IDENTITY_FILE: &str = "default.public.json";

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("identity error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("invalid identity file at {0}")]
    InvalidIdentity(PathBuf),

    #[error("private key not found for identity {0}")]
    PrivateKeyNotFound(String),
}

pub type Result<T> = std::result::Result<T, IdentityError>;

pub struct Identity {
    pub identity_id: String,
    pub public: PublicIdentity,
    signing_key: DalekSigningKey,
}

impl Identity {
    pub fn ensure_default(store_root: &Path) -> Result<Self> {
        let identities_dir = store_root.join(crate::IDENTITIES_DIR);
        fs::create_dir_all(&identities_dir)?;
        let public_path = identities_dir.join(DEFAULT_IDENTITY_FILE);

        if public_path.is_file() {
            return Self::load(store_root);
        }

        let signing_key = generate_signing_key();
        let public_key_bytes = signing_key.verifying_key().to_bytes();
        let identity_id = identity_id_from_public_key(&public_key_bytes);
        let public = PublicIdentity::new(
            identity_id.clone(),
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, public_key_bytes),
        );

        fs::write(public_path, serde_json::to_string_pretty(&public)?)?;
        write_private_key(&identity_id, signing_key.to_bytes())?;

        Ok(Self { identity_id, public, signing_key })
    }

    pub fn load(store_root: &Path) -> Result<Self> {
        let public_path = store_root.join(crate::IDENTITIES_DIR).join(DEFAULT_IDENTITY_FILE);
        let raw = fs::read_to_string(&public_path)?;
        let public: PublicIdentity = serde_json::from_str(&raw)?;
        let private_bytes = read_private_key(&public.identity_id)?;
        let signing_key = DalekSigningKey::from_bytes(&private_bytes);
        Ok(Self { identity_id: public.identity_id.clone(), public, signing_key })
    }

    pub fn signing_key(&self) -> &DalekSigningKey {
        &self.signing_key
    }

    pub fn verifying_key(&self) -> ed25519_dalek::VerifyingKey {
        self.signing_key.verifying_key()
    }
}

pub fn user_keys_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".attestack").join("keys"))
        .unwrap_or_else(|| PathBuf::from(".attestack/keys"))
}

fn private_key_path(identity_id: &str) -> PathBuf {
    user_keys_dir().join(format!("{identity_id}.key"))
}

fn write_private_key(identity_id: &str, key_bytes: [u8; 32]) -> Result<()> {
    let dir = user_keys_dir();
    fs::create_dir_all(&dir)?;
    fs::write(
        private_key_path(identity_id),
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, key_bytes),
    )?;
    Ok(())
}

fn read_private_key(identity_id: &str) -> Result<[u8; 32]> {
    let path = private_key_path(identity_id);
    if !path.is_file() {
        return Err(IdentityError::PrivateKeyNotFound(identity_id.into()));
    }
    let raw = fs::read_to_string(path)?;
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, raw.trim())
        .map_err(|_| IdentityError::InvalidIdentity(private_key_path(identity_id)))?;
    bytes.try_into().map_err(|_| IdentityError::InvalidIdentity(private_key_path(identity_id)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn ensure_and_reload_identity() {
        let dir = tempdir().unwrap();
        let store_root = dir.path().join(".attestack");
        fs::create_dir_all(&store_root).unwrap();
        let identity = Identity::ensure_default(&store_root).unwrap();
        let reloaded = Identity::load(&store_root).unwrap();
        assert_eq!(identity.identity_id, reloaded.identity_id);
        assert!(private_key_path(&identity.identity_id).is_file());
    }
}
