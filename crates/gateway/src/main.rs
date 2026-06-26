use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};
use uuid::Uuid;
use core_protocol::models::{NodeInfo, TunnelMessage};

type NodeRegistry = Arc<DashMap<Uuid, NodeState>>;
type StreamRegistry = Arc<DashMap<Uuid, tokio::sync::mpsc::Sender<Vec<u8>>>>;

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
    
    let node_registry: NodeRegistry = Arc::new(DashMap::new());
    let stream_registry: StreamRegistry = Arc::new(DashMap::new());
    
    let ws_listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    let proxy_listener = TcpListener::bind("0.0.0.0:1080").await.unwrap();
    
    let nr1 = node_registry.clone();
    let sr1 = stream_registry.clone();
    tokio::spawn(async move {
        while let Ok((stream, _)) = ws_listener.accept().await {
            let nr = nr1.clone();
            let sr = sr1.clone();
            tokio::spawn(handle_ws_client(stream, nr, sr));
        }
    });

    info!("Gateway running. WebSocket on 8080. HTTP Proxy on 1080.");
    while let Ok((mut stream, _)) = proxy_listener.accept().await {
        // Find an active node
        if let Some(node) = node_registry.iter().next() {
            let node_tx = node.tx.clone();
            let sr = stream_registry.clone();
            tokio::spawn(handle_proxy_client(stream, node_tx, sr));
        } else {
            warn!("No active nodes to route proxy traffic!");
        }
    }
}

async fn handle_ws_client(stream: TcpStream, nodes: NodeRegistry, streams: StreamRegistry) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => { error!("WS handshake failed: {}", e); return; }
    };
    
    let (mut ws_tx, mut ws_rx) = ws_stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<TunnelMessage>(1024);
    
    let mut current_node_id = None;

    // Sender task
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let bytes = bincode::serialize(&msg).unwrap();
            if ws_tx.send(Message::Binary(bytes)).await.is_err() { break; }
        }
    });

    // Receiver loop
    while let Some(Ok(Message::Binary(data))) = ws_rx.next().await {
        if let Ok(msg) = bincode::deserialize::<TunnelMessage>(&data) {
            match msg {
                TunnelMessage::Register { info, signature } => {
                    let verify_bytes = serde_json::to_vec(&info).unwrap();
                    if info.public_key.verify_strict(&verify_bytes, &signature).is_ok() {
                        info!("Node {} registered", info.node_id);
                        current_node_id = Some(info.node_id);
                        nodes.insert(info.node_id, NodeState {
                            info,
                            last_heartbeat: std::time::Instant::now(),
                            tx: tx.clone(),
                        });
                    } else {
                        warn!("Invalid signature from node");
                        break;
                    }
                },
                TunnelMessage::Heartbeat => {
                    if let Some(id) = current_node_id {
                        if let Some(mut state) = nodes.get_mut(&id) {
                            state.last_heartbeat = std::time::Instant::now();
                        }
                    }
                },
                TunnelMessage::Payload { stream_id, data } => {
                    if let Some(stream_tx) = streams.get(&stream_id) {
                        let _ = stream_tx.send(data).await;
                    }
                },
                TunnelMessage::StreamClose { stream_id } => {
                    streams.remove(&stream_id);
                },
                _ => {}
            }
        }
    }
    
    if let Some(id) = current_node_id {
        nodes.remove(&id);
        info!("Node {} disconnected", id);
    }
}

async fn handle_proxy_client(mut stream: TcpStream, node_tx: tokio::sync::mpsc::Sender<TunnelMessage>, streams: StreamRegistry) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    
    let mut buf = [0u8; 4096];
    if let Ok(n) = stream.read(&mut buf).await {
        let req = String::from_utf8_lossy(&buf[..n]);
        if req.starts_with("CONNECT ") {
            let parts: Vec<&str> = req.split_whitespace().collect();
            if parts.len() >= 2 {
                let target = parts[1];
                let mut host_port = target.split(':');
                let target_host = host_port.next().unwrap_or("").to_string();
                let target_port: u16 = host_port.next().unwrap_or("443").parse().unwrap_or(443);
                
                let stream_id = Uuid::new_v4();
                let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(1024);
                streams.insert(stream_id, tx);
                
                let _ = stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await;
                let _ = node_tx.send(TunnelMessage::ProxyRequest { stream_id, target_host, target_port }).await;
                
                let (mut ri, mut wi) = stream.into_split();
                
                // Downstream (Scraper -> Gateway -> Node)
                let node_tx_clone = node_tx.clone();
                tokio::spawn(async move {
                    let mut buf = [0u8; 8192];
                    while let Ok(n) = ri.read(&mut buf).await {
                        if n == 0 { break; }
                        let _ = node_tx_clone.send(TunnelMessage::Payload { stream_id, data: buf[..n].to_vec() }).await;
                    }
                    let _ = node_tx_clone.send(TunnelMessage::StreamClose { stream_id }).await;
                });
                
                // Upstream (Node -> Gateway -> Scraper)
                tokio::spawn(async move {
                    while let Some(data) = rx.recv().await {
                        if wi.write_all(&data).await.is_err() { break; }
                    }
                    streams.remove(&stream_id);
                });
            }
        }
    }
}