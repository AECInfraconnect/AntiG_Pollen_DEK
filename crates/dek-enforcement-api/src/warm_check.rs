use dek_domain_schema::user_event::LocalizedText;
use async_trait::async_trait;

pub struct WarmCheckCtx {
    // Basic context for warm checks
}

pub enum WarmCheckResult {
    Ok,
    Degraded { reason: LocalizedText },
    Failed { reason: LocalizedText },
}

#[async_trait]
pub trait WarmCheck: Send + Sync {
    fn method_id(&self) -> &str;
    async fn verify(&self, ctx: &WarmCheckCtx) -> WarmCheckResult;
}

