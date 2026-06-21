// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

//! dek-resilience — SaaS-scale fail-closed primitives for the PEP.
pub mod breaker;
pub mod admission;

pub use admission::{AdmissionControl, AdmitPermit};
pub use breaker::{Admit, CircuitBreaker, CircuitConfig};

