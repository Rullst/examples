use crate::controllers::billing_controller::PaymentMode;
use crate::controllers::gateway_catalog::{GATEWAYS, GatewayKind, LiveCheckoutStatus};
use crate::controllers::legal_controller::MerchantNotice;
use rullst::html;
use rullst::response::Html;

pub struct PaymentPageState {
    pub selected_provider: String,
    pub payment_mode: PaymentMode,
    pub production_deployment: bool,
    pub expected_price: String,
    pub setup_error: Option<String>,
    pub signed_in: bool,
    pub merchant_notice: Option<MerchantNotice>,
}

fn pricing_navbar(
    csrf_token: &str,
    signed_in: bool,
    live: bool,
    production_deployment: bool,
) -> String {
    let account_actions = if signed_in {
        format!(
            "<span class=\"pricing-nav__status\">Signed in</span><a href=\"/dashboard\" class=\"pricing-nav__link\">Dashboard</a><form method=\"post\" action=\"/logout\" class=\"pricing-nav__form\"><input type=\"hidden\" name=\"_token\" value=\"{}\"><button type=\"submit\" class=\"pricing-nav__link pricing-nav__button\">Sign out</button></form>",
            rullst::html::escape_str(csrf_token)
        )
    } else {
        "<a href=\"/login\" class=\"pricing-nav__link\">Sign in</a><a href=\"/register\" class=\"pricing-nav__link\">Create account</a>".to_owned()
    };
    let terms_label = if live {
        "Purchase terms"
    } else if production_deployment {
        "Pre-launch terms"
    } else {
        "Sandbox terms"
    };
    html! {
        <nav class="pricing-nav" aria-label="Application links">
            <a href="/privacy" class="pricing-nav__link">"Privacy"</a>
            <a href="/terms" class="pricing-nav__link">{terms_label}</a>
            { rullst::html::RawHtml(account_actions) }
            <a href="/nexus" class="pricing-nav__link pricing-nav__link--solid">"Nexus CMS"</a>
        </nav>
    }
}

fn setup_banner(state: &PaymentPageState) -> String {
    let (title, message, class_name) = if let Some(error) = state.setup_error.as_deref() {
        (
            "Configuration rejected",
            error,
            "setup-banner setup-banner--blocked",
        )
    } else {
        match state.payment_mode {
            PaymentMode::Test => (
                "Provider test mode is enabled",
                "This environment uses provider test data and cannot create a real charge. Use only the provider's documented test payment methods.",
                "setup-banner setup-banner--test",
            ),
            PaymentMode::Live => (
                "Live one-time purchase is enabled",
                "Stripe will create a real charge. The server verifies the live Price and grants access only after signed provider reconciliation.",
                "setup-banner setup-banner--live",
            ),
            PaymentMode::Disabled => (
                "Payment checkout is disabled",
                if state.production_deployment {
                    "The production site is online, but real-money Checkout remains locked. Complete the merchant, Stripe, private-artifact, backup and launch gates before enabling it."
                } else {
                    "This is the fail-closed default. Configure a provider-owned test price, test credentials and a signed webhook before enabling test mode."
                },
                "setup-banner",
            ),
        }
    };

    html! {
        <section class={class_name} role="status">
            <div class="setup-banner-icon" aria-hidden="true">"Warning"</div>
            <div class="setup-banner-content">
                <h2>{title}</h2>
                <p>{message}</p>
                <p class="setup-banner-meta">
                    "Selected provider: "
                    <strong>{state.selected_provider.as_str()}</strong>
                    " | Mode: "
                    <strong>{state.payment_mode.label()}</strong>
                    " | Expected one-time price: "
                    <strong>{state.expected_price.as_str()}</strong>
                </p>
            </div>
        </section>
    }
}

