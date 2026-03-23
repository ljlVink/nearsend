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
const MAX_AUTO_SCAN_CANDIDATES: usize = 4096;
const MAX_CUSTOM_SCAN_CANDIDATES: usize = 4096;
const MIN_CUSTOM_CIDR_PREFIX: u8 = 24;

#[derive(Clone, Copy, Debug)]
enum DiscoveryTargetRule {
    Single(Ipv4Addr),
    Cidr { network: Ipv4Addr, prefix: u8 },
}

#[derive(Clone, Copy, Debug)]
struct LocalInterfaceV4 {
    ip: Ipv4Addr,
    network: Ipv4Addr,
    prefixlen: u8,
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

    let interfaces = local_interface_ipv4s().await;
    if interfaces.is_empty() && discovery_target_subnets.is_empty() {
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

    let local_ip_set = interfaces
        .iter()
        .map(|iface| iface.ip.to_string())
        .collect::<BTreeSet<_>>();
    let mut candidate_set = BTreeSet::new();

    for ip in collect_auto_discovery_candidates(&interfaces, &local_ip_set) {
        candidate_set.insert(ip);
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
    prefer_https: bool,
    self_fingerprint: &str,
) -> Option<DiscoveredDevice> {
    let schemes = if prefer_https {
        [true, false]
    } else {
        [false, true]
    };
    let mut attempts = Vec::new();

    for https in schemes {
        let scheme = if https { "https" } else { "http" };
        let url = format!("{}://{}:{}/api/localsend/v1/info", scheme, ip, port);
        let req = client.get(&url).query(&[("fingerprint", self_fingerprint)]);
        let res = match req.send().await {
            Ok(r) => r,
            Err(err) => {
                attempts.push(format!("{scheme}: request {err}"));
                continue;
            }
        };
        if res.status() == reqwest::StatusCode::PRECONDITION_FAILED {
            return None;
        }
        if !res.status().is_success() {
            attempts.push(format!("{scheme}: status {}", res.status()));
            continue;
        }
        let dto = match res.json::<InfoDto>().await {
            Ok(dto) => dto,
            Err(err) => {
                attempts.push(format!("{scheme}: parse {err}"));
                continue;
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
        return Some(DiscoveredDevice {
            info,
            ip: ip.to_string(),
            port,
            https,
        });
    }

    if !attempts.is_empty() {
        log::debug!(
            "discovery probe failed for {}:{} -> {}",
            ip,
            port,
            attempts.join("; ")
        );
    }
    None
}

async fn local_interface_ipv4s() -> Vec<LocalInterfaceV4> {
    let mut local_interfaces = Vec::<LocalInterfaceV4>::new();

    if let Ok(interfaces) = if_addrs::get_if_addrs() {
        for iface in interfaces {
            if iface.is_loopback() {
                continue;
            }
            if let if_addrs::IfAddr::V4(v4) = iface.addr {
                if v4.ip.is_link_local() {
                    continue;
                }
                let prefixlen = v4.prefixlen.min(32);
                let mask = if prefixlen == 0 {
                    0
                } else {
                    u32::MAX << (32 - prefixlen as u32)
                };
                local_interfaces.push(LocalInterfaceV4 {
                    ip: v4.ip,
                    network: Ipv4Addr::from(u32::from(v4.ip) & mask),
                    prefixlen,
                });
            }
        }
    }

    let primary = detect_primary_route_ipv4();
    rank_interface_ipv4s(&mut local_interfaces, primary);
    let mut ranked_interfaces = Vec::new();
    let mut seen = BTreeSet::new();
    for iface in local_interfaces {
        if seen.insert(iface.ip) {
            ranked_interfaces.push(iface);
        }
    }
    log::info!(
        "discovery scan interfaces: {:?}",
        ranked_interfaces
            .iter()
            .map(|iface| format!("{}/{}", iface.ip, iface.prefixlen))
            .collect::<Vec<_>>()
    );
    ranked_interfaces
}

fn collect_auto_discovery_candidates(
    interfaces: &[LocalInterfaceV4],
    local_ip_set: &BTreeSet<String>,
) -> Vec<String> {
    let mut set = BTreeSet::new();

    for iface in interfaces.iter().take(MAX_AUTO_SCAN_PREFIX_COUNT) {
        log::info!(
            "discovery auto subnet: ip={} network={}/{}",
            iface.ip,
            iface.network,
            iface.prefixlen
        );

        let host_bits = 32u32.saturating_sub(iface.prefixlen as u32);
        let host_count = if host_bits == 32 {
            u32::MAX
        } else {
            1u32 << host_bits
        };
        if host_count <= 1 {
            continue;
        }

        let network_base = u32::from(iface.network);
        let local_host = u32::from(iface.ip).saturating_sub(network_base);
        let search_window = host_count
            .saturating_sub(1)
            .min((MAX_AUTO_SCAN_CANDIDATES as u32).saturating_mul(2));
        let before = set.len();

        for distance in 1..=search_window {
            if let Some(host) = local_host.checked_sub(distance) {
                push_auto_candidate(
                    &mut set,
                    local_ip_set,
                    network_base,
                    host,
                    host_count,
                    iface.prefixlen,
                );
            }
            if let Some(host) = local_host.checked_add(distance).filter(|h| *h < host_count) {
                push_auto_candidate(
                    &mut set,
                    local_ip_set,
                    network_base,
                    host,
                    host_count,
                    iface.prefixlen,
                );
            }

            if set.len() >= MAX_AUTO_SCAN_CANDIDATES {
                log::warn!(
                    "auto discovery candidate limit reached ({}), stop expanding subnets",
                    MAX_AUTO_SCAN_CANDIDATES
                );
                return set.into_iter().collect();
            }
        }

        log::info!(
            "discovery auto subnet {}/{} contributed {} candidates",
            iface.network,
            iface.prefixlen,
            set.len().saturating_sub(before)
        );
    }

    set.into_iter().collect()
}

fn push_auto_candidate(
    set: &mut BTreeSet<String>,
    local_ip_set: &BTreeSet<String>,
    network_base: u32,
    host: u32,
    host_count: u32,
    prefixlen: u8,
) {
    if prefixlen <= 30 && (host == 0 || host == host_count.saturating_sub(1)) {
        return;
    }

    let ip_text = Ipv4Addr::from(network_base.saturating_add(host)).to_string();
    if local_ip_set.contains(&ip_text) {
        return;
    }
    set.insert(ip_text);
}

fn detect_primary_route_ipv4() -> Option<Ipv4Addr> {
    let probes = [("224.0.0.167", 53317), ("1.1.1.1", 80), ("8.8.8.8", 80)];
    for (host, port) in probes {
        let socket = match std::net::UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => s,
            Err(_) => continue,
        };
        if socket.connect((host, port)).is_err() {
            continue;
        }
        let local = match socket.local_addr() {
            Ok(addr) => addr,
            Err(_) => continue,
        };
        if let std::net::IpAddr::V4(ipv4) = local.ip() {
            return Some(ipv4);
        }
    }
    None
}

fn rank_interface_ipv4s(list: &mut Vec<LocalInterfaceV4>, primary: Option<Ipv4Addr>) {
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
        score(&b.ip)
            .cmp(&score(&a.ip))
            .then_with(|| a.ip.octets().cmp(&b.ip.octets()))
    });
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
