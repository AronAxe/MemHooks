use serde::de::{DeserializeOwned, Error as DeError};
use serde::{Deserialize, Deserializer, Serialize};
use serde_yaml_ng::{Mapping, Value};
use std::collections::BTreeMap;

pub type BackendMap = BTreeMap<String, Value>;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum RecallQuery {
    Simple(String),
    Structured(StructuredRecallQuery),
}

impl<'de> Deserialize<'de> for RecallQuery {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Ok(match value {
            Value::String(value) => Self::Simple(value),
            Value::Mapping(mapping) => {
                Self::Structured(StructuredRecallQuery::from_mapping(mapping))
            }
            invalid => Self::Structured(StructuredRecallQuery::invalid(invalid)),
        })
    }
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

    pub fn entities(&self) -> &[Entity] {
        match self {
            Self::Simple(_) => &[],
            Self::Structured(value) => &value.entities,
        }
    }

    pub fn resources(&self) -> &[Resource] {
        match self {
            Self::Simple(_) => &[],
            Self::Structured(value) => &value.resources,
        }
    }

    pub fn tags(&self) -> &[String] {
        match self {
            Self::Simple(_) => &[],
            Self::Structured(value) => &value.tags,
        }
    }

    pub fn backends(&self) -> &BackendMap {
        static EMPTY: std::sync::OnceLock<BackendMap> = std::sync::OnceLock::new();
        match self {
            Self::Simple(_) => EMPTY.get_or_init(BackendMap::new),
            Self::Structured(value) => &value.backends,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct StructuredRecallQuery {
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<QueryCondition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<Entity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resources: Vec<Resource>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub backends: BackendMap,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl StructuredRecallQuery {
    fn from_mapping(mut mapping: Mapping) -> Self {
        let mut extra = BTreeMap::new();
        let query = take_typed(&mut mapping, "query", &mut extra).unwrap_or_default();
        let priority = take_typed(&mut mapping, "priority", &mut extra);
        let when = take_typed(&mut mapping, "when", &mut extra);
        let entities = take_typed(&mut mapping, "entities", &mut extra).unwrap_or_default();
        let resources = take_typed(&mut mapping, "resources", &mut extra).unwrap_or_default();
        let tags = take_typed(&mut mapping, "tags", &mut extra).unwrap_or_default();
        let backends = take_typed(&mut mapping, "backends", &mut extra).unwrap_or_default();
        extend_extra(&mut extra, mapping);
        Self {
            query,
            priority,
            when,
            entities,
            resources,
            tags,
            backends,
            extra,
        }
    }

    fn invalid(value: Value) -> Self {
        let mut extra = BTreeMap::new();
        extra.insert("_invalid_value".into(), value);
        Self {
            extra,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct QueryCondition {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<String>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Entity {
    Simple(String),
    Structured(StructuredEntity),
}

impl<'de> Deserialize<'de> for Entity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Ok(match value {
            Value::String(value) => Self::Simple(value),
            Value::Mapping(mapping) => Self::Structured(StructuredEntity::from_mapping(mapping)),
            invalid => Self::Structured(StructuredEntity::invalid(invalid)),
        })
    }
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

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct StructuredEntity {
    pub name: String,
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub salience: Option<f64>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl StructuredEntity {
    fn from_mapping(mut mapping: Mapping) -> Self {
        let mut extra = BTreeMap::new();
        let name = take_typed(&mut mapping, "name", &mut extra).unwrap_or_default();
        let entity_type = take_typed(&mut mapping, "type", &mut extra);
        let salience = take_typed(&mut mapping, "salience", &mut extra);
        extend_extra(&mut extra, mapping);
        Self {
            name,
            entity_type,
            salience,
            extra,
        }
    }

    fn invalid(value: Value) -> Self {
        let mut extra = BTreeMap::new();
        extra.insert("_invalid_value".into(), value);
        Self {
            extra,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Resource {
    Simple(String),
    Structured(StructuredResource),
}

impl<'de> Deserialize<'de> for Resource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Ok(match value {
            Value::String(value) => Self::Simple(value),
            Value::Mapping(mapping) => Self::Structured(StructuredResource::from_mapping(mapping)),
            invalid => Self::Structured(StructuredResource::invalid(invalid)),
        })
    }
}

impl Resource {
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

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct StructuredResource {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub salience: Option<f64>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl StructuredResource {
    fn from_mapping(mut mapping: Mapping) -> Self {
        let mut extra = BTreeMap::new();
        let name = take_typed(&mut mapping, "name", &mut extra).unwrap_or_default();
        let kind = take_typed(&mut mapping, "kind", &mut extra);
        let salience = take_typed(&mut mapping, "salience", &mut extra);
        extend_extra(&mut extra, mapping);
        Self {
            name,
            kind,
            salience,
            extra,
        }
    }

    fn invalid(value: Value) -> Self {
        let mut extra = BTreeMap::new();
        extra.insert("_invalid_value".into(), value);
        Self {
            extra,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct HookFrontmatter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherits: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recall_queries: Vec<RecallQuery>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<Entity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resources: Vec<Resource>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sensitivity: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub backends: BackendMap,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl HookFrontmatter {
    pub fn inherits(&self) -> bool {
        self.inherits.unwrap_or(true)
    }
}

fn take_typed<T: DeserializeOwned>(
    mapping: &mut Mapping,
    key: &str,
    extra: &mut BTreeMap<String, Value>,
) -> Option<T> {
    let yaml_key = Value::String(key.to_string());
    let value = mapping.remove(&yaml_key)?;
    match serde_yaml_ng::from_value::<T>(value.clone()) {
        Ok(value) => Some(value),
        Err(_) => {
            extra.insert(key.to_string(), value);
            None
        }
    }
}

fn extend_extra(extra: &mut BTreeMap<String, Value>, mapping: Mapping) {
    for (key, value) in mapping {
        let key = match key {
            Value::String(key) => key,
            other => format!("<non-string-key:{other:?}>"),
        };
        extra.insert(key, value);
    }
}
