use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use wasmtime::*;
use wasmtime_wasi::pipe::{MemoryInputPipe, MemoryOutputPipe};
use wasmtime_wasi::preview1;
use wasmtime_wasi::WasiCtxBuilder;

pub trait PluginHost {
    fn invoke(&self, plugin_id: &str, input: Value) -> Result<Value>;
}

pub struct WasmtimePluginHost {
    engine: Engine,
    instances_pre: HashMap<String, InstancePre<wasmtime_wasi::preview1::WasiP1Ctx>>,
}

impl WasmtimePluginHost {
    pub fn new(plugin_paths: HashMap<String, String>) -> Result<Self> {
        let engine = Engine::default();
        let mut instances_pre = HashMap::new();

        let mut linker = Linker::new(&engine);
        preview1::add_to_linker_sync(&mut linker, |s| s)?;

        for (plugin_id, path) in plugin_paths {
            if std::path::Path::new(&path).exists() {
                let module = Module::from_file(&engine, &path)
                    .with_context(|| format!("Failed to load plugin WASM module: {}", path))?;
                let instance_pre = linker.instantiate_pre(&module)?;
                instances_pre.insert(plugin_id, instance_pre);
            } else {
                eprintln!("Plugin path not found: {}", path);
            }
        }

        Ok(Self {
            engine,
            instances_pre,
        })
    }
}

impl PluginHost for WasmtimePluginHost {
    fn invoke(&self, plugin_id: &str, input: Value) -> Result<Value> {
        let instance_pre = self
            .instances_pre
            .get(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Unknown or uninitialized plugin: {}", plugin_id))?;

        let input_str = serde_json::to_string(&input)?;
        let stdin = MemoryInputPipe::new(bytes::Bytes::from(input_str.into_bytes()));
        let stdout = MemoryOutputPipe::new(10 * 1024 * 1024);

        let mut builder = WasiCtxBuilder::new();
        builder.stdin(stdin.clone());
        builder.stdout(stdout.clone());
        builder.inherit_stderr();
        let wasi = builder.build_p1();

        let mut store = Store::new(&self.engine, wasi);

        // Instantiate and run _start
        let instance = instance_pre.instantiate(&mut store)?;
        let func = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
        func.call(&mut store, ())?;

        // Read output produced by WASI guest via memory pipe
        let out_bytes = stdout.contents();
        let output_str = String::from_utf8_lossy(&out_bytes);

        let output_val: Value = serde_json::from_str(&output_str)?;

        Ok(output_val)
    }
}
