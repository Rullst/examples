use crate::pages::legal;
use rullst::server::Extension;

fn nonce(csp_nonce: &Option<Extension<rullst::security::CspNonce>>) -> &str {
    csp_nonce
        .as_ref()
        .map(|Extension(nonce)| nonce.as_str())
        .unwrap_or_default()
}

pub async fn privacy_notice(
    csp_nonce: Option<Extension<rullst::security::CspNonce>>,
) -> rullst::response::Html<String> {
    legal::privacy_notice_page(nonce(&csp_nonce))
}

pub async fn sandbox_terms(
    csp_nonce: Option<Extension<rullst::security::CspNonce>>,
) -> rullst::response::Html<String> {
    legal::sandbox_terms_page(nonce(&csp_nonce))
}
