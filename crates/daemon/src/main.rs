use core_protocol::models::{NodeInfo, TunnelMessage};
use ed25519_dalek::{Signer, SigningKey};
use futures_util::{SinkExt, StreamExt};
use rand::rngs::OsRng;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};
use uuid::Uuid;
use backoff::ExponentialBackoff;

type StreamMap = Arc<Mutex<HashMap<Uuid, mpsc::Sender<Vec<u8>>>>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let gateway_url = env::var("GATEWAY_URL").unwrap_or_else(|_| "ws://127.0.0.1:8080".to_string());
    let max_bandwidth: u32 = env::var("MAX_BANDWIDTH_MBPS").unwrap_or_else(|_| "10".to_string()).parse().unwrap();
    
    info!("Starting Headless Client Daemon...");
    info!("Target Gateway: {}", gateway_url);
    
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let public_key = signing_key.verifying_key();
    let node_id = Uuid::new_v4();
    
    let backoff = ExponentialBackoff {
        max_elapsed_time: None,
        max_interval: Duration::from_secs(60),
        ..ExponentialBackoff::default()
    };

    let connect_operation = || async {
        info!("Connecting to {}...", gateway_url);
        
        let (ws_stream, _) = match connect_async(&gateway_url).await {
            Ok(ws) => ws,
            Err(e) => {
                warn!("Connection failed: {}", e);
                let err: Result<(), backoff::Error<std::io::Error>> = Err(backoff::Error::transient(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "disconnected")));
                return err;
            }
        };
        
        info!("Connected to Gateway!");
        let (mut ws_tx, mut ws_rx) = ws_stream.split();
        let (tx, mut rx) = mpsc::channel::<TunnelMessage>(1024);
        
        // Registration
        let info_payload = NodeInfo {
            node_id,
            public_key,
            country: "US".to_string(),
            city: "New York".to_string(),
            isp: "Comcast".to_string(),
            max_bandwidth_mbps: max_bandwidth,
            vpn_mode_active: true,
        };
        
        let msg_bytes = serde_json::to_vec(&info_payload).unwrap();
        let signature = signing_key.sign(&msg_bytes);
        
        let register_msg = TunnelMessage::Register {
            info: info_payload,
            signature,
        };
        let _ = tx.send(register_msg).await;
        
        let streams: StreamMap = Arc::new(Mutex::new(HashMap::new()));
        
        // Writer task
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let bytes = bincode::serialize(&msg).unwrap();
                if ws_tx.send(Message::Binary(bytes)).await.is_err() { break; }
            }
        });
        
        // Heartbeat task
        let h_tx = tx_clone.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;
                if h_tx.send(TunnelMessage::Heartbeat).await.is_err() { break; }
            }
        });

        // Reader task
        while let Some(Ok(Message::Binary(data))) = ws_rx.next().await {
            if let Ok(msg) = bincode::deserialize::<TunnelMessage>(&data) {
                match msg {
                    TunnelMessage::ProxyRequest { stream_id, target_host, target_port } => {
                        let st_tx = tx_clone.clone();
                        let streams_c = streams.clone();
                        tokio::spawn(async move {
                            info!("Dialing proxy target: {}:{}", target_host, target_port);
                            if let Ok(target_stream) = TcpStream::connect((target_host.as_str(), target_port)).await {
                                let (mut ri, mut wi) = target_stream.into_split();
                                let (local_tx, mut local_rx) = mpsc::channel::<Vec<u8>>(1024);
                                streams_c.lock().await.insert(stream_id, local_tx);
                                
                                let s_id = stream_id;
                                let up_tx = st_tx.clone();
                                tokio::spawn(async move {
                                    let mut buf = [0u8; 8192];
                                    while let Ok(n) = ri.read(&mut buf).await {
                                        if n == 0 { break; }
                                        let _ = up_tx.send(TunnelMessage::Payload { stream_id: s_id, data: buf[..n].to_vec() }).await;
                                    }
                                    let _ = up_tx.send(TunnelMessage::StreamClose { stream_id: s_id }).await;
                                });
                                
                                tokio::spawn(async move {
                                    while let Some(data) = local_rx.recv().await {
                                        if wi.write_all(&data).await.is_err() { break; }
                                    }
                                    streams_c.lock().await.remove(&stream_id);
                                });
                            } else {
                                let _ = st_tx.send(TunnelMessage::StreamClose { stream_id }).await;
                            }
                        });
                    },
                    TunnelMessage::Payload { stream_id, data } => {
                        if let Some(st) = streams.lock().await.get(&stream_id) {
                            let _ = st.send(data).await;
                        }
                    },
                    TunnelMessage::StreamClose { stream_id } => {
                        streams.lock().await.remove(&stream_id);
                    },
                    _ => {}
                }
            }
        }
        
        warn!("Disconnected from gateway. Retrying...");
        let err: Result<(), backoff::Error<std::io::Error>> = Err(backoff::Error::transient(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "disconnected")));
        err
    };

    if let Err(e) = backoff::future::retry(backoff, connect_operation).await {
        error!("Fatal connection error: {}", e);
    }
}