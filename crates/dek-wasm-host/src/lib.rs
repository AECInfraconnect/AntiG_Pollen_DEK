use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use wasmtime::*;
use wasmtime_wasi::preview1;
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::pipe::{MemoryInputPipe, MemoryOutputPipe};

pub trait PluginHost {
    fn invoke(&self, plugin_id: &str, input: Value) -> Result<Value>;
}

pub struct WasmtimePluginHost {
    engine: Engine,
    modules: HashMap<String, Module>,
}

impl WasmtimePluginHost {
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        let mut modules = HashMap::new();

        // Dynamically resolve WASM path based on debug vs release
        let paths = vec![
            "target/wasm32-wasip1/release/dummy_policy.wasm",
            "target/wasm32-wasip1/debug/dummy_policy.wasm",
            "../../target/wasm32-wasip1/release/dummy_policy.wasm",
            "../../target/wasm32-wasip1/debug/dummy_policy.wasm",
        ];

        let mut wasm_path = None;
        for p in paths {
            if std::path::Path::new(p).exists() {
                wasm_path = Some(p);
                break;
            }
        }

        // For testing, we use dummy_policy.wasm as the "pii-redactor" for now
        // In reality, this should be configurable
        if let Some(path) = wasm_path {
            let module =
                Module::from_file(&engine, path).context("Failed to load plugin WASM module")?;
            modules.insert("pii-redactor".to_string(), module);
        }

        Ok(Self { engine, modules })
    }
}

impl PluginHost for WasmtimePluginHost {
    fn invoke(&self, plugin_id: &str, input: Value) -> Result<Value> {
        let module = self
            .modules
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
        let mut linker = Linker::new(&self.engine);
        preview1::add_to_linker_sync(&mut linker, |s| s)?;

        // Instantiate and run _start
        let instance = linker.instantiate(&mut store, module)?;
        let func = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
        func.call(&mut store, ())?;

        // Read output produced by WASI guest via memory pipe
        let out_bytes = stdout.contents();
        let output_str = String::from_utf8_lossy(&out_bytes);

        let output_val: Value = serde_json::from_str(&output_str)?;

        Ok(output_val)
    }
}
