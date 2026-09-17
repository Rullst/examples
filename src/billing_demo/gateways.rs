//! Audited Rullst Capital v12 adapter catalogue used by the public Showcase.
//!
//! The Showcase exercises deterministic fixtures only. Exporting an adapter is
//! not the same as having a current, production-ready checkout integration.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayKind {
    Billing,
    Payout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterStatus {
    /// A guarded path exists in the separate SaaS blueprint.
    AuditedInSaas,
    /// The exported v12 adapter needs a framework contract correction.
    FrameworkFixRequired,
    /// The exported v12 adapter rejects plan-only live checkout.
    UnsupportedByV12,
    /// This adapter sends funds and is not an incoming checkout gateway.
    PayoutOnly,
}

#[derive(Debug, Clone, Copy)]
pub struct GatewayInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: GatewayKind,
    pub status: AdapterStatus,
    pub audit_note: &'static str,
}

impl GatewayInfo {
    pub fn kind_label(self) -> &'static str {
        match self.kind {
            GatewayKind::Billing => "Incoming billing adapter",
            GatewayKind::Payout => "Outgoing payout adapter",
        }
    }

    pub fn status_badge(self) -> (&'static str, &'static str) {
        match self.status {
            AdapterStatus::AuditedInSaas => ("SaaS path implemented", "status-audited"),
            AdapterStatus::FrameworkFixRequired => ("Framework fix required", "status-blocked"),
            AdapterStatus::UnsupportedByV12 => ("No live checkout in v12", "status-unavailable"),
            AdapterStatus::PayoutOnly => ("Payout only", "status-payout"),
        }
    }
}

/// Returns all adapters exported by Rullst Capital 12.0.0.
pub fn all_gateways() -> Vec<GatewayInfo> {
    vec![
        GatewayInfo {
            id: "stripe",
            name: "Stripe",
            kind: GatewayKind::Billing,
            status: AdapterStatus::AuditedInSaas,
            audit_note: "The separate SaaS blueprint verifies a server-owned recurring price before redirecting. Its published staging environment must use Stripe Test Mode.",
        },
        GatewayInfo {
            id: "razorpay",
            name: "Razorpay",
            kind: GatewayKind::Billing,
            status: AdapterStatus::AuditedInSaas,
            audit_note: "The separate SaaS blueprint contains a guarded provider-plan path. Merchant eligibility, currencies and payment methods remain account-dependent.",
        },
        GatewayInfo {
            id: "lemonsqueezy",
            name: "Lemon Squeezy",
            kind: GatewayKind::Billing,
            status: AdapterStatus::FrameworkFixRequired,
            audit_note: "The v12 request hard-codes store ID 1 instead of accepting the merchant's store ID.",
        },
        GatewayInfo {
            id: "paddle",
            name: "Paddle",
            kind: GatewayKind::Billing,
            status: AdapterStatus::FrameworkFixRequired,
            audit_note: "The v12 transaction payload does not match the current provider transaction contract.",
        },
        GatewayInfo {
            id: "polar",
            name: "Polar",
            kind: GatewayKind::Billing,
            status: AdapterStatus::FrameworkFixRequired,
            audit_note: "The v12 adapter uses the former price-based checkout request instead of the current products-based contract.",
        },
        GatewayInfo {
            id: "infinitepay",
            name: "InfinitePay",
            kind: GatewayKind::Billing,
            status: AdapterStatus::UnsupportedByV12,
            audit_note: "Live plan-only checkout returns UnsupportedOperation in rullst-capital 12.0.0.",
        },
        GatewayInfo {
            id: "mercadopago",
            name: "Mercado Pago",
            kind: GatewayKind::Billing,
            status: AdapterStatus::UnsupportedByV12,
            audit_note: "Live plan-only checkout and the body-only live webhook verifier are unavailable in v12.",
        },
        GatewayInfo {
            id: "coinbase",
            name: "Coinbase Commerce",
            kind: GatewayKind::Billing,
            status: AdapterStatus::UnsupportedByV12,
            audit_note: "Live plan-only checkout returns UnsupportedOperation in rullst-capital 12.0.0.",
        },
        GatewayInfo {
            id: "picpay",
            name: "PicPay",
            kind: GatewayKind::Billing,
            status: AdapterStatus::UnsupportedByV12,
            audit_note: "Live plan-only checkout returns UnsupportedOperation in rullst-capital 12.0.0.",
        },
        GatewayInfo {
            id: "alipay",
            name: "Alipay",
            kind: GatewayKind::Billing,
            status: AdapterStatus::UnsupportedByV12,
            audit_note: "Live RSA2 checkout signing and live webhook verification are explicitly disabled in v12.",
        },
        GatewayInfo {
            id: "wise",
            name: "Wise",
            kind: GatewayKind::Payout,
            status: AdapterStatus::PayoutOnly,
            audit_note: "Wise sends funds to recipients. It is not a gateway for receiving SaaS checkout payments.",
        },
    ]
}

