// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

#![warn(clippy::print_stdout, clippy::print_stderr)]
#![deny(clippy::unwrap_used, clippy::expect_used)]
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;
use wasmtime::*;
use wasmtime_wasi::pipe::{MemoryInputPipe, MemoryOutputPipe};
use wasmtime_wasi::preview1;
use wasmtime_wasi::WasiCtxBuilder;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    #[error("Execution error: {0}")]
    Execution(String),
    #[error("Invalid input or output format: {0}")]
    InvalidFormat(String),
}

pub trait PluginHost: Send + Sync {
    fn invoke(&self, plugin_id: &str, input: Value) -> std::result::Result<Value, PluginError>;
}

pub struct WasmtimePluginHost {
    engine: Engine,
    instances_pre: HashMap<String, InstancePre<wasmtime_wasi::preview1::WasiP1Ctx>>,
}

use sha2::{Digest, Sha256};

impl WasmtimePluginHost {
    pub fn new(plugin_paths: HashMap<String, String>) -> anyhow::Result<Self> {
        let engine = Engine::default();
        let mut instances_pre = HashMap::new();

        let mut linker = Linker::new(&engine);
        preview1::add_to_linker_sync(&mut linker, |s| s)?;

        linker.func_wrap(
            "pollen_env",
            "ner_predict",
            |mut caller: Caller<'_, wasmtime_wasi::preview1::WasiP1Ctx>,
             ptr: u32,
             len: u32,
             out_ptr: u32,
             max_out_len: u32|
             -> u32 {
                let memory = match caller.get_export("memory") {
                    Some(Extern::Memory(m)) => m,
                    _ => return 0,
                };
                let mem_slice = memory.data(&caller);

                let start = ptr as usize;
                let end = start + len as usize;
                if end > mem_slice.len() {
                    return 0;
                }

                let text = match std::str::from_utf8(&mem_slice[start..end]) {
                    Ok(s) => s,
                    Err(_) => return 0,
                };

                // NER mock for gliner-pii-small local inference
                // In a real scenario, we'd make a blocking HTTP call to a local sidecar
                let redacted = text
                    .replace("John Doe", "[REDACTED_NAME]")
                    .replace("Acme Corp", "[REDACTED_ORG]");
                let redacted_bytes = redacted.as_bytes();

                let out_len = redacted_bytes.len().min(max_out_len as usize);

                let out_start = out_ptr as usize;
                let out_end = out_start + out_len;
                let mem_mut = memory.data_mut(&mut caller);
                if out_end > mem_mut.len() {
                    return 0;
                }
                mem_mut[out_start..out_end].copy_from_slice(&redacted_bytes[..out_len]);

                out_len as u32
            },
        )?;

        for (plugin_id, path) in plugin_paths {
            let p = std::path::Path::new(&path);
            if p.exists() {
                // Optionally verify sha256 hash if manifest.json exists in the same directory
                if let Some(parent) = p.parent() {
                    let manifest_path = parent.join("manifest.json");
                    if manifest_path.exists() {
                        let manifest_content = std::fs::read_to_string(&manifest_path)?;
                        if let Ok(manifest) =
                            serde_json::from_str::<HashMap<String, String>>(&manifest_content)
                        {
                            let file_name = p.file_name().unwrap_or_default().to_string_lossy();
                            if let Some(expected_hash) = manifest.get(file_name.as_ref()) {
                                let bytes = std::fs::read(p)?;
                                let mut hasher = Sha256::new();
                                hasher.update(&bytes);
                                let actual_hash = hex::encode(hasher.finalize());

                                if actual_hash != *expected_hash {
                                    anyhow::bail!(
                                        "SHA256 hash mismatch for plugin {}: expected {}, got {}",
                                        path,
                                        expected_hash,
                                        actual_hash
                                    );
                                }
                            }
                        }
                    }
                }

                let module = Module::from_file(&engine, &path).map_err(|e| {
                    anyhow::anyhow!("Failed to load plugin WASM module: {}: {}", path, e)
                })?;
                let instance_pre = linker.instantiate_pre(&module)?;
                instances_pre.insert(plugin_id, instance_pre);
            } else {
                tracing::warn!("Plugin path not found: {}", path);
            }
        }

        Ok(Self {
            engine,
            instances_pre,
        })
    }
}

impl PluginHost for WasmtimePluginHost {
    fn invoke(&self, plugin_id: &str, input: Value) -> std::result::Result<Value, PluginError> {
        let instance_pre = self
            .instances_pre
            .get(plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?;

        let input_str =
            serde_json::to_string(&input).map_err(|e| PluginError::InvalidFormat(e.to_string()))?;
        let stdin = MemoryInputPipe::new(bytes::Bytes::from(input_str.into_bytes()));
        let stdout = MemoryOutputPipe::new(10 * 1024 * 1024);

        let mut builder = WasiCtxBuilder::new();
        builder.stdin(stdin.clone());
        builder.stdout(stdout.clone());
        builder.inherit_stderr();
        let wasi = builder.build_p1();

        let mut store = Store::new(&self.engine, wasi);

        // Instantiate and run _start
        let instance = instance_pre
            .instantiate(&mut store)
            .map_err(|e| PluginError::Execution(format!("Instantiate failed: {}", e)))?;
        let func = instance
            .get_typed_func::<(), ()>(&mut store, "_start")
            .map_err(|e| PluginError::Execution(format!("Missing _start function: {}", e)))?;
        func.call(&mut store, ())
            .map_err(|e| PluginError::Execution(format!("Execution failed: {}", e)))?;

        // Read output produced by WASI guest via memory pipe
        let out_bytes = stdout.contents();
        let output_str = String::from_utf8_lossy(&out_bytes);

        let output_val: Value = serde_json::from_str(&output_str).map_err(|e| {
            PluginError::InvalidFormat(format!("Failed to parse output JSON: {}", e))
        })?;

        Ok(output_val)
    }
}
