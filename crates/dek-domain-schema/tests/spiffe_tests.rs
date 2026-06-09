// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

#![allow(clippy::unwrap_used, clippy::expect_used)]
use dek_domain_schema::spiffe::{SpiffeBuilder, SpiffeId, validate_tenant_isolation};
use dek_domain_schema::tenant::TrustDomainStrategy;

#[test]
fn test_spiffe_parsing() {
    let id = SpiffeId::parse("spiffe://pollen.cloud/tenant/t-1/agent/a-1").unwrap();
    assert_eq!(id.trust_domain, "pollen.cloud");
    assert_eq!(id.path, "/tenant/t-1/agent/a-1");
    assert_eq!(id.to_uri(), "spiffe://pollen.cloud/tenant/t-1/agent/a-1");

    assert!(SpiffeId::parse("https://pollen.cloud").is_err());
    assert!(SpiffeId::parse("spiffe://").is_err());
}

#[test]
fn test_spiffe_builder_shared() {
    let builder = SpiffeBuilder::new(
        TrustDomainStrategy::Shared,
        "t-1".to_string(),
        "pollen.cloud".to_string(),
    );

    let agent_id = builder.build_agent_id("a-1");
    assert_eq!(
        agent_id.to_uri(),
        "spiffe://pollen.cloud/tenant/t-1/agent/a-1"
    );

    let device_id = builder.build_device_id("d-1");
    assert_eq!(
        device_id.to_uri(),
        "spiffe://pollen.cloud/tenant/t-1/device/d-1"
    );
}

#[test]
fn test_spiffe_builder_dedicated() {
    let builder = SpiffeBuilder::new(
        TrustDomainStrategy::Dedicated,
        "acme".to_string(),
        "pollen.cloud".to_string(),
    );

    let agent_id = builder.build_agent_id("a-1");
    assert_eq!(agent_id.to_uri(), "spiffe://acme.pollen.cloud/agent/a-1");
}

#[test]
fn test_spiffe_builder_custom() {
    let builder = SpiffeBuilder::new(
        TrustDomainStrategy::Custom("secure.acme.corp".to_string()),
        "acme".to_string(),
        "pollen.cloud".to_string(),
    );

    let device_id = builder.build_device_id("d-1");
    assert_eq!(device_id.to_uri(), "spiffe://secure.acme.corp/device/d-1");
}

#[test]
fn test_validate_tenant_isolation_shared() {
    let strategy = TrustDomainStrategy::Shared;
    let valid_id = SpiffeId::parse("spiffe://pollen.cloud/tenant/t-1/device/d-1").unwrap();

    // Valid
    assert!(validate_tenant_isolation(&valid_id, &strategy, "t-1", "pollen.cloud").is_ok());

    // Wrong tenant
    assert!(validate_tenant_isolation(&valid_id, &strategy, "t-2", "pollen.cloud").is_err());

    // Wrong trust domain
    let invalid_id = SpiffeId::parse("spiffe://evil.cloud/tenant/t-1/device/d-1").unwrap();
    assert!(validate_tenant_isolation(&invalid_id, &strategy, "t-1", "pollen.cloud").is_err());
}

#[test]
fn test_validate_tenant_isolation_dedicated() {
    let strategy = TrustDomainStrategy::Dedicated;
    let valid_id = SpiffeId::parse("spiffe://acme.pollen.cloud/device/d-1").unwrap();

    // Valid
    assert!(validate_tenant_isolation(&valid_id, &strategy, "acme", "pollen.cloud").is_ok());

    // Wrong tenant
    assert!(validate_tenant_isolation(&valid_id, &strategy, "other", "pollen.cloud").is_err());

    // Wrong trust domain (Shared structure used with Dedicated strategy)
    let invalid_id = SpiffeId::parse("spiffe://pollen.cloud/tenant/acme/device/d-1").unwrap();
    assert!(validate_tenant_isolation(&invalid_id, &strategy, "acme", "pollen.cloud").is_err());
}

#[test]
fn test_validate_tenant_isolation_custom() {
    let strategy = TrustDomainStrategy::Custom("corp.acme.internal".to_string());
    let valid_id = SpiffeId::parse("spiffe://corp.acme.internal/device/d-1").unwrap();

    // Valid
    assert!(validate_tenant_isolation(&valid_id, &strategy, "acme", "pollen.cloud").is_ok());

    // Wrong trust domain
    let invalid_id = SpiffeId::parse("spiffe://other.acme.internal/device/d-1").unwrap();
    assert!(validate_tenant_isolation(&invalid_id, &strategy, "acme", "pollen.cloud").is_err());
}

