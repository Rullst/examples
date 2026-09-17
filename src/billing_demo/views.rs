//! Public, offline Rullst Capital catalogue.

use super::gateways::{GatewayInfo, GatewayKind, all_gateways};
use axum::http::Uri;
use rullst::html;

/// Renders the pricing, adapter audit and deterministic fixture explorer.
pub fn render_pricing_page(
    nav: String,
    styles: String,
    free_can_post: bool,
    xml_snippet: String,
    csrf_token: &str,
    simulated_checkout_url: Option<(String, String)>,
) -> String {
    let gateways = all_gateways();
    let adapter_options = render_adapter_options(&gateways);
    let gateway_cards = render_gateway_cards(&gateways);
    let audit_details = render_audit_details(&gateways);
    let fixture_result = render_fixture_result(simulated_checkout_url);
    let saas_cta = render_saas_cta(configured_saas_url());

    let extra_styles = r#"
        .pricing-hero { margin-bottom: 2rem; }
        .hero-stats { display: flex; gap: .7rem; flex-wrap: wrap; margin-top: 1rem; }
        .stat-badge { display: inline-flex; align-items: center; gap: .4rem; background: rgba(255,255,255,.05); border: 1px solid rgba(255,255,255,.12); border-radius: 9999px; padding: .35rem .75rem; font-size: .78rem; color: #cbd5e1; }
        .status-audited { border-color: rgba(16,185,129,.45); color: #6ee7b7; }
        .status-blocked { border-color: rgba(245,158,11,.5); color: #fbbf24; }
        .status-unavailable { border-color: rgba(248,113,113,.45); color: #fca5a5; }
        .status-payout { border-color: rgba(34,211,238,.45); color: #67e8f9; }
        .offer-grid, .gateway-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(100%, 280px), 1fr)); gap: 1rem; margin-top: 1.25rem; }
        .offer-card, .gateway-card { min-width: 0; background: #070a12; border: 1px solid #1e293b; border-radius: .75rem; padding: 1.2rem; }
        .gateway-card { display: flex; flex-direction: column; justify-content: space-between; }
        .gateway-card h3 { color: #f8fafc; margin: 0; font-size: 1.08rem; }
        .gateway-meta { color: #94a3b8; font-size: .78rem; margin: .45rem 0 .8rem; }
        .gateway-note { color: #cbd5e1; font-size: .85rem; line-height: 1.55; }
        .gateway-actions { display: flex; gap: .55rem; margin-top: 1rem; }
        .gateway-actions .btn { flex: 1; text-align: center; font-size: .78rem; padding: .45rem; }
        .fixture-box { background: rgba(59,130,246,.08); border: 1px solid rgba(59,130,246,.35); border-radius: .75rem; padding: 1.25rem; margin-top: 1.5rem; }
        .fixture-form { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)) auto; gap: .85rem; align-items: end; }
        .fixture-field { min-width: 0; }
        .fixture-field label { display: block; font-size: .78rem; color: #94a3b8; margin-bottom: .35rem; }
        .fixture-field select, .fixture-field input { box-sizing: border-box; width: 100%; min-width: 0; padding: .65rem; background: #070a12; border: 1px solid #334155; border-radius: .375rem; color: #fff; font-size: .9rem; }
        .fixture-result { margin-top: 1rem; padding: 1rem; background: rgba(16,185,129,.1); border: 1px solid rgba(16,185,129,.5); border-radius: .5rem; }
        .code-box { max-width: 100%; box-sizing: border-box; background: #030712; border: 1px solid #1e293b; border-radius: .5rem; padding: 1rem; font-family: monospace; font-size: .82rem; color: #38bdf8; overflow-wrap: anywhere; white-space: pre-wrap; }
        .audit-list { margin-top: 1rem; background: #070a12; border: 1px solid #1e293b; border-radius: .75rem; overflow: hidden; }
        .audit-list details { border-bottom: 1px solid #1e293b; }
        .audit-list details:last-child { border-bottom: 0; }
        .audit-list summary { padding: 1rem; cursor: pointer; color: #f8fafc; font-weight: 700; }
        .audit-body { padding: 0 1rem 1rem; color: #cbd5e1; line-height: 1.55; }
        .saas-cta { margin: 1.5rem 0 0; padding: 1.2rem; border: 1px solid rgba(16,185,129,.5); border-radius: .75rem; background: linear-gradient(135deg, rgba(16,185,129,.12), rgba(59,130,246,.1)); display: flex; justify-content: space-between; align-items: center; gap: 1rem; }
        .saas-cta h2 { margin: 0 0 .35rem; color: #f8fafc; font-size: 1.15rem; }
        .saas-cta p { margin: 0; color: #cbd5e1; font-size: .88rem; }
        .notice { color: #fbbf24; font-size: .85rem; line-height: 1.5; }
        @media (max-width: 820px) {
            .fixture-form { grid-template-columns: 1fr 1fr; }
            .fixture-submit { grid-column: 1 / -1; }
            .saas-cta { align-items: stretch; flex-direction: column; }
            .saas-cta .btn { width: 100%; box-sizing: border-box; text-align: center; }
        }
        @media (max-width: 560px) {
            .container { width: auto; padding-left: .75rem; padding-right: .75rem; }
            .card, .fixture-box, .offer-card, .gateway-card { padding: 1rem; }
            .fixture-form { grid-template-columns: 1fr; }
            .fixture-submit { grid-column: auto; }
            .gateway-actions { flex-direction: column; }
            .hero-stats .stat-badge { width: 100%; box-sizing: border-box; border-radius: .5rem; }
        }
    "#;

    html! {
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover" />
                <title>"Rullst Capital - Offline Adapter Showcase"</title>
                <link rel="icon" type="image/png" href="https://raw.githubusercontent.com/Rullst/Rullst/main/Rullst.png" />
                <style>{ rullst::html::RawHtml(styles) }</style>
                <style>{ rullst::html::RawHtml(extra_styles.to_owned()) }</style>
            </head>
            <body>
                { rullst::html::RawHtml(nav) }
                <div class="container">
                    <section class="card pricing-hero">
                        <h1 class="card-title" style="margin-bottom: .5rem;">
                            "SaaS pricing and Capital adapter audit"
                            <span class="feature-tag tag-cap">"rullst-capital"</span>
                        </h1>
                        <p style="color: var(--text-muted); max-width: 850px; margin: 0;">
                            "This public Showcase demonstrates quota logic and deterministic adapter fixtures. It never reads deployment payment credentials, contacts a payment provider or creates a charge."
                        </p>
                        <div class="hero-stats">
                            <span class="stat-badge">"10 incoming billing adapters"</span>
                            <span class="stat-badge">"1 outgoing payout adapter"</span>
                            <span class="stat-badge status-audited">"2 guarded paths implemented in the SaaS blueprint"</span>
                            <span class="stat-badge">"All actions on this page are offline"</span>
                        </div>
                        { rullst::html::RawHtml(saas_cta) }
                        <div class="offer-grid">
                            <article class="offer-card">
                                <h2 style="color: #94a3b8; margin: 0 0 .5rem; font-size: 1.05rem;">"Community Free"</h2>
                                <p style="font-size: 1.8rem; font-weight: 800; color: #fff; margin: 0;">"$0 / month"</p>
                                <p style="color: var(--text-muted);">"Illustrative offer used to exercise the Billable quota API."</p>
                                <div class="stat-badge status-audited">
                                    {if free_can_post { "Quota fixture: allowed (2/3)" } else { "Quota fixture: reached" }}
                                </div>
                            </article>
                            <article class="offer-card">
                                <h2 style="color: #38bdf8; margin: 0 0 .5rem; font-size: 1.05rem;">"Pro Author"</h2>
                                <p style="font-size: 1.8rem; font-weight: 800; color: #fff; margin: 0;">"$29 / month"</p>
                                <p style="color: var(--text-muted);">"Illustrative fixture only. This is not a purchasable offer in the Showcase."</p>
                                <a href="#fixture-explorer" class="btn" style="display: block; text-align: center;">"Explore adapter fixtures"</a>
                            </article>
                            <article class="offer-card">
                                <h2 style="color: #c084fc; margin: 0 0 .5rem; font-size: 1.05rem;">"Enterprise"</h2>
                                <p style="font-size: 1.8rem; font-weight: 800; color: #fff; margin: 0;">"$99 / month"</p>
                                <p style="color: var(--text-muted);">"Illustrative fixture with an offline fiscal XML preview; no invoice is issued."</p>
                                <a href="#adapter-audit" class="btn" style="display: block; text-align: center;">"Read the capability audit"</a>
                            </article>
                        </div>
                    </section>

                    <section id="fixture-explorer" class="card fixture-box">
                        <h2 class="card-title" style="margin-bottom: .5rem; color: #38bdf8;">"Offline adapter fixture explorer"</h2>
                        <p style="color: var(--text-muted); font-size: .9rem;">
                            "The output is synthetic and may resemble a URL, but it is not a checkout session and should not be used as provider-integration documentation. Wise produces a payout fixture, not a customer checkout."
                        </p>
                        <form method="POST" action="/checkout#fixture-explorer" class="fixture-form">
                            <input type="hidden" name="_token" value={csrf_token} />
                            <div class="fixture-field">
                                <label>"Adapter"</label>
                                <select name="provider">{ rullst::html::RawHtml(adapter_options) }</select>
                            </div>
                            <div class="fixture-field">
                                <label>"Fixture scenario"</label>
                                <select name="plan">
                                    <option value="pro_plan">"Pro sample"</option>
                                    <option value="enterprise_plan">"Enterprise sample"</option>
                                </select>
                            </div>
                            <div class="fixture-field">
                                <label>"Synthetic customer email"</label>
                                <input type="email" name="email" value="customer@example.invalid" required="true" />
                            </div>
                            <div class="fixture-submit">
                                <button type="submit" class="btn btn-primary" style="width: 100%; padding: .68rem 1rem;">"Generate offline fixture"</button>
                            </div>
                        </form>
                        { rullst::html::RawHtml(fixture_result) }
                    </section>

                    <section id="adapter-audit" class="card" style="margin-top: 2rem;">
                        <h2 class="card-title" style="margin-bottom: .4rem;">"Rullst Capital 12.0.0 capability audit"</h2>
                        <p style="color: var(--text-muted); font-size: .9rem;">
                            "These labels describe the audited v12 code paths, not provider availability for every country or account. Fees, settlement times, tax treatment and regional eligibility are intentionally omitted because providers and merchant contracts change."
                        </p>
                        <p class="notice">"Exported by the crate does not mean production-ready. Only Stripe and Razorpay have guarded paths in the separate SaaS blueprint; the published staging target will initially enable Stripe Test Mode only."</p>
                        <div class="gateway-grid">{ rullst::html::RawHtml(gateway_cards) }</div>
                        <div class="audit-list">{ rullst::html::RawHtml(audit_details) }</div>
                    </section>

                    <section class="card" style="margin-top: 2rem;">
                        <h2 class="card-title">"Contained offline fiscal preview"</h2>
                        <p style="color: var(--text-muted); font-size: .9rem;">
                            "This escaped DPS XML fragment is not signed, transmitted, homologated or authorized."
                        </p>
                        <div class="code-box">{xml_snippet}</div>
                    </section>
                </div>
            </body>
        </html>
    }
}

fn render_adapter_options(gateways: &[GatewayInfo]) -> String {
    gateways
        .iter()
        .map(|gateway| {
            let suffix = match gateway.kind {
                GatewayKind::Billing => "billing fixture",
                GatewayKind::Payout => "payout fixture",
            };
            format!(
                "<option value=\"{}\">{} ({})</option>",
                gateway.id,
                rullst::html::escape_str(gateway.name),
                suffix
            )
        })
        .collect()
}

fn render_gateway_cards(gateways: &[GatewayInfo]) -> String {
    gateways
        .iter()
        .map(|gateway| {
            let (status, class_name) = gateway.status_badge();
            let fixture_label = if gateway.kind == GatewayKind::Payout {
                "Inspect payout fixture"
            } else {
                "Inspect billing fixture"
            };
            html! {
                <article class="gateway-card">
                    <div>
                        <h3>{gateway.name}</h3>
                        <p class="gateway-meta">{gateway.kind_label()}</p>
                        <span class={format!("stat-badge {class_name}")}>{status}</span>
                        <p class="gateway-note">{gateway.audit_note}</p>
                    </div>
                    <div class="gateway-actions">
                        <a href={format!("/checkout?provider={}&plan=pro_plan#fixture-explorer", gateway.id)} class="btn btn-primary">{fixture_label}</a>
                        <a href={format!("#audit-{}", gateway.id)} class="btn">"Audit note"</a>
                    </div>
                </article>
            }
        })
        .collect()
}

fn render_audit_details(gateways: &[GatewayInfo]) -> String {
    gateways
        .iter()
        .map(|gateway| {
            let (status, _) = gateway.status_badge();
            html! {
                <details id={format!("audit-{}", gateway.id)}>
                    <summary>{format!("{} - {}", gateway.name, status)}</summary>
                    <div class="audit-body">
                        <p><strong>"Adapter kind: "</strong>{gateway.kind_label()}</p>
                        <p>{gateway.audit_note}</p>
                    </div>
                </details>
            }
        })
        .collect()
}

fn render_fixture_result(simulated: Option<(String, String)>) -> String {
    let Some((provider, output)) = simulated else {
        return String::new();
    };
    html! {
        <div class="fixture-result" role="status">
            <strong style="color: #6ee7b7;">{format!("{} offline fixture", provider.to_ascii_uppercase())}</strong>
            <p style="color: #cbd5e1; font-size: .85rem;">"No live provider request was made. The synthetic output is shown as text and is not opened as a checkout link."</p>
            <div class="code-box">{output}</div>
            <a href="/pricing" class="btn" style="display: inline-block; margin-top: .75rem;">"Clear fixture"</a>
        </div>
    }
}

fn configured_saas_url() -> Option<String> {
    std::env::var("SAAS_BLUEPRINT_URL")
        .ok()
        .and_then(|value| validated_saas_url(&value))
}

fn validated_saas_url(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 2048
        || value.chars().any(char::is_whitespace)
        || value.chars().any(char::is_control)
    {
        return None;
    }
    let uri = value.parse::<Uri>().ok()?;
    let authority = uri.authority()?;
    if uri.scheme_str() != Some("https")
        || authority.as_str().contains('@')
        || authority.host().is_empty()
    {
        return None;
    }
    Some(value.to_owned())
}

fn render_saas_cta(url: Option<String>) -> String {
    let Some(url) = url else {
        return String::new();
    };
    html! {
        <aside class="saas-cta">
            <div>
                <h2>"See the deployed SaaS payment flow"</h2>
                <p>"Production-style architecture with persistent PostgreSQL and Stripe Test Mode. Test data only; no real charge is created."</p>
            </div>
            <a href={url} target="_blank" rel="noopener noreferrer" class="btn btn-emerald">"Open SaaS blueprint"</a>
        </aside>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::billing_demo::gateways::AdapterStatus;

    #[test]
    fn accepts_only_bounded_https_saas_links_without_userinfo() {
        assert_eq!(
            validated_saas_url("https://saas.example.com/pricing"),
            Some("https://saas.example.com/pricing".to_owned())
        );
        assert_eq!(validated_saas_url("http://saas.example.com"), None);
        assert_eq!(validated_saas_url("javascript:alert(1)"), None);
        assert_eq!(validated_saas_url("https://user@example.com"), None);
        assert_eq!(validated_saas_url("https://example.com/ bad"), None);
    }

    #[test]
    fn public_catalogue_copy_does_not_expose_secret_presence() {
        let page = render_pricing_page(
            String::new(),
            String::new(),
            true,
            "fixture".to_owned(),
            "csrf",
            None,
        );
        assert!(!page.contains("Credentials Detected"));
        assert!(!page.contains("STRIPE_SECRET_KEY"));
        assert!(page.contains("never reads deployment payment credentials"));
    }

    #[test]
    fn status_copy_distinguishes_payouts_from_checkout() {
        let gateways = all_gateways();
        let wise = gateways
            .iter()
            .find(|gateway| gateway.id == "wise")
            .unwrap();
        assert_eq!(wise.status, AdapterStatus::PayoutOnly);
        assert_eq!(wise.kind, GatewayKind::Payout);
    }
}
