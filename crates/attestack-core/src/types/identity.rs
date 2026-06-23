use serde::{Deserialize, Serialize};

pub const IDENTITY_SCHEMA_V1: &str = "attestack.identity.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublicIdentity {
    pub schema_version: String,
    pub identity_id: String,
    pub public_key: String,
    pub created_at: time::OffsetDateTime,
}

impl PublicIdentity {
    pub fn new(identity_id: String, public_key: String) -> Self {
        Self {
            schema_version: IDENTITY_SCHEMA_V1.into(),
            identity_id,
            public_key,
            created_at: time::OffsetDateTime::now_utc(),
        }
    }
}
