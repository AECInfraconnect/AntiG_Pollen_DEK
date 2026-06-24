use crate::{CapabilityStatus, PepCapability};
use dek_domain_schema::control_level::ControlLevel;

pub fn detect_pep_capabilities() -> Vec<PepCapability> {
    let mut caps = vec![
        PepCapability {
            r#type: "mcp-stdio".into(),
            transports: vec!["stdio".into()],
            control_level: ControlLevel::Enforce,
            status: CapabilityStatus::Ready,
            status_reason: None,
        },
        PepCapability {
            r#type: "mcp-http".into(),
            transports: vec!["http".into()],
            control_level: ControlLevel::Enforce,
            status: CapabilityStatus::Ready,
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
            CapabilityStatus::MissingBinary,
            Some(dek_domain_schema::deployment_session::LocalizedText {
                en: "BPF filesystem not mounted".to_string(),
                th: "ไม่ได้เมานต์ BPF filesystem บนเครื่องนี้".to_string(),
            }),
            ControlLevel::Observe,
        )
    } else if !has_root {
        (
            CapabilityStatus::MissingPermission,
            Some(dek_domain_schema::deployment_session::LocalizedText {
                en: "Root privileges required for eBPF".to_string(),
                th: "ต้องการสิทธิ์ Root เพื่อรัน eBPF".to_string(),
            }),
            ControlLevel::Observe,
        )
    } else {
        (CapabilityStatus::Ready, None, ControlLevel::Enforce)
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
            (CapabilityStatus::Ready, None)
        }
        Ok(_) => (
            CapabilityStatus::MissingDriver,
            Some(dek_domain_schema::deployment_session::LocalizedText {
                en: "BFE service not running".to_string(),
                th: "ไม่ได้เปิดใช้งานบริการ BFE".to_string(),
            }),
        ),
        Err(e) => (
            CapabilityStatus::UnsupportedOs,
            Some(dek_domain_schema::deployment_session::LocalizedText {
                en: format!("Could not query BFE: {}", e),
                th: format!("ไม่สามารถตรวจสอบบริการ BFE: {}", e),
            }),
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
            (CapabilityStatus::Ready, None)
        }
        _ => (
            CapabilityStatus::MissingDriver,
            Some(dek_domain_schema::deployment_session::LocalizedText {
                en: "Network Extension not installed or active".to_string(),
                th: "ไม่ได้ติดตั้งหรือไม่ได้เปิดใช้งาน Network Extension".to_string(),
            }),
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
