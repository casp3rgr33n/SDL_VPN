use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ProxyType {
    Socks5,
    Http,
    Https,
    Wireguard,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ProxyNodeStatus {
    Active,
    Degraded,
    Offline,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyNode {
    pub node_id: Uuid,
    pub region: String,
    pub proxy_type: ProxyType,
    pub latency_ms: u32,
    pub current_load: u32, // percentage 0-100
    pub status: ProxyNodeStatus,
    pub fallback_node_id: Option<Uuid>,
    pub last_health_check: DateTime<Utc>,
}

impl ProxyNode {
    pub fn is_healthy(&self) -> bool {
        self.status == ProxyNodeStatus::Active && self.current_load < 90
    }

    pub fn get_optimal_node<'a>(&'a self, pool: &'a [ProxyNode]) -> &'a ProxyNode {
        if self.is_healthy() {
            return self;
        }

        if let Some(fallback_id) = self.fallback_node_id {
            if let Some(fallback_node) = pool.iter().find(|n| n.node_id == fallback_id) {
                if fallback_node.is_healthy() {
                    return fallback_node;
                }
            }
        }

        // Ultimate fallback: find any healthy node in the same region
        pool.iter()
            .filter(|n| n.is_healthy() && n.region == self.region)
            .min_by_key(|n| n.latency_ms + (n.current_load * 2))
            .unwrap_or(self)
    }
}
