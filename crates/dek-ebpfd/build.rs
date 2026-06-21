use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    
    // eBPF is only applicable on Linux
    if target_os != "linux" {
        return;
    }

    if let Err(e) = aya_build::build_ebpf() {
        println!("cargo:warning=Failed to build eBPF programs: {}", e);
    }
}
