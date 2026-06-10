#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityHealth {
    Healthy,
    RenewDegraded,
    Expired,
}

pub fn classify(remaining_secs: i64, last_renew_ok: bool) -> IdentityHealth {
    if last_renew_ok {
        IdentityHealth::Healthy
    } else if remaining_secs > 60 {
        IdentityHealth::RenewDegraded
    } else {
        IdentityHealth::Expired
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify() {
        // renew สำเร็จ → Healthy
        assert_eq!(classify(100, true), IdentityHealth::Healthy);
        assert_eq!(classify(10, true), IdentityHealth::Healthy);

        // renew ล้ม แต่ SVID เหลือ > 60s → RenewDegraded
        assert_eq!(classify(61, false), IdentityHealth::RenewDegraded);
        assert_eq!(classify(3600, false), IdentityHealth::RenewDegraded);

        // SVID เหลือ ≤ 60s + renew ล้ม → Expired
        assert_eq!(classify(60, false), IdentityHealth::Expired);
        assert_eq!(classify(0, false), IdentityHealth::Expired);
        assert_eq!(classify(-10, false), IdentityHealth::Expired);
    }
}
