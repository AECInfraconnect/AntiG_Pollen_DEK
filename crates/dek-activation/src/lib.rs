// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use std::path::PathBuf;

pub mod coordinator;
pub mod hydration;
pub mod lkg;
pub mod modes;
pub mod preflight;
pub mod signature;
pub mod snapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivationSource {
    PollSync,
    CloudPush,
    LocalAdmin,
    EmergencyDeny,
}

#[derive(Debug, Clone)]
pub struct ActivationRequest {
    pub manifest_path: PathBuf,
    pub source: ActivationSource,
    pub tenant_id: String,
    pub device_id: String,
}

#[derive(Debug, Clone)]
pub struct ActivationReceipt {
    pub timestamp_version: u64,
    pub bundle_id: String,
    pub mode: dek_config::ActivationMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivationError {
    SchemaFailed(String),
    RuntimeHydrationFailed(String),
    PreflightFailed(String),
    CanaryFailed(String),
    SnapshotSwapFailed(String),
    RollbackFailed(String),
    ProfileViolation(String),
    Timeout,
}

impl std::fmt::Display for ActivationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SchemaFailed(msg) => write!(f, "Schema Failed: {}", msg),
            Self::RuntimeHydrationFailed(msg) => write!(f, "Runtime Hydration Failed: {}", msg),
            Self::PreflightFailed(msg) => write!(f, "Preflight Failed: {}", msg),
            Self::CanaryFailed(msg) => write!(f, "Canary Failed: {}", msg),
            Self::SnapshotSwapFailed(msg) => write!(f, "Snapshot Swap Failed: {}", msg),
            Self::RollbackFailed(msg) => write!(f, "Rollback Failed: {}", msg),
            Self::ProfileViolation(msg) => write!(f, "Profile Violation: {}", msg),
            Self::Timeout => write!(f, "Activation Timeout"),
        }
    }
}

impl std::error::Error for ActivationError {}

#[derive(Debug, Clone)]
pub enum ActivationDecision {
    Activated(ActivationReceipt),
    Rejected(ActivationError),
    Deferred(String),
}
