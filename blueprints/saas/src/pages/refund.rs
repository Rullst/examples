use rullst::response::Html;

pub fn refund_page(
    eligible: bool,
    status: Option<&str>,
    refund_window_days: u16,
    csrf_token: &str,
    csp_nonce: &str,
) -> Html<String> {
    let action = match status {
        Some("requested" | "processing") => {
            "<div class=\"notice\"><strong>Refund requested</strong><p>The request is awaiting review. This is not a bank dispute and does not move money immediately. If approved, the full refund is issued in Stripe; access remains active until Stripe confirms it.</p></div>".to_owned()
        }
        Some("completed") => {
            "<div class=\"notice notice--done\"><strong>Refund completed</strong><p>Stripe confirmed the refund and the associated access was revoked. The credit usually takes 5&ndash;10 business days to appear, depending on the bank.</p></div>".to_owned()
        }
        _ if eligible => format!(
            "<form method=\"post\" action=\"/refund\"><input type=\"hidden\" name=\"_token\" value=\"{}\"><input type=\"hidden\" name=\"confirmation\" value=\"request_full_refund\"><label><input type=\"checkbox\" required> I understand that a completed refund revokes the purchased guide and certificate.</label><button type=\"submit\">Request full refund</button></form><div class=\"explanation\"><h2>Before requesting</h2><ul><li>This sends a request to the seller; it is not a bank dispute and does not refund the payment immediately.</li><li>If approved, the full charged amount is returned through Stripe. Banks usually display it within 5&ndash;10 business days.</li><li>A refund completed before a card dispute is opened may prevent a later dispute fee, but simultaneous cases cannot be guaranteed. Once a dispute is opened, its fee generally cannot be avoided.</li><li>Stripe generally does not return the original payment-processing fee to the seller.</li></ul></div><p class=\"muted\">Online requests are available for {refund_window_days} calendar days after purchase. Contact support if a non-waivable legal right requires a different period.</p>",
            rullst::html::escape_str(csrf_token),
        ),
        _ => "<div class=\"notice\"><strong>Online request window closed</strong><p>Contact officialrullst@gmail.com so any applicable statutory right can be reviewed.</p></div>".to_owned(),
    };
    Html(format!(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta name="robots" content="noindex,nofollow,noarchive"><title>Refund request - Rullst</title><style nonce="{}">{}</style></head><body><main><a class="back" href="/dashboard">Back to dashboard</a><section><p class="eyebrow">Stripe refund workflow</p><h1>Request a refund</h1><p>Submitting this form does not move money automatically. The seller reviews the request in Nexus and issues the refund through Stripe. A signed Stripe webhook then revokes access and certificate validity.</p>{}</section></main></body></html>"#,
        rullst::html::escape_str(csp_nonce),
        REFUND_CSS,
        action,
    ))
}

pub fn refund_unavailable_page(csp_nonce: &str) -> Html<String> {
    Html(format!(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta name="robots" content="noindex,nofollow,noarchive"><title>Refund request temporarily unavailable - Rullst</title><style nonce="{}">{}</style></head><body><main><a class="back" href="/dashboard">Back to dashboard</a><section><p class="eyebrow">Your refund right is preserved</p><h1>Refund request temporarily unavailable</h1><p>Your purchase has not been changed. Please return to the dashboard and try again shortly. If the problem continues, email <a href="mailto:officialrullst@gmail.com">officialrullst@gmail.com</a>; the time of your first contact can be used when reviewing the published refund window.</p></section></main></body></html>"#,
        rullst::html::escape_str(csp_nonce),
        REFUND_CSS,
    ))
}

const REFUND_CSS: &str = r#":root{color-scheme:dark}*{box-sizing:border-box}body{margin:0;background:#070b14;color:#e5e7eb;font-family:Inter,system-ui,sans-serif;line-height:1.6}main{width:min(100% - 2rem,720px);margin:0 auto;padding:2rem 0}.back,section a{color:#6ee7b7}.back{display:inline-block;margin-bottom:1rem}section{padding:clamp(1.25rem,5vw,2.5rem);border:1px solid #334155;border-radius:1rem;background:#0f172a}.eyebrow{color:#6ee7b7;font-size:.78rem;font-weight:800;text-transform:uppercase}h1{margin:.25rem 0 1rem;color:#fff}.explanation{margin-top:1rem;padding:1rem;border:1px solid #334155;border-radius:.75rem;background:#0b1220}.explanation h2{margin:0 0 .5rem;font-size:1rem}.explanation ul{margin:.5rem 0;padding-left:1.2rem}.explanation li+li{margin-top:.45rem}form,.notice{margin-top:1.5rem;padding:1rem;border:1px solid #475569;border-radius:.75rem;background:#111827}label{display:flex;gap:.65rem;align-items:flex-start}button{width:100%;margin-top:1rem;padding:.8rem;border:0;border-radius:.6rem;background:#10b981;color:#032117;font:inherit;font-weight:800;cursor:pointer}.muted{color:#94a3b8;font-size:.9rem}.notice--done{border-color:#10b981}@media(max-width:600px){main{width:min(100% - 1rem,720px);padding-top:.75rem}}"#;

#[cfg(test)]
mod tests {
    use super::{refund_page, refund_unavailable_page};

    #[test]
    fn refund_page_distinguishes_a_refund_request_from_a_dispute() {
        let page = refund_page(true, None, 14, "csrf", "nonce").0;
        assert!(page.contains("not a bank dispute"));
        assert!(page.contains("5&ndash;10 business days"));
        assert!(page.contains("14 calendar days"));
    }

    #[test]
    fn temporary_failure_preserves_a_support_path() {
        let page = refund_unavailable_page("nonce").0;
        assert!(page.contains("Your refund right is preserved"));
        assert!(page.contains("officialrullst@gmail.com"));
    }
}
