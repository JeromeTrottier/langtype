//! Knowledge bases and their members (DOMAIN_MODEL §3, §6).

use crate::ids::{KbId, UserId, WorkspaceId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// `paused` blocks agent access; `upgrading` blocks writes during a migration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KbStatus {
    #[default]
    Active,
    Paused,
    Archived,
    Upgrading,
}

/// The aggregate root: one space built from one exact kit version.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeBase {
    pub id: KbId,
    pub workspace_id: WorkspaceId,
    pub name: String,
    pub kit_id: String,
    pub kit_version: String,
    pub kit_sha256: String,
    /// Values for the kit's settings schema.
    pub settings: Value,
    pub status: KbStatus,
    pub schema_version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A member's access to one KB.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Access {
    Owner,
    Editor,
    Viewer,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KbMember {
    pub kb_id: KbId,
    pub user_id: UserId,
    pub access: Access,
    /// Roles from the kit (`gm`, `player`, `learner`…).
    #[serde(default)]
    pub kit_roles: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn knowledge_base_round_trips() {
        let kb = KnowledgeBase {
            id: KbId::new(),
            workspace_id: WorkspaceId::new(),
            name: "Russian".into(),
            kit_id: "@test/minimal".into(),
            kit_version: "0.1.0".into(),
            kit_sha256: "ab".repeat(32),
            settings: json!({ "session_length": 15 }),
            status: KbStatus::Upgrading,
            schema_version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_value(&kb).unwrap();
        assert_eq!(json["status"], "upgrading");
        assert_eq!(serde_json::from_value::<KnowledgeBase>(json).unwrap(), kb);
    }

    #[test]
    fn kb_member_round_trips_with_access_and_kit_roles() {
        let member = KbMember {
            kb_id: KbId::new(),
            user_id: UserId::new(),
            access: Access::Editor,
            kit_roles: vec!["player".into()],
        };
        let json = serde_json::to_value(&member).unwrap();
        assert_eq!(json["access"], "editor");
        assert_eq!(json["kit_roles"], json!(["player"]));
        assert_eq!(serde_json::from_value::<KbMember>(json).unwrap(), member);
    }
}
