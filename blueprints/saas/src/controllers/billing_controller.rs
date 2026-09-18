use crate::models::purchase_attempt::PurchaseAttempt;
use crate::models::user::User;
use crate::pages::billing::{self, PaymentPageState};
use reqwest::{Client, Response as HttpResponse, Url};
use rullst::db::{Orm, sqlx};
use rullst::server::{Extension, Form, HeaderMap, IntoResponse, Redirect, Response, StatusCode};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const CHECKOUT_OFFER: &str = "gateway-report-stripe";
const SANDBOX_ARTIFACT_VERSION: &str = "stripe-gateway-field-report-v1";
const LIVE_ARTIFACT_VERSION: &str = "stripe-rullst-production-guide-v1";
const MAX_LOW_VALUE_AMOUNT_MINOR: u64 = 1_000;
const MAX_PROVIDER_RESPONSE_BYTES: usize = 64 * 1024;
const MAX_WEBHOOK_BYTES: usize = 64 * 1024;
const STRIPE_WEBHOOK_TOLERANCE_SECONDS: u64 = 5 * 60;
static RECONCILIATION_RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct BillingIdentity {
    /// Authenticated owner ID, derived from a session plus tenant membership.
    pub owner_id: i32,
    /// Normalized email belonging to that authenticated owner.
    pub email: String,
}

#[derive(Deserialize)]
pub struct CheckoutForm {
    pub offer: String,
    pub purchase_authority: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentMode {
    Disabled,
    Test,
    Live,
}

impl PaymentMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Test => "Stripe sandbox",
            Self::Live => "live money",
        }
    }

    fn accepts_checkout(self) -> bool {
        matches!(self, Self::Test | Self::Live)
    }

    fn expects_live_object(self) -> bool {
        self == Self::Live
    }
}

#[derive(Clone)]
struct BillingConfig {
    provider: String,
    api_key: String,
    webhook_secret: String,
    redirect_url: String,
    price_id: String,
    expected_currency: String,
    expected_amount_minor: u64,
    founding_customer_limit: u32,
    reconciliation_token: String,
    mode: PaymentMode,
}

#[derive(Debug)]
pub struct BillingConfigError(String);

impl BillingConfigError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for BillingConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for BillingConfigError {}

#[derive(Debug)]
struct VerifiedCheckout {
    attempt_id: i32,
    checkout_token_hash: String,
    session_id: String,
    payment_intent_id: String,
}

fn env_trimmed(name: &str) -> String {
    std::env::var(name).unwrap_or_default().trim().to_owned()
}

fn is_mock_credential(value: &str) -> bool {
    value.to_ascii_lowercase().starts_with("mock_")
}

fn strong_webhook_secret(secret: &str) -> bool {
    secret.starts_with("whsec_") && secret.len() >= 24 && !is_mock_credential(secret)
}

fn valid_provider_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 200
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn valid_currency(value: &str) -> bool {
    value.len() == 3 && value.bytes().all(|byte| byte.is_ascii_uppercase())
}

fn valid_checkout_session_id(value: &str, mode: PaymentMode) -> bool {
    let prefix = if mode == PaymentMode::Live {
        "cs_live_"
    } else {
        "cs_test_"
    };
    value.starts_with(prefix) && valid_provider_id(value)
}

fn valid_redirect_url(value: &str, mode: PaymentMode) -> bool {
    let Ok(url) = Url::parse(value) else {
        return false;
    };
    if !url.username().is_empty() || url.password().is_some() || url.host_str().is_none() {
        return false;
    }
    if url.scheme() == "https" {
        return true;
    }
    mode == PaymentMode::Test
        && url.scheme() == "http"
        && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"))
}

fn checkout_return_url(base: &str, outcome: &str) -> Result<String, BillingConfigError> {
    let mut url =
        Url::parse(base).map_err(|_| BillingConfigError::new("BILLING_REDIRECT_URL is invalid"))?;
    url.query_pairs_mut().append_pair("checkout", outcome);
    Ok(url.into())
}

fn payment_mode() -> Result<PaymentMode, BillingConfigError> {
    parse_payment_mode(&std::env::var("PAYMENTS_MODE").unwrap_or_else(|_| "disabled".to_owned()))
}

fn parse_payment_mode(value: &str) -> Result<PaymentMode, BillingConfigError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "disabled" => Ok(PaymentMode::Disabled),
        "test" => Ok(PaymentMode::Test),
        "live" => Ok(PaymentMode::Live),
        _ => Err(BillingConfigError::new(
            "PAYMENTS_MODE must be disabled, test or live",
        )),
    }
}

fn billing_config() -> Result<BillingConfig, BillingConfigError> {
    let provider = std::env::var("BILLING_PROVIDER")
        .unwrap_or_else(|_| "stripe".to_owned())
        .trim()
        .to_ascii_lowercase();
    if provider != "stripe" {
        return Err(BillingConfigError::new(
            "the first audited one-time rollout supports Stripe only",
        ));
    }

    let expected_currency = std::env::var("BILLING_EXPECTED_CURRENCY")
        .unwrap_or_else(|_| "BRL".to_owned())
        .trim()
        .to_ascii_uppercase();
    if !valid_currency(&expected_currency) {
        return Err(BillingConfigError::new(
            "BILLING_EXPECTED_CURRENCY must be a three-letter currency code",
        ));
    }

    let expected_amount_minor = std::env::var("BILLING_EXPECTED_AMOUNT_MINOR")
        .unwrap_or_else(|_| "100".to_owned())
        .parse::<u64>()
        .map_err(|_| BillingConfigError::new("BILLING_EXPECTED_AMOUNT_MINOR must be an integer"))?;
    if !(1..=MAX_LOW_VALUE_AMOUNT_MINOR).contains(&expected_amount_minor) {
        return Err(BillingConfigError::new(format!(
            "BILLING_EXPECTED_AMOUNT_MINOR must be between 1 and {MAX_LOW_VALUE_AMOUNT_MINOR}"
        )));
    }

    let mode = payment_mode()?;
    let config = BillingConfig {
        provider,
        api_key: env_trimmed("STRIPE_SECRET_KEY"),
        webhook_secret: env_trimmed("STRIPE_WEBHOOK_SECRET"),
        redirect_url: std::env::var("BILLING_REDIRECT_URL")
            .unwrap_or_else(|_| "http://localhost:3000/dashboard".to_owned())
            .trim()
            .to_owned(),
        price_id: env_trimmed("BILLING_PRICE_ID"),
        expected_currency,
        expected_amount_minor,
        founding_customer_limit: std::env::var("FOUNDING_CUSTOMER_LIMIT")
            .unwrap_or_else(|_| "100".to_owned())
            .parse::<u32>()
            .map_err(|_| BillingConfigError::new("FOUNDING_CUSTOMER_LIMIT must be an integer"))?,
        reconciliation_token: env_trimmed("RECONCILIATION_TOKEN"),
        mode,
    };

    if config.mode.accepts_checkout() {
        if !config.price_id.starts_with("price_") || !valid_provider_id(&config.price_id) {
            return Err(BillingConfigError::new(
                "BILLING_PRICE_ID must contain a Stripe price_... identifier",
            ));
        }
        if !valid_redirect_url(&config.redirect_url, config.mode) {
            return Err(BillingConfigError::new(
                "BILLING_REDIRECT_URL must use HTTPS (HTTP is allowed only for local test mode)",
            ));
        }
        let expected_key_prefix = if config.mode == PaymentMode::Live {
            "sk_live_"
        } else {
            "sk_test_"
        };
        if !config.api_key.starts_with(expected_key_prefix) || is_mock_credential(&config.api_key) {
            return Err(BillingConfigError::new(
                if config.mode == PaymentMode::Live {
                    "Stripe live mode requires an sk_live_ secret key"
                } else {
                    "Stripe sandbox mode requires an sk_test_ secret key"
                },
            ));
        }
        if !strong_webhook_secret(&config.webhook_secret) {
            return Err(BillingConfigError::new(
                "Stripe checkout requires a non-placeholder whsec_ webhook secret",
            ));
        }
        if config.mode == PaymentMode::Live {
            if env_trimmed("LIVE_PAYMENTS_ACKNOWLEDGEMENT") != "accept-real-money" {
                return Err(BillingConfigError::new(
                    "live checkout requires LIVE_PAYMENTS_ACKNOWLEDGEMENT=accept-real-money",
                ));
            }
            if config.founding_customer_limit == 0 || config.founding_customer_limit > 10_000 {
                return Err(BillingConfigError::new(
                    "FOUNDING_CUSTOMER_LIMIT must be between 1 and 10000",
                ));
            }
            if config.reconciliation_token.len() < 32 || config.reconciliation_token.len() > 200 {
                return Err(BillingConfigError::new(
                    "RECONCILIATION_TOKEN must contain between 32 and 200 characters",
                ));
            }
            crate::controllers::artifact_controller::validate_live_artifact_configuration()?;
            crate::controllers::legal_controller::validate_live_merchant_configuration()?;
        }
    }

    Ok(config)
}

