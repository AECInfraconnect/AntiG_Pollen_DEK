fn main() {
    let p = rcgen::CertificateParams::new(vec!["device".to_string()]);
    let c = rcgen::Certificate::from_params(p).unwrap();
    let cert_pem = c.serialize_pem().unwrap();
    let key_pem = c.serialize_private_key_pem();
    let mut pem = cert_pem.into_bytes();
    pem.extend_from_slice(b"\n");
    pem.extend_from_slice(key_pem.as_bytes());
    let id = reqwest::Identity::from_pem(&pem).unwrap();
    println!("OK");
}
