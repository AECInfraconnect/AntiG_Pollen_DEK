use crate::PepCapability;
use dek_domain_schema::control_level::ControlLevel;

pub fn detect_pep_capabilities() -> Vec<PepCapability> {
    let mut caps = vec![
        PepCapability {
            r#type: "mcp-stdio".into(),
            transports: vec!["stdio".into()],
            control_level: ControlLevel::Enforce,
        },
        PepCapability {
            r#type: "mcp-http".into(),
            transports: vec!["http".into()],
            control_level: ControlLevel::Enforce,
        },
    ];

    #[cfg(target_os = "linux")]
    caps.push(detect_ebpf());

    #[cfg(target_os = "windows")]
    caps.push(PepCapability {
        r#type: "windows-wfp".into(),
        transports: vec!["wfp".into()],
        control_level: ControlLevel::Observe,
    });

    #[cfg(target_os = "macos")]
    caps.push(PepCapability {
        r#type: "macos-nefilter".into(),
        transports: vec!["nefilter".into()],
        control_level: ControlLevel::Observe,
    });

    caps
}

#[cfg(target_os = "linux")]
fn detect_ebpf() -> PepCapability {
    // Fake probe for demo
    let has_bpf = std::path::Path::new("/sys/fs/bpf").exists();
    PepCapability {
        r#type: "linux-ebpf".into(),
        transports: vec!["ebpf".into()],
        control_level: if has_bpf {
            ControlLevel::Enforce
        } else {
            ControlLevel::Observe
        },
    }
}
