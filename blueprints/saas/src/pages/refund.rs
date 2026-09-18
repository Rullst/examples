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
            "<div class=\"notice\"><strong>Refund requested</strong><p>The request is awaiting review. The refund will be issued in Stripe; access remains active until Stripe confirms it.</p></div>".to_owned()
        }
        Some("completed") => {
            "<div class=\"notice notice--done\"><strong>Refund completed</strong><p>Stripe confirmed the refund and the associated access was revoked.</p></div>".to_owned()
        }
        _ if eligible => format!(
            "<form method=\"post\" action=\"/refund\"><input type=\"hidden\" name=\"_token\" value=\"{}\"><input type=\"hidden\" name=\"confirmation\" value=\"request_full_refund\"><label><input type=\"checkbox\" required> I understand that a completed refund revokes the purchased guide and certificate.</label><button type=\"submit\">Request full refund</button></form><p class=\"muted\">Requests are available for {refund_window_days} calendar days after purchase. Contact support if a non-waivable legal right requires a different period.</p>",
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

const REFUND_CSS: &str = r#":root{color-scheme:dark}*{box-sizing:border-box}body{margin:0;background:#070b14;color:#e5e7eb;font-family:Inter,system-ui,sans-serif;line-height:1.6}main{width:min(100% - 2rem,720px);margin:0 auto;padding:2rem 0}.back{display:inline-block;margin-bottom:1rem;color:#6ee7b7}section{padding:clamp(1.25rem,5vw,2.5rem);border:1px solid #334155;border-radius:1rem;background:#0f172a}.eyebrow{color:#6ee7b7;font-size:.78rem;font-weight:800;text-transform:uppercase}h1{margin:.25rem 0 1rem;color:#fff}form,.notice{margin-top:1.5rem;padding:1rem;border:1px solid #475569;border-radius:.75rem;background:#111827}label{display:flex;gap:.65rem;align-items:flex-start}button{width:100%;margin-top:1rem;padding:.8rem;border:0;border-radius:.6rem;background:#10b981;color:#032117;font:inherit;font-weight:800;cursor:pointer}.muted{color:#94a3b8;font-size:.9rem}.notice--done{border-color:#10b981}@media(max-width:600px){main{width:min(100% - 1rem,720px);padding-top:.75rem}}"#;
