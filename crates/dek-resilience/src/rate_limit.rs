use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

/// token-bucket แบบเบา ต่อ key (เช่น "agent:tool")
pub struct RateLimiter {
    buckets: Mutex<HashMap<String, Bucket>>,
    capacity: f64,
    refill_per_sec: f64,
}

struct Bucket {
    tokens: f64,
    last: Instant,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RateDecision {
    Allow,
    Throttled,
}

impl RateLimiter {
    pub fn new(capacity: f64, refill_per_sec: f64) -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
            capacity,
            refill_per_sec,
        }
    }

    pub fn check(&self, key: &str) -> RateDecision {
        self.check_at(key, Instant::now())
    }

    pub fn check_at(&self, key: &str, now: Instant) -> RateDecision {
        let mut map = match self.buckets.lock() {
            Ok(m) => m,
            Err(_) => return RateDecision::Allow, // fail-open เฉพาะ rate limit (ไม่ใช่ security gate)
        };
        let b = map.entry(key.to_string()).or_insert(Bucket {
            tokens: self.capacity,
            last: now,
        });
        let elapsed = now.saturating_duration_since(b.last).as_secs_f64();
        b.tokens = (b.tokens + elapsed * self.refill_per_sec).min(self.capacity);
        b.last = now;
        if b.tokens >= 1.0 {
            b.tokens -= 1.0;
            RateDecision::Allow
        } else {
            RateDecision::Throttled
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn throttles_after_capacity() {
        let rl = RateLimiter::new(2.0, 0.0); // 2 calls แล้วหมด, ไม่ refill
        let now = Instant::now();
        assert_eq!(rl.check_at("agent1:db.query", now), RateDecision::Allow);
        assert_eq!(rl.check_at("agent1:db.query", now), RateDecision::Allow);
        assert_eq!(rl.check_at("agent1:db.query", now), RateDecision::Throttled);
    }
}
