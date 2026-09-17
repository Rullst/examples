//! Audited view of the adapters exported by `rullst-capital` 12.0.0.
//!
//! "Exported" is deliberately not synonymous with "safe live checkout".
//! The status text below records the operation boundary used by this example.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayKind {
    Billing,
    Payout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveCheckoutStatus {
    /// Enabled in provider sandbox by this example after a server-side check.
    Enabled,
    /// Exported by v12, but not wired to this one-time product contract.
    ApplicationContractRequired,
    /// The v12 adapter issues an outbound checkout request, but its current
    /// provider contract has a known blocking mismatch.
    FrameworkFixRequired,
    /// The v12 adapter intentionally rejects live plan-only checkout.
    UnsupportedByV12,
    /// This adapter sends money and is not a customer checkout gateway.
    PayoutOnly,
}

#[derive(Debug, Clone, Copy)]
pub struct GatewayInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: GatewayKind,
    pub status: LiveCheckoutStatus,
    pub test_note: &'static str,
}

pub const GATEWAYS: [GatewayInfo; 11] = [
    GatewayInfo {
        id: "stripe",
        name: "Stripe",
        kind: GatewayKind::Billing,
        status: LiveCheckoutStatus::Enabled,
        test_note: "The application-owned sandbox path requires an active one-time BRL 1.00 Price; live money remains blocked.",
    },
    GatewayInfo {
        id: "razorpay",
        name: "Razorpay",
        kind: GatewayKind::Billing,
        status: LiveCheckoutStatus::ApplicationContractRequired,
        test_note: "The exported v12 path is subscription-oriented; this one-time report needs a separate provider contract and account review.",
    },
    GatewayInfo {
        id: "lemonsqueezy",
        name: "Lemon Squeezy",
        kind: GatewayKind::Billing,
        status: LiveCheckoutStatus::FrameworkFixRequired,
        test_note: "The v12 request hard-codes store ID 1 instead of accepting the merchant's store ID.",
    },
    GatewayInfo {
        id: "paddle",
        name: "Paddle",
        kind: GatewayKind::Billing,
        status: LiveCheckoutStatus::FrameworkFixRequired,
        test_note: "The v12 transaction payload does not match the current Paddle transaction contract; Paddle's BRL minimum is R$4.00.",
    },
    GatewayInfo {
        id: "polar",
        name: "Polar",
        kind: GatewayKind::Billing,
        status: LiveCheckoutStatus::FrameworkFixRequired,
        test_note: "The v12 adapter sends the former product_price_id checkout shape instead of the current products-based contract.",
    },
    GatewayInfo {
        id: "infinitepay",
        name: "InfinitePay",
        kind: GatewayKind::Billing,
        status: LiveCheckoutStatus::UnsupportedByV12,
        test_note: "Live plan-only checkout returns UnsupportedOperation in rullst-capital 12.0.0.",
    },
    GatewayInfo {
        id: "mercadopago",
        name: "Mercado Pago",
        kind: GatewayKind::Billing,
        status: LiveCheckoutStatus::UnsupportedByV12,
        test_note: "Live plan-only checkout and the body-only live webhook verifier are unavailable in v12.",
    },
    GatewayInfo {
        id: "coinbase",
        name: "Coinbase Commerce",
        kind: GatewayKind::Billing,
        status: LiveCheckoutStatus::UnsupportedByV12,
        test_note: "Live plan-only checkout returns UnsupportedOperation in rullst-capital 12.0.0.",
    },
    GatewayInfo {
        id: "picpay",
        name: "PicPay",
        kind: GatewayKind::Billing,
        status: LiveCheckoutStatus::UnsupportedByV12,
        test_note: "Live plan-only checkout returns UnsupportedOperation in rullst-capital 12.0.0.",
    },
    GatewayInfo {
        id: "alipay",
        name: "Alipay",
        kind: GatewayKind::Billing,
        status: LiveCheckoutStatus::UnsupportedByV12,
        test_note: "Live RSA2 checkout signing and live webhook verification are explicitly disabled in v12.",
    },
    GatewayInfo {
        id: "wise",
        name: "Wise",
        kind: GatewayKind::Payout,
        status: LiveCheckoutStatus::PayoutOnly,
        test_note: "Wise is a payout adapter. It sends funds to recipients; it is not used to receive SaaS checkout payments.",
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
    fn catalog_has_eleven_unique_adapters() {
        let ids = GATEWAYS
            .iter()
            .map(|gateway| gateway.id)
            .collect::<HashSet<_>>();
        assert_eq!(GATEWAYS.len(), 11);
        assert_eq!(ids.len(), 11);
        assert_eq!(
            GATEWAYS
                .iter()
                .filter(|gateway| gateway.kind == GatewayKind::Billing)
                .count(),
            10
        );
        assert_eq!(
            GATEWAYS
                .iter()
                .filter(|gateway| gateway.kind == GatewayKind::Payout)
                .count(),
            1
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
