use rullst::response::Html;

const EFFECTIVE_DATE: &str = "2026-09-17";
const VERSION: &str = "1.0";

fn page(title: &str, subtitle: &str, content: &str, csp_nonce: &str) -> Html<String> {
    Html(format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta name=\"robots\" content=\"noindex,nofollow,noarchive\"><title>{}</title><link rel=\"icon\" type=\"image/png\" href=\"/static/rullst.png\"><style nonce=\"{}\">{}</style></head><body><main><nav><a href=\"/\">SaaS blueprint</a><a href=\"/privacy\">Privacy notice</a><a href=\"/terms\">Sandbox terms</a></nav><header><p class=\"eyebrow\">Rullst SaaS staging</p><h1>{}</h1><p class=\"subtitle\">{}</p><p class=\"version\">Effective {} · Version {}</p></header>{}<footer><p>Privacy and support contact: <a href=\"mailto:officialrullst@gmail.com\">officialrullst@gmail.com</a></p><p>This engineering notice is not a claim that the staging example automatically complies with every law in every country.</p></footer></main></body></html>",
        rullst::html::escape_str(title),
        rullst::html::escape_str(csp_nonce),
        LEGAL_CSS,
        rullst::html::escape_str(title),
        rullst::html::escape_str(subtitle),
        EFFECTIVE_DATE,
        VERSION,
        content,
    ))
}

pub fn privacy_notice_page(csp_nonce: &str) -> Html<String> {
    page(
        "Privacy notice",
        "What the public Stripe sandbox collects, why it is used and how to exercise data rights.",
        r#"
        <section><h2>Scope and controller contact</h2><p>This notice covers the public Rullst SaaS staging blueprint. It is a technical test environment, not the future live-money service. The operator is an individual seller in Brazil using the public brand Rullst. Privacy requests are handled at <a href="mailto:officialrullst@gmail.com">officialrullst@gmail.com</a>. Do not send passwords, card data or government identifiers by email.</p></section>
        <section><h2>Data processed</h2><ul><li>Account name, normalized email address and an Argon2id password hash.</li><li>Internal account, purchase-attempt, entitlement and certificate references.</li><li>Provider session, payment and event references needed for signature verification, idempotency and reconciliation. These are confidential and never shown publicly.</li><li>Essential encrypted-session and CSRF cookies. Advertising, analytics and personalization cookies are not enabled.</li></ul><p>Rullst does not receive or store raw card numbers. Stripe hosts payment-method collection. Use only Stripe's documented test payment methods in this environment.</p></section>
        <section><h2>Purposes and sharing</h2><p>Data is used only to authenticate the requested account, create and reconcile a sandbox Checkout, grant the versioned test entitlement, issue or verify the Sandbox Pioneer certificate, prevent abuse and investigate failures. Starting Checkout sends the account email and server-owned offer metadata to Stripe. The application runs in Microsoft Azure and stores staging records in Neon PostgreSQL hosted in an AWS US region. This can involve international processing; a production launch requires a reviewed transfer and subprocessor assessment for the actual users served.</p></section>
        <section><h2>Public certificate boundary</h2><p>The authenticated certificate can display the account name. The public verification URL contains a random identifier and exposes only badge type, issue date, test environment and current validity. It does not expose name, email, amount, country, payment method or Stripe identifiers. Sharing the random URL is the holder's choice and never grants account access.</p></section>
        <section><h2>Retention and rights</h2><p>Staging records may be reset and are not production records. Automated export and deletion are not yet offered in the UI, which is one reason live-money production remains blocked. Contact the address above to request access, correction, a portable copy, closure/deletion, restriction or human review. Some security and reconciliation evidence can require temporary restricted retention; the response will explain any applicable exception. Requests are verified before account data is disclosed or changed.</p></section>
        <section><h2>Security and minors</h2><p>TLS, encrypted session cookies, CSRF protection, password hashing, provider-hosted payment entry and signed webhook reconciliation protect the staging flow. No internet service can promise absolute security. Public content can be viewed by minors, but an account purchase flow must be operated by an adult or a parent/legal guardian. The application does not ask for a birth date.</p></section>
        <section><h2>Notices and complaints</h2><p>Material changes receive a new version and effective date. You may contact the operator first and may also complain to the data-protection authority applicable to you. Production use requires jurisdiction-specific notices and qualified legal review.</p></section>
        "#,
        csp_nonce,
    )
}

