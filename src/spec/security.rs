use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use utoipa::openapi;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum SecurityScheme {
    Oauth2(Oauth2),
}

impl From<SecurityScheme> for openapi::security::SecurityScheme {
    fn from(value: SecurityScheme) -> Self {
        match value {
            SecurityScheme::Oauth2(oauth) => Self::OAuth2(oauth.into()),
            //_ => unimplemented!(),
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Oauth2 {
    pub description: Option<String>,
    #[serde(flatten)]
    pub flow: Flow,
    pub scopes: Option<BTreeMap<String, String>>,
}

impl From<Oauth2> for openapi::security::OAuth2 {
    fn from(value: Oauth2) -> Self {
        let mut oauth2 = Self::new([value.flow.into_openapi_flow(value.scopes)]);
        oauth2.description = value.description;
        oauth2
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase", tag = "flow")]
pub enum Flow {
    #[serde(rename_all = "camelCase")]
    Implicit { authorization_url: String },
    #[serde(rename_all = "camelCase")]
    Password { token_url: String },
    #[serde(rename_all = "camelCase")]
    Application { token_url: String },
    #[serde(rename_all = "camelCase")]
    AccessCode {
        authorization_url: String,
        token_url: String,
    },
}

impl Flow {
    fn into_openapi_flow(
        self,
        scopes: Option<BTreeMap<String, String>>,
    ) -> openapi::security::Flow {
        use openapi::security::Flow as OpenApiFlow;
        use openapi::security::Scopes as OpenApiScopes;
        let scopes: OpenApiScopes = scopes.unwrap_or_default().into_iter().collect();
        match self {
            Flow::Implicit { authorization_url } => {
                OpenApiFlow::Implicit(openapi::security::Implicit::new(authorization_url, scopes))
            }
            Flow::Password { token_url } => {
                OpenApiFlow::Password(openapi::security::Password::new(token_url, scopes))
            }
            Flow::Application { token_url } => OpenApiFlow::ClientCredentials(
                openapi::security::ClientCredentials::new(token_url, scopes),
            ),
            Flow::AccessCode {
                authorization_url,
                token_url,
            } => OpenApiFlow::AuthorizationCode(openapi::security::AuthorizationCode::new(
                authorization_url,
                token_url,
                scopes,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::include_json;

    use super::*;

    #[test]
    fn serde_security() {
        let security_raw = include_json!("../../tests/swagger.json", "/securityDefinitions");

        let de: BTreeMap<String, SecurityScheme> =
            serde_json::from_str(&security_raw.clone().to_string()).unwrap();
        let de = serde_json::to_value(de).unwrap();

        assert_eq!(de, security_raw);
    }

    #[test]
    fn into_openapi_security() {
        let security_raw =
            include_json!("../../tests/swagger.json", "/securityDefinitions").to_string();
        let security_openapi_raw =
            include_json!("../../tests/openapi.json", "/components/securitySchemes");

        let security: BTreeMap<String, SecurityScheme> =
            serde_json::from_str(&security_raw).unwrap();
        let openapi_security: BTreeMap<String, openapi::security::SecurityScheme> =
            security.into_iter().map(|(k, v)| (k, v.into())).collect();

        assert_eq!(
            security_openapi_raw,
            serde_json::to_value(openapi_security).unwrap()
        );
    }
}
