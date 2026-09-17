use crate::controllers::billing_controller::PaymentMode;
use crate::controllers::gateway_catalog::{GATEWAYS, GatewayKind, LiveCheckoutStatus};
use rullst::html;
use rullst::response::Html;

pub struct PaymentPageState {
    pub selected_provider: String,
    pub payment_mode: PaymentMode,
    pub expected_price: String,
    pub setup_error: Option<String>,
    pub signed_in: bool,
}

fn pricing_navbar(csrf_token: &str, signed_in: bool) -> String {
    let account_actions = if signed_in {
        format!(
            "<span class=\"pricing-nav__status\">Signed in</span><a href=\"/dashboard\" class=\"pricing-nav__link\">Dashboard</a><form method=\"post\" action=\"/logout\" class=\"pricing-nav__form\"><input type=\"hidden\" name=\"_token\" value=\"{}\"><button type=\"submit\" class=\"pricing-nav__link pricing-nav__button\">Sign out</button></form>",
            rullst::html::escape_str(csrf_token)
        )
    } else {
        "<a href=\"/login\" class=\"pricing-nav__link\">Sign in</a><a href=\"/register\" class=\"pricing-nav__link\">Create account</a>".to_owned()
    };
    html! {
        <nav class="pricing-nav" aria-label="Application links">
            <a href="/privacy" class="pricing-nav__link">"Privacy"</a>
            <a href="/terms" class="pricing-nav__link">"Sandbox terms"</a>
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
                "Live-money mode is blocked",
                "Live checkout remains fail-closed until sandbox evidence, refunds, monitoring and the production legal notices pass review.",
                "setup-banner setup-banner--blocked",
            ),
            PaymentMode::Disabled => (
                "Payment checkout is disabled",
                "This is the fail-closed default. Configure a provider-owned test price, test credentials and a signed webhook before enabling test mode.",
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
        format!(
            "<form id=\"checkout-form\" method=\"post\" action=\"/billing/checkout\"><input type=\"hidden\" name=\"_token\" value=\"{}\"><input type=\"hidden\" name=\"offer\" value=\"gateway-report-stripe\"><label class=\"purchase-authority\"><input type=\"checkbox\" name=\"purchase_authority\" value=\"adult_or_guardian\" required> I am 18 or older, or I am the parent/legal guardian making this purchase.</label><button id=\"checkout-submit\" type=\"submit\" class=\"btn-checkout primary\">{}</button><p id=\"checkout-status\" class=\"checkout-status\" role=\"status\" aria-live=\"polite\" hidden>Verifying the sandbox Price and opening Stripe Checkout&hellip;</p></form>",
            rullst::html::escape_str(csrf_token),
            button_label,
        )
    } else if checkout_available {
        "<a href=\"/login\" class=\"btn-checkout primary\">Sign in to open sandbox checkout</a><p class=\"checkout-status checkout-status--visible\">Checkout is tied to your authenticated account. Sign in first, then return here.</p>".to_owned()
    } else {
        "<button type=\"button\" class=\"btn-checkout\" disabled>Payment checkout is disabled</button>"
            .to_owned()
    };

    html! {
        <section class="pricing-card pricing-card--featured">
            <p class="eyebrow">"Server-controlled one-time checkout"</p>
            <h2 class="plan-name">"Stripe Gateway Field Report"</h2>
            <p class="plan-desc">"The browser never supplies a Stripe Price ID or amount. Immediately before Checkout, the server verifies the configured Price is active, one-time, belongs to the sandbox and exactly matches the expected amount and currency."</p>
            <div class="price-container">
                <span class="price-label">{state.expected_price.as_str()}</span>
                <span class="period">"one-time purchase"</span>
            </div>
            <ul class="features-list">
                <li>"Authenticated local customer binding"</li>
                <li>"Provider price verified before redirect"</li>
                <li>"Signed, fresh and idempotent webhook boundary"</li>
                <li>"Paid session reconciled with Stripe before access"</li>
                <li>"HTTP 303 checkout handoff"</li>
            </ul>
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
                    { rullst::html::RawHtml(pricing_navbar(csrf_token, state.signed_in)) }
                    <header class="header">
                        <span class="badge">"Rullst SaaS Blueprint"</span>
                        <h1>"Payment testing without hidden assumptions"</h1>
                        <p class="subtitle">"A deliberately constrained one-time Checkout harness. The published staging environment uses Stripe sandbox data; live money remains blocked until the production acceptance gate is complete."</p>
                    </header>
                    { rullst::html::RawHtml(setup_banner(state)) }
                    <div class="pricing-grid">
                        { rullst::html::RawHtml(checkout_card(csrf_token, state)) }
                        <aside class="safety-card">
                            <p class="eyebrow">"Safe rollout order"</p>
                            <h2>"Test-mode checklist"</h2>
                            <ol>
                                <li>"Create the exact one-time sandbox Price in the provider dashboard."</li>
                                <li>"Store test keys only in Azure Container Apps secrets."</li>
                                <li>"Use the provider's documented test card or payment method."</li>
                                <li>"Reconcile the signed webhook and persistent local state."</li>
                                <li>"Test duplicate events, failed payments, expiration and refunds before any live rollout."</li>
                            </ol>
                            <p class="fine-print">"Never use a real card to test Stripe live mode. The published demo must stay in Stripe Test Mode and creates no real charge."</p>
                        </aside>
                    </div>
                    { rullst::html::RawHtml(gateway_matrix()) }
                </main>
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
            expected_price: "BRL 1.00".to_owned(),
            setup_error: None,
            signed_in,
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
}
