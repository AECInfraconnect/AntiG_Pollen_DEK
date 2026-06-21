use anyhow::Result;
use dek_agent_observer::trust::{enforce_trust, TrustAction, TrustScore};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMessage {
    pub sender_id: String,
    pub receiver_id: String,
    pub payload_encrypted: String,
    pub signature: String,      // base64 Ed25519 signature
    pub public_key_b64: String, // base64 public key
}

use dek_control_plane_api::capability::{CapabilityMaturity, RuntimeCapability};

pub struct IATPBroker {
    pub is_active: bool,
}

impl IATPBroker {
    pub fn capability() -> RuntimeCapability {
        RuntimeCapability {
            capability_id: "a2a.mediator".into(),
            name: "IATP Mediator".into(),
            pep_type: "a2a_mediator".into(),
            maturity: CapabilityMaturity::EnforceBeta,
            supported_os: vec!["linux".into(), "macos".into(), "windows".into()],
            limitations: vec![],
        }
    }
}

impl Default for IATPBroker {
    fn default() -> Self {
        Self::new()
    }
}

impl IATPBroker {
    pub fn new() -> Self {
        Self { is_active: true }
    }

    /// Process an A2A message intercepting it via the DEK mediator.
    /// This paves the way for ASI07 Inter-Agent Trust Protocol.
    pub fn route_message(&self, msg: &A2AMessage, sender_trust: &TrustScore) -> Result<()> {
        tracing::info!(
            "Intercepting A2A message from {} to {}",
            msg.sender_id,
            msg.receiver_id
        );

        // 1. Verify cryptographic signature
        use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
        use ed25519_dalek::{Signature, Verifier, VerifyingKey};

        let pubkey_bytes = b64
            .decode(&msg.public_key_b64)
            .map_err(|_| anyhow::anyhow!("Invalid base64 public key"))?;

        let pubkey_arr: [u8; 32] = pubkey_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid Ed25519 public key length"))?;
        let public_key = VerifyingKey::from_bytes(&pubkey_arr)
            .map_err(|_| anyhow::anyhow!("Invalid Ed25519 public key format"))?;

        let sig_bytes = b64
            .decode(&msg.signature)
            .map_err(|_| anyhow::anyhow!("Invalid base64 signature"))?;
        let signature = Signature::from_slice(&sig_bytes)
            .map_err(|_| anyhow::anyhow!("Invalid Ed25519 signature format"))?;

        // Construct the digest payload that should have been signed
        // Format: sender_id | receiver_id | payload_encrypted
        let digest_input = format!(
            "{}|{}|{}",
            msg.sender_id, msg.receiver_id, msg.payload_encrypted
        );

        public_key
            .verify(digest_input.as_bytes(), &signature)
            .map_err(|_| anyhow::anyhow!("Cryptographic signature verification failed"))?;

        tracing::info!("A2A signature verified successfully");

        // Enforce trust score
        let action = enforce_trust(sender_trust);
        match action {
            TrustAction::KillSwitch => {
                tracing::warn!("A2A Blocked: Sender {} is a rogue agent", msg.sender_id);
                return Err(anyhow::anyhow!(
                    "Sender blocked due to critical trust score"
                ));
            }
            TrustAction::RequireApproval => {
                tracing::warn!(
                    "A2A Paused: Message from {} requires human approval",
                    msg.sender_id
                );
                return Err(anyhow::anyhow!("A2A message requires human approval"));
            }
            TrustAction::Normal => {
                tracing::info!("A2A Allowed: Routing message to {}", msg.receiver_id);
            }
        }

        // Mocking the successful routing
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;

    fn generate_test_msg(sender: &str, receiver: &str, payload: &str) -> A2AMessage {
        let mut csprng = OsRng {};
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let digest_input = format!("{}|{}|{}", sender, receiver, payload);
        let signature = signing_key.sign(digest_input.as_bytes());

        A2AMessage {
            sender_id: sender.into(),
            receiver_id: receiver.into(),
            payload_encrypted: payload.into(),
            signature: b64.encode(signature.to_bytes()),
            public_key_b64: b64.encode(signing_key.verifying_key().to_bytes()),
        }
    }

    #[test]
    fn test_a2a_route_message_allowed() {
        let broker = IATPBroker::new();
        let msg = generate_test_msg("agent-a", "agent-b", "encrypted_data");
        let trust = TrustScore {
            agent_id: "agent-a".into(),
            score: 0.9,
            reasons: vec![],
        };
        assert!(broker.route_message(&msg, &trust).is_ok());
    }

    #[test]
    fn test_a2a_route_message_blocked() {
        let broker = IATPBroker::new();
        let msg = generate_test_msg("rogue-agent", "agent-b", "malicious_data");
        let trust = TrustScore {
            agent_id: "rogue-agent".into(),
            score: 0.1, // Will trigger KillSwitch
            reasons: vec!["novel_tool".into()],
        };
        assert!(broker.route_message(&msg, &trust).is_err());
    }

    #[test]
    fn test_a2a_route_message_invalid_signature() {
        let broker = IATPBroker::new();
        let mut msg = generate_test_msg("agent-a", "agent-b", "data");
        msg.payload_encrypted = "tampered_data".into(); // Break the signature

        let trust = TrustScore {
            agent_id: "agent-a".into(),
            score: 0.9,
            reasons: vec![],
        };
        let res = broker.route_message(&msg, &trust);
        assert!(res.is_err());
        if let Err(e) = res {
            assert_eq!(e.to_string(), "Cryptographic signature verification failed");
        }
    }
}
