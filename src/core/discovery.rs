use futures_util::stream::{self, StreamExt};
use localsend::http::state::ClientInfo;
use localsend::model::discovery::DeviceType;
use serde::Deserialize;
use std::collections::{BTreeSet, HashMap};
use std::net::Ipv4Addr;
use std::sync::{OnceLock, RwLock};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct DiscoveredDevice {
    pub info: ClientInfo,
    pub ip: String,
    pub port: u16,
    pub https: bool,
}

fn passive_discovery_map() -> &'static RwLock<HashMap<String, DiscoveredDevice>> {
    static PASSIVE: OnceLock<RwLock<HashMap<String, DiscoveredDevice>>> = OnceLock::new();
    PASSIVE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn device_key(info: &ClientInfo, ip: &str, port: u16) -> String {
    if info.token.is_empty() {
        format!("{}:{}", ip, port)
    } else {
        info.token.clone()
    }
}

pub fn register_passive_device(device: DiscoveredDevice) {
    let key = device_key(&device.info, &device.ip, device.port);
    if let Ok(mut map) = passive_discovery_map().write() {
        map.insert(key, device);
    }
}

pub fn list_passive_devices(self_fingerprint: Option<&str>) -> Vec<DiscoveredDevice> {
    let mut out = Vec::new();
    let own = self_fingerprint.unwrap_or_default();
    if let Ok(map) = passive_discovery_map().read() {
        for device in map.values() {
            if !own.is_empty() && device.info.token == own {
                continue;
            }
            out.push(device.clone());
        }
    }
    out
}

