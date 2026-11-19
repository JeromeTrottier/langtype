//! Kit manifests: load `kit.yaml`, validate it against `schemas/kit.schema.json`, then check
//! references (types, roles, engines, activities, skills, starters, libraries).
//!
//! Expression (CEL) type-checking is added in M1 (see docs/FILTERS.md).

use serde_json::Value;
use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};

/// The kit format contract, embedded at build time.
pub const KIT_SCHEMA: &str = include_str!("../../../../schemas/kit.schema.json");
/// The library format contract, embedded at build time.
pub const LIBRARY_SCHEMA: &str = include_str!("../../../../schemas/library.schema.json");

#[derive(Debug, thiserror::Error)]
pub enum KitError {
    #[error("cannot read {path}: {source}")]
    Io { path: PathBuf, source: std::io::Error },
    #[error("invalid YAML in {path}: {source}")]
    Yaml { path: PathBuf, source: serde_yaml::Error },
}

/// One validation problem, with a JSON-pointer-like location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub at: String,
    pub message: String,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", if self.at.is_empty() { "/" } else { &self.at }, self.message)
    }
}

/// Load `<dir>/kit.yaml` as JSON.
pub fn load_manifest(dir: &Path) -> Result<Value, KitError> {
    let path = dir.join("kit.yaml");
    let text =
        std::fs::read_to_string(&path).map_err(|source| KitError::Io { path: path.clone(), source })?;
    serde_yaml::from_str(&text).map_err(|source| KitError::Yaml { path, source })
}

/// Validate the kit in `dir`. Returns every problem found (empty = valid).
pub fn validate_dir(dir: &Path) -> Result<Vec<Problem>, KitError> {
    let manifest = load_manifest(dir)?;
    let mut problems = validate_schema(&manifest);
    if problems.is_empty() {
        problems.extend(check_references(&manifest, Some(dir)));
    }
    Ok(problems)
}

/// Check a manifest against the JSON Schema.
pub fn validate_schema(manifest: &Value) -> Vec<Problem> {
    let schema: Value = serde_json::from_str(KIT_SCHEMA).expect("kit schema is valid JSON");
    let validator = jsonschema::validator_for(&schema).expect("kit schema compiles");
    validator
        .iter_errors(manifest)
        .map(|e| Problem { at: e.instance_path.to_string(), message: e.to_string() })
        .collect()
}

const BASE_TYPES: &[&str] = &["text", "markdown", "int", "float", "bool", "date", "timestamp", "json"];

/// Check that a field type string is well formed and refers to known entity types.
fn check_field_type(ty: &str, entity_types: &BTreeSet<String>) -> Result<(), String> {
    let ty = ty.trim();
    if BASE_TYPES.contains(&ty) {
        return Ok(());
    }
    if let Some(inner) = ty.strip_prefix("ref(").and_then(|r| r.strip_suffix(')')) {
        return if entity_types.contains(inner.trim()) {
            Ok(())
        } else {
            Err(format!("ref to unknown entity type `{}`", inner.trim()))
        };
    }
    if let Some(inner) = ty.strip_prefix("list(").and_then(|r| r.strip_suffix(')')) {
        return check_field_type(inner, entity_types);
    }
    if let Some(inner) = ty.strip_prefix("enum[").and_then(|r| r.strip_suffix(']')) {
        let values: Vec<&str> = inner.split(',').map(str::trim).filter(|v| !v.is_empty()).collect();
        return if values.is_empty() { Err("enum with no values".into()) } else { Ok(()) };
    }
    Err(format!("unknown field type `{ty}`"))
}

fn field_type(field: &Value) -> Option<&str> {
    match field {
        Value::String(s) => Some(s),
        Value::Object(o) => o.get("type").and_then(Value::as_str),
        _ => None,
    }
}

fn keys(value: Option<&Value>) -> BTreeSet<String> {
    value.and_then(Value::as_object).map(|o| o.keys().cloned().collect()).unwrap_or_default()
}

fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(String::from).collect())
        .unwrap_or_default()
}

