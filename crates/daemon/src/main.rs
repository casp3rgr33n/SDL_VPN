use core_protocol::models::{NodeInfo, TunnelMessage};
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use std::env;
use std::time::Duration;
use tracing::{info, warn, error};
use uuid::Uuid;
use backoff::ExponentialBackoff;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let gateway_url = env::var("GATEWAY_URL").unwrap_or_else(|_| "wss://gateway.example.com/tunnel".to_string());
    let max_bandwidth: u32 = env::var("MAX_BANDWIDTH_MBPS").unwrap_or_else(|_| "10".to_string()).parse().unwrap();
    
    info!("Starting Headless Client Daemon...");
    info!("Target Gateway: {}", gateway_url);
    info!("Bandwidth Limit: {} Mbps", max_bandwidth);

    // Generate identity
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let public_key = signing_key.verifying_key();
    let node_id = Uuid::new_v4();
    
    info!("Generated Node Identity: {}", node_id);
    
    let backoff = ExponentialBackoff {
        max_elapsed_time: None,
        max_interval: Duration::from_secs(60),
        ..ExponentialBackoff::default()
    };

    let connect_operation = || async {
        info!("Attempting connection to gateway...");
        // Implement WebSocket handshake here
        // If it fails, return Err(backoff::Error::transient("connection failed"))
        
        let info = NodeInfo {
            node_id,
            public_key,
            country: "US".to_string(),
            city: "New York".to_string(),
            isp: "Comcast".to_string(), // In reality, fetch via IP lookup
            max_bandwidth_mbps: max_bandwidth,
            vpn_mode_active: true,
        };
        
        let msg_bytes = serde_json::to_vec(&info).unwrap();
        let signature = signing_key.sign(&msg_bytes);
        
        let _register_msg = TunnelMessage::Register {
            info,
            signature,
        };
        
        // Loop listening for proxy requests
        // loop { ... }
        
        // Simulating a disconnect to test backoff logic
        warn!("Disconnected from gateway. Retrying...");
        Err(backoff::Error::transient(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "disconnected")))
    };

    if let Err(e) = backoff::future::retry(backoff, connect_operation).await {
        error!("Fatal connection error: {}", e);
    }
}