/// Validates the fail-closed payment configuration during process startup.
pub fn initialize_billing_provider() -> Result<(), BillingConfigError> {
    billing_config().map(|_| ())
}

fn normalize_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

fn valid_email(email: &str) -> bool {
    !email.is_empty()
        && email.len() <= 254
        && !email.contains(['\r', '\n'])
        && email.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty() && domain.contains('.') && !domain.ends_with('.')
        })
}

fn valid_identity(identity: &BillingIdentity) -> bool {
    identity.owner_id > 0 && valid_email(&identity.email)
}

fn checkout_rate_limited(owner_id: i32) -> bool {
    static LIMITER: OnceLock<rullst_security::RateLimiter> = OnceLock::new();
    LIMITER
        .get_or_init(|| rullst_security::RateLimiter::new(2, Duration::from_secs(10 * 60)))
        .check(&format!("billing-user:{owner_id}"))
}

fn sha256_hex(value: &[u8]) -> String {
    hex::encode(Sha256::digest(value))
}

fn new_certificate_id(mode: PaymentMode) -> String {
    let environment = if mode == PaymentMode::Live {
        "LIVE"
    } else {
        "SBX"
    };
    format!(
        "RST-{environment}-{}",
        Uuid::new_v4().simple().to_string().to_ascii_uppercase()
    )
}

fn artifact_version(mode: PaymentMode) -> &'static str {
    if mode == PaymentMode::Live {
        LIVE_ARTIFACT_VERSION
    } else {
        SANDBOX_ARTIFACT_VERSION
    }
}

fn verify_stripe_signature_at(
    payload: &[u8],
    signature_header: &str,
    webhook_secret: &str,
    now_unix_seconds: i64,
) -> Result<(), BillingConfigError> {
    let mut timestamps = signature_header.split(',').filter_map(|part| {
        let (key, value) = part.trim().split_once('=')?;
        (key == "t").then_some(value)
    });
    let timestamp_text = timestamps
        .next()
        .filter(|_| timestamps.next().is_none())
        .ok_or_else(|| BillingConfigError::new("Stripe signature timestamp is invalid"))?;
    let timestamp = timestamp_text
        .parse::<i64>()
        .map_err(|_| BillingConfigError::new("Stripe signature timestamp is invalid"))?;

    let mut signed_payload = Vec::with_capacity(timestamp_text.len() + 1 + payload.len());
    signed_payload.extend_from_slice(timestamp_text.as_bytes());
    signed_payload.push(b'.');
    signed_payload.extend_from_slice(payload);
    let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, webhook_secret.as_bytes());

    let signature_matches = signature_header.split(',').any(|part| {
        let Some((name, value)) = part.trim().split_once('=') else {
            return false;
        };
        name == "v1"
            && hex::decode(value).ok().is_some_and(|signature| {
                ring::hmac::verify(&key, &signed_payload, &signature).is_ok()
            })
    });
    if !signature_matches {
        return Err(BillingConfigError::new(
            "Stripe webhook signature verification failed",
        ));
    }

    if now_unix_seconds.abs_diff(timestamp) > STRIPE_WEBHOOK_TOLERANCE_SECONDS {
        return Err(BillingConfigError::new(
            "Stripe webhook timestamp is outside the acceptance window",
        ));
    }
    Ok(())
}

fn verify_stripe_signature(
    payload: &[u8],
    signature_header: &str,
    webhook_secret: &str,
) -> Result<(), BillingConfigError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| BillingConfigError::new("system clock is before the Unix epoch"))?;
    let now = i64::try_from(now.as_secs())
        .map_err(|_| BillingConfigError::new("system clock value is unsupported"))?;
    verify_stripe_signature_at(payload, signature_header, webhook_secret, now)
}

fn provider_http_client() -> Result<Client, BillingConfigError> {
    Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(12))
        .https_only(true)
        .no_proxy()
        .build()
        .map_err(|_| BillingConfigError::new("could not build provider HTTP client"))
}

async fn bounded_json(response: HttpResponse) -> Result<Value, BillingConfigError> {
    if !response.status().is_success() {
        return Err(BillingConfigError::new(format!(
            "provider returned HTTP {}",
            response.status().as_u16()
        )));
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_PROVIDER_RESPONSE_BYTES as u64)
    {
        return Err(BillingConfigError::new("provider response is too large"));
    }

    let mut response = response;
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| BillingConfigError::new("provider response could not be read"))?
    {
        if body.len().saturating_add(chunk.len()) > MAX_PROVIDER_RESPONSE_BYTES {
            return Err(BillingConfigError::new("provider response is too large"));
        }
        body.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body)
        .map_err(|_| BillingConfigError::new("provider returned invalid JSON"))
}

async fn verify_stripe_price(config: &BillingConfig) -> Result<(), BillingConfigError> {
    let response = provider_http_client()?
        .get(format!(
            "https://api.stripe.com/v1/prices/{}",
            config.price_id
        ))
        .bearer_auth(&config.api_key)
        .send()
        .await
        .map_err(|_| BillingConfigError::new("Stripe price lookup failed"))?;
    let body = bounded_json(response).await?;

    let amount_matches = body["unit_amount"].as_u64() == Some(config.expected_amount_minor);
    let currency_matches = body["currency"]
        .as_str()
        .map(str::to_ascii_uppercase)
        .as_deref()
        == Some(config.expected_currency.as_str());
    let mode_matches = body["livemode"].as_bool() == Some(config.mode.expects_live_object());
    let is_one_time = body["type"].as_str() == Some("one_time") && body["recurring"].is_null();

    if body["id"].as_str() != Some(config.price_id.as_str())
        || body["active"].as_bool() != Some(true)
        || !amount_matches
        || !currency_matches
        || !mode_matches
        || !is_one_time
    {
        return Err(BillingConfigError::new(
            "Stripe Price must be active, one-time, in the selected environment, and match the expected BRL amount",
        ));
    }
    Ok(())
}

fn trusted_stripe_checkout_url(body: &Value) -> Result<String, BillingConfigError> {
    let checkout_url = body["url"]
        .as_str()
        .ok_or_else(|| BillingConfigError::new("Stripe did not return a Checkout URL"))?;
    let parsed = Url::parse(checkout_url)
        .map_err(|_| BillingConfigError::new("Stripe returned an invalid Checkout URL"))?;
    if parsed.scheme() != "https"
        || parsed.host_str() != Some("checkout.stripe.com")
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(BillingConfigError::new(
            "Stripe returned an untrusted Checkout URL",
        ));
    }
    Ok(checkout_url.to_owned())
}

