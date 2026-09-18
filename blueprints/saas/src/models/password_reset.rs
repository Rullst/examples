use ring::hmac;
use ring::rand::{SecureRandom, SystemRandom};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

pub const RESET_LIFETIME_SECONDS: i64 = 15 * 60;
const CODE_HASH_DOMAIN: &[u8] = b"rullst.saas.password-reset-code.v1\0";
const REQUEST_KEY_DOMAIN: &[u8] = b"rullst.saas.password-reset-request.v1\0";
const CODE_MAC_DOMAIN: &[u8] = b"rullst.saas.password-reset-mac.v1\0";

#[derive(Debug, Clone)]
pub struct ResetCodeMaterial {
    pub selector: String,
    pub code: String,
    pub token_hash: String,
    pub expires_unix: i64,
}

fn unix_now() -> Result<i64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .map_err(|_| "system clock is before the Unix epoch".to_owned())
}

fn code_mac(selector: &[u8], user_id: i32, expires_unix: i64, app_key: &[u8]) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA256, app_key);
    let mut context = hmac::Context::with_key(&key);
    context.update(CODE_MAC_DOMAIN);
    context.update(selector);
    context.update(&user_id.to_be_bytes());
    context.update(&expires_unix.to_be_bytes());
    hex::encode(context.sign().as_ref())
}

pub fn generate_code(user_id: i32, app_key: &[u8]) -> Result<ResetCodeMaterial, String> {
    let mut selector = [0_u8; 32];
    SystemRandom::new()
        .fill(&mut selector)
        .map_err(|_| "secure random generation failed".to_owned())?;
    let expires_unix = unix_now()? + RESET_LIFETIME_SECONDS;
    let selector_hex = hex::encode(selector);
    let code = format!(
        "{}.{}",
        selector_hex,
        code_mac(&selector, user_id, expires_unix, app_key)
    );
    Ok(ResetCodeMaterial {
        token_hash: hash_code(&code),
        selector: selector_hex,
        code,
        expires_unix,
    })
}

pub fn reconstruct_code(
    selector_hex: &str,
    user_id: i32,
    expires_unix: i64,
    app_key: &[u8],
) -> Result<String, String> {
    let selector = hex::decode(selector_hex).map_err(|_| "invalid reset selector".to_owned())?;
    if selector.len() != 32 {
        return Err("invalid reset selector length".to_owned());
    }
    Ok(format!(
        "{}.{}",
        selector_hex,
        code_mac(&selector, user_id, expires_unix, app_key)
    ))
}

pub fn hash_code(code: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(CODE_HASH_DOMAIN);
    digest.update(code.as_bytes());
    hex::encode(digest.finalize())
}

pub fn request_key(email: &str, app_key: &[u8]) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA256, app_key);
    let mut context = hmac::Context::with_key(&key);
    context.update(REQUEST_KEY_DOMAIN);
    context.update(email.as_bytes());
    hex::encode(context.sign().as_ref())
}

pub fn valid_code_shape(code: &str) -> bool {
    let bytes = code.as_bytes();
    bytes.len() == 129
        && bytes.get(64) == Some(&b'.')
        && bytes[..64].iter().all(u8::is_ascii_hexdigit)
        && bytes[65..].iter().all(u8::is_ascii_hexdigit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_code_can_be_reconstructed_without_storing_it() {
        let key = b"a sufficiently long test-only application key";
        let material = generate_code(42, key).expect("generate reset material");
        assert!(valid_code_shape(&material.code));
        assert_eq!(material.token_hash.len(), 64);
        assert_eq!(
            reconstruct_code(&material.selector, 42, material.expires_unix, key)
                .expect("reconstruct code"),
            material.code
        );
    }

    #[test]
    fn code_is_bound_to_account_expiry_and_application_key() {
        let key = b"a sufficiently long test-only application key";
        let material = generate_code(42, key).expect("generate reset material");
        assert_ne!(
            reconstruct_code(&material.selector, 43, material.expires_unix, key)
                .expect("reconstruct code"),
            material.code
        );
        assert_ne!(
            reconstruct_code(&material.selector, 42, material.expires_unix + 1, key)
                .expect("reconstruct code"),
            material.code
        );
    }

    #[test]
    fn request_keys_do_not_store_plain_email_addresses() {
        let key = b"a sufficiently long test-only application key";
        let hashed = request_key("person@example.com", key);
        assert_eq!(hashed.len(), 64);
        assert!(!hashed.contains("person"));
    }
}
