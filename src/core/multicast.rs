use serde::{Deserialize, Serialize};

const DEFAULT_MULTICAST_GROUP: &str = "224.0.0.167";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ProtocolType {
    Http,
    Https,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum DeviceType {
    Mobile,
    Desktop,
    Web,
    Headless,
    Server,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MulticastDto {
    alias: String,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_type: Option<DeviceType>,
    fingerprint: String,
    port: u16,
    protocol: ProtocolType,
    download: bool,
    announcement: bool,
    announce: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RegisterDto {
    alias: String,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_type: Option<DeviceType>,
    fingerprint: String,
    port: u16,
    protocol: ProtocolType,
    download: bool,
}

pub async fn start_multicast_service(
    alias: String,
    fingerprint: String,
    port: u16,
    device_model: Option<String>,
    device_type: Option<localsend::model::discovery::DeviceType>,
) -> anyhow::Result<()> {
    let socket = tokio::net::UdpSocket::bind(("0.0.0.0", port)).await?;
    socket.join_multicast_v4(
        DEFAULT_MULTICAST_GROUP
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid multicast group"))?,
        "0.0.0.0"
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid multicast iface"))?,
    )?;
    socket.set_broadcast(true)?;

    let info = MulticastDto {
        alias: alias.clone(),
        version: "2.1".to_string(),
        device_model: device_model.clone(),
        device_type: map_device_type(device_type.as_ref()),
        fingerprint: fingerprint.clone(),
        port,
        protocol: ProtocolType::Http,
        download: false,
        announcement: true,
        announce: true,
    };

    // initial announcement bursts
    for delay_ms in [100u64, 500, 2000] {
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        let _ = socket
            .send_to(
                &serde_json::to_vec(&info)?,
                format!("{}:{}", DEFAULT_MULTICAST_GROUP, port),
            )
            .await;
    }

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_millis(1200))
        .build()?;

    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let (size, addr) = match socket.recv_from(&mut buf).await {
            Ok(v) => v,
            Err(err) => {
                log::warn!("multicast recv error: {}", err);
                continue;
            }
        };
        let payload = &buf[..size];
        let dto = match serde_json::from_slice::<MulticastDto>(payload) {
            Ok(dto) => dto,
            Err(_) => continue,
        };
        if dto.fingerprint == fingerprint {
            continue;
        }
        if !(dto.announce || dto.announcement) {
            continue;
        }

        let register = RegisterDto {
            alias: alias.clone(),
            version: "2.1".to_string(),
            device_model: device_model.clone(),
            device_type: map_device_type(device_type.as_ref()),
            fingerprint: fingerprint.clone(),
            port,
            protocol: ProtocolType::Http,
            download: false,
        };

        let peer_ip = addr.ip().to_string();
        let peer_port = dto.port;
        let scheme = match dto.protocol {
            ProtocolType::Https => "https",
            ProtocolType::Http => "http",
        };
        let url = format!(
            "{}://{}:{}/api/localsend/v2/register",
            scheme, peer_ip, peer_port
        );

        if let Err(err) = client.post(&url).json(&register).send().await {
            log::debug!("multicast register response failed {}: {}", url, err);
        }
    }
}

fn map_device_type(value: Option<&localsend::model::discovery::DeviceType>) -> Option<DeviceType> {
    match value {
        Some(localsend::model::discovery::DeviceType::Mobile) => Some(DeviceType::Mobile),
        Some(localsend::model::discovery::DeviceType::Desktop) => Some(DeviceType::Desktop),
        Some(localsend::model::discovery::DeviceType::Web) => Some(DeviceType::Web),
        Some(localsend::model::discovery::DeviceType::Headless) => Some(DeviceType::Headless),
        Some(localsend::model::discovery::DeviceType::Server) => Some(DeviceType::Server),
        None => None,
    }
}
