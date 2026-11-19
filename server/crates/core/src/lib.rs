//! Mnemo domain core: identifiers, actors and the rules from `docs/DOMAIN_MODEL.md`.
//!
//! This crate performs no I/O. Storage, HTTP and MCP live in other crates.

pub mod actor;
pub mod ids;

pub use actor::{Actor, Channel};
pub use ids::{ChangesetId, EntityId, EventId, KbId, SessionId, UserId, WorkspaceId};

/// Version of the database schema this build expects (see `store` migrations).
pub const SCHEMA_VERSION: u32 = 1;
