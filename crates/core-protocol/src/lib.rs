pub mod models;
pub mod error;
pub mod subscription;
pub mod proxy;

#[cfg(test)]
mod tests {
    use super::models::*;
    use super::subscription::*;
    use super::proxy::*;
    use ed25519_dalek::{Signer, SigningKey, Verifier};
    use rand::rngs::OsRng;
    use uuid::Uuid;
    use chrono::{Utc, Duration};

    #[test]
    fn test_node_registration_crypto() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key();

        let info = NodeInfo {
            node_id: Uuid::new_v4(),
            public_key,
            country: "US".to_string(),
            city: "New York".to_string(),
            isp: "Comcast".to_string(),
            max_bandwidth_mbps: 100,
            vpn_mode_active: true,
        };

        let message_bytes = serde_json::to_vec(&info).unwrap();
        let signature = signing_key.sign(&message_bytes);

        let msg = TunnelMessage::Register {
            info: info.clone(),
            signature,
        };

        if let TunnelMessage::Register { info, signature } = msg {
            let verify_bytes = serde_json::to_vec(&info).unwrap();
            assert!(info.public_key.verify(&verify_bytes, &signature).is_ok());
        } else {
            panic!("Wrong message type");
        }
    }

    #[test]
    fn test_subscription_fallback() {
        let mut sub = UserSubscription::new(Uuid::new_v4());
        sub.tier = SubscriptionTier::Pro;
        sub.active_until = Some(Utc::now() - Duration::days(1)); // Expired yesterday
        sub.bandwidth_limit_bytes = None; // Pro was unlimited
        
        assert!(!sub.is_active());
        
        sub.apply_fallback();
        
        assert_eq!(sub.tier, SubscriptionTier::Free);
        assert!(sub.active_until.is_none());
        assert_eq!(sub.bandwidth_limit_bytes, Some(5_000_000_000));
    }

    #[test]
    fn test_subscription_bandwidth_tracking() {
        let mut sub = UserSubscription::new(Uuid::new_v4());
        assert!(sub.has_bandwidth_remaining());
        
        sub.bandwidth_used_bytes = 5_000_000_001;
        assert!(!sub.has_bandwidth_remaining());
    }

    #[test]
    fn test_proxy_node_optimal_fallback() {
        let fallback_id = Uuid::new_v4();
        
        let mut pool = vec![];
        
        let mut primary = ProxyNode {
            node_id: Uuid::new_v4(),
            region: "US-East".to_string(),
            proxy_type: ProxyType::Wireguard,
            latency_ms: 10,
            current_load: 95, // overloaded -> unhealthy
            status: ProxyNodeStatus::Active,
            fallback_node_id: Some(fallback_id),
            last_health_check: Utc::now(),
        };
        
        let fallback = ProxyNode {
            node_id: fallback_id,
            region: "US-East".to_string(),
            proxy_type: ProxyType::Wireguard,
            latency_ms: 50,
            current_load: 20, // healthy
            status: ProxyNodeStatus::Active,
            fallback_node_id: None,
            last_health_check: Utc::now(),
        };
        
        let generic_healthy = ProxyNode {
            node_id: Uuid::new_v4(),
            region: "US-East".to_string(),
            proxy_type: ProxyType::Wireguard,
            latency_ms: 40,
            current_load: 10,
            status: ProxyNodeStatus::Active,
            fallback_node_id: None,
            last_health_check: Utc::now(),
        };

        pool.push(primary.clone());
        pool.push(fallback.clone());
        pool.push(generic_healthy.clone());
        
        // Should return the fallback node
        let optimal = primary.get_optimal_node(&pool);
        assert_eq!(optimal.node_id, fallback_id);
        
        // What if fallback is offline?
        pool[1].status = ProxyNodeStatus::Offline;
        
        // Should fallback to generic_healthy in same region
        let optimal_ultimate = primary.get_optimal_node(&pool);
        assert_eq!(optimal_ultimate.node_id, generic_healthy.node_id);
    }
}