pub async fn account_has_stripe_report(user_id: i32) -> Result<bool, BillingConfigError> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM entitlements WHERE user_id = $1 AND product_sku = $2 AND status = 'active')",
    )
    .bind(user_id)
    .bind(CHECKOUT_OFFER)
    .fetch_one(Orm::pool().map_err(|_| BillingConfigError::new("database is unavailable"))?)
    .await
    .map_err(|_| BillingConfigError::new("entitlement lookup failed"))
}

async fn open_purchase_attempt(
    user_id: i32,
) -> Result<Option<PurchaseAttempt>, BillingConfigError> {
    sqlx::query_as::<_, PurchaseAttempt>(
        "SELECT * FROM purchase_attempts WHERE user_id = $1 AND product_sku = $2 AND status IN ('pending', 'unknown', 'checkout_created') ORDER BY id DESC LIMIT 1",
    )
    .bind(user_id)
    .bind(CHECKOUT_OFFER)
    .fetch_optional(Orm::pool().map_err(|_| BillingConfigError::new("database is unavailable"))?)
    .await
    .map_err(|_| BillingConfigError::new("open checkout lookup failed"))
}

fn attempt_matches_config(attempt: &PurchaseAttempt, config: &BillingConfig) -> bool {
    let Ok(expected_amount_minor) = i32::try_from(config.expected_amount_minor) else {
        return false;
    };
    attempt.provider == config.provider
        && attempt.product_sku == CHECKOUT_OFFER
        && attempt.provider_price_id == config.price_id
        && attempt.expected_amount_minor == expected_amount_minor
        && attempt.expected_currency == config.expected_currency
}

fn resumed_session_matches_attempt(
    body: &Value,
    attempt: &PurchaseAttempt,
    mode: PaymentMode,
) -> bool {
    let attempt_id = attempt.id.to_string();
    body["id"].as_str() == attempt.provider_session_id.as_deref()
        && body["livemode"].as_bool() == Some(mode.expects_live_object())
        && body["mode"].as_str() == Some("payment")
        && body["metadata"]["purchase_attempt_id"].as_str() == Some(attempt_id.as_str())
        && body["metadata"]["product_sku"].as_str() == Some(CHECKOUT_OFFER)
        && body["client_reference_id"]
            .as_str()
            .is_some_and(|token| sha256_hex(token.as_bytes()) == attempt.checkout_token_hash)
}

async fn resume_open_checkout(
    identity: &BillingIdentity,
    config: &BillingConfig,
) -> Result<Option<Response>, BillingConfigError> {
    let Some(attempt) = open_purchase_attempt(identity.owner_id).await? else {
        return Ok(None);
    };
    if !attempt_matches_config(&attempt, config) {
        return Err(BillingConfigError::new(
            "open checkout does not match the active server configuration",
        ));
    }
    if attempt.status != "checkout_created" {
        return Ok(Some(
            (
                StatusCode::CONFLICT,
                "A previous checkout has an uncertain outcome and is being reconciled; wait before trying again",
            )
                .into_response(),
        ));
    }

    let session_id = attempt
        .provider_session_id
        .as_deref()
        .filter(|id| valid_checkout_session_id(id, config.mode))
        .ok_or_else(|| BillingConfigError::new("open checkout has no valid provider session"))?;
    let response = provider_http_client()?
        .get(format!(
            "https://api.stripe.com/v1/checkout/sessions/{session_id}"
        ))
        .bearer_auth(&config.api_key)
        .send()
        .await
        .map_err(|_| BillingConfigError::new("Stripe Checkout recovery failed"))?;
    let body = bounded_json(response).await?;
    if !resumed_session_matches_attempt(&body, &attempt, config.mode) {
        return Err(BillingConfigError::new(
            "Stripe Checkout recovery did not match the authenticated purchase attempt",
        ));
    }

    match body["status"].as_str() {
        Some("open") => {
            let checkout_url = trusted_stripe_checkout_url(&body)?;
            Ok(Some(Redirect::to(&checkout_url).into_response()))
        }
        Some("complete") => {
            let processing_url = checkout_return_url(&config.redirect_url, "processing")?;
            Ok(Some(Redirect::to(&processing_url).into_response()))
        }
        Some("expired") => {
            set_attempt_status(attempt.id, "expired").await;
            Ok(None)
        }
        _ => Err(BillingConfigError::new(
            "Stripe returned an unsupported Checkout Session status",
        )),
    }
}

async fn create_purchase_attempt(
    identity: &BillingIdentity,
    config: &BillingConfig,
    token_hash: String,
) -> Result<PurchaseAttempt, BillingConfigError> {
    let amount = i32::try_from(config.expected_amount_minor)
        .map_err(|_| BillingConfigError::new("configured amount is too large"))?;
    sqlx::query_as::<_, PurchaseAttempt>(
        "INSERT INTO purchase_attempts (user_id, provider, product_sku, provider_price_id, expected_amount_minor, expected_currency, checkout_token_hash, status) VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending') RETURNING *",
    )
    .bind(identity.owner_id)
    .bind(&config.provider)
    .bind(CHECKOUT_OFFER)
    .bind(&config.price_id)
    .bind(amount)
    .bind(&config.expected_currency)
    .bind(token_hash)
    .fetch_one(Orm::pool().map_err(|_| BillingConfigError::new("database is unavailable"))?)
    .await
    .map_err(|_| BillingConfigError::new("an open checkout already exists or could not be saved"))
}

async fn set_attempt_status(attempt_id: i32, status: &str) {
    let Ok(pool) = Orm::pool() else {
        return;
    };
    if let Err(error) = sqlx::query(
        "UPDATE purchase_attempts SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
    )
    .bind(status)
    .bind(attempt_id)
    .execute(pool)
    .await
    {
        eprintln!("Purchase attempt status update failed: {error}");
    }
}

async fn create_stripe_checkout(
    identity: &BillingIdentity,
    config: &BillingConfig,
    attempt: &PurchaseAttempt,
    checkout_token: &str,
) -> Result<String, BillingConfigError> {
    let success_url = checkout_return_url(&config.redirect_url, "success")?;
    let cancel_url = checkout_return_url(&config.redirect_url, "cancelled")?;
    let attempt_id = attempt.id.to_string();
    let idempotency_key = format!("rullst-checkout-{}", attempt.id);
    let form = [
        ("mode", "payment".to_owned()),
        ("adaptive_pricing[enabled]", "false".to_owned()),
        ("success_url", success_url),
        ("cancel_url", cancel_url),
        ("customer_email", normalize_email(&identity.email)),
        ("customer_creation", "always".to_owned()),
        ("client_reference_id", checkout_token.to_owned()),
        ("line_items[0][price]", config.price_id.clone()),
        ("line_items[0][quantity]", "1".to_owned()),
        ("metadata[purchase_attempt_id]", attempt_id.clone()),
        ("metadata[product_sku]", CHECKOUT_OFFER.to_owned()),
        (
            "payment_intent_data[metadata][purchase_attempt_id]",
            attempt_id,
        ),
        (
            "payment_intent_data[metadata][product_sku]",
            CHECKOUT_OFFER.to_owned(),
        ),
    ];
    let response = provider_http_client()?
        .post("https://api.stripe.com/v1/checkout/sessions")
        .bearer_auth(&config.api_key)
        .header("Idempotency-Key", idempotency_key)
        .form(&form)
        .send()
        .await
        .map_err(|_| BillingConfigError::new("Stripe Checkout creation had an unknown outcome"))?;
    let body = bounded_json(response).await?;

    let session_id = body["id"]
        .as_str()
        .filter(|id| valid_checkout_session_id(id, config.mode))
        .ok_or_else(|| BillingConfigError::new("Stripe returned an invalid Checkout Session ID"))?;
    if body["livemode"].as_bool() != Some(config.mode.expects_live_object())
        || body["mode"].as_str() != Some("payment")
    {
        return Err(BillingConfigError::new(
            "Stripe returned a Checkout Session for the wrong mode",
        ));
    }
    let checkout_url = trusted_stripe_checkout_url(&body)?;

    sqlx::query(
        "UPDATE purchase_attempts SET provider_session_id = $1, status = 'checkout_created', updated_at = CURRENT_TIMESTAMP WHERE id = $2 AND status = 'pending'",
    )
    .bind(session_id)
    .bind(attempt.id)
    .execute(Orm::pool().map_err(|_| BillingConfigError::new("database is unavailable"))?)
    .await
    .map_err(|_| BillingConfigError::new("Checkout Session could not be bound locally"))?;

    Ok(checkout_url)
}

