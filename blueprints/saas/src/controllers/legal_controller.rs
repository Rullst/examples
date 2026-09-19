use crate::pages::legal;
use rullst::server::Extension;

#[derive(Debug, Clone)]
pub struct MerchantNotice {
    pub legal_name: String,
    pub tax_id: String,
    pub physical_address: String,
    pub country: String,
    pub support_email: String,
    pub refund_window_days: u16,
}

fn valid_public_field(value: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&value.chars().count()) && !value.chars().any(char::is_control)
}

fn tax_id_digits(value: &str) -> Option<Vec<u8>> {
    if value.is_empty()
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_digit() && !matches!(byte, b'.' | b'-' | b'/' | b' '))
    {
        return None;
    }
    Some(
        value
            .bytes()
            .filter(u8::is_ascii_digit)
            .map(|byte| byte - b'0')
            .collect(),
    )
}

fn valid_cpf(digits: &[u8]) -> bool {
    if digits.len() != 11 || digits.iter().all(|digit| *digit == digits[0]) {
        return false;
    }
    let first_sum: u32 = digits[..9]
        .iter()
        .enumerate()
        .map(|(index, digit)| u32::from(*digit) * (10 - index as u32))
        .sum();
    let first = ((first_sum * 10) % 11) % 10;
    let second_sum: u32 = digits[..10]
        .iter()
        .enumerate()
        .map(|(index, digit)| u32::from(*digit) * (11 - index as u32))
        .sum();
    let second = ((second_sum * 10) % 11) % 10;
    u32::from(digits[9]) == first && u32::from(digits[10]) == second
}

fn cnpj_check_digit(digits: &[u8], weights: &[u32]) -> u32 {
    let sum: u32 = digits
        .iter()
        .zip(weights)
        .map(|(digit, weight)| u32::from(*digit) * weight)
        .sum();
    match sum % 11 {
        0 | 1 => 0,
        remainder => 11 - remainder,
    }
}

fn valid_cnpj(digits: &[u8]) -> bool {
    if digits.len() != 14 || digits.iter().all(|digit| *digit == digits[0]) {
        return false;
    }
    let first = cnpj_check_digit(&digits[..12], &[5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]);
    let second = cnpj_check_digit(&digits[..13], &[6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]);
    u32::from(digits[12]) == first && u32::from(digits[13]) == second
}

fn valid_brazilian_tax_id(value: &str) -> bool {
    tax_id_digits(value).is_some_and(|digits| valid_cpf(&digits) || valid_cnpj(&digits))
}

fn valid_email(value: &str) -> bool {
    value.len() <= 254
        && !value.contains(['\r', '\n'])
        && value
            .split_once('@')
            .is_some_and(|(local, domain)| !local.is_empty() && domain.contains('.'))
}

pub fn live_merchant_notice() -> Result<MerchantNotice, String> {
    let legal_name = std::env::var("MERCHANT_LEGAL_NAME")
        .unwrap_or_default()
        .trim()
        .to_owned();
    if !valid_public_field(&legal_name, 2, 120) {
        return Err("MERCHANT_LEGAL_NAME must contain the seller's public legal name".to_owned());
    }
    let tax_id = std::env::var("MERCHANT_TAX_ID")
        .unwrap_or_default()
        .trim()
        .to_owned();
    if !valid_brazilian_tax_id(&tax_id) {
        return Err("MERCHANT_TAX_ID must contain a valid Brazilian CPF or CNPJ".to_owned());
    }
    let physical_address = std::env::var("MERCHANT_PHYSICAL_ADDRESS")
        .unwrap_or_default()
        .trim()
        .to_owned();
    if !valid_public_field(&physical_address, 10, 240) {
        return Err(
            "MERCHANT_PHYSICAL_ADDRESS must contain the seller's public physical address"
                .to_owned(),
        );
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
        tax_id,
        physical_address,
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

pub fn production_deployment() -> bool {
    std::env::var("DEPLOYMENT_TIER")
        .is_ok_and(|value| value.trim().eq_ignore_ascii_case("production"))
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
    } else if production_deployment() {
        legal::prelaunch_privacy_notice_page(nonce(&csp_nonce))
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
    } else if production_deployment() {
        legal::prelaunch_terms_page(nonce(&csp_nonce))
    } else {
        legal::sandbox_terms_page(nonce(&csp_nonce))
    }
}

pub async fn cookies_notice(
    csp_nonce: Option<Extension<rullst::security::CspNonce>>,
) -> rullst::response::Html<String> {
    if live_mode() {
        match live_merchant_notice() {
            Ok(merchant) => legal::cookies_notice_page(nonce(&csp_nonce), Some(&merchant), true),
            Err(_) => legal::configuration_unavailable_page(nonce(&csp_nonce)),
        }
    } else {
        legal::cookies_notice_page(nonce(&csp_nonce), None, production_deployment())
    }
}

#[cfg(test)]
mod tests {
    use super::{valid_brazilian_tax_id, valid_public_field};

    #[test]
    fn accepts_valid_brazilian_tax_identifiers() {
        assert!(valid_brazilian_tax_id("529.982.247-25"));
        assert!(valid_brazilian_tax_id("11.222.333/0001-81"));
    }

    #[test]
    fn rejects_invalid_or_repeated_tax_identifiers() {
        assert!(!valid_brazilian_tax_id("111.111.111-11"));
        assert!(!valid_brazilian_tax_id("529.982.247-24"));
        assert!(!valid_brazilian_tax_id("11.222.333/0001-80"));
        assert!(!valid_brazilian_tax_id("52998224725<script>"));
    }

    #[test]
    fn public_fields_reject_control_characters() {
        assert!(valid_public_field("Public seller address", 10, 240));
        assert!(!valid_public_field("Address\nInjected", 10, 240));
    }
}
