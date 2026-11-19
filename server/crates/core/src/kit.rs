//! Typed kit definitions (DOMAIN_MODEL §4), converted from a schema-valid `kit.yaml`.
//!
//! Loading and validation (schema + references) live in the `kits` crate; this module only holds
//! the types and the pure conversion [`KitDef::from_manifest`].

mod field;
mod manifest;

pub use field::{FieldDef, FieldType, FieldTypeError};
pub use manifest::KitDefError;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Who sees an entity type, field or event type by default (DOMAIN_MODEL §10).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisibilityPolicy {
    pub audience: Audience,
    /// Event type whose audience is added to an entity's `revealed_to` when it is logged
    /// (written `reveal_on: event(<type>)` in kits).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reveal_on: Option<String>,
}

/// The roles a [`VisibilityPolicy`] allows. KB owners always see everything.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Audience {
    /// Every member (the default when a kit gives no `roles`).
    #[default]
    All,
    /// Only members holding one of these kit roles.
    Roles(Vec<String>),
}

/// A kit role (`gm`, `player`, `learner`…). `owner` is implicit and never listed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleDef {
    pub id: String,
}

/// Who may edit entities of a type, besides KB owners.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum Editor {
    /// The member named in the entity's `owner_user_id`.
    Owner,
    /// Members holding this kit role.
    Role(String),
}

impl From<String> for Editor {
    fn from(s: String) -> Self {
        if s == "owner" { Self::Owner } else { Self::Role(s) }
    }
}

impl From<Editor> for String {
    fn from(e: Editor) -> Self {
        match e {
            Editor::Owner => "owner".into(),
            Editor::Role(r) => r,
        }
    }
}

/// How entities of a type age (DOMAIN_MODEL §6 `status`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lifecycle {
    Note,
    Thread,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EntityTypeDef {
    pub id: String,
    /// Fields, ordered by name.
    pub fields: Vec<FieldDef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_field: Option<String>,
    #[serde(default)]
    pub visibility: VisibilityPolicy,
    #[serde(default)]
    pub editable_by: Vec<Editor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<Lifecycle>,
    /// Fields used for duplicate checks; `None` means the title field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dedupe_on: Option<Vec<String>>,
    /// Library id -> (field -> CEL expression over `record`).
    #[serde(default)]
    pub import_from: BTreeMap<String, BTreeMap<String, String>>,
}

impl EntityTypeDef {
    pub fn field(&self, name: &str) -> Option<&FieldDef> {
        self.fields.iter().find(|f| f.name == name)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cardinality {
    OneToOne,
    OneToMany,
    #[default]
    ManyToMany,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationTypeDef {
    pub id: String,
    pub from_type: String,
    pub to_type: String,
    #[serde(default)]
    pub cardinality: Cardinality,
}

/// What an event means to the platform (DOMAIN_MODEL §6 Event `class`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventClass {
    /// What happened.
    Story,
    /// Observations engines learn from.
    Evidence,
    /// Engine housekeeping.
    Bookkeeping,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventTypeDef {
    pub id: String,
    pub class: EventClass,
    /// Payload fields, ordered by name.
    #[serde(default)]
    pub payload: Vec<FieldDef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<VisibilityPolicy>,
}

/// An engine the kit uses: its version requirement (from `requires.engines`) and its config
/// (from `engines`, validated against the engine's `config_schema` later).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EngineBinding {
    pub engine: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_req: Option<String>,
    #[serde(default)]
    pub config: Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grading {
    SelfRated,
    Auto,
    None,
}

/// One input slot of an activity's `map`: a CEL expression or a filter.
///
/// Filters stay raw JSON here; `core::filter` gives them a type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityInput {
    Expr(String),
    Filter(Value),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActivityDef {
    pub id: String,
    pub title: String,
    pub engine: String,
    pub presenter: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub select: Option<Value>,
    #[serde(default)]
    pub map: BTreeMap<String, ActivityInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grading: Option<Grading>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimaryAction {
    pub activity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetupDef {
    pub skill: String,
    #[serde(default)]
    pub starters: Vec<String>,
    #[serde(default)]
    pub libraries: Vec<String>,
}

/// `requires`: what the kit needs from the platform.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requires {
    pub platform: String,
    pub engine_api: String,
    #[serde(default)]
    pub engines: BTreeMap<String, String>,
    #[serde(default)]
    pub libraries: BTreeMap<String, String>,
}

/// A whole kit version, typed. Maps are keyed by kit-local id.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KitDef {
    /// `@scope/name`.
    pub id: String,
    pub version: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub license: String,
    pub requires: Requires,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
    #[serde(default)]
    pub roles: Vec<RoleDef>,
    /// The settings schema: user-facing settings with defaults.
    #[serde(default)]
    pub settings: BTreeMap<String, FieldDef>,
    pub entity_types: BTreeMap<String, EntityTypeDef>,
    #[serde(default)]
    pub relation_types: BTreeMap<String, RelationTypeDef>,
    #[serde(default)]
    pub event_types: BTreeMap<String, EventTypeDef>,
    /// Bound engines: those configured in `engines` and those listed in `requires.engines`.
    #[serde(default)]
    pub engines: BTreeMap<String, EngineBinding>,
    #[serde(default)]
    pub activities: BTreeMap<String, ActivityDef>,
    #[serde(default)]
    pub primary_actions: Vec<PrimaryAction>,
    /// Role -> ordered briefing section ids.
    #[serde(default)]
    pub briefing: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub views: Vec<String>,
    pub setup: SetupDef,
    #[serde(default)]
    pub skills: Vec<String>,
}

impl KitDef {
    pub fn entity_type(&self, id: &str) -> Option<&EntityTypeDef> {
        self.entity_types.get(id)
    }

    pub fn has_role(&self, id: &str) -> bool {
        self.roles.iter().any(|r| r.id == id)
    }
}
