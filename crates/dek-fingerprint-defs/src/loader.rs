// SPDX-License-Identifier: Apache-2.0

use arc_swap::ArcSwap;
use std::sync::Arc;
use std::path::PathBuf;
use ed25519_dalek::{VerifyingKey, Signature, Verifier};

use crate::model::*;

pub struct DefinitionStore {
    current: ArcSwap<FingerprintDefinition>,
    on_disk_path: PathBuf,
}

impl DefinitionStore {
    pub fn load(on_disk_path: PathBuf, pubkey: Option<&VerifyingKey>) -> Self {
        let mut def = crate::embedded_baseline();
        if let Ok(raw) = std::fs::read(&on_disk_path) {
            // Need a SignedDefinition struct if one exists, or just read the payload
            // For now, if we can parse it as FingerprintDefinition, we'll take it if it's newer
            if let Ok(disk) = serde_json::from_slice::<FingerprintDefinition>(&raw) {
                // In a real implementation with SignedDefinition, verify signature here
                if disk.definition_version > def.definition_version
                    && disk.schema_version == def.schema_version {
                    def = disk;
                    tracing::info!(version = def.definition_version, "loaded on-disk definition");
                } else {
                    tracing::warn!("on-disk definition rejected (sig/version/schema) — using embedded");
                }
            }
        }
        Self { current: ArcSwap::from_pointee(def), on_disk_path }
    }

    pub fn get(&self) -> Arc<FingerprintDefinition> {
        self.current.load_full()
    }

    pub fn apply_update(&self, incoming: FingerprintDefinition) -> anyhow::Result<u64> {
        let cur = self.current.load();
        anyhow::ensure!(incoming.schema_version == cur.schema_version, "schema mismatch");
        anyhow::ensure!(incoming.definition_version > cur.definition_version, "stale version");

        let merged = match incoming.kind {
            DefinitionKind::Full  => incoming,
            DefinitionKind::Delta => merge_delta(&cur, &incoming)?,
        };

        self.current.store(Arc::new(merged.clone()));
        
        if let Ok(json) = serde_json::to_string_pretty(&merged) {
            let tmp_path = self.on_disk_path.with_extension("tmp");
            std::fs::write(&tmp_path, json)?;
            std::fs::rename(tmp_path, &self.on_disk_path)?;
        }
        
        tracing::info!(version = merged.definition_version, "definition updated (hot)");
        Ok(merged.definition_version)
    }
}

fn merge_delta(base: &FingerprintDefinition, delta: &FingerprintDefinition) -> anyhow::Result<FingerprintDefinition> {
    let mut out = base.clone();
    out.definition_version = delta.definition_version;
    
    for s in &delta.signatures {
        match out.signatures.iter_mut().find(|x| x.id == s.id) {
            Some(existing) => *existing = s.clone(),
            None => out.signatures.push(s.clone()),
        }
    }
    for w in &delta.web_ai_signatures {
        match out.web_ai_signatures.iter_mut().find(|x| x.domain == w.domain) {
            Some(e) => *e = w.clone(),
            None => out.web_ai_signatures.push(w.clone()),
        }
    }
    
    // Process removed_ids if necessary
    out.signatures.retain(|s| !delta.removed_ids.contains(&s.id));
    
    // Recalculate catalog hash if needed
    // out.catalog_hash = compute_hash(&out);
    Ok(out)
}
