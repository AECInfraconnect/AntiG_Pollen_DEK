use anyhow::Result;
use dek_agent_observer::trust::{enforce_trust, TrustAction, TrustScore};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMessage {
    pub sender_id: String,
    pub receiver_id: String,
    pub payload_encrypted: String,
    pub signature: String,
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
            maturity: CapabilityMaturity::Stub,
            supported_os: vec!["linux".into(), "macos".into(), "windows".into()],
            limitations: vec!["Cryptographic signature validation is mocked".into()],
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

    #[test]
    fn test_a2a_route_message_allowed() {
        let broker = IATPBroker::new();
        let msg = A2AMessage {
            sender_id: "agent-a".into(),
            receiver_id: "agent-b".into(),
            payload_encrypted: "encrypted_data".into(),
            signature: "sig".into(),
        };
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
        let msg = A2AMessage {
            sender_id: "rogue-agent".into(),
            receiver_id: "agent-b".into(),
            payload_encrypted: "malicious_data".into(),
            signature: "sig".into(),
        };
        let trust = TrustScore {
            agent_id: "rogue-agent".into(),
            score: 0.1, // Will trigger KillSwitch
            reasons: vec!["novel_tool".into()],
        };
        assert!(broker.route_message(&msg, &trust).is_err());
    }
}
