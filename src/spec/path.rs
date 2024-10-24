use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use utoipa::openapi::{self};

use super::{nullable_or_type, Extensions, RefOr, Responses, Schema};

#[derive(Debug, thiserror::Error)]
#[error("invalid path parameter type")]
pub struct InvalidPathParameter;

/// https://swagger.io/specification/v2/#paths-object
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Paths {
    #[serde(flatten)]
    pub paths: BTreeMap<String, PathItem>,
    #[serde(
        flatten,
        skip_serializing_if = "HashMap::is_empty",
        default = "HashMap::new"
    )]
    pub extensions: Extensions,
}

impl From<Paths> for openapi::Paths {
    fn from(value: Paths) -> Self {
        let mut openapi_paths = openapi::PathsBuilder::new()
            .extensions(value.extensions.into_openapi_extensions())
            .build();
        openapi_paths.paths = value
            .paths
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect();
        openapi_paths
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct PathItem {
    pub get: Option<Operation>,
    pub put: Option<Operation>,
    pub post: Option<Operation>,
    pub delete: Option<Operation>,
    pub options: Option<Operation>,
    pub head: Option<Operation>,
    pub patch: Option<Operation>,
    pub trace: Option<Operation>,
    pub parameters: Option<Vec<Parameter>>,
}

impl From<PathItem> for openapi::PathItem {
    fn from(value: PathItem) -> Self {
        let openapi_params: Option<Vec<openapi::path::Parameter>> = value
            .parameters
            .map(|p| p.into_iter().filter_map(|p| p.try_into().ok()).collect());
        let mut openapi_path_item = openapi::path::PathItemBuilder::new()
            .parameters(openapi_params)
            .build();

        openapi_path_item.get = value.get.map(Into::into);
        openapi_path_item.put = value.put.map(Into::into);
        openapi_path_item.post = value.post.map(Into::into);
        openapi_path_item.delete = value.delete.map(Into::into);
        openapi_path_item.options = value.options.map(Into::into);
        openapi_path_item.head = value.head.map(Into::into);
        openapi_path_item.patch = value.patch.map(Into::into);
        openapi_path_item.trace = value.trace.map(Into::into);

        openapi_path_item
    }
}

/// https://swagger.io/specification/v2/#operation-object
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub tags: Option<Vec<String>>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub external_docs: Option<openapi::ExternalDocs>,
    pub operation_id: Option<String>,
    pub consumes: Option<Vec<String>>,
    pub produces: Option<Vec<String>>,
    pub parameters: Option<Vec<Parameter>>,
    pub responses: Responses,
    pub schemes: Option<Vec<String>>,
    pub deprecated: Option<openapi::Deprecated>,
    pub security: Option<Vec<openapi::SecurityRequirement>>,
    #[serde(
        flatten,
        skip_serializing_if = "HashMap::is_empty",
        default = "HashMap::new"
    )]
    pub extensions: Extensions,
}

impl From<Operation> for openapi::path::Operation {
    fn from(value: Operation) -> Self {
        let mut openapi_operation = openapi::path::OperationBuilder::new()
            .tags(value.tags)
            .summary(value.summary)
            .description(value.description)
            .operation_id(value.operation_id)
            .deprecated(value.deprecated)
            .responses(value.responses)
            .extensions(value.extensions.into_openapi_extensions())
            .build();

        openapi_operation.security = value.security;

        if let Some(params) = value.parameters {
            let mut openapi_params: Vec<openapi::path::Parameter> = Vec::with_capacity(10);
            for param in params {
                match param.parameter_in {
                    ParameterIn::FormData(form_body) => {
                        let openapi_content = openapi::content::Content::new(Some(
                            openapi::RefOr::T(openapi::Schema::from(form_body)),
                        ));
                        let openapi_req_body = openapi::request_body::RequestBodyBuilder::new()
                            .description(param.description)
                            .required(Some(is_required(param.required)))
                            .content("application/x-www-form-urlencoded", openapi_content)
                            .build();

                        openapi_operation.request_body = Some(openapi_req_body);
                    }
                    ParameterIn::Body(body) => {
                        let openapi_content =
                            openapi::content::Content::new(Some(body.schema.into_openapi_ref()));
                        let openapi_req_body = openapi::request_body::RequestBodyBuilder::new()
                            .description(param.description)
                            .required(Some(is_required(param.required)))
                            .content("application/json", openapi_content)
                            .build();

                        openapi_operation.request_body = Some(openapi_req_body);
                    }
                    _ => {
                        if let Ok(param) = param.try_into() {
                            openapi_params.push(param);
                        }
                    }
                }
            }

            openapi_operation.parameters = Some(openapi_params);
        }

        openapi_operation
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(flatten)]
    pub parameter_in: ParameterIn,
    #[serde(
        flatten,
        skip_serializing_if = "HashMap::is_empty",
        default = "HashMap::new"
    )]
    pub extensions: Extensions,
}

impl TryFrom<Parameter> for openapi::path::Parameter {
    type Error = InvalidPathParameter;

