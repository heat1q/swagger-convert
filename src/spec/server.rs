use serde::{Deserialize, Serialize};
use utoipa::openapi;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub enum ProtocolSchemes {
    Http,
    Https,
    Ws,
    Wss,
}

pub(crate) fn openapi_servers_from_host(
    schemes: Option<Vec<ProtocolSchemes>>,
    host: Option<String>,
    base_path: Option<String>,
) -> Option<Vec<openapi::Server>> {
    let host = host?;
    let servers = schemes?
        .into_iter()
        .map(|s| {
            let prefix = match s {
                ProtocolSchemes::Http => "http",
                ProtocolSchemes::Https => "https",
                ProtocolSchemes::Ws => "ws",
                ProtocolSchemes::Wss => "wss",
            };
            let base_path = base_path.as_deref().unwrap_or("/");
            let url = format!("{prefix}://{host}{base_path}");
            openapi::Server::new(url)
        })
        .collect();
    Some(servers)
}
