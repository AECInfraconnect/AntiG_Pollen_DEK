// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use dek_domain_schema::deployment_session::{
    DeploymentEvent, DeploymentPhase, DeploymentSession, DeploymentSessionStatus, EventStatus,
    LocalizedText,
};
use tokio::sync::mpsc;

pub trait DeploymentEventSink: Send + Sync {
    #[allow(async_fn_in_trait)]
    async fn emit(&self, event: DeploymentEvent) -> anyhow::Result<()>;
}

pub struct MemoryEventSink {
    sender: mpsc::Sender<DeploymentEvent>,
}

impl MemoryEventSink {
    pub fn new(sender: mpsc::Sender<DeploymentEvent>) -> Self {
        Self { sender }
    }
}

impl DeploymentEventSink for MemoryEventSink {
    async fn emit(&self, event: DeploymentEvent) -> anyhow::Result<()> {
        let _ = self.sender.send(event).await;
        Ok(())
    }
}

pub struct StoreEventSink {
    // In a real implementation, this would hold database pool and telemetry spool references.
}

impl StoreEventSink {
    pub fn new() -> Self {
        Self {}
    }
}

impl DeploymentEventSink for StoreEventSink {
    async fn emit(&self, event: DeploymentEvent) -> anyhow::Result<()> {
        // Pseudo-code for secure telemetry and timeline integration:
        // 1. Write to local event store (SQLite) for the timeline view.
        // 2. Write to secure telemetry spool for cloud/admin sync.

        // Ensure correlation ID, policy ID, and agent/entity IDs are present.
        let _correlation_id = &event.correlation_id;
        let _policy_id = &event.policy_id;
        let _agent_id = &event.agent_id;

        // Emitting to local log
        println!("Emitting deployment event: {:?}", event.event_id);

        Ok(())
    }
}

pub struct DeploymentOrchestrator<T: DeploymentEventSink> {
    event_sink: std::sync::Arc<T>,
}

impl<T: DeploymentEventSink> DeploymentOrchestrator<T> {
    pub fn new(event_sink: std::sync::Arc<T>) -> Self {
        Self { event_sink }
    }

    pub async fn transition(
        &self,
        session: &mut DeploymentSession,
        new_status: DeploymentSessionStatus,
    ) -> anyhow::Result<()> {
        session.status = new_status.clone();
        session.updated_at = chrono::Utc::now();

        let phase = match new_status {
            DeploymentSessionStatus::ScanStarted
            | DeploymentSessionStatus::ScanCompleted
            | DeploymentSessionStatus::CapabilitySnapshotCreated => DeploymentPhase::AgentDiscovery,
            DeploymentSessionStatus::PolicyFeasibilityEvaluated
            | DeploymentSessionStatus::UserSelectedPolicy
            | DeploymentSessionStatus::DeploymentPlanCreated => DeploymentPhase::RoutePlanning,
            DeploymentSessionStatus::ApprovalRequired => DeploymentPhase::RoutePlanning,
            DeploymentSessionStatus::BundleCreated | DeploymentSessionStatus::BundleActivated => {
                DeploymentPhase::PepDeploy
            }
            DeploymentSessionStatus::WarmCheckPassed => DeploymentPhase::WarmCheck,
            DeploymentSessionStatus::Active
            | DeploymentSessionStatus::PartialActive
            | DeploymentSessionStatus::ObserveOnlyActive => DeploymentPhase::Enforcement,
            DeploymentSessionStatus::Failed => DeploymentPhase::Rollback,
            DeploymentSessionStatus::RolledBack => DeploymentPhase::Rollback,
        };

        let event = DeploymentEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            deployment_id: session.deployment_id.clone(),
            agent_id: None,
            entity_id: None,
            policy_id: session.policy_id.clone(),
            phase,
            status: EventStatus::Info,
            title: LocalizedText {
                en: format!("Transitioned to {:?}", new_status),
                th: format!("เปลี่ยนสถานะเป็น {:?}", new_status),
            },
            detail: LocalizedText {
                en: "".into(),
                th: "".into(),
            },
            technical_detail: None,
            user_action: None,
            created_at: chrono::Utc::now(),
            correlation_id: session.deployment_id.clone(),
        };

        self.event_sink.emit(event).await?;
        Ok(())
    }
}
