pub struct SuggesterConfig {
    pub cost_alert_threshold_usd: f64,
}

impl Default for SuggesterConfig {
    fn default() -> Self {
        Self {
            cost_alert_threshold_usd: 25.0,
        }
    }
}