pub async fn pricing_view(
    headers: HeaderMap,
    csrf: Option<Extension<rullst::security::CsrfToken>>,
    csp_nonce: Option<Extension<rullst::security::CspNonce>>,
) -> impl IntoResponse {
    let csrf_token = csrf
        .as_ref()
        .map(|Extension(token)| token.as_str())
        .unwrap_or_default();
    let nonce = csp_nonce
        .as_ref()
        .map(|Extension(nonce)| nonce.as_str())
        .unwrap_or_default();
    let signed_in = authenticated_pricing_identity(&headers).await.is_some();

    let state = match billing_config() {
        Ok(config) => PaymentPageState {
            selected_provider: config.provider,
            payment_mode: config.mode,
            production_deployment: crate::controllers::legal_controller::production_deployment(),
            expected_price: format_amount(config.expected_amount_minor, &config.expected_currency),
            setup_error: None,
            signed_in,
            merchant_notice: (config.mode == PaymentMode::Live)
                .then(crate::controllers::legal_controller::live_merchant_notice)
                .transpose()
                .ok()
                .flatten(),
        },
        Err(error) => PaymentPageState {
            selected_provider: "unavailable".to_owned(),
            payment_mode: PaymentMode::Disabled,
            production_deployment: crate::controllers::legal_controller::production_deployment(),
            expected_price: "not configured".to_owned(),
            setup_error: Some(error.to_string()),
            signed_in,
            merchant_notice: None,
        },
    };
    billing::pricing_page(csrf_token, nonce, &state)
}

async fn authenticated_pricing_identity(headers: &HeaderMap) -> Option<BillingIdentity> {
    let cookie = rullst::auth::extract_session_cookie(headers)?;
    let app_key = rullst::auth::get_app_key().ok()?;
    let user_id = rullst::auth::decrypt_session(&cookie, &app_key).ok()?;
    let user = User::find(user_id).await.ok()??;
    let identity = BillingIdentity {
        owner_id: user.id,
        email: normalize_email(&user.email),
    };
    valid_identity(&identity).then_some(identity)
}

