use rullst::response::Html;

use crate::controllers::legal_controller::MerchantNotice;

const EFFECTIVE_DATE: &str = "2026-09-19";
const VERSION: &str = "1.4";

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
        <section><h2>Data processed</h2><ul><li>Permanent account/certificate name, normalized email address and an Argon2id password hash.</li><li>Hashed, short-lived password-reset codes, keyed request-limit identifiers, transactional-mail delivery state and a PostgreSQL session registry. Complete reset codes are not stored.</li><li>Internal account, purchase-attempt, entitlement and certificate references.</li><li>Provider session, payment and event references needed for signature verification, idempotency and reconciliation. These are confidential and never shown publicly.</li><li>Bounded connection and security metadata, such as IP address, user agent, timestamps and rejected-request class, can be processed by the hosting and security boundary. Application logs must not contain passwords, reset codes, card data or raw webhook bodies.</li><li>Essential encrypted-session and CSRF cookies. Advertising, analytics and personalization cookies are not enabled.</li></ul><p>Rullst does not receive or store raw card numbers. Stripe hosts payment-method collection. Use only Stripe's documented test payment methods in this environment.</p></section>
        <section><h2>Purposes and sharing</h2><p>Data is used only to authenticate or recover the requested account, create and reconcile a sandbox Checkout, grant the versioned test entitlement, issue or verify the Sandbox Pioneer certificate, deliver one transactional confirmation with the requested access links, prevent abuse and investigate failures. Starting Checkout sends the account email and server-owned offer metadata to Stripe. When account mail is enabled, the recipient address and deterministic password-recovery or purchase-confirmation message are sent to Resend for transactional delivery; no marketing or open/click tracking is added. The application runs in Microsoft Azure and stores staging records in Neon PostgreSQL hosted in an AWS US region. This can involve international processing; a production launch requires a reviewed transfer and subprocessor assessment for the actual users served.</p></section>
        <section><h2>Public certificate boundary</h2><p>The authenticated certificate can display the account name. The public verification URL contains a random identifier and exposes only badge type, issue date, test environment and current validity. It does not expose name, email, amount, country, payment method or Stripe identifiers. Sharing the random URL is the holder's choice and never grants account access. Sandbox activity is kept separate and never increases the production Founding Customer count.</p></section>
        <section><h2>Retention and rights</h2><p>Expired password-reset request and token records are removed after a 24-hour abuse-investigation window. Other staging records may be reset and are not production records. An authenticated account can export its application data from the dashboard. Contact the address above to request access, correction, a portable copy, closure/deletion, restriction or human review. Some security and reconciliation evidence can require temporary restricted retention; the response will explain any applicable exception. Requests are verified before account data is disclosed or changed.</p></section>
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
        <section><h2>Who may use it</h2><p>You must be at least 18 years old or use the flow through a parent or legal guardian. Do not create an account for another person without authority. Use an email address you control and avoid unnecessary personal information. The registered name is permanent for this showcase and is used on any sandbox certificate issued to the account.</p></section>
        <section><h2>Acceptable use</h2><ul><li>Use only Stripe's documented test payment methods.</li><li>Do not probe other users, bypass authorization, automate abuse, overload the service or submit secrets and real financial information.</li><li>Do not present a sandbox certificate as evidence that money moved or that Rullst endorsed a person or organization.</li><li>Report security issues privately to <a href="mailto:officialrullst@gmail.com">officialrullst@gmail.com</a>.</li></ul></section>
        <section><h2>Availability and resets</h2><p>The staging service is provided for evaluation and can scale to zero, start slowly, change, reset data or be unavailable. No uptime, permanence or production-support commitment is made. The future live service will require separate production terms, refund rules and merchant notices.</p></section>
        <section><h2>No real payment or refund</h2><p>Because the published staging flow creates no real charge, there is no real amount to refund. If a real charge ever appears, stop and contact the operator and Stripe; that would not be an expected staging result.</p></section>
        <section><h2>Account action</h2><p>Creating an account after these links are presented records no optional marketing consent. It only starts the requested sandbox account. You may stop using the service and request closure through the contact address in the privacy notice.</p></section>
        "#,
        csp_nonce,
    )
}

