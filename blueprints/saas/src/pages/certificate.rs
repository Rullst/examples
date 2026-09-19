use rullst::response::Html;

fn badge_label(badge_kind: &str) -> &'static str {
    match badge_kind {
        "sandbox_pioneer" => "Rullst Sandbox Pioneer",
        "founding_customer" => "Rullst Founding Customer",
        _ => "Rullst Tester",
    }
}

fn issued_date(created_at: &str) -> &str {
    created_at.get(..10).unwrap_or(created_at)
}

fn page_shell(title: &str, body: &str, extra_head: &str, csp_nonce: &str) -> Html<String> {
    Html(format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta name=\"robots\" content=\"noindex,nofollow,noarchive\"><title>{}</title><link rel=\"icon\" type=\"image/png\" href=\"/static/rullst.png\"><style nonce=\"{}\">{}</style>{}</head><body>{}</body></html>",
        rullst::html::escape_str(title),
        rullst::html::escape_str(csp_nonce),
        CERTIFICATE_CSS,
        extra_head,
        body
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn owner_certificate_page(
    holder_name: &str,
    public_id: &str,
    badge_kind: &str,
    environment: &str,
    certificate_status: &str,
    entitlement_status: &str,
    created_at: &str,
    csp_nonce: &str,
) -> Html<String> {
    let active = certificate_status == "active" && entitlement_status == "active";
    let state_label = if active { "Valid" } else { "Revoked" };
    let live = environment == "live";
    let eyebrow = if live {
        "Rullst verified founding customer"
    } else {
        "Rullst verified sandbox participation"
    };
    let description = if live {
        "For completing a provider-confirmed one-time purchase from the Rullst SaaS live showcase. Refunds and payment disputes revoke this certificate."
    } else {
        "For completing a provider-confirmed Stripe sandbox purchase through the Rullst SaaS staging blueprint. No real-money charge is represented by this certificate."
    };
    let terms_label = if live {
        "Purchase terms"
    } else {
        "Sandbox terms"
    };
    let body = format!(
        r#"<main class="page"><nav class="actions" aria-label="Certificate actions"><a href="/dashboard">Back to dashboard</a><a href="/privacy">Privacy</a><a href="/terms">{terms_label}</a><button id="print-certificate" type="button">Print or save as PDF</button></nav><article class="certificate"><div class="mark" aria-hidden="true">R</div><p class="eyebrow">{eyebrow}</p><h1>{badge}</h1><p class="awarded">Awarded to</p><p class="holder">{holder}</p><p class="description">{description}</p><dl><div><dt>Verification ID</dt><dd>{public_id}</dd></div><div><dt>Issued</dt><dd>{issued}</dd></div><div><dt>Environment</dt><dd>{environment}</dd></div><div><dt>Status</dt><dd class="state state--{state_class}">{state}</dd></div></dl><p class="verify">Verify without exposing personal data at <a href="/verify/{public_id}">/verify/{public_id}</a></p></article><p class="privacy">Your name appears only on this authenticated private page. The public verification page contains no name, email, payment identifier or financial details. See the <a href="/privacy">Privacy notice</a> and <a href="/terms">{terms_label}</a>.</p></main>"#,
        badge = rullst::html::escape_str(badge_label(badge_kind)),
        holder = rullst::html::escape_str(holder_name),
        public_id = rullst::html::escape_str(public_id),
        issued = rullst::html::escape_str(issued_date(created_at)),
        environment = rullst::html::escape_str(environment),
        state = state_label,
        state_class = if active { "valid" } else { "revoked" },
        eyebrow = rullst::html::escape_str(eyebrow),
        description = rullst::html::escape_str(description),
        terms_label = rullst::html::escape_str(terms_label),
    );
    let script = format!(
        "<script nonce=\"{}\">document.addEventListener('DOMContentLoaded',function(){{const button=document.getElementById('print-certificate');if(button){{button.addEventListener('click',function(){{window.print();}});}}}});</script>",
        rullst::html::escape_str(csp_nonce)
    );
    page_shell(
        if live {
            "Rullst Founding Customer certificate"
        } else {
            "Rullst Sandbox Pioneer certificate"
        },
        &body,
        &script,
        csp_nonce,
    )
}

pub fn public_verification_page(
    public_id: &str,
    badge_kind: &str,
    environment: &str,
    active: bool,
    created_at: &str,
    csp_nonce: &str,
) -> Html<String> {
    let live = environment == "live";
    let boundary = if live {
        "<div class=\"live-warning\"><strong>Live purchase certificate</strong><p>This confirms a provider-reconciled real-money purchase. A refund or dispute revokes its validity.</p></div>"
    } else {
        "<div class=\"sandbox-warning\"><strong>Sandbox certificate</strong><p>This confirms a Stripe test-mode transaction. It does not prove a real-money purchase or customer status.</p></div>"
    };
    let body = format!(
        r#"<main class="page page--verify"><article class="verification"><div class="mark" aria-hidden="true">R</div><p class="eyebrow">Public certificate verification</p><h1>{badge}</h1><p class="verification-result state state--{state_class}">{state}</p><dl><div><dt>Verification ID</dt><dd>{public_id}</dd></div><div><dt>Issued</dt><dd>{issued}</dd></div><div><dt>Environment</dt><dd>{environment}</dd></div></dl>{boundary}<p class="privacy">Privacy by default: this page does not disclose the holder's name, email, provider IDs or payment details. See the <a href="/privacy">Privacy notice</a>.</p><a class="home-link" href="/">Rullst SaaS blueprint</a> <a class="home-link" href="/terms">Terms</a></article></main>"#,
        badge = rullst::html::escape_str(badge_label(badge_kind)),
        state = if active { "Valid" } else { "Revoked" },
        state_class = if active { "valid" } else { "revoked" },
        public_id = rullst::html::escape_str(public_id),
        issued = rullst::html::escape_str(issued_date(created_at)),
        environment = rullst::html::escape_str(environment),
        boundary = boundary,
    );
    page_shell("Verify Rullst certificate", &body, "", csp_nonce)
}

const CERTIFICATE_CSS: &str = r#"
:root{color-scheme:dark}*{box-sizing:border-box}body{margin:0;min-height:100vh;background:#070b14;color:#e5e7eb;font-family:Inter,ui-sans-serif,system-ui,sans-serif}.page{width:min(100% - 2rem,980px);margin:0 auto;padding:2rem 0 4rem}.actions{display:flex;justify-content:space-between;gap:1rem;margin-bottom:1rem}.actions a,.actions button,.home-link{border:1px solid #334155;border-radius:.65rem;background:#111827;color:#e5e7eb;padding:.7rem 1rem;font:inherit;text-decoration:none;cursor:pointer}.certificate,.verification{position:relative;overflow:hidden;border:1px solid rgba(52,211,153,.55);border-radius:1.5rem;background:radial-gradient(circle at top right,rgba(249,115,22,.16),transparent 35%),linear-gradient(145deg,#101827,#0b1220);box-shadow:0 28px 80px rgba(0,0,0,.4);padding:clamp(1.5rem,6vw,4.5rem);text-align:center}.certificate:before{position:absolute;inset:1rem;border:1px solid rgba(255,255,255,.1);border-radius:1rem;content:"";pointer-events:none}.mark{display:grid;width:4.5rem;height:4.5rem;place-items:center;margin:0 auto 1.5rem;border-radius:50%;background:linear-gradient(135deg,#10b981,#f97316);color:#fff;font-size:2rem;font-weight:900}.eyebrow{color:#6ee7b7;font-size:.78rem;font-weight:800;letter-spacing:.12em;text-transform:uppercase}h1{margin:.7rem auto 2rem;color:#fff;font-size:clamp(2rem,7vw,4.3rem);line-height:1}.awarded{margin:0;color:#94a3b8}.holder{margin:.35rem 0 1.4rem;color:#fff;font-family:Georgia,serif;font-size:clamp(1.8rem,6vw,3.4rem)}.description{max-width:680px;margin:0 auto 2rem;color:#cbd5e1;line-height:1.7}dl{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:1rem;margin:2rem 0;text-align:left}dl div{min-width:0;padding:1rem;border:1px solid rgba(255,255,255,.08);border-radius:.75rem;background:rgba(2,6,23,.45)}dt{margin-bottom:.4rem;color:#94a3b8;font-size:.72rem;font-weight:750;letter-spacing:.06em;text-transform:uppercase}dd{margin:0;overflow-wrap:anywhere;color:#f8fafc}.state{font-weight:800}.state--valid{color:#6ee7b7}.state--revoked{color:#fca5a5}.verify,.privacy{color:#94a3b8;font-size:.86rem;line-height:1.55}.verify a,.privacy a,.home-link{color:#a7f3d0}.page--verify{max-width:760px}.verification-result{font-size:1.4rem}.verification dl{grid-template-columns:2fr 1fr 1fr}.sandbox-warning,.live-warning{margin:1.5rem 0;padding:1rem;border:1px solid rgba(59,130,246,.5);border-radius:.8rem;background:rgba(30,64,175,.18);text-align:left}.sandbox-warning p,.live-warning p{margin:.35rem 0 0;color:#bfdbfe;line-height:1.55}.live-warning{border-color:rgba(16,185,129,.55);background:rgba(6,78,59,.25)}.live-warning p{color:#a7f3d0}.home-link{display:inline-block;margin-top:1rem}@media(max-width:700px){.page{width:min(100% - 1rem,980px);padding-top:.5rem}.actions{align-items:stretch;flex-direction:column}.actions a,.actions button{text-align:center}.certificate,.verification{padding:2.6rem 1.15rem}.certificate:before{inset:.5rem}dl,.verification dl{grid-template-columns:1fr}.holder{overflow-wrap:anywhere}}@media print{body{background:#fff;color:#111}.page{width:100%;padding:0}.actions,.privacy{display:none}.certificate{min-height:95vh;border:3px double #047857;background:#fff;box-shadow:none;color:#111}.certificate:before{border-color:#d1d5db}.certificate h1,.certificate .holder,.certificate dd{color:#111}.description,.awarded,.verify,.certificate dt{color:#374151}.mark{print-color-adjust:exact;-webkit-print-color-adjust:exact}}
"#;

#[cfg(test)]
mod tests {
    use super::{issued_date, owner_certificate_page, public_verification_page};

    #[test]
    fn private_certificate_wires_print_after_the_document_is_ready() {
        let page = owner_certificate_page(
            "Certificate Holder",
            "RST-LIVE-0123456789ABCDEF0123456789ABCDEF",
            "founding_customer",
            "live",
            "active",
            "active",
            "2026-09-17 12:00:00",
            "test-nonce",
        )
        .0;
        assert!(page.contains("DOMContentLoaded"));
        assert!(page.contains("window.print()"));
    }

    #[test]
    fn public_page_omits_holder_and_payment_fields() {
        let page = public_verification_page(
            "RST-SBX-0123456789ABCDEF0123456789ABCDEF",
            "sandbox_pioneer",
            "test",
            true,
            "2026-09-17 12:00:00",
            "test-nonce",
        )
        .0;
        assert!(page.contains("Sandbox certificate"));
        assert!(!page.contains("holder_name"));
        assert!(!page.contains("provider_payment_id"));
    }

    #[test]
    fn display_date_does_not_require_database_specific_parsing() {
        assert_eq!(issued_date("2026-09-17 12:00:00"), "2026-09-17");
        assert_eq!(issued_date("unknown"), "unknown");
    }

    #[test]
    fn live_page_is_distinct_and_privacy_preserving() {
        let page = public_verification_page(
            "RST-LIVE-0123456789ABCDEF0123456789ABCDEF",
            "founding_customer",
            "live",
            true,
            "2026-09-17 12:00:00",
            "test-nonce",
        )
        .0;
        assert!(page.contains("Rullst Founding Customer"));
        assert!(page.contains("Live purchase certificate"));
        assert!(!page.contains("provider_payment_id"));
    }
}
