//! Per-entity engine state (DOMAIN_MODEL §6 EngineState).

use crate::ids::{EntityId, KbId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One state of one engine for one entity. `key` lets an engine keep several states per entity
/// (e.g. one FSRS card per skill); the default key is `""`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EngineStateRow {
    pub kb_id: KbId,
    pub engine: String,
    pub entity_id: EntityId,
    #[serde(default)]
    pub key: String,
    pub state: Value,
    /// Optimistic locking, like entities.
    pub version: i64,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn engine_state_round_trips() {
        let row = EngineStateRow {
            kb_id: KbId::new(),
            engine: "fsrs".into(),
            entity_id: EntityId::new(),
            key: "recall".into(),
            state: json!({ "due": "2026-12-01", "stability": 2.5 }),
            version: 4,
            updated_at: Utc::now(),
        };
        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(serde_json::from_value::<EngineStateRow>(json).unwrap(), row);
    }
}
