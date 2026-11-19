//! Conversion from the `kit.yaml` shape (`schemas/kit.schema.json`) to [`KitDef`].
//!
//! The input is expected to be schema-valid; anything the schema can't express (field type
//! syntax, `reveal_on` syntax) is checked here and reported with its location.

use super::field::is_local_id;
use super::{
    ActivityDef, ActivityInput, Audience, Cardinality, Editor, EngineBinding, EntityTypeDef, EventClass,
    EventTypeDef, FieldDef, FieldType, Grading, KitDef, Lifecycle, PrimaryAction, RelationTypeDef, Requires,
    RoleDef, SetupDef, VisibilityPolicy,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// A problem converting a manifest, with a JSON-pointer-like location.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{at}: {message}")]
pub struct KitDefError {
    pub at: String,
    pub message: String,
}

impl KitDefError {
    fn new(at: impl Into<String>, message: impl Into<String>) -> Self {
        Self { at: at.into(), message: message.into() }
    }
}

#[derive(Deserialize)]
struct RawKit {
    id: String,
    version: String,
    name: String,
    description: Option<String>,
    license: String,
    requires: RawRequires,
    extends: Option<String>,
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
    settings: BTreeMap<String, Value>,
    entities: BTreeMap<String, RawEntity>,
    #[serde(default)]
    relations: BTreeMap<String, RawRelation>,
    #[serde(default)]
    events: BTreeMap<String, RawEvent>,
    #[serde(default)]
    engines: BTreeMap<String, Value>,
    #[serde(default)]
    activities: BTreeMap<String, RawActivity>,
    #[serde(default)]
    primary_actions: Vec<RawPrimaryAction>,
    #[serde(default)]
    briefing: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    tools: Vec<String>,
    #[serde(default)]
    views: Vec<String>,
    setup: SetupDef,
    #[serde(default)]
    skills: Vec<String>,
}

#[derive(Deserialize)]
struct RawRequires {
    platform: String,
    #[serde(rename = "engine-api")]
    engine_api: String,
    #[serde(default)]
    engines: BTreeMap<String, String>,
    #[serde(default)]
    libraries: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct RawVisibility {
    roles: Option<Vec<String>>,
    reveal_on: Option<String>,
}

#[derive(Deserialize)]
struct RawEntity {
    fields: BTreeMap<String, Value>,
    title_field: Option<String>,
    visibility: Option<RawVisibility>,
    #[serde(default)]
    editable_by: Vec<Editor>,
    lifecycle: Option<Lifecycle>,
    dedupe_on: Option<Vec<String>>,
    #[serde(default)]
    import_from: BTreeMap<String, BTreeMap<String, String>>,
}

#[derive(Deserialize)]
struct RawRelation {
    from: String,
    to: String,
    cardinality: Option<Cardinality>,
}

#[derive(Deserialize)]
struct RawEvent {
    class: EventClass,
    #[serde(default)]
    payload: BTreeMap<String, Value>,
    visibility: Option<RawVisibility>,
}

#[derive(Deserialize)]
struct RawActivity {
    title: String,
    engine: String,
    presenter: String,
    skill: Option<String>,
    select: Option<Value>,
    #[serde(default)]
    map: BTreeMap<String, Value>,
    grading: Option<Grading>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawPrimaryAction {
    Id(String),
    Full(PrimaryAction),
}

#[derive(Deserialize)]
struct RawLongField {
    #[serde(rename = "type")]
    ty: String,
    #[serde(default)]
    required: bool,
    default: Option<Value>,
    description: Option<String>,
    visibility: Option<RawVisibility>,
}

impl VisibilityPolicy {
    /// Build a policy from a kit's `{ roles?, reveal_on? }`. No `roles` means everyone;
    /// `reveal_on` must read `event(<event_type>)`.
    pub fn from_parts(roles: Option<Vec<String>>, reveal_on: Option<&str>) -> Result<Self, String> {
        let audience = roles.map_or(Audience::All, Audience::Roles);
        let reveal_on = reveal_on.map(parse_reveal_on).transpose()?;
        Ok(Self { audience, reveal_on })
    }
}

fn parse_reveal_on(s: &str) -> Result<String, String> {
    s.trim()
        .strip_prefix("event(")
        .and_then(|r| r.strip_suffix(')'))
        .map(str::trim)
        .filter(|t| is_local_id(t))
        .map(String::from)
        .ok_or_else(|| format!("malformed `reveal_on` `{s}`: expected `event(<event_type>)`"))
}

fn visibility(raw: Option<RawVisibility>, at: &str) -> Result<Option<VisibilityPolicy>, KitDefError> {
    raw.map(|v| VisibilityPolicy::from_parts(v.roles, v.reveal_on.as_deref()))
        .transpose()
        .map_err(|m| KitDefError::new(format!("{at}/visibility/reveal_on"), m))
}

impl FieldDef {
    /// Convert a field spec in short (`"markdown"`) or long (`{ type, required, … }`) form.
    /// Error locations are relative to the field.
    pub fn from_manifest(name: &str, spec: &Value) -> Result<Self, KitDefError> {
        let long = match spec {
            Value::String(ty) => RawLongField {
                ty: ty.clone(),
                required: false,
                default: None,
                description: None,
                visibility: None,
            },
            other => RawLongField::deserialize(other).map_err(|e| KitDefError::new("", e.to_string()))?,
        };
        let ty = FieldType::parse(&long.ty).map_err(|e| KitDefError::new("", e.to_string()))?;
        Ok(Self {
            name: name.to_string(),
            ty,
            required: long.required,
            default: long.default,
            description: long.description,
            visibility: visibility(long.visibility, "")?,
        })
    }
}

fn fields(specs: BTreeMap<String, Value>, at: &str, errors: &mut Vec<KitDefError>) -> Vec<FieldDef> {
    specs
        .iter()
        .filter_map(|(name, spec)| {
            FieldDef::from_manifest(name, spec)
                .map_err(|e| errors.push(KitDefError::new(format!("{at}/{name}{}", e.at), e.message)))
                .ok()
        })
        .collect()
}

impl KitDef {
    /// Convert a schema-valid manifest (as JSON). Returns every problem found.
    pub fn from_manifest(manifest: &Value) -> Result<Self, Vec<KitDefError>> {
        let raw = RawKit::deserialize(manifest).map_err(|e| vec![KitDefError::new("", e.to_string())])?;
        let mut errors = Vec::new();

        let settings =
            fields(raw.settings, "/settings", &mut errors).into_iter().map(|f| (f.name.clone(), f)).collect();

        let mut entity_types = BTreeMap::new();
        for (id, e) in raw.entities {
            let at = format!("/entities/{id}");
            let fields = fields(e.fields, &format!("{at}/fields"), &mut errors);
            let visibility = visibility(e.visibility, &at).unwrap_or_else(|err| {
                errors.push(err);
                None
            });
            entity_types.insert(
                id.clone(),
                EntityTypeDef {
                    id,
                    fields,
                    title_field: e.title_field,
                    visibility: visibility.unwrap_or_default(),
                    editable_by: e.editable_by,
                    lifecycle: e.lifecycle,
                    dedupe_on: e.dedupe_on,
                    import_from: e.import_from,
                },
            );
        }

        let relation_types = raw
            .relations
            .into_iter()
            .map(|(id, r)| {
                let def = RelationTypeDef {
                    id: id.clone(),
                    from_type: r.from,
                    to_type: r.to,
                    cardinality: r.cardinality.unwrap_or_default(),
                };
                (id, def)
            })
            .collect();

        let mut event_types = BTreeMap::new();
        for (id, e) in raw.events {
            let at = format!("/events/{id}");
            let payload = fields(e.payload, &format!("{at}/payload"), &mut errors);
            let visibility = visibility(e.visibility, &at).unwrap_or_else(|err| {
                errors.push(err);
                None
            });
            event_types.insert(id.clone(), EventTypeDef { id, class: e.class, payload, visibility });
        }

        let mut engines: BTreeMap<String, EngineBinding> = BTreeMap::new();
        for engine in raw.engines.keys().chain(raw.requires.engines.keys()) {
            engines.entry(engine.clone()).or_insert_with(|| EngineBinding {
                engine: engine.clone(),
                version_req: raw.requires.engines.get(engine).cloned(),
                config: raw.engines.get(engine).cloned().unwrap_or_else(|| Value::Object(Default::default())),
            });
        }

        let activities = raw
            .activities
            .into_iter()
            .map(|(id, a)| {
                let map = a
                    .map
                    .into_iter()
                    .map(|(slot, v)| {
                        let input = match v {
                            Value::String(expr) => ActivityInput::Expr(expr),
                            filter => ActivityInput::Filter(filter),
                        };
                        (slot, input)
                    })
                    .collect();
                let def = ActivityDef {
                    id: id.clone(),
                    title: a.title,
                    engine: a.engine,
                    presenter: a.presenter,
                    skill: a.skill,
                    select: a.select,
                    map,
                    grading: a.grading,
                };
                (id, def)
            })
            .collect();

        let primary_actions = raw
            .primary_actions
            .into_iter()
            .map(|a| match a {
                RawPrimaryAction::Id(activity) => PrimaryAction { activity, label: None },
                RawPrimaryAction::Full(action) => action,
            })
            .collect();

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(Self {
            id: raw.id,
            version: raw.version,
            name: raw.name,
            description: raw.description,
            license: raw.license,
            requires: Requires {
                platform: raw.requires.platform,
                engine_api: raw.requires.engine_api,
                engines: raw.requires.engines,
                libraries: raw.requires.libraries,
            },
            extends: raw.extends,
            roles: raw.roles.into_iter().map(|id| RoleDef { id }).collect(),
            settings,
            entity_types,
            relation_types,
            event_types,
            engines,
            activities,
            primary_actions,
            briefing: raw.briefing,
            tools: raw.tools,
            views: raw.views,
            setup: raw.setup,
            skills: raw.skills,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn minimal(entities: Value) -> Value {
        json!({
            "id": "@test/unit",
            "version": "0.1.0",
            "name": "Unit",
            "license": "CC0-1.0",
            "requires": { "platform": ">=0.1", "engine-api": "^1" },
            "entities": entities,
            "setup": { "skill": "setup" },
        })
    }

    #[test]
    fn short_and_long_fields_convert_to_the_same_def() {
        let short = FieldDef::from_manifest("body", &json!("markdown")).unwrap();
        let long = FieldDef::from_manifest("body", &json!({ "type": "markdown" })).unwrap();
        assert_eq!(short, long);
        assert_eq!(
            short,
            FieldDef {
                name: "body".into(),
                ty: FieldType::Markdown,
                required: false,
                default: None,
                description: None,
                visibility: None,
            }
        );
    }

    #[test]
    fn long_fields_keep_every_option() {
        let def = FieldDef::from_manifest(
            "twist",
            &json!({ "type": "enum[a, b]", "required": true, "default": "a",
                     "description": "Hidden", "visibility": { "roles": ["gm"] } }),
        )
        .unwrap();
        assert_eq!(def.ty, FieldType::Enum(vec!["a".into(), "b".into()]));
        assert!(def.required);
        assert_eq!(def.default, Some(json!("a")));
        assert_eq!(def.description.as_deref(), Some("Hidden"));
        assert_eq!(
            def.visibility,
            Some(VisibilityPolicy { audience: Audience::Roles(vec!["gm".into()]), reveal_on: None })
        );
    }

    #[test]
    fn bad_field_types_are_errors() {
        let err = FieldDef::from_manifest("x", &json!("colour")).unwrap_err();
        assert_eq!(err.message, "unknown field type `colour`");
    }

    #[test]
    fn visibility_defaults_to_all() {
        let kit =
            KitDef::from_manifest(&minimal(json!({ "note": { "fields": { "title": "text" } } }))).unwrap();
        assert_eq!(kit.entity_types["note"].visibility, VisibilityPolicy::default());
        assert_eq!(VisibilityPolicy::default().audience, Audience::All);
        assert_eq!(VisibilityPolicy::from_parts(None, None).unwrap().audience, Audience::All);
    }

    #[test]
    fn visibility_models_roles_and_reveal_on() {
        let policy = VisibilityPolicy::from_parts(Some(vec!["gm".into()]), Some("event( npc_met )")).unwrap();
        assert_eq!(policy.audience, Audience::Roles(vec!["gm".into()]));
        assert_eq!(policy.reveal_on.as_deref(), Some("npc_met"));
    }

    #[test]
    fn malformed_reveal_on_is_an_error() {
        for bad in ["npc_met", "event()", "event(NpcMet)", "events(npc_met)", "event(npc_met"] {
            assert!(VisibilityPolicy::from_parts(None, Some(bad)).is_err(), "{bad}");
        }
        let manifest = minimal(json!({
            "npc": { "fields": { "name": "text" }, "visibility": { "roles": ["gm"], "reveal_on": "npc_met" } },
        }));
        let errors = KitDef::from_manifest(&manifest).unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].at, "/entities/npc/visibility/reveal_on");
        assert!(errors[0].message.contains("malformed `reveal_on`"), "{}", errors[0]);
    }

    #[test]
    fn every_conversion_error_is_reported_with_its_location() {
        let manifest = minimal(json!({
            "a": { "fields": { "x": "colour", "y": { "type": "int", "visibility": { "reveal_on": "oops" } } } },
        }));
        let errors = KitDef::from_manifest(&manifest).unwrap_err();
        let at: Vec<&str> = errors.iter().map(|e| e.at.as_str()).collect();
        assert_eq!(at, ["/entities/a/fields/x", "/entities/a/fields/y/visibility/reveal_on"]);
    }
}
