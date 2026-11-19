//! Entities and relations (DOMAIN_MODEL §6).

use crate::ids::{EntityId, KbId, RelationId, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Where an entity was created. Matches the `entities.source` CHECK.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Setup,
    Starter,
    Library,
    Member,
    Agent,
    Activity,
    Engine,
    Import,
}

/// Lifecycle status, only for types with a `lifecycle`. Matches the `entities.status` CHECK.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityStatus {
    Open,
    Resolved,
    Superseded,
}

/// A grant in `revealed_to`: a kit role or one user. Serialises as a plain string.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Grantee {
    /// Tried first: kit role ids are snake_case and never parse as UUIDs.
    User(UserId),
    Role(String),
}

/// The library record an entity was imported from (DOMAIN_MODEL §5).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryRef {
    pub library_id: String,
    pub record_id: String,
    pub library_version: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub kb_id: KbId,
    /// An entity type of the KB's pinned kit.
    #[serde(rename = "type")]
    pub ty: String,
    /// Always valid against the type's fields.
    pub data: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_user_id: Option<UserId>,
    #[serde(default)]
    pub revealed_to: Vec<Grantee>,
    /// Optimistic locking: every update must send the version it read.
    pub version: i64,
    pub source: Source,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library_ref: Option<LibraryRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<EntityStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

/// A typed link between two entities of the same KB. Visible only if both ends are.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Relation {
    pub id: RelationId,
    pub kb_id: KbId,
    #[serde(rename = "type")]
    pub ty: String,
    pub from_entity: EntityId,
    pub to_entity: EntityId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn entity_round_trips() {
        let user = UserId::new();
        let entity = Entity {
            id: EntityId::new(),
            kb_id: KbId::new(),
            ty: "npc".into(),
            data: json!({ "name": "Mira" }),
            owner_user_id: Some(user),
            revealed_to: vec![Grantee::Role("player".into()), Grantee::User(user)],
            version: 3,
            source: Source::Agent,
            library_ref: Some(LibraryRef {
                library_id: "@official/srd".into(),
                record_id: "goblin".into(),
                library_version: "1.0.0".into(),
            }),
            status: Some(EntityStatus::Superseded),
            superseded_by: Some(EntityId::new()),
            valid_until: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };
        let json = serde_json::to_value(&entity).unwrap();
        assert_eq!(json["type"], "npc");
        assert_eq!(json["source"], "agent");
        assert_eq!(json["status"], "superseded");
        assert_eq!(json["revealed_to"], json!(["player", user.to_string()]));
        assert_eq!(serde_json::from_value::<Entity>(json).unwrap(), entity);
    }

    #[test]
    fn minimal_entity_round_trips() {
        let entity = Entity {
            id: EntityId::new(),
            kb_id: KbId::new(),
            ty: "note".into(),
            data: json!({}),
            owner_user_id: None,
            revealed_to: vec![],
            version: 1,
            source: Source::Member,
            library_ref: None,
            status: None,
            superseded_by: None,
            valid_until: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };
        let json = serde_json::to_value(&entity).unwrap();
        assert!(json.get("status").is_none());
        assert_eq!(serde_json::from_value::<Entity>(json).unwrap(), entity);
    }

    #[test]
    fn relation_round_trips() {
        let relation = Relation {
            id: RelationId::new(),
            kb_id: KbId::new(),
            ty: "member_of".into(),
            from_entity: EntityId::new(),
            to_entity: EntityId::new(),
            data: Some(json!({ "since": "2026-01-01" })),
            created_at: Utc::now(),
        };
        let json = serde_json::to_value(&relation).unwrap();
        assert_eq!(json["type"], "member_of");
        assert_eq!(serde_json::from_value::<Relation>(json).unwrap(), relation);
    }

    #[test]
    fn enums_match_the_migration_checks() {
        let sources = [
            (Source::Setup, "setup"),
            (Source::Starter, "starter"),
            (Source::Library, "library"),
            (Source::Member, "member"),
            (Source::Agent, "agent"),
            (Source::Activity, "activity"),
            (Source::Engine, "engine"),
            (Source::Import, "import"),
        ];
        for (source, text) in sources {
            assert_eq!(serde_json::to_value(source).unwrap(), json!(text));
        }
        let statuses = [
            (EntityStatus::Open, "open"),
            (EntityStatus::Resolved, "resolved"),
            (EntityStatus::Superseded, "superseded"),
        ];
        for (status, text) in statuses {
            assert_eq!(serde_json::to_value(status).unwrap(), json!(text));
        }
    }
}
