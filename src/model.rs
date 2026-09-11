use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum RecallQuery {
    Simple(String),
    Structured(StructuredRecallQuery),
}

impl RecallQuery {
    pub fn text(&self) -> &str {
        match self {
            Self::Simple(value) => value,
            Self::Structured(value) => &value.query,
        }
    }

    pub fn priority(&self) -> Option<f64> {
        match self {
            Self::Simple(_) => None,
            Self::Structured(value) => value.priority,
        }
    }

    pub fn roles(&self) -> &[String] {
        match self {
            Self::Simple(_) => &[],
            Self::Structured(value) => value
                .when
                .as_ref()
                .map(|when| when.roles.as_slice())
                .unwrap_or(&[]),
        }
    }

    pub fn memory_types(&self) -> &[String] {
        match self {
            Self::Simple(_) => &[],
            Self::Structured(value) => &value.memory_types,
        }
    }

    pub fn connection_types(&self) -> &[String] {
        match self {
            Self::Simple(_) => &[],
            Self::Structured(value) => &value.connection_types,
        }
    }

    pub fn entities(&self) -> &[Entity] {
        match self {
            Self::Simple(_) => &[],
            Self::Structured(value) => &value.entities,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StructuredRecallQuery {
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<QueryCondition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub memory_types: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connection_types: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<Entity>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct QueryCondition {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<String>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Entity {
    Simple(String),
    Structured(StructuredEntity),
}

impl Entity {
    pub fn name(&self) -> &str {
        match self {
            Self::Simple(value) => value,
            Self::Structured(value) => &value.name,
        }
    }

    pub fn salience(&self) -> Option<f64> {
        match self {
            Self::Simple(_) => None,
            Self::Structured(value) => value.salience,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StructuredEntity {
    pub name: String,
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub salience: Option<f64>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct HookFrontmatter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bank: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherits: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub memory_types: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connection_types: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mental_models: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub knowledge_pages: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recall_queries: Vec<RecallQuery>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<Entity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sensitivity: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl HookFrontmatter {
    pub fn inherits(&self) -> bool {
        self.inherits.unwrap_or(true)
    }
}
