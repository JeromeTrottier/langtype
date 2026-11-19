//! Field types and definitions (DOMAIN_MODEL §4 `FieldDef`).

use super::VisibilityPolicy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// The type of an entity field, event payload field or setting.
///
/// Written in kits as a string: `text`, `int`, `enum[a, b]`, `ref(npc)`, `list(ref(npc))`…
/// Serialises back to the same canonical string.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum FieldType {
    Text,
    Markdown,
    Int,
    Float,
    Bool,
    /// `YYYY-MM-DD`.
    Date,
    /// RFC 3339, UTC.
    Timestamp,
    /// Any JSON value.
    Json,
    /// One of the listed values.
    Enum(Vec<String>),
    /// The id of an entity of the named type.
    Ref(String),
    /// A list of values of the inner type.
    List(Box<FieldType>),
}

/// Why a field type string could not be parsed.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum FieldTypeError {
    #[error("unknown field type `{0}`")]
    Unknown(String),
    #[error("enum with no values")]
    EmptyEnum,
    #[error("enum value `{0}` is listed twice")]
    DuplicateEnumValue(String),
    #[error("`ref(..)` needs an entity type id, got `{0}`")]
    BadRef(String),
}

impl FieldType {
    /// Parse a kit type string. Whitespace around tokens is ignored.
    pub fn parse(ty: &str) -> Result<Self, FieldTypeError> {
        let ty = ty.trim();
        let base = match ty {
            "text" => Some(Self::Text),
            "markdown" => Some(Self::Markdown),
            "int" => Some(Self::Int),
            "float" => Some(Self::Float),
            "bool" => Some(Self::Bool),
            "date" => Some(Self::Date),
            "timestamp" => Some(Self::Timestamp),
            "json" => Some(Self::Json),
            _ => None,
        };
        if let Some(base) = base {
            return Ok(base);
        }
        if let Some(inner) = ty.strip_prefix("ref(").and_then(|r| r.strip_suffix(')')) {
            let inner = inner.trim();
            return if is_local_id(inner) {
                Ok(Self::Ref(inner.to_string()))
            } else {
                Err(FieldTypeError::BadRef(inner.to_string()))
            };
        }
        if let Some(inner) = ty.strip_prefix("list(").and_then(|r| r.strip_suffix(')')) {
            return Ok(Self::List(Box::new(Self::parse(inner)?)));
        }
        if let Some(inner) = ty.strip_prefix("enum[").and_then(|r| r.strip_suffix(']')) {
            let mut values: Vec<String> = Vec::new();
            for value in inner.split(',').map(str::trim).filter(|v| !v.is_empty()) {
                if values.iter().any(|v| v == value) {
                    return Err(FieldTypeError::DuplicateEnumValue(value.to_string()));
                }
                values.push(value.to_string());
            }
            return if values.is_empty() { Err(FieldTypeError::EmptyEnum) } else { Ok(Self::Enum(values)) };
        }
        Err(FieldTypeError::Unknown(ty.to_string()))
    }

    /// Entity types this type refers to through `ref(..)`, at any depth.
    pub fn referenced_types(&self) -> Vec<&str> {
        match self {
            Self::Ref(t) => vec![t.as_str()],
            Self::List(inner) => inner.referenced_types(),
            _ => Vec::new(),
        }
    }
}

/// Kit-local ids: `^[a-z][a-z0-9_]*$` (schema `localId`).
pub(crate) fn is_local_id(s: &str) -> bool {
    let mut chars = s.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text => f.write_str("text"),
            Self::Markdown => f.write_str("markdown"),
            Self::Int => f.write_str("int"),
            Self::Float => f.write_str("float"),
            Self::Bool => f.write_str("bool"),
            Self::Date => f.write_str("date"),
            Self::Timestamp => f.write_str("timestamp"),
            Self::Json => f.write_str("json"),
            Self::Enum(values) => write!(f, "enum[{}]", values.join(", ")),
            Self::Ref(t) => write!(f, "ref({t})"),
            Self::List(inner) => write!(f, "list({inner})"),
        }
    }
}

