use crate::pages::legal;
use rullst::server::Extension;

#[derive(Debug, Clone)]
pub struct MerchantNotice {
    pub legal_name: String,
    pub country: String,
    pub support_email: String,
    pub refund_window_days: u16,
}

fn valid_email(value: &str) -> bool {
    value.len() <= 254
        && !value.contains(['\r', '\n'])
        && value
            .split_once('@')
            .is_some_and(|(local, domain)| !local.is_empty() && domain.contains('.'))
}

fn live_merchant_notice() -> Result<MerchantNotice, String> {
    let legal_name = std::env::var("MERCHANT_LEGAL_NAME")
        .unwrap_or_default()
        .trim()
        .to_owned();
    if legal_name.len() < 2 || legal_name.len() > 120 {
        return Err("MERCHANT_LEGAL_NAME must contain the seller's public legal name".to_owned());
    }
    let country = std::env::var("MERCHANT_COUNTRY")
        .unwrap_or_default()
        .trim()
        .to_ascii_uppercase();
    if country.len() != 2 || !country.bytes().all(|byte| byte.is_ascii_uppercase()) {
        return Err("MERCHANT_COUNTRY must be a two-letter country code".to_owned());
    }
    let support_email = std::env::var("SUPPORT_EMAIL")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if !valid_email(&support_email) {
        return Err("SUPPORT_EMAIL must be a valid monitored address".to_owned());
    }
    let refund_window_days = std::env::var("REFUND_WINDOW_DAYS")
        .unwrap_or_else(|_| "14".to_owned())
        .parse::<u16>()
        .map_err(|_| "REFUND_WINDOW_DAYS must be an integer".to_owned())?;
    if !(7..=30).contains(&refund_window_days) {
        return Err("REFUND_WINDOW_DAYS must be between 7 and 30".to_owned());
    }
    Ok(MerchantNotice {
        legal_name,
        country,
        support_email,
        refund_window_days,
    })
}

pub fn validate_live_merchant_configuration()
-> Result<(), crate::controllers::billing_controller::BillingConfigError> {
    live_merchant_notice()
        .map(|_| ())
        .map_err(crate::controllers::billing_controller::BillingConfigError::new)
}

pub fn live_mode() -> bool {
    std::env::var("PAYMENTS_MODE").is_ok_and(|value| value.trim().eq_ignore_ascii_case("live"))
}

pub fn refund_window_days() -> u16 {
    live_merchant_notice()
        .map(|merchant| merchant.refund_window_days)
        .unwrap_or(14)
}

fn nonce(csp_nonce: &Option<Extension<rullst::security::CspNonce>>) -> &str {
    csp_nonce
        .as_ref()
        .map(|Extension(nonce)| nonce.as_str())
        .unwrap_or_default()
}

pub async fn privacy_notice(
    csp_nonce: Option<Extension<rullst::security::CspNonce>>,
) -> rullst::response::Html<String> {
    if live_mode() {
        match live_merchant_notice() {
            Ok(merchant) => legal::production_privacy_notice_page(&merchant, nonce(&csp_nonce)),
            Err(_) => legal::configuration_unavailable_page(nonce(&csp_nonce)),
        }
    } else {
        legal::privacy_notice_page(nonce(&csp_nonce))
    }
}

pub async fn sandbox_terms(
    csp_nonce: Option<Extension<rullst::security::CspNonce>>,
) -> rullst::response::Html<String> {
    if live_mode() {
        match live_merchant_notice() {
            Ok(merchant) => legal::production_terms_page(&merchant, nonce(&csp_nonce)),
            Err(_) => legal::configuration_unavailable_page(nonce(&csp_nonce)),
        }
    } else {
        legal::sandbox_terms_page(nonce(&csp_nonce))
    }
}