fn prelaunch_page(title: &str, subtitle: &str, content: &str, csp_nonce: &str) -> Html<String> {
    Html(format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta name=\"robots\" content=\"noindex,nofollow,noarchive\"><title>{}</title><link rel=\"icon\" type=\"image/png\" href=\"/static/rullst.png\"><style nonce=\"{}\">{}</style></head><body><main><nav><a href=\"/\">SaaS blueprint</a><a href=\"/privacy\">Privacy notice</a><a href=\"/terms\">Pre-launch terms</a></nav><header><p class=\"eyebrow\">Rullst SaaS production pre-launch</p><h1>{}</h1><p class=\"subtitle\">{}</p><p class=\"version\">Effective {} &middot; Version {}</p></header>{}<footer><p>Privacy and support contact: <a href=\"mailto:officialrullst@gmail.com\">officialrullst@gmail.com</a></p><p>Real-money Checkout is disabled. These pre-launch notices do not replace the seller disclosure and purchase terms required before launch.</p></footer></main></body></html>",
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

pub fn prelaunch_privacy_notice_page(csp_nonce: &str) -> Html<String> {
    prelaunch_page(
        "Pre-launch privacy notice",
        "What the production pre-launch site processes while real-money Checkout remains disabled.",
        r#"
        <section><h2>Scope and contact</h2><p>This notice covers the Rullst SaaS production infrastructure before payment launch. Real-money Checkout is disabled. Privacy requests are handled at <a href="mailto:officialrullst@gmail.com">officialrullst@gmail.com</a>. Do not send passwords, card data or government identifiers by email.</p></section>
        <section><h2>Account data</h2><p>If you create an account before launch, the application stores the supplied name, normalized email, an Argon2id password hash and bounded authentication/security metadata. If password recovery is enabled, it also stores hashed short-lived reset codes, keyed request-limit identifiers, delivery state and revocable session references; complete reset codes are not stored. The recipient address and deterministic security message are sent to Resend for transactional delivery without marketing tracking. Essential encrypted-session and CSRF cookies are used. Advertising, analytics and personalization cookies are not enabled.</p></section>
        <section><h2>No payment processing</h2><p>The application cannot initiate a Stripe Checkout while the production payment gate is disabled. Do not enter real payment information anywhere on this site. Use the separate staging environment only with Stripe's documented test methods.</p></section>
        <section><h2>Infrastructure and rights</h2><p>The application runs in Microsoft Azure and stores application records in Neon PostgreSQL hosted in an AWS US region. You may request access, correction, export, restriction or closure through the contact above. Identity is verified before account information is disclosed or changed.</p></section>
        "#,
        csp_nonce,
    )
}

pub fn prelaunch_terms_page(csp_nonce: &str) -> Html<String> {
    prelaunch_page(
        "Production pre-launch terms",
        "Conditions for using the production site before real-money Checkout is enabled.",
        r#"
        <section><h2>No offer or charge yet</h2><p>The production application is online for readiness validation, but Checkout is disabled. No button on this site can currently create a real payment, purchase entitlement or Founding Customer certificate.</p></section>
        <section><h2>Account use</h2><p>You must control the email address used for an account and must not probe other users, bypass authorization, automate abuse, overload the service or submit secrets or real financial information. Creating an account records no optional marketing consent.</p></section>
        <section><h2>Launch boundary</h2><p>A real offer will appear only after the seller identity, exact price, refund terms, private deliverable, signed Stripe webhook, reconciliation and backup controls are configured. The page will then state clearly that Checkout creates a real charge and will publish the applicable purchase terms before payment.</p></section>
        <section><h2>Availability</h2><p>This low-cost serverless showcase can scale to zero, start slowly or be temporarily unavailable. No uptime commitment is made during pre-launch.</p></section>
        "#,
        csp_nonce,
    )
}

fn production_page(
    title: &str,
    subtitle: &str,
    content: &str,
    support_email: &str,
    csp_nonce: &str,
) -> Html<String> {
    Html(format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta name=\"robots\" content=\"index,follow\"><title>{}</title><link rel=\"icon\" type=\"image/png\" href=\"/static/rullst.png\"><style nonce=\"{}\">{}</style></head><body><main><nav><a href=\"/\">SaaS blueprint</a><a href=\"/privacy\">Privacy notice</a><a href=\"/terms\">Purchase and refund terms</a></nav><header><p class=\"eyebrow\">Rullst SaaS live showcase</p><h1>{}</h1><p class=\"subtitle\">{}</p><p class=\"version\">Effective {} &middot; Version {}</p></header>{}<footer><p>Privacy, refunds and support: <a href=\"mailto:{}\">{}</a></p><p>This notice does not replace rights that cannot lawfully be excluded in the customer's jurisdiction.</p></footer></main></body></html>",
        rullst::html::escape_str(title),
        rullst::html::escape_str(csp_nonce),
        LEGAL_CSS,
        rullst::html::escape_str(title),
        rullst::html::escape_str(subtitle),
        EFFECTIVE_DATE,
        VERSION,
        content,
        rullst::html::escape_str(support_email),
        rullst::html::escape_str(support_email),
    ))
}

pub fn production_privacy_notice_page(merchant: &MerchantNotice, csp_nonce: &str) -> Html<String> {
    let legal_name = rullst::html::escape_str(&merchant.legal_name);
    let tax_id = rullst::html::escape_str(&merchant.tax_id);
    let physical_address = rullst::html::escape_str(&merchant.physical_address);
    let country = rullst::html::escape_str(&merchant.country);
    let support_email = rullst::html::escape_str(&merchant.support_email);
    let content = format!(
        r#"<section><h2>Controller and contact</h2><p><strong>{legal_name}</strong>, operating under the Rullst brand in {country}, controls the application account and purchase records.</p><dl><dt>Brazilian tax registration</dt><dd>{tax_id}</dd><dt>Physical address</dt><dd>{physical_address}</dd><dt>Electronic and support address</dt><dd><a href="mailto:{support_email}">{support_email}</a></dd></dl><p>Do not email passwords, card numbers or identity documents unless a verified support process specifically requires them.</p></section>
        <section><h2>Data processed</h2><p>The service stores the permanent account/certificate name, normalized email, Argon2id password hash, internal identifiers, purchase attempts, entitlement state, certificate state, refund-request state, transactional-mail delivery state and bounded security metadata. Password recovery additionally uses hashed short-lived reset codes, keyed request-limit identifiers and revocable session references; complete reset codes are not stored. Stripe hosts payment collection; Rullst does not receive or store complete card numbers or security codes.</p></section>
        <section><h2>Purposes and providers</h2><p>Data is processed to create, authenticate and recover accounts, prevent abuse, complete the requested one-time purchase, reconcile signed Stripe events, deliver the purchased artifact, issue and verify a certificate, send one purchase confirmation with the requested account and community links, handle refunds and disputes, maintain security and meet legal obligations. The application runs in Microsoft Azure, application records are stored in Neon PostgreSQL, payments are processed by Stripe, and enabled password-recovery and purchase-confirmation messages are delivered by Resend using the account email without marketing or open/click tracking. These providers can process data internationally under their respective contractual safeguards.</p></section>
        <section><h2>Public certificate and aggregate-count boundary</h2><p>The authenticated certificate can display the account name. Public verification uses a random identifier and discloses only certificate type, issue date, environment and current validity. It never publishes the holder's name, email, amount, payment method or provider identifiers. The offer page publishes only an aggregate count of distinct accounts with an active reconciled live entitlement and active Founding Customer certificate. Refunded, disputed, revoked and sandbox records are excluded; no buyer identifier is included in the count.</p></section>
        <section><h2>Retention and rights</h2><p>Expired password-reset request and token records are removed after a 24-hour abuse-investigation window. Account and purchase evidence is retained only as necessary for delivery, fraud prevention, refunds, disputes, accounting and applicable legal obligations. An authenticated account can download its application data from the dashboard. You may request correction, restriction, objection, account closure or deletion through the contact above. Identity is verified before account data is disclosed or changed; legally required records can be restricted rather than immediately erased.</p></section>
        <section><h2>Security and minors</h2><p>TLS, encrypted sessions, CSRF protection, password hashing, server-owned prices, hosted payment entry, signed webhook verification, replay controls and private artifact integrity checks protect the service. No internet service can promise absolute security. A purchase must be made by an adult or by a parent or legal guardian acting for a minor.</p></section>"#,
    );
    production_page(
        "Privacy notice",
        "How the live Rullst SaaS showcase processes account and purchase data.",
        &content,
        &merchant.support_email,
        csp_nonce,
    )
}

pub fn production_terms_page(merchant: &MerchantNotice, csp_nonce: &str) -> Html<String> {
    let legal_name = rullst::html::escape_str(&merchant.legal_name);
    let tax_id = rullst::html::escape_str(&merchant.tax_id);
    let physical_address = rullst::html::escape_str(&merchant.physical_address);
    let support_email = rullst::html::escape_str(&merchant.support_email);
    let days = merchant.refund_window_days;
    let content = format!(
        r#"<section><h2>Seller identification</h2><p><strong>{legal_name}</strong>, operating under the Rullst brand.</p><dl><dt>Brazilian tax registration</dt><dd>{tax_id}</dd><dt>Physical address</dt><dd>{physical_address}</dd><dt>Electronic and support address</dt><dd><a href="mailto:{support_email}">{support_email}</a></dd></dl></section>
        <section><h2>Product</h2><p>The one-time digital product is a private, versioned Markdown guide containing the Rullst Stripe gateway field report and a sanitized implementation tutorial covering Stripe, PostgreSQL, GitHub OIDC, Azure Container Apps, DNS/TLS, signed webhooks, deployment validation and production promotion. Every reconciled live purchase also receives a Rullst Founding Customer certificate, without a quantity cap. This is not a subscription, investment, professional licence or guarantee of commercial results.</p></section>
        <section><h2>Price and delivery</h2><p>The exact price is presented and charged in Brazilian reais (BRL) before the customer leaves for Stripe Checkout. A foreign card issuer may convert the charge, add exchange or international fees, or decline it. Availability is not guaranteed in every country or for every payment method. Access is granted only after a signed provider event is independently reconciled. The authenticated dashboard delivers the current purchased artifact; a redirect alone never proves payment.</p></section>
        <section><h2>Refund policy</h2><p>The customer may request a full refund within {days} calendar days through the authenticated dashboard or by emailing <a href="mailto:{support_email}">{support_email}</a>. A dashboard request is not a bank dispute and does not move money automatically; the seller reviews it and processes approved refunds through Stripe. Stripe usually returns the full charged amount to the customer within 5&ndash;10 business days, subject to the bank. Stripe generally does not return the original payment-processing fee to the seller. Access and certificate validity are revoked only after Stripe confirms the refund. This voluntary policy does not reduce any longer or non-waivable consumer right that applies by law.</p></section>
        <section><h2>Disputes and support</h2><p>Contact support before opening a bank dispute so the seller can investigate delivery or issue a refund. This does not prevent a customer from exercising rights with a bank, payment provider or authority. Fraudulent use, credential sharing and attempts to bypass authorization are prohibited.</p></section>
        <section><h2>Availability and changes</h2><p>This is a public framework showcase using low-cost serverless infrastructure and can have cold-start delay or temporary interruption. Purchased artifact access will be restored after recoverable outages, but no uninterrupted-availability promise is made. Material terms changes apply prospectively and do not remove rights attached to an earlier purchase.</p></section>
        <section><h2>Certificate name</h2><p>The name submitted during account registration is permanent for this showcase and is printed on the authenticated certificate after purchase. The registration screen requires explicit acknowledgement before the account is created. Contact support before purchasing if the name was entered incorrectly.</p></section>
        <section><h2>Purchaser authority</h2><p>The purchaser confirms that they are at least 18 years old or are a parent or legal guardian authorized to make the purchase for a minor. The service does not request a birth date.</p></section>"#,
    );
    production_page(
        "Purchase and refund terms",
        "Terms for the one-time live purchase offered by the Rullst SaaS showcase.",
        &content,
        &merchant.support_email,
        csp_nonce,
    )
}

pub fn configuration_unavailable_page(csp_nonce: &str) -> Html<String> {
    production_page(
        "Notice unavailable",
        "The live merchant notice is not configured, so live Checkout remains unavailable.",
        "<section><h2>Fail-closed configuration</h2><p>No purchase should be attempted until the merchant legal name, Brazilian tax registration, physical address, support contact and refund window are configured.</p></section>",
        "officialrullst@gmail.com",
        csp_nonce,
    )
}

const LEGAL_CSS: &str = r#"
:root{color-scheme:dark}*{box-sizing:border-box}body{margin:0;background:#080d17;color:#e5e7eb;font-family:Inter,ui-sans-serif,system-ui,sans-serif;line-height:1.7}main{width:min(100% - 2rem,860px);margin:0 auto;padding:2rem 0 4rem}nav{display:flex;flex-wrap:wrap;gap:.75rem;margin-bottom:3rem}nav a,footer a,section a{color:#6ee7b7}nav a{padding:.55rem .8rem;border:1px solid #334155;border-radius:.6rem;text-decoration:none}header{padding:clamp(1.5rem,5vw,3rem);border:1px solid rgba(52,211,153,.35);border-radius:1.25rem;background:linear-gradient(145deg,#111827,#0b1220)}.eyebrow{margin:0;color:#6ee7b7;font-size:.78rem;font-weight:800;letter-spacing:.1em;text-transform:uppercase}h1{margin:.4rem 0;color:#fff;font-size:clamp(2.2rem,8vw,4rem);line-height:1.05}.subtitle,.version{color:#94a3b8}.version{font-size:.85rem}section{margin-top:1.25rem;padding:clamp(1.25rem,4vw,2rem);border:1px solid rgba(255,255,255,.08);border-radius:1rem;background:rgba(15,23,42,.65)}h2{margin-top:0;color:#f8fafc;font-size:1.25rem}dt{margin-top:.75rem;color:#94a3b8;font-size:.82rem;font-weight:800;text-transform:uppercase}dd{margin:.1rem 0;overflow-wrap:anywhere}li+li{margin-top:.5rem}footer{margin-top:2rem;padding-top:1.5rem;border-top:1px solid #1f2937;color:#94a3b8;font-size:.88rem}@media(max-width:600px){main{width:min(100% - 1rem,860px);padding-top:.75rem}nav{align-items:stretch;flex-direction:column;margin-bottom:1rem}nav a{text-align:center}ul{padding-left:1.25rem}}
"#;

#[cfg(test)]
mod tests {
    use super::{
        prelaunch_terms_page, privacy_notice_page, production_privacy_notice_page,
        production_terms_page, sandbox_terms_page,
    };
    use crate::controllers::legal_controller::MerchantNotice;

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

    #[test]
    fn production_terms_publish_escaped_required_seller_identification() {
        let merchant = merchant_notice();
        let page = production_terms_page(&merchant, "nonce").0;
        assert!(page.contains("Seller &amp; Owner"));
        assert!(page.contains("529.982.247-25"));
        assert!(page.contains("123 Public Street, Brazil"));
    }

    #[test]
    fn production_privacy_notice_explains_the_anonymous_public_count() {
        let page = production_privacy_notice_page(&merchant_notice(), "nonce").0;

        assert!(page.contains("aggregate count of distinct accounts"));
        assert!(page.contains("Refunded, disputed, revoked and sandbox records are excluded"));
        assert!(page.contains("Version 1.4"));
    }

    fn merchant_notice() -> MerchantNotice {
        MerchantNotice {
            legal_name: "Seller & Owner".to_owned(),
            tax_id: "529.982.247-25".to_owned(),
            physical_address: "123 Public Street, Brazil".to_owned(),
            country: "BR".to_owned(),
            support_email: "support@example.com".to_owned(),
            refund_window_days: 14,
        }
    }

    #[test]
    fn prelaunch_terms_do_not_claim_staging_or_real_checkout() {
        let page = prelaunch_terms_page("nonce").0;
        assert!(page.contains("production application is online"));
        assert!(page.contains("Checkout is disabled"));
        assert!(!page.contains("Rullst SaaS staging"));
    }
}
