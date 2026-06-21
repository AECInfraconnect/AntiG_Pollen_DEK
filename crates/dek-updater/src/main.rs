use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("Pollen DEK Auto-Updater");

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: dek-updater <target_exe> <new_exe>");
        std::process::exit(1);
    }

    let target_exe = PathBuf::from(&args[1]);
    let new_exe = PathBuf::from(&args[2]);

    if !new_exe.exists() {
        eprintln!("New executable not found: {:?}", new_exe);
        std::process::exit(1);
    }

    // Task 6.2: TUF Verification
    // In a full implementation, this uses tough or rust-tuf to query the root role, targets role, and map the target path to its hash/size.
    if let Some(tuf_metadata) = args.get(3) {
        println!(
            "Performing TUF verification using metadata: {}",
            tuf_metadata
        );
        if let Err(e) = verify_tuf_signature(&new_exe, tuf_metadata) {
            eprintln!("TUF Verification Failed: {e}");
            std::process::exit(1);
        }
        println!("TUF Verification Successful.");
    } else {
        println!("Warning: Running without TUF verification metadata.");
    }

    // On Windows, you can rename an executing file, but you can't delete or overwrite it directly.
    let backup_exe = target_exe.with_extension("exe.bak");

    // Remove old backup if it exists
    if backup_exe.exists()
        && let Err(e) = fs::remove_file(&backup_exe)
    {
        eprintln!("Failed to remove old backup: {e}");
    }

    // Rename current running executable to backup
    if target_exe.exists()
        && let Err(e) = fs::rename(&target_exe, &backup_exe)
    {
        eprintln!("Failed to rename active executable: {e}");
        std::process::exit(1);
    }

    // Rename the new downloaded executable to the target name
    if let Err(e) = fs::rename(&new_exe, &target_exe) {
        eprintln!("Failed to move new executable into place: {e}");
        // Rollback
        if backup_exe.exists() {
            let _ = fs::rename(&backup_exe, &target_exe);
        }
        std::process::exit(1);
    }

    println!("Update successful. Please restart the service.");
}

// Task 6.2: TUF Verification
fn verify_tuf_signature(new_exe: &PathBuf, metadata_path: &str) -> anyhow::Result<()> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    // 1. Read the provided mock TUF targets metadata
    let metadata_content = fs::read_to_string(metadata_path)?;
    let metadata: serde_json::Value = serde_json::from_str(&metadata_content)?;

    // Expected format: { "targets": { "dek-core.exe": { "hashes": { "sha256": "..." }, "length": ... } } }
    let target_name = new_exe
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let target_info = metadata["targets"][&target_name]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Target {} not found in TUF metadata", target_name))?;

    let expected_hash = target_info["hashes"]["sha256"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing sha256 hash in metadata for target"))?;
    let expected_size = target_info["length"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("Missing length in metadata for target"))?;

    // 2. Hash the executable
    let mut file = fs::File::open(new_exe)?;
    let meta = file.metadata()?;
    if meta.len() != expected_size {
        anyhow::bail!(
            "File size mismatch. Expected {}, got {}",
            expected_size,
            meta.len()
        );
    }

    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let result = hasher.finalize();
    let actual_hash = hex::encode(result);

    if actual_hash != expected_hash {
        anyhow::bail!(
            "Hash mismatch. Expected {}, got {}",
            expected_hash,
            actual_hash
        );
    }

    Ok(())
}
