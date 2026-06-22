// SPDX-License-Identifier: Apache-2.0
pub fn install_panic_hook() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info.location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "<unknown>".into());
        // Do not include the raw payload to prevent leaking PII/secrets.
        tracing::error!(target: "panic", location = %location, "FATAL panic; aborting");
        
        // Wait a bit to flush telemetry (no blocking flush API available, so just sleep briefly)
        std::thread::sleep(std::time::Duration::from_millis(500));
        prev(info);
    }));
}