fn checkout_card(csrf_token: &str, state: &PaymentPageState) -> String {
    let checkout_available =
        state.payment_mode != PaymentMode::Disabled && state.setup_error.is_none();
    let action = if checkout_available && state.signed_in {
        let button_label = if state.payment_mode == PaymentMode::Test {
            "Continue to provider test checkout"
        } else {
            "Continue to live checkout"
        };
        let progress = if state.payment_mode == PaymentMode::Live {
            "Verifying the live Price and opening Stripe Checkout&hellip;"
        } else {
            "Verifying the sandbox Price and opening Stripe Checkout&hellip;"
        };
        format!(
            "<form id=\"checkout-form\" method=\"post\" action=\"/billing/checkout\"><input type=\"hidden\" name=\"_token\" value=\"{}\"><input type=\"hidden\" name=\"offer\" value=\"gateway-report-stripe\"><label class=\"purchase-authority\"><input type=\"checkbox\" name=\"purchase_authority\" value=\"adult_or_guardian\" required> I am 18 or older, or I am the parent/legal guardian making this purchase.</label><button id=\"checkout-submit\" type=\"submit\" class=\"btn-checkout primary\">{}</button><p id=\"checkout-status\" class=\"checkout-status\" role=\"status\" aria-live=\"polite\" hidden>{}</p></form>",
            rullst::html::escape_str(csrf_token),
            button_label,
            progress,
        )
    } else if checkout_available {
        let label = if state.payment_mode == PaymentMode::Live {
            "Sign in to purchase"
        } else {
            "Sign in to open sandbox checkout"
        };
        format!(
            "<a href=\"/login\" class=\"btn-checkout primary\">{label}</a><p class=\"checkout-status checkout-status--visible\">Checkout is tied to your authenticated account. Sign in first, then return here.</p>"
        )
    } else {
        "<button type=\"button\" class=\"btn-checkout\" disabled>Payment checkout is disabled</button>"
            .to_owned()
    };
    let seller_disclosure = state
        .merchant_notice
        .as_ref()
        .map(|merchant| {
            let mailto = format!("mailto:{}", merchant.support_email);
            html! {
                <aside class="seller-disclosure" aria-label="Seller identification">
                    <h3>"Seller identification"</h3>
                    <p><strong>{merchant.legal_name.as_str()}</strong>" (Rullst)"</p>
                    <p>"Brazilian tax registration: "<strong>{merchant.tax_id.as_str()}</strong></p>
                    <address>{merchant.physical_address.as_str()}</address>
                    <p><a href={mailto.as_str()}>{merchant.support_email.as_str()}</a></p>
                    <p><a href="/terms">"Read the purchase and refund terms before paying."</a></p>
                </aside>
            }
        })
        .unwrap_or_default();
    let deliverables = if state.payment_mode == PaymentMode::Live {
        "<div class=\"deliverables\"><h3>What this purchase includes</h3><ul><li>A private, versioned Markdown guide containing the Stripe gateway field report and a sanitized implementation tutorial for Stripe, PostgreSQL, GitHub OIDC, Azure Container Apps, DNS/TLS, webhooks and production promotion.</li><li>A Rullst Founding Customer certificate for every reconciled live purchase. There is no certificate quota or expiration based on purchase order.</li></ul></div>".to_owned()
    } else {
        "<div class=\"deliverables\"><h3>What this sandbox flow includes</h3><ul><li>A downloadable Stripe sandbox field-report summary.</li><li>A test-only Rullst Sandbox Pioneer certificate after the signed test payment is reconciled. It is not proof of a real purchase.</li></ul></div>".to_owned()
    };

    html! {
        <section class="pricing-card pricing-card--featured">
            <p class="eyebrow">"Server-controlled one-time checkout"</p>
            <h2 class="plan-name">"Stripe Gateway Field Report"</h2>
            <p class="plan-desc">"The browser never supplies a Stripe Price ID or amount. Immediately before Checkout, the server verifies that the configured Price is active, one-time, belongs to this Stripe environment and exactly matches the expected amount and currency."</p>
            <div class="price-container">
                <span class="price-label">{state.expected_price.as_str()}</span>
                <span class="period">"one-time purchase"</span>
            </div>
            { rullst::html::RawHtml(deliverables) }
            <ul class="features-list">
                <li>"Authenticated local customer binding"</li>
                <li>"Provider price verified before redirect"</li>
                <li>"Signed, fresh and idempotent webhook boundary"</li>
                <li>"Paid session reconciled with Stripe before access"</li>
                <li>"HTTP 303 checkout handoff"</li>
            </ul>
            { rullst::html::RawHtml(seller_disclosure) }
            { rullst::html::RawHtml(action) }
            <p class="fine-print">"A successful redirect never grants access. Application state changes only after verified provider evidence. The provider dashboard remains authoritative for charges, cancellations and refunds."</p>
        </section>
    }
}

fn status_text(status: LiveCheckoutStatus) -> (&'static str, &'static str) {
    match status {
        LiveCheckoutStatus::Enabled => ("Sandbox path ready", "status status--enabled"),
        LiveCheckoutStatus::ApplicationContractRequired => {
            ("Application contract required", "status status--blocked")
        }
        LiveCheckoutStatus::FrameworkFixRequired => {
            ("Framework fix required", "status status--blocked")
        }
        LiveCheckoutStatus::UnsupportedByV12 => {
            ("No live checkout in v12", "status status--unavailable")
        }
        LiveCheckoutStatus::PayoutOnly => ("Payout only", "status status--payout"),
    }
}

fn gateway_rows() -> String {
    GATEWAYS
        .iter()
        .map(|gateway| {
            let (status, class_name) = status_text(gateway.status);
            let kind = match gateway.kind {
                GatewayKind::Billing => "Billing",
                GatewayKind::Payout => "Outgoing payout",
            };
            format!(
                "<tr><th scope=\"row\">{}</th><td>{}</td><td><span class=\"{}\">{}</span></td><td>{}</td></tr>",
                rullst::html::escape_str(gateway.name),
                kind,
                class_name,
                status,
                rullst::html::escape_str(gateway.test_note),
            )
        })
        .collect()
}

