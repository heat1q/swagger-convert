use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use utoipa::openapi::{self};

use super::{AdditionalProperties, Extensions, RefOr};

/// https://swagger.io/specification/v2/#definitions-object
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Definitions {
    #[serde(flatten)]
    pub defintions: BTreeMap<String, RefOr<Schema>>,
}

impl From<Definitions> for BTreeMap<String, openapi::RefOr<openapi::Schema>> {
    fn from(value: Definitions) -> Self {
        value
            .defintions
            .into_iter()
            .map(|(k, v)| (k, v.into_openapi_ref()))
            .collect()
    }
}

/// https://swagger.io/specification/v2/#schema-object
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(untagged, rename_all = "camelCase")]
pub enum Schema {
    Array(Array),
    Object(Object),
    AllOf(AllOf),
}

impl From<Schema> for openapi::Schema {
    fn from(value: Schema) -> Self {
        match value {
            Schema::Array(array) => {
                let openapi_array = openapi::ArrayBuilder::new()
                    .title(array.title)
                    .items(array.items.into_openapi_ref())
                    .description(array.description)
                    .default(array.default)
                    .example(array.example)
                    .xml(array.xml)
                    .max_items(array.max_items)
                    .min_items(array.min_items)
                    .unique_items(array.unique_items)
                    .nullable(array.extensions.nullable())
                    .extensions(array.extensions.into_openapi_extensions())
                    .build();

                Self::Array(openapi_array)
            }
            Schema::Object(object) => {
                let mut openapi_object = openapi::ObjectBuilder::new()
                    .schema_type(object.schema_type)
                    .title(object.title)
                    .format(object.format)
                    .description(object.description)
                    .default(object.default)
                    .enum_values(object.enum_values)
                    .example(object.example)
                    .read_only(object.read_only)
                    .xml(object.xml)
                    .nullable(object.extensions.nullable())
                    .multiple_of(object.multiple_of)
                    .maximum(object.maximum)
                    .minimum(object.minimum)
                    .exclusive_maximum(object.exclusive_maximum)
                    .exclusive_minimum(object.exclusive_minimum)
                    .max_length(object.max_length)
                    .min_length(object.min_length)
                    .pattern(object.pattern)
                    .max_properties(object.max_properties)
                    .min_properties(object.min_properties)
                    .extensions(object.extensions.into_openapi_extensions())
                    .build();

                openapi_object.required = object.required;
                openapi_object.properties = object
                    .properties
                    .into_iter()
                    .map(|(k, v)| (k, v.into_openapi_ref()))
                    .collect();
                openapi_object.additional_properties = object
                    .additional_properties
                    .map(|p| Box::new(p.into_openapi_additional_properties()));

                Self::Object(openapi_object)
            }
            Schema::AllOf(all_of) => {
                let mut openapi_all_of = openapi::AllOfBuilder::new()
                    .title(all_of.title)
                    .description(all_of.description)
                    .default(all_of.default)
                    .example(all_of.example)
                    .discriminator(all_of.discriminator.map(openapi::Discriminator::new))
                    .nullable(all_of.extensions.nullable())
                    .extensions(all_of.extensions.into_openapi_extensions())
                    .build();

                openapi_all_of.items = all_of
                    .items
                    .into_iter()
                    .map(|i| i.into_openapi_ref())
                    .collect();

                Self::AllOf(openapi_all_of)
            }
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Array {
    #[serde(rename = "type")]
    pub schema_type: openapi::SchemaType,
    pub title: Option<String>,
    pub items: Box<RefOr<Schema>>,
    pub description: Option<String>,
    pub example: Option<serde_json::Value>,
    pub default: Option<serde_json::Value>,
    pub max_items: Option<usize>,
    pub min_items: Option<usize>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub unique_items: bool,
    pub xml: Option<openapi::xml::Xml>,

    #[serde(
        flatten,
        skip_serializing_if = "HashMap::is_empty",
        default = "HashMap::new"
    )]
    pub extensions: Extensions,
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Object {
    pub format: Option<openapi::SchemaFormat>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub default: Option<serde_json::Value>,

    pub multiple_of: Option<f64>,
    pub maximum: Option<f64>,
    pub exclusive_maximum: Option<f64>,
    pub minimum: Option<f64>,
    pub exclusive_minimum: Option<f64>,
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
    pub pattern: Option<String>,
    pub max_properties: Option<usize>,
    pub min_properties: Option<usize>,

    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub required: Vec<String>,
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<serde_json::Value>>,
    #[serde(rename = "type")]
    pub schema_type: openapi::SchemaType,

    #[serde(skip_serializing_if = "BTreeMap::is_empty", default = "BTreeMap::new")]
    pub properties: BTreeMap<String, RefOr<Schema>>,
    pub additional_properties: Option<Box<AdditionalProperties<Schema>>>,

    pub read_only: Option<bool>,
    pub xml: Option<openapi::xml::Xml>,
    pub example: Option<serde_json::Value>,

    #[serde(
        flatten,
        skip_serializing_if = "HashMap::is_empty",
        default = "HashMap::new"
    )]
    pub extensions: Extensions,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct AllOf {
    pub items: Vec<RefOr<Schema>>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub default: Option<serde_json::Value>,
    pub example: Option<serde_json::Value>,
    pub discriminator: Option<String>,

    #[serde(
        flatten,
        skip_serializing_if = "HashMap::is_empty",
        default = "HashMap::new"
    )]
    pub extensions: Extensions,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use assert_json_diff::assert_json_eq;
    use serde_json::Value;

    use crate::include_json;

    use super::*;

    #[test]
    fn deserialize_definition() {
        let swagger: Value =
            serde_json::from_str(include_str!("../../tests/swagger.json")).unwrap();
        let definitions = swagger.get("definitions").unwrap().to_string();
        let definitions: Definitions = serde_json::from_str(&definitions).unwrap();

        let s = serde_json::to_string_pretty(&definitions).unwrap();
        fs::write("definitions.json", s).unwrap();
    }

    #[test]
    fn into_openapi_schemas() {
        let definitions = include_json!("../../tests/swagger.json", "/definitions").to_string();
        let schemas = include_json!("../../tests/openapi.json", "/components/schemas");

        let definitions: Definitions = serde_json::from_str(&definitions).unwrap();
        let openapi_schemas: BTreeMap<String, openapi::RefOr<openapi::Schema>> = definitions.into();

        let s = serde_json::to_string_pretty(&openapi_schemas).unwrap();
        fs::write("schemas.json", s).unwrap();

        assert_json_eq!(schemas, serde_json::to_value(openapi_schemas).unwrap());
    }
}
