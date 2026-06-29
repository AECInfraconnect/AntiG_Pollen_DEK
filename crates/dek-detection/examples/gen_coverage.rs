//! Emit coverage.yaml from the installed detection packs.
//! Run: cargo run --example gen_coverage -- detections/core-v1
use dek_detection::{build_coverage, coverage::coverage_to_yaml, load_pack_dir};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../../contracts/detections/packs/core-v1".into());
    let rules = load_pack_dir(&dir)?;
    let cov = build_coverage(&rules);
    let yaml = coverage_to_yaml(&cov)?;
    std::io::stdout().write_all(yaml.as_bytes())?;
    Ok(())
}