fn gateway_matrix() -> String {
    html! {
        <section class="gateway-section" aria-labelledby="gateway-heading">
            <div class="section-heading">
                <p class="eyebrow">"Rullst Capital 12.0.0"</p>
                <h2 id="gateway-heading">"Adapter capability matrix"</h2>
                <p>"The framework exports 10 incoming billing adapters and 1 outgoing payout adapter. Exported does not mean every adapter has a usable current live checkout."</p>
            </div>
            <div class="table-scroll" tabindex="0">
                <table>
                    <thead><tr><th>"Adapter"</th><th>"Kind"</th><th>"SaaS path"</th><th>"Audit note"</th></tr></thead>
                    <tbody>{ rullst::html::RawHtml(gateway_rows()) }</tbody>
                </table>
            </div>
        </section>
    }
}

pub fn pricing_page(csrf_token: &str, csp_nonce: &str, state: &PaymentPageState) -> Html<String> {
    let checkout_script = if state.signed_in
        && state.payment_mode != PaymentMode::Disabled
        && state.setup_error.is_none()
    {
        format!(
            r#"<script nonce="{}">(() => {{
const form = document.getElementById('checkout-form');
const button = document.getElementById('checkout-submit');
const status = document.getElementById('checkout-status');
if (!form || !button || !status) return;
const originalLabel = button.textContent;
const reset = () => {{ button.disabled = false; button.textContent = originalLabel; status.hidden = true; }};
form.addEventListener('submit', () => {{
  if (!form.checkValidity()) return;
  button.disabled = true;
  button.textContent = 'Opening Stripe Checkout…';
  status.hidden = false;
}});
window.addEventListener('pageshow', reset);
}})();</script>"#,
            rullst::html::escape_str(csp_nonce)
        )
    } else {
        String::new()
    };
    let live = state.payment_mode == PaymentMode::Live;
    let production_prelaunch = state.production_deployment && !live;
    let subtitle = if live {
        "A real one-time purchase of the private Rullst and Stripe implementation guide. Stripe hosts payment collection; access is granted only after provider reconciliation."
    } else if production_prelaunch {
        "Production infrastructure is online while real-money Checkout remains deliberately locked. No payment can be initiated until every launch gate is completed."
    } else {
        "A deliberately constrained one-time Checkout harness. The published staging environment uses Stripe sandbox data and creates no real charge."
    };
    let checklist_title = if live {
        "Live purchase safeguards"
    } else if production_prelaunch {
        "Production launch checklist"
    } else {
        "Test-mode checklist"
    };
    let checklist_items = if live {
        "<li>The exact live one-time Price is verified by the server before redirect.</li><li>Live keys and webhook secrets remain in Azure Container Apps secrets.</li><li>Stripe Checkout collects payment details; this application never receives complete card data.</li><li>Signed webhooks reconcile payment, refund and dispute state.</li><li>The private guide is downloaded only after entitlement and SHA-256 verification.</li>"
    } else if production_prelaunch {
        "<li>Publish the required seller identity and reviewed purchase terms.</li><li>Configure the private artifact, exact live Stripe Price and signed webhook.</li><li>Complete backup and restore validation before accepting money.</li><li>Enable live mode only through the protected production workflow.</li><li>Treat the first payment as a genuine independent customer sale, never as test data.</li>"
    } else {
        "<li>Create the exact one-time sandbox Price in the provider dashboard.</li><li>Store test keys only in Azure Container Apps secrets.</li><li>Use the provider's documented test card or payment method.</li><li>Reconcile the signed webhook and persistent local state.</li><li>Test duplicate events, failed payments, expiration and refunds before any live rollout.</li>"
    };
    let heading = if live {
        "A real purchase with explicit boundaries"
    } else if production_prelaunch {
        "Production checkout is safely locked"
    } else {
        "Payment testing without hidden assumptions"
    };
    let final_note = if live {
        "This is a real purchase, not an integration test. Use the sandbox environment for test cards. Refunds are requested from the authenticated dashboard and processed through Stripe."
    } else if production_prelaunch {
        "No payment can be initiated while Checkout is disabled. Use the separate staging site for Stripe test cards."
    } else {
        "Never use a real card to test Stripe live mode. This environment stays in Stripe Test Mode and creates no real charge."
    };
    let document = html! {
        <html lang="en">
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>"SaaS payments - Rullst"</title>
                <link rel="icon" type="image/png" href="/static/rullst.png" />
                <link rel="stylesheet" href="/static/rullst.css" />
            </head>
            <body>
                <div class="glow-bg"></div>
                <div class="glow-bg-right"></div>
                <main class="container">
                    { rullst::html::RawHtml(pricing_navbar(csrf_token, state.signed_in, live, state.production_deployment)) }
                    <header class="header">
                        <span class="badge">"Rullst SaaS Blueprint"</span>
                        <h1>{heading}</h1>
                        <p class="subtitle">{subtitle}</p>
                    </header>
                    { rullst::html::RawHtml(setup_banner(state)) }
                    <div class="pricing-grid">
                        { rullst::html::RawHtml(checkout_card(csrf_token, state)) }
                        <aside class="safety-card">
                            <p class="eyebrow">"Safe rollout order"</p>
                            <h2>{checklist_title}</h2>
                            <ol>{ rullst::html::RawHtml(checklist_items.to_owned()) }</ol>
                            <p class="fine-print">{final_note}</p>
                        </aside>
                    </div>
                    { rullst::html::RawHtml(gateway_matrix()) }
                </main>
                <footer class="community-footer">
                    <div class="community-footer__mark" aria-hidden="true">"R"</div>
                    <div>
                        <p class="community-footer__eyebrow">"Build with us"</p>
                        <h2>"Join our community on Discord"</h2>
                        <p>"Meet Rullst builders, exchange ideas, and help shape what comes next."</p>
                    </div>
                    <a href="https://discord.gg/2ntKFtsSjw" target="_blank" rel="noopener noreferrer">"Join the Rullst Discord"</a>
                </footer>
                { rullst::html::RawHtml(checkout_script) }
            </body>
        </html>
    };
    Html(format!("<!DOCTYPE html>{document}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(signed_in: bool) -> PaymentPageState {
        PaymentPageState {
            selected_provider: "stripe".to_owned(),
            payment_mode: PaymentMode::Test,
            production_deployment: false,
            expected_price: "BRL 1.00".to_owned(),
            setup_error: None,
            signed_in,
            merchant_notice: None,
        }
    }

    #[test]
    fn signed_in_page_keeps_account_context_and_submit_feedback() {
        let page = pricing_page("csrf-token", "csp-nonce", &state(true)).0;

        assert!(page.contains("Signed in"));
        assert!(page.contains("href=\"/dashboard\""));
        assert!(page.contains("id=\"checkout-form\""));
        assert!(page.contains("Opening Stripe Checkout"));
        assert!(!page.contains("href=\"/login\" class=\"pricing-nav__link\""));
    }

    #[test]
    fn signed_out_page_requires_login_before_checkout() {
        let page = pricing_page("csrf-token", "csp-nonce", &state(false)).0;

        assert!(page.contains("Sign in to open sandbox checkout"));
        assert!(page.contains("href=\"/login\" class=\"pricing-nav__link\""));
        assert!(!page.contains("id=\"checkout-form\""));
    }

    #[test]
    fn live_offer_places_required_seller_identity_before_checkout() {
        let mut state = state(true);
        state.payment_mode = PaymentMode::Live;
        state.merchant_notice = Some(MerchantNotice {
            legal_name: "Public Seller".to_owned(),
            tax_id: "529.982.247-25".to_owned(),
            physical_address: "123 Public Street, Brazil".to_owned(),
            country: "BR".to_owned(),
            support_email: "support@example.com".to_owned(),
            refund_window_days: 14,
        });
        let page = pricing_page("csrf-token", "csp-nonce", &state).0;
        assert!(page.contains("Seller identification"));
        assert!(page.contains("Purchase terms"));
        assert!(page.contains("529.982.247-25"));
        assert!(page.contains("Read the purchase and refund terms before paying."));
        assert!(page.contains("What this purchase includes"));
        assert!(page.contains("every reconciled live purchase"));
        assert!(!page.contains("first 100"));
        assert!(page.contains("sanitized implementation tutorial"));
    }

    #[test]
    fn pricing_page_links_to_the_discord_community() {
        let page = pricing_page("csrf-token", "csp-nonce", &state(false)).0;
        assert!(page.contains("Join our community on Discord"));
        assert!(page.contains("https://discord.gg/2ntKFtsSjw"));
        assert!(page.contains("rel=\"noopener noreferrer\""));
    }

    #[test]
    fn disabled_production_is_not_described_as_staging_or_sandbox() {
        let mut state = state(false);
        state.payment_mode = PaymentMode::Disabled;
        state.production_deployment = true;
        let page = pricing_page("csrf-token", "csp-nonce", &state).0;

        assert!(page.contains("Production checkout is safely locked"));
        assert!(page.contains("Pre-launch terms"));
        assert!(page.contains("No payment can be initiated"));
        assert!(!page.contains("published staging environment"));
        assert!(!page.contains("This environment stays in Stripe Test Mode"));
    }
}
