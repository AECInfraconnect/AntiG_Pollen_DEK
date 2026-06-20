pub struct DiscoveryConfig {
    pub min_fingerprint_confidence: f64,
    pub cost_alert_threshold_usd: f64,
    pub default_retention_days: u32,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            min_fingerprint_confidence: 0.5,
            cost_alert_threshold_usd: 25.0,
            default_retention_days: 14,
        }
    }
}