pub fn sandbox_terms_page(csp_nonce: &str) -> Html<String> {
    page(
        "Sandbox terms",
        "Conditions for using the public technical demonstration.",
        r#"
        <section><h2>Test environment only</h2><p>This site demonstrates a constrained Stripe sandbox integration. It must use provider test data and cannot create a real charge. A sandbox result, report entitlement or Sandbox Pioneer certificate is not proof of a real purchase, professional certification or customer relationship.</p></section>
        <section><h2>Who may use it</h2><p>You must be at least 18 years old or use the flow through a parent or legal guardian. Do not create an account for another person without authority. Use an email address you control and avoid unnecessary personal information.</p></section>
        <section><h2>Acceptable use</h2><ul><li>Use only Stripe's documented test payment methods.</li><li>Do not probe other users, bypass authorization, automate abuse, overload the service or submit secrets and real financial information.</li><li>Do not present a sandbox certificate as evidence that money moved or that Rullst endorsed a person or organization.</li><li>Report security issues privately to <a href="mailto:officialrullst@gmail.com">officialrullst@gmail.com</a>.</li></ul></section>
        <section><h2>Availability and resets</h2><p>The staging service is provided for evaluation and can scale to zero, start slowly, change, reset data or be unavailable. No uptime, permanence or production-support commitment is made. The future live service will require separate production terms, refund rules and merchant notices.</p></section>
        <section><h2>No real payment or refund</h2><p>Because the published staging flow creates no real charge, there is no real amount to refund. If a real charge ever appears, stop and contact the operator and Stripe; that would not be an expected staging result.</p></section>
        <section><h2>Account action</h2><p>Creating an account after these links are presented records no optional marketing consent. It only starts the requested sandbox account. You may stop using the service and request closure through the contact address in the privacy notice.</p></section>
        "#,
        csp_nonce,
    )
}

const LEGAL_CSS: &str = r#"
:root{color-scheme:dark}*{box-sizing:border-box}body{margin:0;background:#080d17;color:#e5e7eb;font-family:Inter,ui-sans-serif,system-ui,sans-serif;line-height:1.7}main{width:min(100% - 2rem,860px);margin:0 auto;padding:2rem 0 4rem}nav{display:flex;flex-wrap:wrap;gap:.75rem;margin-bottom:3rem}nav a,footer a,section a{color:#6ee7b7}nav a{padding:.55rem .8rem;border:1px solid #334155;border-radius:.6rem;text-decoration:none}header{padding:clamp(1.5rem,5vw,3rem);border:1px solid rgba(52,211,153,.35);border-radius:1.25rem;background:linear-gradient(145deg,#111827,#0b1220)}.eyebrow{margin:0;color:#6ee7b7;font-size:.78rem;font-weight:800;letter-spacing:.1em;text-transform:uppercase}h1{margin:.4rem 0;color:#fff;font-size:clamp(2.2rem,8vw,4rem);line-height:1.05}.subtitle,.version{color:#94a3b8}.version{font-size:.85rem}section{margin-top:1.25rem;padding:clamp(1.25rem,4vw,2rem);border:1px solid rgba(255,255,255,.08);border-radius:1rem;background:rgba(15,23,42,.65)}h2{margin-top:0;color:#f8fafc;font-size:1.25rem}li+li{margin-top:.5rem}footer{margin-top:2rem;padding-top:1.5rem;border-top:1px solid #1f2937;color:#94a3b8;font-size:.88rem}@media(max-width:600px){main{width:min(100% - 1rem,860px);padding-top:.75rem}nav{align-items:stretch;flex-direction:column;margin-bottom:1rem}nav a{text-align:center}ul{padding-left:1.25rem}}
"#;

#[cfg(test)]
mod tests {
    use super::{privacy_notice_page, sandbox_terms_page};

    #[test]
    fn privacy_notice_names_data_rights_and_public_certificate_boundary() {
        let page = privacy_notice_page("nonce").0;
        assert!(page.contains("access, correction, a portable copy"));
        assert!(page.contains("does not expose name, email"));
        assert!(page.contains("officialrullst@gmail.com"));
    }

    #[test]
    fn sandbox_terms_never_describe_test_activity_as_a_purchase() {
        let page = sandbox_terms_page("nonce").0;
        assert!(page.contains("cannot create a real charge"));
        assert!(page.contains("not proof of a real purchase"));
    }
}
