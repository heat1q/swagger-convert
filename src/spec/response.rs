use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use utoipa::openapi::{self};

use super::{Extensions, ParameterGeneric, RefOr, Schema};

/// https://swagger.io/specification/v2/#responses-object
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Responses {
    #[serde(flatten)]
    pub responses: BTreeMap<String, RefOr<Response>>,
    pub default: Option<RefOr<Response>>,
    #[serde(flatten)]
    pub extensions: Option<Extensions>,
}

impl From<Responses> for openapi::Responses {
    fn from(value: Responses) -> Self {
        let resp_iter = value
            .responses
            .into_iter()
            .map(|(k, v)| (k, v.into_openapi_ref()));
        openapi::ResponsesBuilder::new()
            .responses_from_iter(resp_iter)
            .build()
    }
}

/// https://swagger.io/specification/v2/#response-object
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub description: String,
    pub schema: Option<RefOr<Schema>>,
    pub headers: Option<BTreeMap<String, ParameterHeader>>,
    pub examples: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(flatten)]
    pub extensions: Option<Extensions>,
}

impl From<Response> for openapi::Response {
    fn from(value: Response) -> Self {
        let mut content = openapi::Content::default();
        if let Some(schema) = value.schema {
            content.schema = Some(schema.into_openapi_ref());
        }

        if let Some(examples) = value.examples {
            content.examples = examples
                .into_iter()
                .map(|(k, v)| {
                    let mut example = openapi::example::Example::default();
                    example.value = Some(v);
                    (k, openapi::RefOr::T(example))
                })
                .collect();
        }

        let mut response = openapi::ResponseBuilder::new()
            .description(value.description)
            .content("application/json", content) // swagger only supports json
            .extensions(value.extensions.map(Into::into))
            .build();

        response.headers = value
            .headers
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect();

        response
    }
}

/// https://swagger.io/specification/v2/#header-object
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct ParameterHeader {
    pub description: String,
    #[serde(flatten)]
    pub parameter: ParameterGeneric,
}

impl From<ParameterHeader> for openapi::header::Header {
    fn from(value: ParameterHeader) -> Self {
        let mut header = openapi::header::Header::default();
        header.description = Some(value.description);
        header.schema = openapi::RefOr::T(value.parameter.into());
        header
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use assert_json_diff::assert_json_eq;

    use crate::include_json;

    use super::*;

    #[test]
    fn deserialize_responses() {
        let responses =
            include_json!("../../tests/data/petstore_swagger.json", "/responses").to_string();
        let _responses: Responses = serde_json::from_str(&responses).unwrap();
    }

    #[test]
    fn into_openapi_responses() {
        let responses_raw =
            include_json!("../../tests/data/petstore_swagger.json", "/responses").to_string();
        let responses_openapi_raw = include_json!(
            "../../tests/data/petstore_openapi.json",
            "/components/responses"
        );

        let responses: Responses = serde_json::from_str(&responses_raw).unwrap();
        let openapi_responses: openapi::Responses = responses.into();

        let s = serde_json::to_string_pretty(&openapi_responses).unwrap();
        fs::write("responses.json", s).unwrap();

        assert_json_eq!(
            responses_openapi_raw,
            serde_json::to_value(openapi_responses).unwrap()
        );
    }
}
