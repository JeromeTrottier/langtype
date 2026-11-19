//! Events: append-only facts (DOMAIN_MODEL §6). Never edited; a wrong fact is corrected by a
//! new event.

use crate::actor::{Actor, Channel};
use crate::ids::{ActivityRunId, ChangesetId, EntityId, EventId, KbId, SessionId};
use crate::kit::{EventClass, VisibilityPolicy};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub kb_id: KbId,
    /// A kit event type or a platform type (`session_started`, `entity_revealed`…).
    #[serde(rename = "type")]
    pub ty: String,
    pub class: EventClass,
    #[serde(default)]
    pub payload: Value,
    /// Entities the event is about.
    #[serde(default)]
    pub subjects: Vec<EntityId>,
    pub actor: Actor,
    pub channel: Channel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity_run_id: Option<ActivityRunId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changeset_id: Option<ChangesetId>,
    /// `None`: the most restrictive visibility of its subjects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<VisibilityPolicy>,
    pub at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{ConnectionId, UserId};
    use crate::kit::Audience;
    use serde_json::json;

    #[test]
    fn event_round_trips() {
        let event = Event {
            id: EventId::new(),
            kb_id: KbId::new(),
            ty: "npc_met".into(),
            class: EventClass::Story,
            payload: json!({ "where": "tavern" }),
            subjects: vec![EntityId::new(), EntityId::new()],
            actor: Actor::Agent { user_id: UserId::new(), connection_id: ConnectionId::new() },
            channel: Channel::Mcp,
            session_id: Some(SessionId::new()),
            activity_run_id: Some(ActivityRunId::new()),
            changeset_id: Some(ChangesetId::new()),
            visibility: Some(VisibilityPolicy {
                audience: Audience::Roles(vec!["gm".into()]),
                reveal_on: None,
            }),
            at: Utc::now(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "npc_met");
        assert_eq!(json["class"], "story");
        assert_eq!(serde_json::from_value::<Event>(json).unwrap(), event);
    }
}
