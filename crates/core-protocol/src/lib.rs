pub mod models;

#[cfg(test)]
mod tests {
    use super::models::*;
    use ed25519_dalek::{Signer, SigningKey, Verifier};
    use rand::rngs::OsRng;
    use uuid::Uuid;

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

        // Verification logic that the gateway will run
        if let TunnelMessage::Register { info, signature } = msg {
            let verify_bytes = serde_json::to_vec(&info).unwrap();
            assert!(info.public_key.verify(&verify_bytes, &signature).is_ok());
        } else {
            panic!("Wrong message type");
        }
    }
}