/// Generates deterministic fixture output without contacting a payment service.
pub async fn simulate_provider_checkout(
    provider_id: &str,
    customer_email: &str,
    plan_id: &str,
    redirect_url: &str,
) -> Result<String, rullst_capital::CapitalError> {
    use rullst_capital::providers::*;

    match provider_id {
        "stripe" => {
            let provider = StripeProvider::new("mock_stripe_key".to_string(), String::new());
            provider
                .create_checkout_session(customer_email, plan_id, redirect_url)
                .await
        }
        "lemonsqueezy" => {
            let provider = LemonSqueezyProvider::new("mock_lmsq".to_string(), String::new());
            provider
                .create_checkout_session(customer_email, plan_id, redirect_url)
                .await
        }
        "infinitepay" => {
            let provider = InfinitePayProvider::new("mock_inf".to_string(), String::new());
            provider
                .create_checkout_session(customer_email, plan_id, redirect_url)
                .await
        }
        "polar" => {
            let provider = PolarProvider::new("mock_polar".to_string(), String::new());
            provider
                .create_checkout_session(customer_email, plan_id, redirect_url)
                .await
        }
        "paddle" => {
            let provider = PaddleProvider::new("mock_paddle".to_string(), String::new());
            provider
                .create_checkout_session(customer_email, plan_id, redirect_url)
                .await
        }
        "alipay" => {
            let provider = AlipayProvider::new(
                "mock_alipay".to_string(),
                "mock_private_key".to_string(),
                String::new(),
            );
            provider
                .create_checkout_session(customer_email, plan_id, redirect_url)
                .await
        }
        "mercadopago" => {
            let provider = MercadoPagoProvider::new("mock_mp".to_string(), String::new());
            provider
                .create_checkout_session(customer_email, plan_id, redirect_url)
                .await
        }
        "razorpay" => {
            let provider = RazorpayProvider::new(
                "mock_rzp".to_string(),
                "mock_secret".to_string(),
                String::new(),
            );
            provider
                .create_checkout_session(customer_email, plan_id, redirect_url)
                .await
        }
        "coinbase" => {
            let provider = CoinbaseCommerceProvider::new("mock_cb".to_string(), String::new());
            provider
                .create_checkout_session(customer_email, plan_id, redirect_url)
                .await
        }
        "picpay" => {
            let provider = PicPayProvider::new("mock_pic".to_string(), "mock_seller".to_string());
            provider
                .create_checkout_session(customer_email, plan_id, redirect_url)
                .await
        }
        "wise" => Ok(format!(
            "https://wise.example.invalid/mock-payout?recipient={}&scenario={}",
            rullst_capital::url_encode(customer_email),
            rullst_capital::url_encode(plan_id)
        )),
        _ => Err(rullst_capital::CapitalError::ConfigurationError(format!(
            "Unknown adapter: {provider_id}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn catalogue_has_ten_billing_adapters_and_one_payout_adapter() {
        let gateways = all_gateways();
        let ids = gateways
            .iter()
            .map(|gateway| gateway.id)
            .collect::<HashSet<_>>();
        assert_eq!(gateways.len(), 11);
        assert_eq!(ids.len(), 11);
        assert_eq!(
            gateways
                .iter()
                .filter(|gateway| gateway.kind == GatewayKind::Billing)
                .count(),
            10
        );
        assert_eq!(
            gateways
                .iter()
                .filter(|gateway| gateway.kind == GatewayKind::Payout)
                .count(),
            1
        );
    }

    #[test]
    fn only_separate_audited_saas_paths_are_marked_implemented() {
        let enabled = all_gateways()
            .into_iter()
            .filter(|gateway| gateway.status == AdapterStatus::AuditedInSaas)
            .map(|gateway| gateway.id)
            .collect::<Vec<_>>();
        assert_eq!(enabled, vec!["stripe", "razorpay"]);
    }
}
