// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use rcgen::{CertificateParams, KeyPair, SanType};
use std::fs;

fn main() {
    let _ca_key_pem = fs::read_to_string("certs/root_ca.key").unwrap();
    let _ca_cert_pem = fs::read_to_string("certs/root_ca.crt").unwrap();
    
    // In rcgen 0.13, does CertificateSigningRequest exist? Let's check:
    // let csr = rcgen::CertificateSigningRequest::from_pem("...");
}