pub async fn checkout_redirect(
    Extension(identity): Extension<BillingIdentity>,
    Form(form): Form<CheckoutForm>,
) -> Response {
    if form.offer != CHECKOUT_OFFER || form.purchase_authority != "adult_or_guardian" {
        return (
            StatusCode::BAD_REQUEST,
            "The fixed offer and purchaser-authority confirmation are required",
        )
            .into_response();
    }
    if !valid_identity(&identity) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let config = match billing_config() {
        Ok(config) if config.mode.accepts_checkout() => config,
        Ok(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Payment checkout is disabled",
            )
                .into_response();
        }
        Err(error) => {
            eprintln!("Billing configuration rejected: {error}");
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    };

    match account_has_stripe_report(identity.owner_id).await {
        Ok(true) => {
            return (
                StatusCode::CONFLICT,
                "This account already owns the Stripe report",
            )
                .into_response();
        }
        Ok(false) => {}
        Err(error) => {
            eprintln!("Entitlement lookup failed: {error}");
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    }

    match resume_open_checkout(&identity, &config).await {
        Ok(Some(response)) => return response,
        Ok(None) => {}
        Err(error) => {
            eprintln!("Open Stripe Checkout recovery failed: {error}");
            return StatusCode::BAD_GATEWAY.into_response();
        }
    }

    if checkout_rate_limited(identity.owner_id) {
        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            "Checkout creation limit reached; wait before creating a new session",
        )
            .into_response();
        response.headers_mut().insert(
            rullst::server::header::RETRY_AFTER,
            rullst::server::HeaderValue::from_static("600"),
        );
        return response;
    }

    if let Err(error) = verify_stripe_price(&config).await {
        eprintln!("Stripe Price verification failed: {error}");
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }

    let checkout_token = Uuid::new_v4().simple().to_string();
    let attempt =
        match create_purchase_attempt(&identity, &config, sha256_hex(checkout_token.as_bytes()))
            .await
        {
            Ok(attempt) => attempt,
            Err(error) => {
                eprintln!("Purchase attempt creation failed: {error}");
                return StatusCode::CONFLICT.into_response();
            }
        };

    match create_stripe_checkout(&identity, &config, &attempt, &checkout_token).await {
        Ok(url) => Redirect::to(&url).into_response(),
        Err(error) => {
            set_attempt_status(attempt.id, "unknown").await;
            eprintln!("Stripe Checkout creation failed: {error}");
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

fn event_matches_mode(event: &Value, config: &BillingConfig) -> bool {
    event["livemode"].as_bool() == Some(config.mode.expects_live_object())
}

async fn retrieve_verified_checkout(
    config: &BillingConfig,
    event_session: &Value,
) -> Result<Option<VerifiedCheckout>, BillingConfigError> {
    let session_id = event_session["id"]
        .as_str()
        .filter(|id| valid_checkout_session_id(id, config.mode))
        .ok_or_else(|| BillingConfigError::new("invalid Stripe Checkout Session ID"))?;
    let session = retrieve_checkout_session(config, session_id).await?;
    verify_paid_checkout(config, session_id, &session)
}

async fn retrieve_checkout_session(
    config: &BillingConfig,
    session_id: &str,
) -> Result<Value, BillingConfigError> {
    let response = provider_http_client()?
        .get(format!(
            "https://api.stripe.com/v1/checkout/sessions/{session_id}"
        ))
        .bearer_auth(&config.api_key)
        .query(&[("expand[]", "line_items")])
        .send()
        .await
        .map_err(|_| BillingConfigError::new("Stripe Checkout reconciliation failed"))?;
    bounded_json(response).await
}

fn verify_paid_checkout(
    config: &BillingConfig,
    session_id: &str,
    session: &Value,
) -> Result<Option<VerifiedCheckout>, BillingConfigError> {
    if session["payment_status"].as_str() != Some("paid") {
        return Ok(None);
    }
    let attempt_id = session["metadata"]["purchase_attempt_id"]
        .as_str()
        .and_then(|value| value.parse::<i32>().ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| {
            BillingConfigError::new("Checkout metadata has no valid purchase attempt")
        })?;
    let checkout_token = session["client_reference_id"]
        .as_str()
        .filter(|value| value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| BillingConfigError::new("Checkout client reference is invalid"))?;
    let payment_intent_id = session["payment_intent"]
        .as_str()
        .filter(|value| value.starts_with("pi_") && valid_provider_id(value))
        .ok_or_else(|| BillingConfigError::new("Checkout payment intent is invalid"))?;
    let line_items = session["line_items"]["data"]
        .as_array()
        .ok_or_else(|| BillingConfigError::new("Checkout line items are unavailable"))?;
    let price_id = line_items
        .first()
        .and_then(|item| {
            item["price"]["id"]
                .as_str()
                .or_else(|| item["price"].as_str())
        })
        .unwrap_or_default();
    let currency_matches = session["currency"]
        .as_str()
        .map(str::to_ascii_uppercase)
        .as_deref()
        == Some(config.expected_currency.as_str());

    if session["id"].as_str() != Some(session_id)
        || session["livemode"].as_bool() != Some(config.mode.expects_live_object())
        || session["mode"].as_str() != Some("payment")
        || session["status"].as_str() != Some("complete")
        || session["amount_total"].as_u64() != Some(config.expected_amount_minor)
        || !currency_matches
        || session["metadata"]["product_sku"].as_str() != Some(CHECKOUT_OFFER)
        || line_items.len() != 1
        || price_id != config.price_id
        || line_items[0]["quantity"].as_u64() != Some(1)
    {
        return Err(BillingConfigError::new(
            "Stripe Checkout evidence does not match the server-owned offer",
        ));
    }

    Ok(Some(VerifiedCheckout {
        attempt_id,
        checkout_token_hash: sha256_hex(checkout_token.as_bytes()),
        session_id: session_id.to_owned(),
        payment_intent_id: payment_intent_id.to_owned(),
    }))
}

async fn persist_successful_checkout(
    config: &BillingConfig,
    event_id: &str,
    event_type: &str,
    payload_hash: &str,
    checkout: &VerifiedCheckout,
) -> Result<(), BillingConfigError> {
    let mut transaction = Orm::begin_transaction()
        .await
        .map_err(|_| BillingConfigError::new("billing transaction could not start"))?;
    let event_insert = sqlx::query(
        "INSERT INTO provider_events (provider, event_id, event_type, payload_sha256) VALUES ('stripe', $1, $2, $3) ON CONFLICT (provider, event_id) DO NOTHING",
    )
    .bind(event_id)
    .bind(event_type)
    .bind(payload_hash)
    .execute(&mut *transaction)
    .await
    .map_err(|_| BillingConfigError::new("provider event could not be recorded"))?;
    if event_insert.rows_affected() == 0 {
        transaction
            .rollback()
            .await
            .map_err(|_| BillingConfigError::new("duplicate event rollback failed"))?;
        return Ok(());
    }

    let attempt = sqlx::query_as::<_, PurchaseAttempt>(
        "SELECT * FROM purchase_attempts WHERE id = $1 FOR UPDATE",
    )
    .bind(checkout.attempt_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| BillingConfigError::new("purchase attempt lookup failed"))?
    .ok_or_else(|| BillingConfigError::new("purchase attempt does not exist"))?;

    let session_matches = attempt
        .provider_session_id
        .as_deref()
        .is_none_or(|known| known == checkout.session_id);
    if attempt.provider != config.provider
        || attempt.product_sku != CHECKOUT_OFFER
        || attempt.provider_price_id != config.price_id
        || attempt.expected_amount_minor != config.expected_amount_minor as i32
        || attempt.expected_currency != config.expected_currency
        || attempt.checkout_token_hash != checkout.checkout_token_hash
        || !session_matches
    {
        return Err(BillingConfigError::new(
            "purchase attempt does not match the verified Checkout Session",
        ));
    }

    sqlx::query(
        "UPDATE purchase_attempts SET provider_session_id = $1, provider_payment_id = $2, status = 'paid', updated_at = CURRENT_TIMESTAMP WHERE id = $3",
    )
    .bind(&checkout.session_id)
    .bind(&checkout.payment_intent_id)
    .bind(attempt.id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| BillingConfigError::new("purchase attempt could not be completed"))?;

    let entitlement_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO entitlements (user_id, product_sku, provider, provider_payment_id, artifact_version, status, revoked_reason, revoked_at) VALUES ($1, $2, 'stripe', $3, $4, 'active', NULL, NULL) ON CONFLICT (user_id, product_sku) DO UPDATE SET provider_payment_id = EXCLUDED.provider_payment_id, artifact_version = EXCLUDED.artifact_version, status = 'active', revoked_reason = NULL, revoked_at = NULL, updated_at = CURRENT_TIMESTAMP RETURNING id",
    )
    .bind(attempt.user_id)
    .bind(CHECKOUT_OFFER)
    .bind(&checkout.payment_intent_id)
    .bind(artifact_version(config.mode))
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| BillingConfigError::new("entitlement could not be created"))?;

    if config.mode == PaymentMode::Live {
        sqlx::query("SELECT pg_advisory_xact_lock(731_225_991)")
            .execute(&mut *transaction)
            .await
            .map_err(|_| BillingConfigError::new("certificate cohort lock failed"))?;
        let issued = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM tester_certificates WHERE badge_kind = 'founding_customer' AND environment = 'live'",
        )
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| BillingConfigError::new("certificate cohort could not be counted"))?;
        if issued < i64::from(config.founding_customer_limit) {
            sqlx::query(
                "INSERT INTO tester_certificates (entitlement_id, public_id, badge_kind, environment, status) VALUES ($1, $2, 'founding_customer', 'live', 'active') ON CONFLICT (entitlement_id) DO UPDATE SET badge_kind = 'founding_customer', environment = 'live', status = 'active', updated_at = CURRENT_TIMESTAMP",
            )
            .bind(entitlement_id)
            .bind(new_certificate_id(config.mode))
            .execute(&mut *transaction)
            .await
            .map_err(|_| BillingConfigError::new("founding customer certificate could not be issued"))?;
        }
    } else {
        sqlx::query(
            "INSERT INTO tester_certificates (entitlement_id, public_id, badge_kind, environment, status) VALUES ($1, $2, 'sandbox_pioneer', 'test', 'active') ON CONFLICT (entitlement_id) DO UPDATE SET status = 'active', updated_at = CURRENT_TIMESTAMP",
        )
        .bind(entitlement_id)
        .bind(new_certificate_id(config.mode))
        .execute(&mut *transaction)
        .await
        .map_err(|_| BillingConfigError::new("sandbox certificate could not be issued"))?;
    }

    transaction
        .commit()
        .await
        .map_err(|_| BillingConfigError::new("billing transaction commit failed"))
}

#[derive(Debug)]
struct VerifiedRevocation {
    payment_intent_id: String,
    reason: &'static str,
    attempt_status: &'static str,
}

async fn retrieve_verified_revocation(
    config: &BillingConfig,
    event_type: &str,
    event_object: &Value,
) -> Result<VerifiedRevocation, BillingConfigError> {
    let (resource, id, reason, attempt_status) = match event_type {
        "charge.refunded" => (
            "charges",
            event_object["id"]
                .as_str()
                .filter(|value| value.starts_with("ch_") && valid_provider_id(value))
                .ok_or_else(|| BillingConfigError::new("invalid refunded Charge ID"))?,
            "refunded",
            "refunded",
        ),
        "charge.dispute.created" => (
            "disputes",
            event_object["id"]
                .as_str()
                .filter(|value| value.starts_with("dp_") && valid_provider_id(value))
                .ok_or_else(|| BillingConfigError::new("invalid Stripe dispute ID"))?,
            "disputed",
            "disputed",
        ),
        _ => {
            return Err(BillingConfigError::new(
                "unsupported entitlement revocation event",
            ));
        }
    };
    let response = provider_http_client()?
        .get(format!("https://api.stripe.com/v1/{resource}/{id}"))
        .bearer_auth(&config.api_key)
        .send()
        .await
        .map_err(|_| BillingConfigError::new("Stripe revocation reconciliation failed"))?;
    let object = bounded_json(response).await?;
    if object["id"].as_str() != Some(id)
        || object["livemode"].as_bool() != Some(config.mode.expects_live_object())
    {
        return Err(BillingConfigError::new(
            "Stripe revocation evidence belongs to the wrong environment",
        ));
    }
    if event_type == "charge.refunded"
        && (object["refunded"].as_bool() != Some(true)
            || object["amount"].as_u64().is_none_or(|amount| {
                amount == 0 || object["amount_refunded"].as_u64() != Some(amount)
            }))
    {
        return Err(BillingConfigError::new(
            "Stripe Charge is not fully refunded",
        ));
    }
    let payment_intent_id = object["payment_intent"]
        .as_str()
        .filter(|value| value.starts_with("pi_") && valid_provider_id(value))
        .ok_or_else(|| BillingConfigError::new("revocation has no valid PaymentIntent"))?;
    Ok(VerifiedRevocation {
        payment_intent_id: payment_intent_id.to_owned(),
        reason,
        attempt_status,
    })
}

