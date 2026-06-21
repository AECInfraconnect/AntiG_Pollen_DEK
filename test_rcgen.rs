// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use rcgen::{CertificateSigningRequest, KeyPair, CertificateParams};

fn main() {
    let _ = rcgen::CertificateSigningRequest::from("dummy");
}

