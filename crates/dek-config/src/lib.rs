use anyhow::{Context, Result};
use reqwest::{Certificate, Identity};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtlsConfig {
    pub client_cert_path: String,
    pub client_key_path: String,
    pub root_ca_path: String,
}

impl MtlsConfig {
    pub fn build_client(&self) -> Result<reqwest::Client> {
        let root_ca_der = fs::read(&self.root_ca_path).context("Failed to read root CA")?;
        let root_ca = Certificate::from_pem(&root_ca_der)?;

        let client_cert = fs::read(&self.client_cert_path).context("Failed to read client cert")?;
        let client_key = fs::read(&self.client_key_path).context("Failed to read client key")?;
        let mut id_pem = client_cert;
        id_pem.extend_from_slice(b"\n");
        id_pem.extend_from_slice(&client_key);
        let identity = Identity::from_pem(&id_pem)?;

        let client = reqwest::Client::builder()
            .add_root_certificate(root_ca)
            .identity(identity)
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        Ok(client)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    pub device_id: String,
    pub mtls: MtlsConfig,
}

impl BootstrapConfig {
    pub fn load_or_default(path: &str) -> Result<Self> {
        let p = Path::new(path);
        if p.exists() {
            let data = fs::read_to_string(p)?;
            let config: BootstrapConfig = serde_json::from_str(&data)?;
            Ok(config)
        } else {
            let default_config = Self {
                device_id: "device-001".to_string(),
                mtls: MtlsConfig {
                    client_cert_path: "certs/client.crt".to_string(),
                    client_key_path: "certs/client.key".to_string(),
                    root_ca_path: "certs/root_ca.crt".to_string(),
                },
            };
            let json_str = serde_json::to_string_pretty(&default_config)?;
            fs::write(p, json_str)?;
            Ok(default_config)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFgaConfig {
    pub endpoint: String,
    pub store_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CedarConfig {
    pub policy_src: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
    pub policy_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub openfga: Option<OpenFgaConfig>,
    pub cedar: Option<CedarConfig>,
    pub opa_wasm: Option<WasmConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpireServerConfig {
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DekConfig {
    pub device_id: String,
    pub tenant_id: String,
    pub mtls: MtlsConfig,
    pub spire_server: Option<SpireServerConfig>,
    pub policy_config: Option<PolicyConfig>,
}

impl DekConfig {
    pub async fn fetch_from_cloud(
        bootstrap: &BootstrapConfig,
        endpoint_base: &str,
    ) -> Result<Self> {
        let client = bootstrap.mtls.build_client()?;

        let url = format!("{}/config/{}", endpoint_base, bootstrap.device_id);
        println!("Fetching dynamic config from cloud over MTLS: {}", url);

        let res = client.get(&url).send().await?;
        if !res.status().is_success() {
            anyhow::bail!("Failed to fetch config. Status: {}", res.status());
        }

        let config: DekConfig = res.json().await?;
        Ok(config)
    }
}