async fn persist_revocation(
    event_id: &str,
    event_type: &str,
    payload_hash: &str,
    revocation: &VerifiedRevocation,
) -> Result<(), BillingConfigError> {
    let mut transaction = Orm::begin_transaction()
        .await
        .map_err(|_| BillingConfigError::new("revocation transaction could not start"))?;
    let event_insert = sqlx::query(
        "INSERT INTO provider_events (provider, event_id, event_type, payload_sha256) VALUES ('stripe', $1, $2, $3) ON CONFLICT (provider, event_id) DO NOTHING",
    )
    .bind(event_id)
    .bind(event_type)
    .bind(payload_hash)
    .execute(&mut *transaction)
    .await
    .map_err(|_| BillingConfigError::new("revocation event could not be recorded"))?;
    if event_insert.rows_affected() == 0 {
        transaction
            .rollback()
            .await
            .map_err(|_| BillingConfigError::new("duplicate revocation rollback failed"))?;
        return Ok(());
    }

    sqlx::query(
        "UPDATE purchase_attempts SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE provider = 'stripe' AND provider_payment_id = $2",
    )
    .bind(revocation.attempt_status)
    .bind(&revocation.payment_intent_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| BillingConfigError::new("purchase revocation could not be recorded"))?;

    let entitlement_ids = sqlx::query_scalar::<_, i32>(
        "UPDATE entitlements SET status = 'revoked', revoked_reason = $1, revoked_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE provider = 'stripe' AND provider_payment_id = $2 AND status = 'active' RETURNING id",
    )
    .bind(revocation.reason)
    .bind(&revocation.payment_intent_id)
    .fetch_all(&mut *transaction)
    .await
    .map_err(|_| BillingConfigError::new("entitlement could not be revoked"))?;

    for entitlement_id in entitlement_ids {
        sqlx::query(
            "UPDATE tester_certificates SET status = 'revoked', updated_at = CURRENT_TIMESTAMP WHERE entitlement_id = $1 AND status = 'active'",
        )
        .bind(entitlement_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| BillingConfigError::new("certificate could not be revoked"))?;
        if revocation.reason == "refunded" {
            sqlx::query(
                "UPDATE refund_requests SET status = 'completed', updated_at = CURRENT_TIMESTAMP WHERE entitlement_id = $1 AND status IN ('requested', 'processing')",
            )
            .bind(entitlement_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| BillingConfigError::new("refund request could not be completed"))?;
        }
    }

    transaction
        .commit()
        .await
        .map_err(|_| BillingConfigError::new("revocation transaction commit failed"))
}

async fn close_terminal_attempt(event_session: &Value, status: &str) {
    let Some(attempt_id) = event_session["metadata"]["purchase_attempt_id"]
        .as_str()
        .and_then(|value| value.parse::<i32>().ok())
    else {
        return;
    };
    let Some(token) = event_session["client_reference_id"].as_str() else {
        return;
    };
    let Ok(pool) = Orm::pool() else {
        return;
    };
    if let Err(error) = sqlx::query(
        "UPDATE purchase_attempts SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 AND checkout_token_hash = $3 AND status IN ('pending', 'unknown', 'checkout_created')",
    )
    .bind(status)
    .bind(attempt_id)
    .bind(sha256_hex(token.as_bytes()))
    .execute(pool)
    .await
    {
        eprintln!("Terminal checkout state could not be recorded: {error}");
    }
}

