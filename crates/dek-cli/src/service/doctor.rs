use anyhow::Result;
use std::path::Path;

pub fn run() -> Result<()> {
    println!("Pollen DEK Diagnostics");
    println!("----------------------");

    let bootstrap_path = dek_config::paths::get_bootstrap_path();
    check_file("Bootstrap Config", &bootstrap_path);

    let config_dir = dek_config::paths::get_config_dir();
    let client_key = config_dir.join("certs").join("client.key");
    check_file("Client Private Key", &client_key);

    let client_cert = config_dir.join("certs").join("client.crt");
    check_file("Client Certificate", &client_cert);

    let ca_cert = config_dir.join("certs").join("root_ca.crt");
    check_file("Root CA Bundle", &ca_cert);

    println!("\nKeystore:");
    let ks = dek_keystore::get_keystore();
    if ks.load_key("mtls_client_key").is_ok() {
        println!("  mtls_client_key: OK");
    } else {
        println!("  mtls_client_key: NOT FOUND");
    }

    Ok(())
}

fn check_file(name: &str, path: &Path) {
    if path.exists() {
        println!("  {}: FOUND ({})", name, path.display());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(path) {
                let mode = meta.permissions().mode();
                if (mode & 0o077) != 0 {
                    println!("    WARNING: Permissions are too open ({:o})", mode);
                } else {
                    println!("    Permissions: OK");
                }
            }
        }
    } else {
        println!("  {}: NOT FOUND", name);
    }
}
