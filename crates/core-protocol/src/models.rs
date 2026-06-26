use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeInfo {
    pub node_id: Uuid,
    pub country: String,
    pub city: String,
    pub isp: String,
    pub max_bandwidth_mbps: u32,
    pub vpn_mode_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TunnelMessage {
    Register(NodeInfo),
    Heartbeat,
    ProxyRequest {
        stream_id: Uuid,
        target_host: String,
        target_port: u16,
    },
    Payload {
        stream_id: Uuid,
        data: Vec<u8>,
    },
    StreamClose {
        stream_id: Uuid,
    }
}
