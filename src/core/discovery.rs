use futures_util::stream::{self, StreamExt};
use localsend::http::state::ClientInfo;
use localsend::model::discovery::DeviceType;
use serde::Deserialize;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct DiscoveredDevice {
    pub info: ClientInfo,
    pub ip: String,
    pub port: u16,
    pub https: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InfoDto {
    alias: String,
    version: Option<String>,
    device_model: Option<String>,
    device_type: Option<String>,
    fingerprint: Option<String>,
}

pub async fn scan_local_network(
    port: u16,
    timeout: Duration,
    self_fingerprint: Option<String>,
) -> Vec<DiscoveredDevice> {
    let Some(base_prefix) = local_subnet_prefix().await else {
        return Vec::new();
    };

    let client = match reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(timeout)
        .build()
    {
        Ok(c) => c,
        Err(err) => {
            log::error!("failed to create discovery client: {}", err);
            return Vec::new();
        }
    };

    let candidates: Vec<String> = (1..=254).map(|i| format!("{}.{}", base_prefix, i)).collect();
    let self_fingerprint = self_fingerprint.unwrap_or_default();

    stream::iter(candidates)
        .map(|ip| {
            let client = client.clone();
            let self_fingerprint = self_fingerprint.clone();
            async move {
                let protocol_candidates = [false, true];
                for https in protocol_candidates {
                    let url = format!(
                        "{}://{}:{}/api/localsend/v2/info",
                        if https { "https" } else { "http" },
                        ip,
                        port
                    );
                    match client
                        .get(&url)
                        .query(&[("fingerprint", self_fingerprint.as_str())])
                        .send()
                        .await
                    {
                        Ok(res) if res.status().is_success() => {
                            let dto = match res.json::<InfoDto>().await {
                                Ok(dto) => dto,
                                Err(err) => {
                                    log::debug!("parse info failed for {}: {}", url, err);
                                    continue;
                                }
                            };

                            let fingerprint = dto.fingerprint.unwrap_or_default();
                            if !self_fingerprint.is_empty() && fingerprint == self_fingerprint {
                                return None;
                            }

                            let info = ClientInfo {
                                alias: dto.alias,
                                version: dto.version.unwrap_or_else(|| "2.1".to_string()),
                                device_model: dto.device_model,
                                device_type: map_device_type(dto.device_type.as_deref()),
                                token: fingerprint,
                            };
                            return Some(DiscoveredDevice {
                                info,
                                ip,
                                port,
                                https,
                            });
                        }
                        _ => continue,
                    }
                }
                None
            }
        })
        .buffer_unordered(64)
        .filter_map(async move |x| x)
        .collect::<Vec<_>>()
        .await
}

async fn local_subnet_prefix() -> Option<String> {
    // Do not rely on internet reachability. Try LAN/multicast route first.
    let probes = [("224.0.0.167", 53317), ("1.1.1.1", 80), ("8.8.8.8", 80)];
    for (host, port) in probes {
        let socket = match tokio::net::UdpSocket::bind("0.0.0.0:0").await {
            Ok(s) => s,
            Err(_) => continue,
        };
        if socket.connect((host, port)).await.is_err() {
            continue;
        }
        let local = match socket.local_addr() {
            Ok(addr) => addr,
            Err(_) => continue,
        };
        if let std::net::IpAddr::V4(ipv4) = local.ip() {
            let octets = ipv4.octets();
            return Some(format!("{}.{}.{}", octets[0], octets[1], octets[2]));
        }
    }
    None
}

fn map_device_type(value: Option<&str>) -> Option<DeviceType> {
    match value {
        Some("mobile") => Some(DeviceType::Mobile),
        Some("desktop") => Some(DeviceType::Desktop),
        Some("web") => Some(DeviceType::Web),
        Some("headless") => Some(DeviceType::Headless),
        Some("server") => Some(DeviceType::Server),
        Some("MOBILE") => Some(DeviceType::Mobile),
        Some("DESKTOP") => Some(DeviceType::Desktop),
        Some("WEB") => Some(DeviceType::Web),
        Some("HEADLESS") => Some(DeviceType::Headless),
        Some("SERVER") => Some(DeviceType::Server),
        _ => None,
    }
}
