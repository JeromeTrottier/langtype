//! Who did something, and through which channel (DOMAIN_MODEL §3).

use crate::ids::UserId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The origin of an event, changeset or session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Actor {
    /// A person using the dashboard.
    Member { user_id: UserId },
    /// Claude (or another MCP client) acting as a member.
    Agent { user_id: UserId, connection_id: Uuid },
    /// Effects returned by an engine hook.
    Engine { engine: String },
    /// Imports, migrations, scheduled jobs.
    System,
}

/// How a change reached the platform.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    Dashboard,
    Mcp,
    Activity,
    Engine,
    Import,
    Migration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actor_serialises_with_a_kind_tag() {
        let json = serde_json::to_value(Actor::System).unwrap();
        assert_eq!(json, serde_json::json!({ "kind": "system" }));
    }
}
