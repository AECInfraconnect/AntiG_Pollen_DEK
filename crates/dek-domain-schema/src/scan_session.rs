// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalScanSession {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSourceStatus {
    pub source: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalScanSummary {
    pub total_found: u32,
    pub session_id: String,
}