impl TryFrom<String> for FieldType {
    type Error = FieldTypeError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s)
    }
}

impl From<FieldType> for String {
    fn from(t: FieldType) -> Self {
        t.to_string()
    }
}

/// One field of an entity type, event payload or settings schema.
///
/// Kits write it short (`body: markdown`) or long (`{ type, required, default, visibility }`);
/// both forms convert to the same `FieldDef`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: FieldType,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Field-level visibility; `None` means the field follows its entity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<VisibilityPolicy>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> FieldType {
        FieldType::parse(s).unwrap()
    }

    #[test]
    fn base_types_parse() {
        let cases = [
            ("text", FieldType::Text),
            ("markdown", FieldType::Markdown),
            ("int", FieldType::Int),
            ("float", FieldType::Float),
            ("bool", FieldType::Bool),
            ("date", FieldType::Date),
            ("timestamp", FieldType::Timestamp),
            ("json", FieldType::Json),
            ("  int ", FieldType::Int),
        ];
        for (input, expected) in cases {
            assert_eq!(parse(input), expected, "{input}");
        }
    }

    #[test]
    fn enums_refs_and_nested_lists_parse() {
        assert_eq!(parse("enum[a, b]"), FieldType::Enum(vec!["a".into(), "b".into()]));
        assert_eq!(parse("enum[ low,high ]"), FieldType::Enum(vec!["low".into(), "high".into()]));
        assert_eq!(parse("ref(npc)"), FieldType::Ref("npc".into()));
        assert_eq!(parse("ref( npc )"), FieldType::Ref("npc".into()));
        assert_eq!(parse("list(text)"), FieldType::List(Box::new(FieldType::Text)));
        assert_eq!(
            parse("list(list(ref(npc)))"),
            FieldType::List(Box::new(FieldType::List(Box::new(FieldType::Ref("npc".into())))))
        );
        assert_eq!(
            parse("list(enum[x, y])"),
            FieldType::List(Box::new(FieldType::Enum(vec!["x".into(), "y".into()])))
        );
    }

    #[test]
    fn unknown_and_malformed_types_are_rejected() {
        assert_eq!(FieldType::parse("colour"), Err(FieldTypeError::Unknown("colour".into())));
        assert_eq!(FieldType::parse("enum[]"), Err(FieldTypeError::EmptyEnum));
        assert_eq!(FieldType::parse("enum[a, a]"), Err(FieldTypeError::DuplicateEnumValue("a".into())));
        assert_eq!(FieldType::parse("ref()"), Err(FieldTypeError::BadRef(String::new())));
        assert_eq!(FieldType::parse("ref(Npc)"), Err(FieldTypeError::BadRef("Npc".into())));
        assert!(FieldType::parse("list(colour)").is_err());
        assert!(FieldType::parse("list(int").is_err());
        assert!(FieldType::parse("object{a: int}").is_err());
        assert!(FieldType::parse("").is_err());
    }

    #[test]
    fn display_round_trips() {
        for s in ["text", "enum[a, b]", "ref(npc)", "list(list(ref(npc)))", "list(enum[x, y])"] {
            assert_eq!(parse(s).to_string(), s);
            assert_eq!(parse(&parse(s).to_string()), parse(s));
        }
    }

    #[test]
    fn serialises_as_the_kit_string() {
        let ty = parse("list(ref(npc))");
        let json = serde_json::to_value(&ty).unwrap();
        assert_eq!(json, serde_json::json!("list(ref(npc))"));
        assert_eq!(serde_json::from_value::<FieldType>(json).unwrap(), ty);
        assert!(serde_json::from_value::<FieldType>(serde_json::json!("colour")).is_err());
    }

    #[test]
    fn referenced_types_reach_into_lists() {
        assert_eq!(parse("list(list(ref(npc)))").referenced_types(), vec!["npc"]);
        assert!(parse("enum[a]").referenced_types().is_empty());
    }
}
