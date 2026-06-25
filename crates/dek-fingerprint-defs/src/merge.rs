use crate::model::*;
use std::collections::HashMap;

pub struct FingerprintDb {
    pub version: u64,
    pub by_id: HashMap<String, AgentSignatureV2>,
    pub web_ai: HashMap<String, WebAiSignatureDef>,
    pub installed_apps: HashMap<String, InstalledAppSignatureDef>,
}

impl FingerprintDb {
    pub fn from_baseline(base: FingerprintDefinition) -> Self {
        let by_id = base
            .signatures
            .into_iter()
            .map(|s| (s.id.clone(), s))
            .collect();
        let web_ai = base
            .web_ai_signatures
            .into_iter()
            .map(|s| (s.domain.clone(), s))
            .collect();
        let installed_apps = base
            .installed_app_signatures
            .into_iter()
            .map(|s| (s.id.clone(), s))
            .collect();
        Self {
            version: base.definition_version,
            by_id,
            web_ai,
            installed_apps,
        }
    }

    pub fn apply(&mut self, def: FingerprintDefinition) -> anyhow::Result<()> {
        match def.kind {
            DefinitionKind::Full => {
                self.by_id = def
                    .signatures
                    .into_iter()
                    .map(|s| (s.id.clone(), s))
                    .collect();
                self.web_ai = def
                    .web_ai_signatures
                    .into_iter()
                    .map(|s| (s.domain.clone(), s))
                    .collect();
                self.installed_apps = def
                    .installed_app_signatures
                    .into_iter()
                    .map(|s| (s.id.clone(), s))
                    .collect();
            }
            DefinitionKind::Delta => {
                if def.base_version != Some(self.version) {
                    anyhow::bail!(
                        "delta base {:?} != current {} โ€” เธ•เนเธญเธเธ”เธถเธ full",
                        def.base_version,
                        self.version
                    );
                }
                for sig in def.signatures {
                    self.by_id.insert(sig.id.clone(), sig);
                }
                for sig in def.web_ai_signatures {
                    self.web_ai.insert(sig.domain.clone(), sig);
                }
                for sig in def.installed_app_signatures {
                    self.installed_apps.insert(sig.id.clone(), sig);
                }
                for id in &def.removed_ids {
                    self.by_id.remove(id);
                    self.web_ai.remove(id);
                    self.installed_apps.remove(id);
                }
            }
        }
        self.version = def.definition_version;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(id: &str, rev: u32) -> AgentSignatureV2 {
        AgentSignatureV2 {
            id: id.into(),
            display_name: id.into(),
            agent_type: "cli_agent".into(),
            revision: rev,
            meta: SignatureMeta {
                author: "t".into(),
                description: "".into(),
                references: vec![],
                added_in: "1".into(),
                tags: vec![],
            },
            process_names: vec![],
            binary_hashes: vec![],
            config_paths: Default::default(),
            config_parsers: vec![],
            ports: vec![],
            port_probe: None,
            detection_logic: DetectionLogic::AnyOf,
            control_strategies: vec![],
            risk_weight: 0.5,
            cmd_patterns: vec![],
            exe_path_patterns: vec![],
            install_markers: vec![],
            cli_binaries: vec![],
            package_markers: vec![],
            env_markers: vec![],
            egress_hosts: vec![],
            vendor: None,
            product: None,
            capability_tags: vec![],
            signal_weights: None,
        }
    }

    #[test]
    fn delta_adds_and_removes() {
        let base = FingerprintDefinition {
            schema_version: "v2".into(),
            definition_version: 1,
            released_at: "".into(),
            min_engine_version: "1.0.0".into(),
            kind: DefinitionKind::Full,
            base_version: None,
            signatures: vec![sig("ollama", 1)],
            removed_ids: vec![],
            catalog_hash: "".into(),
            model_classifier: None,
            web_ai_signatures: vec![],
            installed_app_signatures: vec![],
            ai_process_hints: Default::default(),
            browser_processes: vec![],
        };
        let mut db = FingerprintDb::from_baseline(base);
        let delta = FingerprintDefinition {
            schema_version: "v2".into(),
            definition_version: 2,
            released_at: "".into(),
            min_engine_version: "1.0.0".into(),
            kind: DefinitionKind::Delta,
            base_version: Some(1),
            signatures: vec![sig("goose_cli", 1)],
            removed_ids: vec!["ollama".into()],
            catalog_hash: "".into(),
            model_classifier: None,
            web_ai_signatures: vec![],
            installed_app_signatures: vec![],
            ai_process_hints: Default::default(),
            browser_processes: vec![],
        };
        db.apply(delta).unwrap(); //
        assert!(db.by_id.contains_key("goose_cli"));
        assert!(!db.by_id.contains_key("ollama"));
        assert_eq!(db.version, 2);
    }

    #[test]
    fn delta_rejects_wrong_base() {
        let mut db = FingerprintDb {
            version: 1,
            by_id: std::collections::HashMap::new(),
            web_ai: std::collections::HashMap::new(),
            installed_apps: std::collections::HashMap::new(),
        };
        let bad = FingerprintDefinition {
            schema_version: "v2".into(),
            definition_version: 7,
            released_at: "".into(),
            min_engine_version: "1.0.0".into(),
            kind: DefinitionKind::Delta,
            base_version: Some(3),
            signatures: vec![],
            removed_ids: vec![],
            catalog_hash: "bad".into(),
            model_classifier: None,
            web_ai_signatures: vec![],
            installed_app_signatures: vec![],
            ai_process_hints: Default::default(),
            browser_processes: vec![],
        };
        assert!(db.apply(bad).is_err());
    }
}
