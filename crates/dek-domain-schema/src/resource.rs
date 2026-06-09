// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Resource {
    pub schema_version: String,
    pub resource_id: String,
    pub tenant_id: String,
    pub resource_type: String,
    pub name: String,
    pub uri: String,
    pub classification: String,
    pub data_tags: Vec<String>,
    pub owner_principal_id: String,
    pub risk_level: String,
}

