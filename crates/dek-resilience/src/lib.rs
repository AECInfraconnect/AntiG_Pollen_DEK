//! dek-resilience — SaaS-scale fail-closed primitives for the PEP.
pub mod admission;
pub mod breaker;
pub mod rate_limit;

pub use admission::{AdmissionControl, AdmitPermit};
pub use breaker::{Admit, CircuitBreaker, CircuitConfig};
pub use rate_limit::{RateDecision, RateLimiter};
