// SPDX-License-Identifier: Apache-2.0
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
#[cfg(target_os = "linux")]
use std::time::Duration;
use tokio_util::sync::CancellationToken;

/// อัปเดตทุกครั้งที่ supervisor loop ทำงานครบรอบ (heartbeat ภายใน).
#[derive(Clone, Default)]
pub struct Liveness(Arc<AtomicU64>);

impl Liveness {
    pub fn tick(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.0.store(now, Ordering::Relaxed);
    }
    #[cfg(target_os = "linux")]
    fn last_tick_secs(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

/// อ่าน WatchdogSec ที่ systemd ตั้งให้ (env `WATCHDOG_USEC`) แล้ว ping ที่ครึ่งหนึ่งของช่วง.
/// ping เฉพาะเมื่อ supervisor ยัง "มีชีวิต" จริง (liveness tick ภายใน 2*interval).
#[cfg(target_os = "linux")]
pub fn spawn(liveness: Liveness, cancel: CancellationToken) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        // sd_notify::watchdog_enabled() คืน Some(interval) ถ้า systemd เปิด watchdog
        let interval = match sd_notify::watchdog_enabled(false, false) {
            Ok(Some(usec)) => Duration::from_micros(usec / 2), // ping ที่ครึ่งช่วง (best practice)
            _ => return, // ไม่ได้รันใต้ systemd watchdog → ไม่ทำอะไร
        };
        let mut ticker = tokio::time::interval(interval);
        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = ticker.tick() => {
                    let stale = now_secs().saturating_sub(liveness.last_tick_secs());
                    if stale < (2 * interval.as_secs()).max(2) {
                        // healthy → แจ้ง systemd ว่ายังไหว
                        let _ = sd_notify::notify(false, &[sd_notify::NotifyState::Watchdog]);
                    } else {
                        // supervisor ค้างจริง → "อย่า" ping ปล่อยให้ systemd restart (fail-fast)
                        tracing::error!(stale_secs = stale, "supervisor heartbeat stale; withholding watchdog ping");
                    }
                }
            }
        }
    })
}

#[cfg(not(target_os = "linux"))]
pub fn spawn(_liveness: Liveness, _cancel: CancellationToken) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {})
}

#[cfg(target_os = "linux")]
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