pub fn clear_passive_devices() {
    if let Ok(mut map) = passive_discovery_map().write() {
        map.clear();
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InfoDto {
    alias: String,
    version: Option<String>,
    device_model: Option<String>,
    device_type: Option<String>,
    fingerprint: Option<String>,
    token: Option<String>,
}

const MAX_AUTO_SCAN_PREFIX_COUNT: usize = 3;
const MAX_CUSTOM_SCAN_CANDIDATES: usize = 4096;
const MIN_CUSTOM_CIDR_PREFIX: u8 = 24;

#[derive(Clone, Copy, Debug)]
enum DiscoveryTargetRule {
    Single(Ipv4Addr),
    Cidr { network: Ipv4Addr, prefix: u8 },
}

pub fn is_discovery_target_rule_valid(rule: &str) -> bool {
    parse_discovery_target_rule(rule).is_some()
}

fn parse_discovery_target_rule(rule: &str) -> Option<DiscoveryTargetRule> {
    let trimmed = rule.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some((ip_part, prefix_part)) = trimmed.split_once('/') {
        let ip = ip_part.trim().parse::<Ipv4Addr>().ok()?;
        let prefix = prefix_part.trim().parse::<u8>().ok()?;
        if prefix > 32 {
            return None;
        }
        if prefix == 32 {
            return Some(DiscoveryTargetRule::Single(ip));
        }
        if prefix < MIN_CUSTOM_CIDR_PREFIX {
            return None;
        }
        let mask = u32::MAX << (32 - prefix as u32);
        let network = Ipv4Addr::from(u32::from(ip) & mask);
        return Some(DiscoveryTargetRule::Cidr { network, prefix });
    }

    let normalized = trimmed.trim_end_matches('.');
    if normalized.contains('*') {
        if normalized.matches('*').count() != 1 || !normalized.ends_with('*') {
            return None;
        }
        let prefix_text = normalized.trim_end_matches('*').trim_end_matches('.');
        let parts = prefix_text
            .split('.')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        if parts.len() != 3 {
            return None;
        }
        let a = parts[0].parse::<u8>().ok()?;
        let b = parts[1].parse::<u8>().ok()?;
        let c = parts[2].parse::<u8>().ok()?;
        return Some(DiscoveryTargetRule::Cidr {
            network: Ipv4Addr::new(a, b, c, 0),
            prefix: 24,
        });
    }

    let parts = normalized
        .split('.')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    match parts.len() {
        4 => Some(DiscoveryTargetRule::Single(
            normalized.parse::<Ipv4Addr>().ok()?,
        )),
        3 => {
            let a = parts[0].parse::<u8>().ok()?;
            let b = parts[1].parse::<u8>().ok()?;
            let c = parts[2].parse::<u8>().ok()?;
            Some(DiscoveryTargetRule::Cidr {
                network: Ipv4Addr::new(a, b, c, 0),
                prefix: 24,
            })
        }
        _ => None,
    }
}

fn collect_custom_discovery_candidates(
    rules: &[String],
    local_ip_set: &BTreeSet<String>,
) -> Vec<String> {
    let mut set = BTreeSet::new();

    for rule in rules {
        let token = rule.trim();
        if token.is_empty() {
            continue;
        }

        let Some(parsed) = parse_discovery_target_rule(token) else {
            log::warn!("skip invalid discovery target rule: {}", token);
            continue;
        };

        match parsed {
            DiscoveryTargetRule::Single(ip) => {
                let ip_text = ip.to_string();
                if !local_ip_set.contains(&ip_text) {
                    set.insert(ip_text);
                }
            }
            DiscoveryTargetRule::Cidr { network, prefix } => {
                let base = u32::from(network);
                let host_count = 1u32 << (32 - prefix as u32);
                for offset in 0..host_count {
                    if set.len() >= MAX_CUSTOM_SCAN_CANDIDATES {
                        log::warn!(
                            "custom discovery candidate limit reached ({}), stop expanding rules",
                            MAX_CUSTOM_SCAN_CANDIDATES
                        );
                        return set.into_iter().collect();
                    }
                    let ip_text = Ipv4Addr::from(base + offset).to_string();
                    if local_ip_set.contains(&ip_text) {
                        continue;
                    }
                    set.insert(ip_text);
                }
            }
        }
    }

    set.into_iter().collect()
}

pub async fn scan_local_network(
    port: u16,
    https: bool,
    timeout: Duration,
    self_fingerprint: Option<String>,
    discovery_target_subnets: Vec<String>,
) -> Vec<DiscoveredDevice> {
    let self_fingerprint = self_fingerprint.unwrap_or_default();
    let mut dedup: HashMap<String, DiscoveredDevice> = HashMap::new();
    for device in list_passive_devices(Some(&self_fingerprint)) {
        let key = device_key(&device.info, &device.ip, device.port);
        dedup.insert(key, device);
    }
    if !dedup.is_empty() {
        log::info!("discovery scan starts with {} passive devices", dedup.len());
    }

    let interface_ips = local_interface_ipv4s().await;
    if interface_ips.is_empty() && discovery_target_subnets.is_empty() {
        return dedup.into_values().collect();
    }

    let client = match reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(timeout)
        .build()
    {
        Ok(c) => c,
        Err(err) => {
            log::error!("failed to create discovery client: {}", err);
            return dedup.into_values().collect();
        }
    };

    let local_ip_set = interface_ips.iter().cloned().collect::<BTreeSet<String>>();
    let mut candidate_set = BTreeSet::new();

    for interface_ip in interface_ips.iter().take(MAX_AUTO_SCAN_PREFIX_COUNT) {
        let Ok(local_v4) = interface_ip.parse::<Ipv4Addr>() else {
            continue;
        };
        let octets = local_v4.octets();
        let base_prefix = format!("{}.{}.{}", octets[0], octets[1], octets[2]);
        for i in 0..=255 {
            let ip = format!("{}.{}", base_prefix, i);
            if local_ip_set.contains(&ip) {
                continue;
            }
            candidate_set.insert(ip);
        }
    }

    if !discovery_target_subnets.is_empty() {
        log::info!(
            "discovery custom target rules: {:?}",
            discovery_target_subnets
        );
        for ip in collect_custom_discovery_candidates(&discovery_target_subnets, &local_ip_set) {
            candidate_set.insert(ip);
        }
    }

    let candidates = candidate_set.into_iter().collect::<Vec<_>>();
    log::info!("discovery scan candidate count: {}", candidates.len());
    if candidates.is_empty() {
        return dedup.into_values().collect();
    }

    let mut out = Vec::new();
    let discovered = stream::iter(candidates.into_iter())
        .map(|ip| {
            let client = client.clone();
            let self_fingerprint = self_fingerprint.clone();
            async move { scan_one_ip(&client, &ip, port, https, &self_fingerprint).await }
        })
        .buffer_unordered(50);

    tokio::pin!(discovered);
    while let Some(item) = discovered.next().await {
        if let Some(device) = item {
            let key = device_key(&device.info, &device.ip, device.port);
            dedup.insert(key, device);
        }
    }
    for device in list_passive_devices(Some(&self_fingerprint)) {
        let key = device_key(&device.info, &device.ip, device.port);
        dedup.insert(key, device);
    }
    out.extend(dedup.into_values());
    out
}

pub async fn probe_device(
    ip: &str,
    port: u16,
    https: bool,
    self_fingerprint: Option<String>,
) -> Option<DiscoveredDevice> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(2))
        .build()
        .ok()?;
    let own = self_fingerprint.unwrap_or_default();
    scan_one_ip(&client, ip, port, https, &own).await
}

