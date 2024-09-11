use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
};
use utoipa::openapi::{self, schema::SchemaType, OpenApiBuilder, Type};

mod definition;
mod path;
mod response;
mod security;
mod server;

pub use definition::*;
pub use path::*;
pub use response::*;
pub use security::*;
pub use server::*;
pub use utoipa::openapi::Info;

#[derive(Default, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Extensions(pub HashMap<String, serde_json::Value>);

impl Extensions {
    pub fn nullable(&self) -> bool {
        self.0
            .get("x-nullable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn into_openapi_extensions(mut self) -> Option<utoipa::openapi::extensions::Extensions> {
        if self.0.is_empty() {
            return None;
        }
        self.0.remove("x-nullable");
        Some(self.into())
    }
}

impl Deref for Extensions {
    type Target = HashMap<String, serde_json::Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Extensions> for HashMap<String, serde_json::Value> {
    fn from(value: Extensions) -> Self {
        value.0
    }
}

impl serde::ser::Serialize for Extensions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::ser::Serialize::serialize(&self.0, serializer)
    }
}

impl<'de> serde::de::Deserialize<'de> for Extensions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map: HashMap<String, serde_json::Value> = HashMap::deserialize(deserializer)?;
        let map = map
            .into_iter()
            .filter(|(k, _)| k.starts_with("x-"))
            .collect();
        Ok(Self(map))
    }
}

impl From<Extensions> for utoipa::openapi::extensions::Extensions {
    fn from(value: Extensions) -> Self {
        let mut builder = openapi::extensions::ExtensionsBuilder::new();
        for (key, value) in value.0 {
            builder = builder.add(key, value);
        }
        builder.build()
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(untagged)]
pub enum RefOr<T> {
    Ref(openapi::Ref),
    T(T),
}

impl<T> RefOr<T> {
    fn into_openapi_ref<V: From<T>>(self) -> openapi::RefOr<V> {
        match self {
            RefOr::T(v) => openapi::RefOr::T(v.into()),
            RefOr::Ref(openapi::Ref { ref_location, .. }) => {
                let ref_location = openapi::Ref::new(Self::openapi_ref_location(&ref_location));
                openapi::RefOr::Ref(ref_location)
            }
        }
    }

    #[allow(dead_code)]
    fn try_into_openapi_ref<V: TryFrom<T>>(self) -> Result<openapi::RefOr<V>, V::Error> {
        match self {
            RefOr::T(v) => Ok(openapi::RefOr::T(v.try_into()?)),
            RefOr::Ref(openapi::Ref { ref_location, .. }) => {
                let ref_location = openapi::Ref::new(Self::openapi_ref_location(&ref_location));
                Ok(openapi::RefOr::Ref(ref_location))
            }
        }
    }

    fn openapi_ref_location(ref_location: &str) -> String {
        let prefix = ["#", "components"].into_iter();
        let ref_location = ref_location
            .split('/')
            .skip(1)
            .map(|element| match element {
                "definitions" => "schemas",
                _ => element,
            });
        Itertools::intersperse(prefix.chain(ref_location), "/").collect()
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(untagged)]
pub enum AdditionalProperties<T> {
    RefOr(RefOr<T>),
    FreeForm(bool),
    Any(BTreeMap<String, serde_json::Value>),
}

impl<T> AdditionalProperties<T> {
    fn into_openapi_additional_properties<V: From<T> + Default>(
        self,
    ) -> openapi::schema::AdditionalProperties<V> {
        use openapi::schema;
        match self {
            Self::RefOr(r) => schema::AdditionalProperties::RefOr(r.into_openapi_ref()),
            Self::FreeForm(f) => schema::AdditionalProperties::FreeForm(f),
            // discard any other invalid properties
            Self::Any(_) => schema::AdditionalProperties::RefOr(openapi::RefOr::T(V::default())),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum SwaggerVersion {
    #[serde(rename = "2.0")]
    Version2,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Swagger {
    pub swagger: SwaggerVersion,
    pub info: Info,
    pub host: Option<String>,
    pub base_path: Option<String>,
    pub schemes: Option<Vec<ProtocolSchemes>>,
    pub consumes: Option<Vec<String>>,
    pub produces: Option<Vec<String>>,
    pub paths: Paths,
    pub definitions: Option<Definitions>,
    pub responses: Option<Responses>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub security_definitions: BTreeMap<String, SecurityScheme>,
    pub security: Option<Vec<openapi::SecurityRequirement>>,
    pub tags: Option<Vec<openapi::Tag>>,
    pub external_docs: Option<openapi::ExternalDocs>,
}

impl From<Swagger> for openapi::OpenApi {
    fn from(swagger: Swagger) -> Self {
        let responses: openapi::Responses = if swagger.responses.is_some() {
            swagger.responses.unwrap().into()
        } else {
            openapi::Responses::new()
        };

        let mut components = openapi::Components::new();
        components.schemas = if swagger.definitions.is_some() {
            swagger.definitions.unwrap().into()
        } else {
            BTreeMap::new()
        };
        components.responses = responses.responses;
        components.security_schemes = swagger
            .security_definitions
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect();
        let servers =
            server::openapi_servers_from_host(swagger.schemes, swagger.host, swagger.base_path);

        OpenApiBuilder::new()
            .info(swagger.info)
            .paths(swagger.paths)
            .servers(servers)
            .components(Some(components))
            .security(swagger.security)
            .external_docs(swagger.external_docs)
            .build()
    }
}

pub(crate) fn nullable_or_type(is_nullable: bool, schema_type: Type) -> SchemaType {
    if is_nullable {
        SchemaType::Array(vec![schema_type, Type::Null])
    } else {
        SchemaType::Type(schema_type)
    }
}
