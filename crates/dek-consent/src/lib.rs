// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgreementType {
    Eula,
    PrivacyNotice,
    Telemetry,
    BrowserHistoryScan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgreementDoc {
    pub agreement_type: AgreementType,
    pub version: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub agreement_type: AgreementType,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub user_identifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsentStoreData {
    pub records: HashMap<AgreementType, ConsentRecord>,
}

pub struct ConsentStore {
    path: PathBuf,
}

impl ConsentStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn load(&self) -> Result<ConsentStoreData> {
        if !self.path.exists() {
            return Ok(ConsentStoreData::default());
        }
        let content = fs::read_to_string(&self.path).context("Failed to read consent store")?;
        let data: ConsentStoreData =
            serde_json::from_str(&content).context("Failed to parse consent store")?;
        Ok(data)
    }

    pub fn save(&self, data: &ConsentStoreData) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).context("Failed to create consent store directory")?;
        }
        let content =
            serde_json::to_string_pretty(data).context("Failed to serialize consent data")?;
        fs::write(&self.path, content).context("Failed to write consent store")?;
        Ok(())
    }

    pub fn record_consent(
        &self,
        agreement_type: AgreementType,
        version: String,
        user_identifier: String,
    ) -> Result<()> {
        let mut data = self.load()?;
        let record = ConsentRecord {
            agreement_type: agreement_type.clone(),
            version,
            timestamp: Utc::now(),
            user_identifier,
        };
        data.records.insert(agreement_type, record);
        self.save(&data)
    }

    pub fn has_consented(&self, agreement_type: &AgreementType) -> Result<bool> {
        let data = self.load()?;
        Ok(data.records.contains_key(agreement_type))
    }
}
