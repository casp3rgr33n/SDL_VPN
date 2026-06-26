use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use core_protocol::models::{NodeInfo, TunnelMessage};

type NodeRegistry = Arc<RwLock<HashMap<Uuid, NodeState>>>;

#[derive(Debug)]
struct NodeState {
    info: NodeInfo,
    last_heartbeat: std::time::Instant,
    tx: tokio::sync::mpsc::Sender<TunnelMessage>,
}

#[tokio::main]
async fn main() {
    println!("Starting Central Orchestration Gateway...");
    let _registry: NodeRegistry = Arc::new(RwLock::new(HashMap::new()));
    
    // Server logic will go here
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
