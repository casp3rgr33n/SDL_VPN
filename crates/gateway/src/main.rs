use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;
use core_protocol::models::{NodeInfo, TunnelMessage};
use tracing::{info, warn, error};

type NodeRegistry = Arc<DashMap<Uuid, NodeState>>;

#[derive(Debug)]
pub struct NodeState {
    pub info: NodeInfo,
    pub last_heartbeat: std::time::Instant,
    pub tx: tokio::sync::mpsc::Sender<TunnelMessage>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting Central Orchestration Gateway...");
    
    let registry: NodeRegistry = Arc::new(DashMap::new());
    
    // Simulate accepting a node
    let (tx, mut rx) = tokio::sync::mpsc::channel::<TunnelMessage>(1024);
    
    // Start health checker daemon
    let reg_clone = registry.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let now = std::time::Instant::now();
            reg_clone.retain(|id, state| {
                if now.duration_since(state.last_heartbeat).as_secs() > 15 {
                    warn!("Node {} timed out. Pruning from active pool.", id);
                    false
                } else {
                    true
                }
            });
        }
    });

    info!("Gateway running. Awaiting connections...");
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}