#![no_main]
use libfuzzer_sys::fuzz_target;
use dek_policy_presets::model::{DeployPresetRequest, ControlMode};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(s) {
            // Get a valid built-in preset to fuzz against
            if let Some(preset) = dek_policy_presets::catalog::builtin_presets().into_iter().next() {
                let req = DeployPresetRequest {
                    preset_id: preset.id.clone(),
                    preset_version: None,
                    control_mode: ControlMode::Observe,
                    selected_pep_types: vec![],
                    targets: Default::default(),
                    params: std::collections::BTreeMap::new(),
                    dry_run_first: false,
                    pdp_route: None,
                };
                
                // Fuzz passing malicious strings as param values
                if let Some(obj) = json.as_object() {
                    let mut req = req;
                    for (k, v) in obj {
                        req.params.insert(k.clone(), v.clone());
                    }
                    let _ = dek_policy_presets::render::render(&preset, &req);
                }
            }
        }
    }
});
