//! Mnemo domain core: identifiers, actors and the rules from `docs/DOMAIN_MODEL.md`.
//!
//! This crate performs no I/O. Storage, HTTP and MCP live in other crates.

pub mod actor;
pub mod changeset;
pub mod engine_state;
pub mod entity;
pub mod event;
pub mod filter;
pub mod ids;
pub mod kb;
pub mod kit;
pub mod validate;
pub mod visibility;

pub use actor::{Actor, Channel};
pub use engine_state::EngineStateRow;
pub use entity::{Entity, EntityStatus, Grantee, LibraryRef, Relation, Source};
pub use event::Event;
pub use ids::{
    ActivityRunId, ChangesetId, ConnectionId, EntityId, EventId, KbId, RelationId, SessionId, UserId,
    WorkspaceId,
};
pub use kb::{Access, KbMember, KbStatus, KnowledgeBase};
pub use kit::KitDef;

/// Version of the database schema this build expects (see `store` migrations).
pub const SCHEMA_VERSION: u32 = 1;
