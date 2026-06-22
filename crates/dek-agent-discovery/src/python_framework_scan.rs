use crate::model::*;
use std::path::{Path, PathBuf};

const AI_FRAMEWORK_PACKAGES: &[(&str, &str)] = &[
    ("langchain", "LangChain"),
    ("llama_index", "LlamaIndex"),
    ("crewai", "CrewAI"),
    ("autogen", "AutoGen"),
    ("smolagents", "smolagents"),
    ("dspy", "DSPy"),
    ("haystack", "Haystack"),
    ("openai", "OpenAI SDK"),
    ("anthropic", "Anthropic SDK"),
    ("ollama", "Ollama Python"),
];

pub fn scan_python_frameworks() -> Vec<DiscoveryEvidenceV2> {
    let mut evidence = Vec::new();
    for venv in find_python_environments() {
        let site_packages = get_site_packages(&venv);
        for (pkg, name) in AI_FRAMEWORK_PACKAGES {
            if has_package(&site_packages, pkg) {
                evidence.push(make_framework_evidence(name, pkg, &venv));
            }
        }
    }
    evidence
}

fn get_site_packages(venv: &Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        venv.join("Lib").join("site-packages")
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(entries) = std::fs::read_dir(venv.join("lib")) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.starts_with("python") {
                        return entry.path().join("site-packages");
                    }
                }
            }
        }
        venv.join("lib").join("python3").join("site-packages")
    }
}

fn has_package(site_packages: &Path, pkg: &str) -> bool {
    site_packages.join(pkg).is_dir()
        || site_packages.join(format!("{}.py", pkg)).is_file()
        || has_dist_info(site_packages, pkg)
}

fn has_dist_info(site_packages: &Path, pkg: &str) -> bool {
    let prefix = pkg.replace('-', "_");
    if let Ok(entries) = std::fs::read_dir(site_packages) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                let lower_name = name.to_lowercase();
                if lower_name.starts_with(&prefix) && lower_name.ends_with(".dist-info") {
                    return true;
                }
            }
        }
    }
    false
}

fn expand_tilde(path: &str, home: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        PathBuf::from(home).join(stripped)
    } else {
        PathBuf::from(path)
    }
}

fn find_python_environments() -> Vec<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_default();
    let mut envs = vec![];
    for p in [
        "~/.virtualenvs",
        "~/miniconda3/envs",
        "~/anaconda3/envs",
        "~/.conda/envs",
        "~/.pyenv/versions",
    ] {
        let expanded = expand_tilde(p, &home);
        if let Ok(rd) = std::fs::read_dir(&expanded) {
            for e in rd.flatten() {
                if e.path().is_dir() {
                    envs.push(e.path());
                }
            }
        }
    }
    envs
}

fn make_framework_evidence(name: &str, pkg: &str, venv: &Path) -> DiscoveryEvidenceV2 {
    DiscoveryEvidenceV2 {
        evidence_id: uuid::Uuid::new_v4().to_string(),
        source: EvidenceSource::PythonFramework,
        confidence: 0.9,
        observed_at: chrono::Utc::now().to_rfc3339(),
        privacy_class: PrivacyClass::InternalMetadata,
        redacted: true,
        data: serde_json::json!({
            "framework": name,
            "package": pkg,
            "venv_path": venv.to_string_lossy(),
        }),
        merge_key: Some(format!("py_framework:{}", pkg)),
        source_path_hash: Some(crate::redaction::sha256_string(&venv.to_string_lossy())),
        source_path_redacted: Some(crate::redaction::redact_path_for_ui(
            &venv.to_string_lossy(),
        )),
    }
}