async fn scan_one_ip(
    client: &reqwest::Client,
    ip: &str,
    port: u16,
    https: bool,
    self_fingerprint: &str,
) -> Option<DiscoveredDevice> {
    // Match LocalSend targeted discovery:
    // - probe peer as v1 first handshake path
    // - use current local TLS mode for scheme
    let url = format!(
        "{}://{}:{}/api/localsend/v1/info",
        if https { "https" } else { "http" },
        ip,
        port
    );
    let req = client.get(&url).query(&[("fingerprint", self_fingerprint)]);
    let res = match req.send().await {
        Ok(r) => r,
        Err(_) => return None,
    };
    if !res.status().is_success() {
        return None;
    }
    let dto = match res.json::<InfoDto>().await {
        Ok(dto) => dto,
        Err(err) => {
            log::debug!("parse info failed for {}: {}", url, err);
            return None;
        }
    };

    let fingerprint = dto.fingerprint.or(dto.token).unwrap_or_default();
    if !self_fingerprint.is_empty() && fingerprint == self_fingerprint {
        return None;
    }
    let info = ClientInfo {
        alias: dto.alias,
        version: dto.version.unwrap_or_else(|| "1.0".to_string()),
        device_model: dto.device_model,
        device_type: map_device_type(dto.device_type.as_deref()),
        token: fingerprint,
    };
    Some(DiscoveredDevice {
        info,
        ip: ip.to_string(),
        port,
        https,
    })
}

async fn local_interface_ipv4s() -> Vec<String> {
    let mut local_ips = Vec::<Ipv4Addr>::new();

    if let Ok(interfaces) = if_addrs::get_if_addrs() {
        for iface in interfaces {
            if iface.is_loopback() {
                continue;
            }
            if let if_addrs::IfAddr::V4(v4) = iface.addr {
                if v4.ip.is_link_local() {
                    continue;
                }
                local_ips.push(v4.ip);
            }
        }
    }

    let primary = local_ips.first().copied();
    rank_ipv4_addresses(&mut local_ips, primary);
    let mut ranked_ips = Vec::new();
    let mut seen = BTreeSet::new();
    for ip in local_ips {
        let ip_text = ip.to_string();
        if seen.insert(ip_text.clone()) {
            ranked_ips.push(ip_text);
        }
    }
    log::info!("discovery scan interfaces: {:?}", ranked_ips);
    ranked_ips
}

fn rank_ipv4_addresses(list: &mut Vec<Ipv4Addr>, primary: Option<Ipv4Addr>) {
    list.sort_by(|a, b| {
        let score = |ip: &Ipv4Addr| -> i32 {
            if Some(*ip) == primary {
                10
            } else if ip.octets()[3] == 1 {
                0
            } else {
                1
            }
        };
        score(b)
            .cmp(&score(a))
            .then_with(|| a.octets().cmp(&b.octets()))
    });
    list.dedup();
}

fn map_device_type(value: Option<&str>) -> Option<DeviceType> {
    match value {
        Some("mobile") | Some("MOBILE") => Some(DeviceType::Mobile),
        Some("desktop") | Some("DESKTOP") => Some(DeviceType::Desktop),
        Some("web") | Some("WEB") => Some(DeviceType::Web),
        Some("headless") | Some("HEADLESS") => Some(DeviceType::Headless),
        Some("server") | Some("SERVER") => Some(DeviceType::Server),
        _ => None,
    }
}
