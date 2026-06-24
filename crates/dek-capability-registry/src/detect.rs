use crate::{CapabilityStatus, PepCapability};
use dek_domain_schema::control_level::ControlLevel;

pub fn detect_pep_capabilities() -> Vec<PepCapability> {
    let mut caps = vec![
        PepCapability {
            r#type: "mcp-stdio".into(),
            transports: vec!["stdio".into()],
            control_level: ControlLevel::Enforce,
            status: CapabilityStatus::Available,
            status_reason: None,
        },
        PepCapability {
            r#type: "mcp-http".into(),
            transports: vec!["http".into()],
            control_level: ControlLevel::Enforce,
            status: CapabilityStatus::Available,
            status_reason: None,
        },
    ];

    #[cfg(target_os = "linux")]
    caps.push(detect_ebpf());

    #[cfg(target_os = "windows")]
    caps.push(detect_windows_wfp());

    #[cfg(target_os = "macos")]
    caps.push(detect_macos_nefilter());

    caps
}

#[cfg(target_os = "linux")]
fn detect_ebpf() -> PepCapability {
    let has_bpf = std::path::Path::new("/sys/fs/bpf").exists();
    let has_root = std::env::var("USER").unwrap_or_default() == "root";

    let (status, reason, level) = if !has_bpf {
        (
            CapabilityStatus::MissingDependencies,
            Some("BPF filesystem not mounted".to_string()),
            ControlLevel::Observe,
        )
    } else if !has_root {
        (
            CapabilityStatus::PermissionDenied,
            Some("Root privileges required for eBPF".to_string()),
            ControlLevel::Observe,
        )
    } else {
        (CapabilityStatus::Available, None, ControlLevel::Enforce)
    };

    PepCapability {
        r#type: "linux-ebpf".into(),
        transports: vec!["ebpf".into()],
        control_level: level,
        status,
        status_reason: reason,
    }
}

#[cfg(target_os = "windows")]
fn detect_windows_wfp() -> PepCapability {
    // Check if BFE service is running
    let output = std::process::Command::new("sc")
        .arg("query")
        .arg("BFE")
        .output();
    let (status, reason) = match output {
        Ok(out) if String::from_utf8_lossy(&out.stdout).contains("RUNNING") => {
            (CapabilityStatus::Available, None)
        }
        Ok(_) => (
            CapabilityStatus::MissingDependencies,
            Some("BFE service not running".to_string()),
        ),
        Err(e) => (
            CapabilityStatus::NotSupported,
            Some(format!("Could not query BFE: {}", e)),
        ),
    };

    PepCapability {
        r#type: "windows-wfp".into(),
        transports: vec!["wfp".into()],
        control_level: ControlLevel::Observe,
        status,
        status_reason: reason,
    }
}

#[cfg(target_os = "macos")]
fn detect_macos_nefilter() -> PepCapability {
    // Check system extension status
    let output = std::process::Command::new("systemextensionsctl")
        .arg("list")
        .output();
    let (status, reason) = match output {
        Ok(out) if String::from_utf8_lossy(&out.stdout).contains("com.pollen.nefilter") => {
            (CapabilityStatus::Available, None)
        }
        _ => (
            CapabilityStatus::MissingDependencies,
            Some("Network Extension not installed or active".to_string()),
        ),
    };

    PepCapability {
        r#type: "macos-nefilter".into(),
        transports: vec!["nefilter".into()],
        control_level: ControlLevel::Observe,
        status,
        status_reason: reason,
    }
}