    fn try_from(value: Parameter) -> Result<Self, Self::Error> {
        let (openapi_param_in, openapi_schema) = match value.parameter_in {
            ParameterIn::Query(query) => (
                openapi::path::ParameterIn::Query,
                openapi::Schema::from(query),
            ),
            ParameterIn::Header(header) => (
                openapi::path::ParameterIn::Header,
                openapi::Schema::from(header),
            ),
            ParameterIn::Path(path) => (
                openapi::path::ParameterIn::Path,
                openapi::Schema::from(path),
            ),
            ParameterIn::FormData(_) | ParameterIn::Body(_) => return Err(InvalidPathParameter),
        };

        Ok(openapi::path::ParameterBuilder::new()
            .name(value.name)
            .description(value.description)
            .schema(Some(openapi_schema))
            .parameter_in(openapi_param_in)
            .required(is_required(value.required))
            .extensions(value.extensions.into_openapi_extensions())
            .build())
    }
}

fn is_required(required: bool) -> openapi::Required {
    if required {
        openapi::Required::True
    } else {
        openapi::Required::False
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(tag = "in", rename_all = "camelCase")]
pub enum ParameterIn {
    Query(ParameterGeneric),
    Header(ParameterGeneric),
    Path(ParameterGeneric),
    FormData(ParameterGeneric),
    Body(ParameterBody),
}

/// https://swagger.io/specification/v2/#parameter-object
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct ParameterGeneric {
    #[serde(rename = "type")]
    pub schema_type: openapi::Type,
    pub format: Option<openapi::SchemaFormat>,
    pub items: Option<Box<ParameterGeneric>>,
    pub allow_empty_value: Option<bool>,
    pub collection_format: Option<String>,
    pub default: Option<serde_json::Value>,

    pub maximum: Option<f64>,
    pub exclusive_maximum: Option<bool>,
    pub minimum: Option<f64>,
    pub exclusive_minimum: Option<bool>,
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
    pub pattern: Option<String>,
    pub max_items: Option<usize>,
    pub min_items: Option<usize>,
    pub unique_items: Option<bool>,
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<serde_json::Value>>,
    pub multiple_of: Option<f64>,
    #[serde(
        flatten,
        skip_serializing_if = "HashMap::is_empty",
        default = "HashMap::new"
    )]
    pub extensions: Extensions,
}

impl From<ParameterGeneric> for openapi::Schema {
    fn from(value: ParameterGeneric) -> Self {
        match value.schema_type {
            openapi::Type::Array => {
                let openapi_array = openapi::ArrayBuilder::new()
                    //.title(value.title)
                    .schema_type(nullable_or_type(
                        value.extensions.nullable(),
                        value.schema_type,
                    ))
                    .items(openapi::RefOr::T(openapi::Schema::from(
                        *value.items.unwrap(),
                    )))
                    //.description(value.description)
                    .default(value.default)
                    //.example(value.example)
                    //.xml(value.xml)
                    .max_items(value.max_items)
                    .min_items(value.min_items)
                    //.unique_items(value.unique_items)
                    //.extensions(value.extensions.into_openapi_extensions())
                    .build();

                Self::Array(openapi_array)
            }
            _ => {
                let openapi_object = openapi::ObjectBuilder::new()
                    .schema_type(nullable_or_type(
                        value.extensions.nullable(),
                        value.schema_type,
                    ))
                    //.title(value.title)
                    .format(value.format)
                    //.description(value.description)
                    .default(value.default)
                    .enum_values(value.enum_values)
                    //.example(value.example)
                    //.read_only(value.read_only)
                    //.xml(value.xml)
                    .multiple_of(value.multiple_of)
                    .maximum(value.maximum)
                    .minimum(value.minimum)
                    //.exclusive_maximum(value.exclusive_maximum)
                    //.exclusive_minimum(value.exclusive_minimum)
                    .max_length(value.max_length)
                    .min_length(value.min_length)
                    .pattern(value.pattern)
                    //.max_properties(value.max_properties)
                    //.min_properties(value.min_properties)
                    //.extensions(value.extensions.into_openapi_extensions())
                    .build();

                Self::Object(openapi_object)
            }
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct ParameterBody {
    pub schema: RefOr<Schema>,
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use assert_json_diff::assert_json_eq;
    use serde_json::json;

    use crate::include_json;
    use testdir::testdir;

    use super::*;

    #[test]
    fn deserialize_paths() {
        let paths = include_json!("../../tests/data/swagger.json", "/paths").to_string();
        let paths: Paths = serde_json::from_str(&paths).unwrap();

        let s = serde_json::to_string_pretty(&paths).unwrap();
        let dir: PathBuf = testdir!();
        fs::write(dir.join("paths.json"), s).unwrap();

        // then
        let path_raw = include_json!("../../tests/data/paths.json");
        assert_json_eq!(serde_json::to_value(path_raw).unwrap(), paths);
    }

    #[test]
    fn into_openapi_paths() {
        let paths = include_json!("../../tests/data/swagger.json", "/paths").to_string();
        let openapi_paths_raw = include_json!("../../tests/data/openapi.json", "/paths");

        let paths: Paths = serde_json::from_str(&paths).unwrap();
        let openapi_paths: openapi::Paths = paths.into();

        fs::write(
            "paths.json",
            serde_json::to_string_pretty(&openapi_paths).unwrap(),
        )
        .unwrap();

        assert_json_eq!(
            json!({"paths": openapi_paths_raw}),
            serde_json::to_value(openapi_paths).unwrap(),
        );
    }
}