/// Verifies Stripe's signature over the untouched body, then reconciles paid
/// Checkout evidence from Stripe before granting a local entitlement.
pub async fn webhook_handler(headers: HeaderMap, body: String) -> Response {
    if body.len() > MAX_WEBHOOK_BYTES {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    }
    let config = match billing_config() {
        Ok(config) if config.mode.accepts_checkout() => config,
        _ => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let signature = match headers
        .get("stripe-signature")
        .and_then(|value| value.to_str().ok())
    {
        Some(signature) => signature,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    if let Err(error) = verify_stripe_signature(body.as_bytes(), signature, &config.webhook_secret)
    {
        eprintln!("Stripe webhook signature rejected: {error}");
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let event: Value = match serde_json::from_str(&body) {
        Ok(event) => event,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let event_id = match event["id"]
        .as_str()
        .filter(|id| id.starts_with("evt_") && valid_provider_id(id))
    {
        Some(event_id) => event_id,
        None => return StatusCode::UNPROCESSABLE_ENTITY.into_response(),
    };
    if !event_matches_mode(&event, &config) {
        return StatusCode::UNPROCESSABLE_ENTITY.into_response();
    }
    let event_type = event["type"].as_str().unwrap_or_default();
    let event_session = &event["data"]["object"];

    match event_type {
        "checkout.session.completed" | "checkout.session.async_payment_succeeded" => {
            let checkout = match retrieve_verified_checkout(&config, event_session).await {
                Ok(Some(checkout)) => checkout,
                Ok(None) => return StatusCode::OK.into_response(),
                Err(error) => {
                    eprintln!("Stripe Checkout evidence rejected: {error}");
                    return StatusCode::UNPROCESSABLE_ENTITY.into_response();
                }
            };
            if let Err(error) = persist_successful_checkout(
                &config,
                event_id,
                event_type,
                &sha256_hex(body.as_bytes()),
                &checkout,
            )
            .await
            {
                eprintln!("Stripe Checkout persistence failed: {error}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
        "checkout.session.async_payment_failed" => {
            close_terminal_attempt(event_session, "failed").await;
        }
        "checkout.session.expired" => {
            close_terminal_attempt(event_session, "expired").await;
        }
        "charge.refunded" | "charge.dispute.created" => {
            let revocation =
                match retrieve_verified_revocation(&config, event_type, event_session).await {
                    Ok(revocation) => revocation,
                    Err(error) => {
                        eprintln!("Stripe revocation evidence rejected: {error}");
                        return StatusCode::UNPROCESSABLE_ENTITY.into_response();
                    }
                };
            if let Err(error) = persist_revocation(
                event_id,
                event_type,
                &sha256_hex(body.as_bytes()),
                &revocation,
            )
            .await
            {
                eprintln!("Stripe revocation persistence failed: {error}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
        _ => {}
    }

    StatusCode::OK.into_response()
}

struct ReconciliationRunGuard;

impl ReconciliationRunGuard {
    fn acquire() -> Option<Self> {
        RECONCILIATION_RUNNING
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            .map(|_| Self)
    }
}

impl Drop for ReconciliationRunGuard {
    fn drop(&mut self) {
        RECONCILIATION_RUNNING.store(false, Ordering::Release);
    }
}

fn reconciliation_authorized(headers: &HeaderMap, expected: &str) -> bool {
    let Some(candidate) = headers
        .get(rullst::server::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
    else {
        return false;
    };
    let expected_key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, expected.as_bytes());
    let expected_tag = ring::hmac::sign(&expected_key, b"rullst-saas-reconciliation-v1");
    let candidate_key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, candidate.as_bytes());
    ring::hmac::verify(
        &candidate_key,
        b"rullst-saas-reconciliation-v1",
        expected_tag.as_ref(),
    )
    .is_ok()
}

async fn reconcile_open_checkouts(config: &BillingConfig) -> Result<u32, BillingConfigError> {
    let attempts = sqlx::query_as::<_, PurchaseAttempt>(
        "SELECT * FROM purchase_attempts WHERE provider = 'stripe' AND product_sku = $1 AND status IN ('pending', 'unknown', 'checkout_created') AND provider_session_id IS NOT NULL ORDER BY last_reconciled_at ASC NULLS FIRST, id ASC LIMIT 50",
    )
    .bind(CHECKOUT_OFFER)
    .fetch_all(Orm::pool().map_err(|_| BillingConfigError::new("database is unavailable"))?)
    .await
    .map_err(|_| BillingConfigError::new("open checkout reconciliation lookup failed"))?;

    let mut processed = 0_u32;
    for attempt in attempts {
        if !attempt_matches_config(&attempt, config) {
            continue;
        }
        let Some(session_id) = attempt
            .provider_session_id
            .as_deref()
            .filter(|id| valid_checkout_session_id(id, config.mode))
        else {
            continue;
        };
        let session = retrieve_checkout_session(config, session_id).await?;
        if let Some(checkout) = verify_paid_checkout(config, session_id, &session)? {
            let event_id = format!("reconcile-checkout-{}", checkout.session_id);
            let payload_hash = sha256_hex(
                serde_json::to_vec(&session)
                    .map_err(|_| BillingConfigError::new("Checkout evidence could not be hashed"))?
                    .as_slice(),
            );
            persist_successful_checkout(
                config,
                &event_id,
                "reconciliation.checkout.paid",
                &payload_hash,
                &checkout,
            )
            .await?;
            processed += 1;
        } else if session["status"].as_str() == Some("expired") {
            let pool =
                Orm::pool().map_err(|_| BillingConfigError::new("database is unavailable"))?;
            let result = sqlx::query(
                "UPDATE purchase_attempts SET status = 'expired', last_reconciled_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = $1 AND provider_session_id = $2 AND status IN ('pending', 'unknown', 'checkout_created')",
            )
            .bind(attempt.id)
            .bind(session_id)
            .execute(pool)
            .await
            .map_err(|_| BillingConfigError::new("expired checkout could not be recorded"))?;
            processed += u32::from(result.rows_affected() > 0);
        } else {
            sqlx::query(
                "UPDATE purchase_attempts SET last_reconciled_at = CURRENT_TIMESTAMP WHERE id = $1 AND status IN ('pending', 'unknown', 'checkout_created')",
            )
            .bind(attempt.id)
            .execute(Orm::pool().map_err(|_| BillingConfigError::new("database is unavailable"))?)
            .await
            .map_err(|_| BillingConfigError::new("checkout reconciliation timestamp could not be recorded"))?;
        }
    }
    Ok(processed)
}

async fn reconcile_active_entitlements(config: &BillingConfig) -> Result<u32, BillingConfigError> {
    let entitlements = sqlx::query_as::<_, (i32, String)>(
        "SELECT id, provider_payment_id FROM entitlements WHERE provider = 'stripe' AND product_sku = $1 AND status = 'active' ORDER BY last_reconciled_at ASC NULLS FIRST, id ASC LIMIT 50",
    )
    .bind(CHECKOUT_OFFER)
    .fetch_all(Orm::pool().map_err(|_| BillingConfigError::new("database is unavailable"))?)
    .await
    .map_err(|_| BillingConfigError::new("active entitlement reconciliation lookup failed"))?;

    let mut processed = 0_u32;
    for (entitlement_id, payment_intent_id) in entitlements {
        if !payment_intent_id.starts_with("pi_") || !valid_provider_id(&payment_intent_id) {
            continue;
        }
        let response = provider_http_client()?
            .get(format!(
                "https://api.stripe.com/v1/payment_intents/{payment_intent_id}"
            ))
            .bearer_auth(&config.api_key)
            .query(&[("expand[]", "latest_charge")])
            .send()
            .await
            .map_err(|_| BillingConfigError::new("Stripe entitlement reconciliation failed"))?;
        let intent = bounded_json(response).await?;
        if intent["id"].as_str() != Some(payment_intent_id.as_str())
            || intent["livemode"].as_bool() != Some(config.mode.expects_live_object())
        {
            return Err(BillingConfigError::new(
                "Stripe entitlement evidence belongs to the wrong environment",
            ));
        }
        let charge = &intent["latest_charge"];
        let Some(charge_id) = charge["id"]
            .as_str()
            .filter(|value| value.starts_with("ch_") && valid_provider_id(value))
        else {
            continue;
        };
        if charge["payment_intent"].as_str() != Some(payment_intent_id.as_str())
            || charge["livemode"].as_bool() != Some(config.mode.expects_live_object())
        {
            return Err(BillingConfigError::new(
                "Stripe Charge evidence does not match its PaymentIntent",
            ));
        }

        let full_refund = charge["refunded"].as_bool() == Some(true)
            && charge["amount"].as_u64().is_some_and(|amount| {
                amount > 0 && charge["amount_refunded"].as_u64() == Some(amount)
            });
        let disputed = charge["disputed"].as_bool() == Some(true);
        let (reason, attempt_status, event_type) = if disputed {
            ("disputed", "disputed", "reconciliation.charge.disputed")
        } else if full_refund {
            ("refunded", "refunded", "reconciliation.charge.refunded")
        } else {
            sqlx::query(
                "UPDATE entitlements SET last_reconciled_at = CURRENT_TIMESTAMP WHERE id = $1 AND status = 'active'",
            )
            .bind(entitlement_id)
            .execute(Orm::pool().map_err(|_| BillingConfigError::new("database is unavailable"))?)
            .await
            .map_err(|_| BillingConfigError::new("entitlement reconciliation timestamp could not be recorded"))?;
            continue;
        };
        let revocation = VerifiedRevocation {
            payment_intent_id: payment_intent_id.clone(),
            reason,
            attempt_status,
        };
        let event_id = format!("reconcile-{reason}-{charge_id}");
        let payload_hash = sha256_hex(
            serde_json::to_vec(charge)
                .map_err(|_| BillingConfigError::new("Charge evidence could not be hashed"))?
                .as_slice(),
        );
        persist_revocation(&event_id, event_type, &payload_hash, &revocation).await?;
        processed += 1;
    }
    Ok(processed)
}

/// Reconciles delayed Stripe state without exposing payment identifiers. This
/// route is intended only for the protected scheduled GitHub Actions workflow.
pub async fn reconciliation_handler(headers: HeaderMap) -> Response {
    let config = match billing_config() {
        Ok(config) if config.mode == PaymentMode::Live => config,
        _ => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    if !reconciliation_authorized(&headers, &config.reconciliation_token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let Some(_guard) = ReconciliationRunGuard::acquire() else {
        return StatusCode::CONFLICT.into_response();
    };
    let processed = match reconcile_open_checkouts(&config).await {
        Ok(processed) => processed,
        Err(error) => {
            eprintln!("Checkout reconciliation failed: {error}");
            return StatusCode::BAD_GATEWAY.into_response();
        }
    };
    let processed = match reconcile_active_entitlements(&config).await {
        Ok(entitlements) => processed.saturating_add(entitlements),
        Err(error) => {
            eprintln!("Entitlement reconciliation failed: {error}");
            return StatusCode::BAD_GATEWAY.into_response();
        }
    };
    (StatusCode::OK, format!("processed={processed}")).into_response()
}

pub async fn download_stripe_report(Extension(user_id): Extension<i32>) -> Response {
    crate::controllers::artifact_controller::download_paid_artifact(user_id).await
}

fn format_amount(amount_minor: u64, currency: &str) -> String {
    format!(
        "{} {}.{:02}",
        currency,
        amount_minor / 100,
        amount_minor % 100
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sandbox_config() -> BillingConfig {
        BillingConfig {
            provider: "stripe".to_owned(),
            api_key: "sk_test_example".to_owned(),
            webhook_secret: "whsec_example_value_long_enough".to_owned(),
            redirect_url: "https://saas-staging.rullst.win/dashboard".to_owned(),
            price_id: "price_example".to_owned(),
            expected_currency: "BRL".to_owned(),
            expected_amount_minor: 100,
            founding_customer_limit: 100,
            reconciliation_token: String::new(),
            mode: PaymentMode::Test,
        }
    }

    fn open_attempt(token: &str) -> PurchaseAttempt {
        PurchaseAttempt {
            id: 42,
            user_id: 7,
            provider: "stripe".to_owned(),
            product_sku: CHECKOUT_OFFER.to_owned(),
            provider_price_id: "price_example".to_owned(),
            expected_amount_minor: 100,
            expected_currency: "BRL".to_owned(),
            checkout_token_hash: sha256_hex(token.as_bytes()),
            provider_session_id: Some("cs_test_example".to_owned()),
            provider_payment_id: None,
            status: "checkout_created".to_owned(),
            last_reconciled_at: None,
            created_at: "2026-09-17T00:00:00Z".to_owned(),
            updated_at: "2026-09-17T00:00:00Z".to_owned(),
        }
    }

    #[test]
    fn validates_provider_owned_ids() {
        assert!(valid_provider_id("price_123-test"));
        assert!(!valid_provider_id(""));
        assert!(!valid_provider_id("price/../../secret"));
    }

    #[test]
    fn checkout_session_ids_are_environment_bound() {
        assert!(valid_checkout_session_id(
            "cs_test_example",
            PaymentMode::Test
        ));
        assert!(!valid_checkout_session_id(
            "cs_live_example",
            PaymentMode::Test
        ));
        assert!(valid_checkout_session_id(
            "cs_live_example",
            PaymentMode::Live
        ));
        assert!(!valid_checkout_session_id(
            "cs_test_example",
            PaymentMode::Live
        ));
    }

    #[test]
    fn reconciliation_requires_the_exact_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            rullst::server::header::AUTHORIZATION,
            rullst::server::HeaderValue::from_static("Bearer an-exact-random-reconciliation-token"),
        );
        assert!(reconciliation_authorized(
            &headers,
            "an-exact-random-reconciliation-token"
        ));
        assert!(!reconciliation_authorized(
            &headers,
            "a-different-random-reconciliation-token"
        ));
    }

    #[test]
    fn resumes_only_the_authenticated_matching_sandbox_session() {
        let token = "opaque-checkout-token";
        let attempt = open_attempt(token);
        let session = serde_json::json!({
            "id": "cs_test_example",
            "livemode": false,
            "mode": "payment",
            "status": "open",
            "client_reference_id": token,
            "metadata": {
                "purchase_attempt_id": "42",
                "product_sku": CHECKOUT_OFFER
            }
        });
        assert!(attempt_matches_config(&attempt, &sandbox_config()));
        assert!(resumed_session_matches_attempt(
            &session,
            &attempt,
            PaymentMode::Test
        ));

        let mut wrong_session = session;
        wrong_session["client_reference_id"] = Value::String("another-token".to_owned());
        assert!(!resumed_session_matches_attempt(
            &wrong_session,
            &attempt,
            PaymentMode::Test
        ));
    }

    #[test]
    fn accepts_only_exact_stripe_checkout_urls() {
        let trusted = serde_json::json!({
            "url": "https://checkout.stripe.com/c/pay/cs_test_example"
        });
        let attacker = serde_json::json!({
            "url": "https://checkout.stripe.com.attacker.example/cs_test_example"
        });
        let embedded_credentials = serde_json::json!({
            "url": "https://user:password@checkout.stripe.com/cs_test_example"
        });
        assert!(trusted_stripe_checkout_url(&trusted).is_ok());
        assert!(trusted_stripe_checkout_url(&attacker).is_err());
        assert!(trusted_stripe_checkout_url(&embedded_credentials).is_err());
    }

    #[test]
    fn formats_low_value_amounts_without_floats() {
        assert_eq!(format_amount(100, "BRL"), "BRL 1.00");
        assert_eq!(format_amount(50, "BRL"), "BRL 0.50");
    }

    #[test]
    fn certificate_ids_are_random_opaque_and_environment_bound() {
        let first = new_certificate_id(PaymentMode::Test);
        let second = new_certificate_id(PaymentMode::Test);
        let live = new_certificate_id(PaymentMode::Live);
        assert!(crate::controllers::certificate_controller::valid_public_certificate_id(&first));
        assert!(crate::controllers::certificate_controller::valid_public_certificate_id(&second));
        assert!(crate::controllers::certificate_controller::valid_public_certificate_id(&live));
        assert!(live.starts_with("RST-LIVE-"));
        assert_ne!(first, second);
    }

    #[test]
    fn rejects_weak_webhook_secrets() {
        assert!(!strong_webhook_secret("short"));
        assert!(!strong_webhook_secret("mock_webhook_secret_that_is_long"));
        assert!(strong_webhook_secret("whsec_example_value_long_enough"));
    }

    #[test]
    fn payment_mode_is_explicit_and_fail_closed() {
        assert_eq!(
            parse_payment_mode("disabled").unwrap(),
            PaymentMode::Disabled
        );
        assert_eq!(parse_payment_mode(" TEST ").unwrap(), PaymentMode::Test);
        assert_eq!(parse_payment_mode("live").unwrap(), PaymentMode::Live);
        assert!(parse_payment_mode("true").is_err());
        assert!(parse_payment_mode("").is_err());
    }

    #[test]
    fn redirect_policy_allows_https_and_local_test_http_only() {
        assert!(valid_redirect_url(
            "https://saas-staging.rullst.win/dashboard",
            PaymentMode::Test
        ));
        assert!(valid_redirect_url(
            "http://127.0.0.1:3000/dashboard",
            PaymentMode::Test
        ));
        assert!(!valid_redirect_url(
            "http://saas-staging.rullst.win/dashboard",
            PaymentMode::Test
        ));
        assert!(!valid_redirect_url(
            "http://localhost:3000/dashboard",
            PaymentMode::Live
        ));
        assert!(!valid_redirect_url(
            "https://user:password@saas.example.com/dashboard",
            PaymentMode::Test
        ));
    }

    #[test]
    fn appends_checkout_state_without_replacing_existing_query() {
        let value =
            checkout_return_url("https://example.com/dashboard?tab=billing", "success").unwrap();
        assert!(value.contains("tab=billing"));
        assert!(value.contains("checkout=success"));
    }

    #[test]
    fn accepts_any_matching_stripe_v1_signature_during_secret_rotation() {
        let payload = br#"{"id":"evt_test"}"#;
        let timestamp = 1_750_000_000_i64;
        let secret = "whsec_rotation_test_value";
        let signed = format!("{timestamp}.{}", String::from_utf8_lossy(payload));
        let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, secret.as_bytes());
        let valid_signature = hex::encode(ring::hmac::sign(&key, signed.as_bytes()).as_ref());
        let header = format!("t={timestamp},v1=00,v1={valid_signature}");

        assert!(verify_stripe_signature_at(payload, &header, secret, timestamp).is_ok());
    }

    #[test]
    fn rejects_stale_stripe_signatures() {
        let payload = b"{}";
        let timestamp = 1_750_000_000_i64;
        let secret = "whsec_stale_test_value";
        let signed = format!("{timestamp}.{{}}");
        let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, secret.as_bytes());
        let signature = hex::encode(ring::hmac::sign(&key, signed.as_bytes()).as_ref());
        let header = format!("t={timestamp},v1={signature}");

        assert!(
            verify_stripe_signature_at(
                payload,
                &header,
                secret,
                timestamp + STRIPE_WEBHOOK_TOLERANCE_SECONDS as i64 + 1,
            )
            .is_err()
        );
    }
}
