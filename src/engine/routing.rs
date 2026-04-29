#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingCandidate {
    pub provider: String,
    pub success_rate_bps: u16, // 0..=10000
    pub priority: u16,         // lower is better
    pub cost_bps: u16,         // lower is better
    pub healthy: bool,
}

/// Deterministic provider picker.
/// Order: healthy first, then success rate, then priority, then cost.
pub fn pick_provider(candidates: &[RoutingCandidate]) -> Option<String> {
    candidates
        .iter()
        .filter(|c| c.healthy)
        .max_by(|a, b| {
            a.success_rate_bps
                .cmp(&b.success_rate_bps)
                .then_with(|| b.priority.cmp(&a.priority))
                .then_with(|| b.cost_bps.cmp(&a.cost_bps))
        })
        .map(|c| c.provider.clone())
}

#[cfg(test)]
mod tests {
    use super::{pick_provider, RoutingCandidate};

    #[test]
    fn picks_highest_success_rate_when_all_healthy() {
        let picked = pick_provider(&[
            RoutingCandidate {
                provider: "razorpay".into(),
                success_rate_bps: 9850,
                priority: 10,
                cost_bps: 200,
                healthy: true,
            },
            RoutingCandidate {
                provider: "payu".into(),
                success_rate_bps: 9920,
                priority: 30,
                cost_bps: 220,
                healthy: true,
            },
        ]);
        assert_eq!(picked.as_deref(), Some("payu"));
    }

    #[test]
    fn skips_unhealthy_provider_even_if_success_rate_higher() {
        let picked = pick_provider(&[
            RoutingCandidate {
                provider: "cashfree".into(),
                success_rate_bps: 9990,
                priority: 5,
                cost_bps: 190,
                healthy: false,
            },
            RoutingCandidate {
                provider: "juspay".into(),
                success_rate_bps: 9800,
                priority: 20,
                cost_bps: 210,
                healthy: true,
            },
        ]);
        assert_eq!(picked.as_deref(), Some("juspay"));
    }
}
