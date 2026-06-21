#[derive(Debug, Clone)]
pub struct AgentBaseline {
    pub average_requests_per_hour: f64,
    pub max_cost_per_day: f64,
}

#[derive(Debug, Clone)]
pub struct TrustScore {
    pub agent_id: String,
    pub score: i32, // 0-100
}

#[derive(Debug, PartialEq, Eq)]
pub enum TrustAction {
    Normal,
    RequireApproval,
    KillSwitch,
}

pub fn enforce_trust(score: &TrustScore) -> TrustAction {
    if score.score < 30 {
        TrustAction::KillSwitch
    } else if score.score < 60 {
        TrustAction::RequireApproval
    } else {
        TrustAction::Normal
    }
}
