use anyhow::{Context, Result};
use rcgen::{Certificate, CertificateParams, DnType, IsCa};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let certs_dir = Path::new("../../certs");
    if !certs_dir.exists() {
        fs::create_dir_all(certs_dir).context("Failed to create certs directory")?;
    }

    // 1. Generate Root CA
    let mut root_params = CertificateParams::new(vec!["Pollen Cloud Root CA".to_string()]);
    root_params.is_ca = IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    root_params
        .distinguished_name
        .push(DnType::OrganizationName, "Pollen DEK Project");
    root_params
        .distinguished_name
        .push(DnType::CommonName, "Pollen Cloud Root CA");
    let root_cert = Certificate::from_params(root_params)?;

    fs::write(certs_dir.join("root_ca.crt"), root_cert.serialize_pem()?)?;
    fs::write(
        certs_dir.join("root_ca.key"),
        root_cert.serialize_private_key_pem(),
    )?;
    println!("Root CA generated.");

    // 2. Generate Server Certificate (mock-cloud)
    let mut server_params =
        CertificateParams::new(vec!["localhost".to_string(), "127.0.0.1".to_string()]);
    server_params
        .distinguished_name
        .push(DnType::OrganizationName, "Pollen DEK Project");
    server_params
        .distinguished_name
        .push(DnType::CommonName, "Pollen Mock Cloud Server");
    let server_cert = Certificate::from_params(server_params)?;

    let server_pem = server_cert.serialize_pem_with_signer(&root_cert)?;
    fs::write(certs_dir.join("server.crt"), server_pem)?;
    fs::write(
        certs_dir.join("server.key"),
        server_cert.serialize_private_key_pem(),
    )?;
    println!("Server Certificate generated.");

    // 3. Generate Client Certificate (DEK Telemetry / Sync)
    let mut client_params = CertificateParams::new(vec!["dek-client".to_string()]);
    client_params
        .distinguished_name
        .push(DnType::OrganizationName, "Pollen DEK Project");
    client_params
        .distinguished_name
        .push(DnType::CommonName, "DEK Edge Client");
    let client_cert = Certificate::from_params(client_params)?;

    let client_pem = client_cert.serialize_pem_with_signer(&root_cert)?;
    fs::write(certs_dir.join("client.crt"), client_pem)?;
    fs::write(
        certs_dir.join("client.key"),
        client_cert.serialize_private_key_pem(),
    )?;
    println!("Client Certificate generated.");

    println!("All MTLS mock certificates successfully generated in `certs/`.");
    Ok(())
}
