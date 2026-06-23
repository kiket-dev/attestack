mod artifact;
mod bundle;
mod event;
mod identity;
mod session;
mod signature;

pub use artifact::*;
pub use bundle::*;
pub use event::*;
pub use identity::*;
pub use session::*;
pub use signature::*;

pub const SESSION_SCHEMA_V1: &str = "attestack.session.v1";
pub const EVENT_SCHEMA_V1: &str = "attestack.event.v1";
pub const BUNDLE_SCHEMA_V1: &str = "attestack.bundle.v1";

/// Maximum bytes captured per command stream (stdout or stderr).
pub const COMMAND_OUTPUT_LIMIT_BYTES: u64 = 10 * 1024 * 1024;