/// Reference checks on a schema-valid manifest. `dir` enables file checks (skills, starters).
pub fn check_references(m: &Value, dir: Option<&Path>) -> Vec<Problem> {
    let mut problems = Vec::new();
    let mut problem = |at: String, message: String| problems.push(Problem { at, message });

    let entity_types = keys(m.get("entities"));
    let roles: BTreeSet<String> = strings(m.get("roles")).into_iter().collect();
    let activities = keys(m.get("activities"));
    let skills: BTreeSet<String> = strings(m.get("skills")).into_iter().collect();
    let mut engines = keys(m.get("engines"));
    engines.extend(keys(m.get("requires").and_then(|r| r.get("engines"))));
    let required_libraries = keys(m.get("requires").and_then(|r| r.get("libraries")));

    let check_roles = |at: &str, vis: Option<&Value>, problem: &mut dyn FnMut(String, String)| {
        for role in strings(vis.and_then(|v| v.get("roles"))) {
            if !roles.contains(&role) {
                problem(format!("{at}/visibility/roles"), format!("unknown role `{role}`"));
            }
        }
    };

    // Entities: field types, visibility roles, title/dedupe fields.
    if let Some(entities) = m.get("entities").and_then(Value::as_object) {
        for (name, def) in entities {
            let at = format!("/entities/{name}");
            let fields = def.get("fields").and_then(Value::as_object);
            for (field, spec) in fields.into_iter().flatten() {
                match field_type(spec) {
                    Some(ty) => {
                        if let Err(e) = check_field_type(ty, &entity_types) {
                            problem(format!("{at}/fields/{field}"), e);
                        }
                    }
                    None => problem(format!("{at}/fields/{field}"), "missing type".into()),
                }
                if let Value::Object(o) = spec {
                    check_roles(&format!("{at}/fields/{field}"), o.get("visibility"), &mut problem);
                }
            }
            check_roles(&at, def.get("visibility"), &mut problem);
            let field_names: BTreeSet<String> =
                fields.map(|f| f.keys().cloned().collect()).unwrap_or_default();
            let mut referenced = strings(def.get("dedupe_on"));
            if let Some(t) = def.get("title_field").and_then(Value::as_str) {
                referenced.push(t.to_string());
            }
            for f in referenced {
                if !field_names.contains(&f) {
                    problem(at.clone(), format!("refers to unknown field `{f}`"));
                }
            }
        }
    }

    // Relations.
    if let Some(relations) = m.get("relations").and_then(Value::as_object) {
        for (name, def) in relations {
            for end in ["from", "to"] {
                if let Some(t) = def.get(end).and_then(Value::as_str)
                    && !entity_types.contains(t)
                {
                    problem(format!("/relations/{name}/{end}"), format!("unknown entity type `{t}`"));
                }
            }
        }
    }

    // Events: payload field types and visibility.
    if let Some(events) = m.get("events").and_then(Value::as_object) {
        for (name, def) in events {
            for (field, spec) in def.get("payload").and_then(Value::as_object).into_iter().flatten() {
                if let Some(Err(e)) = field_type(spec).map(|t| check_field_type(t, &entity_types)) {
                    problem(format!("/events/{name}/payload/{field}"), e);
                }
            }
            check_roles(&format!("/events/{name}"), def.get("visibility"), &mut problem);
        }
    }

    // Activities: engine bound, filter types exist.
    if let Some(acts) = m.get("activities").and_then(Value::as_object) {
        for (name, def) in acts {
            let at = format!("/activities/{name}");
            if let Some(engine) = def.get("engine").and_then(Value::as_str)
                && !engines.contains(engine)
            {
                problem(at.clone(), format!("engine `{engine}` is not bound or required"));
            }
            for (slot, value) in def.get("map").and_then(Value::as_object).into_iter().flatten() {
                if let Some(filter) = value.as_object() {
                    let types = match filter.get("type") {
                        Some(Value::String(s)) => vec![s.clone()],
                        other => strings(other),
                    };
                    for t in types {
                        if !entity_types.contains(&t) {
                            problem(format!("{at}/map/{slot}"), format!("unknown entity type `{t}`"));
                        }
                    }
                }
            }
        }
    }

    // Primary actions refer to activities.
    for (i, action) in m.get("primary_actions").and_then(Value::as_array).into_iter().flatten().enumerate() {
        let id = action.as_str().or_else(|| action.get("activity").and_then(Value::as_str));
        if let Some(id) = id
            && !activities.contains(id)
        {
            problem(format!("/primary_actions/{i}"), format!("unknown activity `{id}`"));
        }
    }

    // Briefing keys are roles (or `owner`).
    for role in keys(m.get("briefing")) {
        if role != "owner" && !roles.contains(&role) {
            problem(format!("/briefing/{role}"), format!("unknown role `{role}`"));
        }
    }

    // Setup.
    if let Some(setup) = m.get("setup") {
        if let Some(skill) = setup.get("skill").and_then(Value::as_str)
            && !skills.contains(skill)
        {
            problem("/setup/skill".into(), format!("setup skill `{skill}` is not listed in `skills`"));
        }
        for lib in strings(setup.get("libraries")) {
            if !required_libraries.contains(&lib) {
                problem("/setup/libraries".into(), format!("library `{lib}` is not in `requires.libraries`"));
            }
        }
        if let Some(dir) = dir {
            for starter in strings(setup.get("starters")) {
                if !dir.join("starters").join(format!("{starter}.yaml")).is_file() {
                    problem("/setup/starters".into(), format!("missing file starters/{starter}.yaml"));
                }
            }
        }
    }

    // Skills have a SKILL.md whose `name` matches.
    if let Some(dir) = dir {
        for skill in &skills {
            let path = dir.join("skills").join(skill).join("SKILL.md");
            match std::fs::read_to_string(&path) {
                Ok(text) => {
                    let declared = text
                        .strip_prefix("---")
                        .and_then(|rest| rest.split("---").next())
                        .and_then(|fm| fm.lines().find_map(|l| l.strip_prefix("name:")))
                        .map(str::trim);
                    if declared != Some(skill.as_str()) {
                        problem(format!("/skills/{skill}"), "SKILL.md frontmatter `name` must match".into());
                    }
                }
                Err(_) => problem(format!("/skills/{skill}"), format!("missing skills/{skill}/SKILL.md")),
            }
        }
    }

    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kits_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../kits")
    }

    #[test]
    fn the_schemas_compile() {
        for schema in [KIT_SCHEMA, LIBRARY_SCHEMA] {
            let value: Value = serde_json::from_str(schema).unwrap();
            jsonschema::validator_for(&value).unwrap();
        }
    }

    #[test]
    fn minimal_test_kit_is_valid() {
        let problems = validate_dir(&kits_dir().join("_test")).unwrap();
        assert!(problems.is_empty(), "unexpected problems: {problems:#?}");
    }

    #[test]
    fn invalid_test_kit_reports_each_problem() {
        let problems = validate_dir(&kits_dir().join("_test/invalid")).unwrap();
        let text: Vec<String> = problems.iter().map(|p| p.to_string()).collect();
        let joined = text.join("\n");
        assert!(joined.contains("unknown entity type `faction`"), "{joined}");
        assert!(joined.contains("unknown role `player`"), "{joined}");
        assert!(joined.contains("setup skill `missing-skill`"), "{joined}");
    }

    #[test]
    fn schema_rejects_unknown_top_level_keys() {
        let mut manifest = load_manifest(&kits_dir().join("_test")).unwrap();
        manifest["surprise"] = Value::Bool(true);
        assert!(!validate_schema(&manifest).is_empty());
    }

    #[test]
    fn field_types_are_parsed() {
        let types: BTreeSet<String> = ["npc".to_string()].into();
        assert!(check_field_type("list(ref(npc))", &types).is_ok());
        assert!(check_field_type("enum[friendly, hostile]", &types).is_ok());
        assert!(check_field_type("ref(faction)", &types).is_err());
        assert!(check_field_type("colour", &types).is_err());
    }
}
