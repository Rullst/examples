//! Audited view of the adapters exported by `rullst-capital` 13.0.0-alpha.3.
//!
//! Note on framework lifecycle:
//! In Rullst 13.0.0-alpha.3, the 9 deprecated v12 adapters (Paddle, Lemon Squeezy,
//! Polar, Razorpay, Mercado Pago, Alipay, Coinbase Commerce, PicPay, Wise) and NFS-e
//! were removed, keeping ONLY Stripe and InfinitePay.
//! "Exported" is deliberately not synonymous with "safe live checkout".
//! The status text below records the operation boundary used by this example.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayKind {
    Billing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveCheckoutStatus {
    /// Enabled in provider sandbox and audited live checkout by this example.
    Enabled,
    /// The v13 adapter intentionally rejects live plan-only checkout (experimental fail-closed).
    ExperimentalFailClosed,
}

#[derive(Debug, Clone, Copy)]
pub struct GatewayInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: GatewayKind,
    pub status: LiveCheckoutStatus,
    pub test_note: &'static str,
}

pub const GATEWAYS: [GatewayInfo; 2] = [
    GatewayInfo {
        id: "stripe",
        name: "Stripe",
        kind: GatewayKind::Billing,
        status: LiveCheckoutStatus::Enabled,
        test_note: "Audited primary gateway in Rullst v13.0.0-alpha.3. The application-owned path verifies an active one-time BRL 1.00 Price and reconciles signed webhooks.",
    },
    GatewayInfo {
        id: "infinitepay",
        name: "InfinitePay",
        kind: GatewayKind::Billing,
        status: LiveCheckoutStatus::ExperimentalFailClosed,
        test_note: "Experimental adapter retained in rullst-capital 13.0.0-alpha.3; live checkout and callbacks remain fail-closed (UnsupportedOperation).",
    },
];

pub fn gateway(id: &str) -> Option<&'static GatewayInfo> {
    GATEWAYS.iter().find(|gateway| gateway.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn catalog_has_two_v13_unique_adapters() {
        let ids = GATEWAYS
            .iter()
            .map(|gateway| gateway.id)
            .collect::<HashSet<_>>();
        assert_eq!(GATEWAYS.len(), 2);
        assert_eq!(ids.len(), 2);
        assert_eq!(
            GATEWAYS
                .iter()
                .filter(|gateway| gateway.kind == GatewayKind::Billing)
                .count(),
            2
        );
    }

    #[test]
    fn only_reviewed_example_paths_are_enabled() {
        let enabled = GATEWAYS
            .iter()
            .filter(|gateway| gateway.status == LiveCheckoutStatus::Enabled)
            .map(|gateway| gateway.id)
            .collect::<Vec<_>>();
        assert_eq!(enabled, vec!["stripe"]);
    }
}
