//! `load_kit`: validate, then convert to a typed `KitDef`.

use mnemo_core::kit::{
    ActivityInput, Audience, Cardinality, EventClass, FieldType, Grading, Lifecycle, VisibilityPolicy,
};
use mnemo_kits::{LoadError, load_kit, validate_dir};
use serde_json::json;
use std::path::{Path, PathBuf};

fn kits_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../kits")
}

#[test]
fn test_kit_converts_to_a_matching_kit_def() {
    let kit = load_kit(&kits_dir().join("_test")).unwrap();

    assert_eq!(kit.id, "@test/minimal");
    assert_eq!(kit.version, "0.1.0");
    assert_eq!(kit.requires.engine_api, "^1");
    assert_eq!(kit.extends, None);

    let roles: Vec<&str> = kit.roles.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(roles, ["owner_only", "guest"]);

    let session_length = &kit.settings["session_length"];
    assert_eq!(session_length.ty, FieldType::Int);
    assert_eq!(session_length.default, Some(json!(15)));

    // Entity types.
    let types: Vec<&str> = kit.entity_types.keys().map(String::as_str).collect();
    assert_eq!(types, ["counter", "note", "thread"]);
    let note = kit.entity_type("note").unwrap();
    assert_eq!(note.title_field.as_deref(), Some("title"));
    assert_eq!(note.lifecycle, Some(Lifecycle::Note));
    assert_eq!(note.dedupe_on, Some(vec!["title".to_string()]));
    assert_eq!(note.visibility, VisibilityPolicy::default());
    let title = note.field("title").unwrap();
    assert_eq!((title.ty.clone(), title.required), (FieldType::Text, true));
    let body = note.field("body").unwrap();
    assert_eq!((body.ty.clone(), body.required), (FieldType::Markdown, false));
    assert_eq!(kit.entity_type("thread").unwrap().lifecycle, Some(Lifecycle::Thread));
    let counter = kit.entity_type("counter").unwrap();
    assert_eq!(counter.visibility.audience, Audience::Roles(vec!["owner_only".into()]));
    assert_eq!(counter.field("value").unwrap().ty, FieldType::Int);

    // Relations.
    let about = &kit.relation_types["about"];
    assert_eq!((about.from_type.as_str(), about.to_type.as_str()), ("note", "counter"));
    assert_eq!(about.cardinality, Cardinality::ManyToMany);

    // Events.
    let changed = &kit.event_types["counter_changed"];
    assert_eq!(changed.class, EventClass::Story);
    assert_eq!(changed.payload.len(), 1);
    assert_eq!((changed.payload[0].name.as_str(), &changed.payload[0].ty), ("delta", &FieldType::Int));
    assert_eq!(kit.event_types["note_reviewed"].class, EventClass::Evidence);
    assert!(kit.event_types["note_reviewed"].payload.is_empty());

    // Engines: config from `engines`, version from `requires.engines`.
    let counters = &kit.engines["counters"];
    assert_eq!(counters.version_req.as_deref(), Some("^1"));
    assert_eq!(counters.config, json!({ "track": ["counter.value"] }));

    // Activities.
    let bump = &kit.activities["bump"];
    assert_eq!(bump.title, "Bump a counter");
    assert_eq!((bump.engine.as_str(), bump.presenter.as_str()), ("counters", "log_form"));
    assert_eq!(bump.grading, Some(Grading::None));
    assert_eq!(
        bump.map["targets"],
        ActivityInput::Filter(json!({ "type": "counter", "where": "entity.data.value < 100", "limit": 5 }))
    );
    assert_eq!(kit.primary_actions[0].activity, "bump");
    assert_eq!(kit.primary_actions[0].label.as_deref(), Some("Bump"));

    // Briefing, setup, skills.
    assert_eq!(kit.briefing["owner_only"], ["last_recap", "open_threads"]);
    assert_eq!(kit.setup.skill, "test-setup");
    assert_eq!(kit.skills, ["test-setup"]);
    assert_eq!(kit.views, ["notes_table"]);
}

#[test]
fn kit_def_round_trips_through_json() {
    let kit = load_kit(&kits_dir().join("_test")).unwrap();
    let json = serde_json::to_value(&kit).unwrap();
    assert_eq!(serde_json::from_value::<mnemo_core::KitDef>(json).unwrap(), kit);
}

#[test]
fn invalid_kit_returns_every_problem() {
    let dir = kits_dir().join("_test/invalid");
    let expected = validate_dir(&dir).unwrap();
    assert!(expected.len() >= 3, "{expected:#?}");
    match load_kit(&dir) {
        Err(LoadError::Invalid(problems)) => {
            assert_eq!(problems, expected);
            let joined: Vec<String> = problems.iter().map(ToString::to_string).collect();
            let joined = joined.join("\n");
            assert!(joined.contains("unknown entity type `faction`"), "{joined}");
            assert!(joined.contains("unknown role `player`"), "{joined}");
            assert!(joined.contains("setup skill `missing-skill`"), "{joined}");
        }
        other => panic!("expected LoadError::Invalid, got {other:?}"),
    }
}

#[test]
fn malformed_reveal_on_is_a_load_error() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("kit.yaml"),
        r#"
id: "@test/reveal"
version: 0.1.0
name: Reveal
license: CC0-1.0
requires: { platform: ">=0.1", engine-api: "^1" }
roles: [gm]
entities:
  npc:
    fields: { name: text }
    visibility: { roles: [gm], reveal_on: "npc_met" }
events:
  npc_met: { class: story }
setup: { skill: setup }
skills: [setup]
"#,
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join("skills/setup")).unwrap();
    std::fs::write(dir.path().join("skills/setup/SKILL.md"), "---\nname: setup\n---\n").unwrap();
    // `kit validate` reports it too.
    let problems = validate_dir(dir.path()).unwrap();
    assert_eq!(problems.len(), 1, "{problems:#?}");
    assert_eq!(problems[0].at, "/entities/npc/visibility/reveal_on");
    match load_kit(dir.path()) {
        Err(LoadError::Invalid(p)) => assert_eq!(p, problems),
        other => panic!("expected LoadError::Invalid, got {other:?}"),
    }
}

#[test]
fn missing_kit_is_a_read_error() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(load_kit(dir.path()), Err(LoadError::Read(_))));
}
