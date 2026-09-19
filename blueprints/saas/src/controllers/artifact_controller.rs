use crate::controllers::billing_controller::{BillingConfigError, account_has_stripe_report};
use reqwest::{Client, Url};
use rullst::server::{HeaderMap, IntoResponse, Response, StatusCode};
use sha2::{Digest, Sha256};
use std::net::IpAddr;
use std::time::Duration;

const MAX_PRIVATE_ARTIFACT_BYTES: usize = 2 * 1024 * 1024;
const MAX_IDENTITY_RESPONSE_BYTES: usize = 32 * 1024;
const AZURE_STORAGE_API_VERSION: &str = "2023-11-03";
const SANDBOX_REPORT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/reports/stripe-gateway-field-report-v1.md"
));

struct ArtifactConfig {
    url: Url,
    sha256: String,
    filename: String,
}

fn live_artifact_config() -> Result<ArtifactConfig, BillingConfigError> {
    let raw_url = std::env::var("PAID_ARTIFACT_URL")
        .unwrap_or_default()
        .trim()
        .to_owned();
    let url = Url::parse(&raw_url)
        .map_err(|_| BillingConfigError::new("PAID_ARTIFACT_URL is invalid"))?;
    let storage_account = std::env::var("PAID_ARTIFACT_STORAGE_ACCOUNT")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if storage_account.len() < 3
        || storage_account.len() > 24
        || !storage_account
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    {
        return Err(BillingConfigError::new(
            "PAID_ARTIFACT_STORAGE_ACCOUNT must be a valid Azure Storage account name",
        ));
    }
    let trusted_host = format!("{storage_account}.blob.core.windows.net");
    if url.scheme() != "https"
        || url.host_str() != Some(trusted_host.as_str())
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(BillingConfigError::new(
            "PAID_ARTIFACT_URL must be a query-free private HTTPS URL in the configured Azure Storage account",
        ));
    }
    let sha256 = std::env::var("PAID_ARTIFACT_SHA256")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if sha256.len() != 64 || !sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(BillingConfigError::new(
            "PAID_ARTIFACT_SHA256 must be a 64-character hexadecimal digest",
        ));
    }
    let filename = std::env::var("PAID_ARTIFACT_FILENAME")
        .unwrap_or_else(|_| "rullst-stripe-production-guide-v1.md".to_owned());
    let filename = filename.trim().to_owned();
    if filename.is_empty()
        || filename.len() > 100
        || !filename.ends_with(".md")
        || !filename
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(BillingConfigError::new(
            "PAID_ARTIFACT_FILENAME must be a safe Markdown filename",
        ));
    }
    Ok(ArtifactConfig {
        url,
        sha256,
        filename,
    })
}

pub fn validate_live_artifact_configuration() -> Result<(), BillingConfigError> {
    live_artifact_config()?;
    managed_identity_configuration().map(|_| ())
}

fn trusted_identity_endpoint(url: &Url) -> bool {
    if url.scheme() != "http"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return false;
    }
    match url.host_str() {
        Some("localhost") => true,
        Some(host) => host.parse::<IpAddr>().is_ok_and(|address| match address {
            IpAddr::V4(address) => {
                address.is_loopback() || address.octets().starts_with(&[169, 254])
            }
            IpAddr::V6(address) => address.is_loopback(),
        }),
        None => false,
    }
}

fn managed_identity_configuration() -> Result<(Url, String), BillingConfigError> {
    let endpoint = std::env::var("IDENTITY_ENDPOINT")
        .unwrap_or_default()
        .trim()
        .to_owned();
    let endpoint = Url::parse(&endpoint)
        .map_err(|_| BillingConfigError::new("Azure managed identity endpoint is unavailable"))?;
    if !trusted_identity_endpoint(&endpoint) {
        return Err(BillingConfigError::new(
            "Azure managed identity endpoint is not a trusted local endpoint",
        ));
    }
    let identity_header = std::env::var("IDENTITY_HEADER")
        .unwrap_or_default()
        .trim()
        .to_owned();
    if identity_header.len() < 16
        || identity_header.len() > 4096
        || identity_header.contains(['\r', '\n'])
    {
        return Err(BillingConfigError::new(
            "Azure managed identity header is unavailable",
        ));
    }
    Ok((endpoint, identity_header))
}

async fn managed_identity_token() -> Result<String, BillingConfigError> {
    let (mut endpoint, identity_header) = managed_identity_configuration()?;
    endpoint.query_pairs_mut().clear().extend_pairs([
        ("resource", "https://storage.azure.com/"),
        ("api-version", "2019-08-01"),
    ]);
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(5))
        .no_proxy()
        .build()
        .map_err(|_| BillingConfigError::new("managed identity client could not be built"))?;
    let response = client
        .get(endpoint)
        .header("X-IDENTITY-HEADER", identity_header)
        .send()
        .await
        .map_err(|_| BillingConfigError::new("managed identity token is unavailable"))?;
    if !response.status().is_success()
        || response
            .content_length()
            .is_some_and(|length| length > MAX_IDENTITY_RESPONSE_BYTES as u64)
    {
        return Err(BillingConfigError::new(
            "managed identity token is unavailable",
        ));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|_| BillingConfigError::new("managed identity response could not be read"))?;
    if bytes.len() > MAX_IDENTITY_RESPONSE_BYTES {
        return Err(BillingConfigError::new(
            "managed identity response is too large",
        ));
    }
    let body: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|_| BillingConfigError::new("managed identity response is invalid"))?;
    let token = body["access_token"]
        .as_str()
        .filter(|token| {
            token.len() >= 32
                && token.len() <= 16 * 1024
                && !token.bytes().any(|byte| byte.is_ascii_control())
        })
        .ok_or_else(|| BillingConfigError::new("managed identity response has no access token"))?;
    Ok(token.to_owned())
}

fn attachment_response(body: String, filename: &str) -> Response {
    let mut response = (StatusCode::OK, body).into_response();
    response.headers_mut().insert(
        rullst::server::header::CONTENT_TYPE,
        rullst::server::HeaderValue::from_static("text/markdown; charset=utf-8"),
    );
    if let Ok(value) =
        rullst::server::HeaderValue::from_str(&format!("attachment; filename=\"{filename}\""))
    {
        response
            .headers_mut()
            .insert(rullst::server::header::CONTENT_DISPOSITION, value);
    }
    response.headers_mut().insert(
        rullst::server::header::CACHE_CONTROL,
        rullst::server::HeaderValue::from_static("private, no-store"),
    );
    response
}

async fn retrieve_live_artifact() -> Result<(String, String), BillingConfigError> {
    let config = live_artifact_config()?;
    let access_token = managed_identity_token().await?;
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .https_only(true)
        .no_proxy()
        .build()
        .map_err(|_| BillingConfigError::new("private artifact client could not be built"))?;
    let mut response = client
        .get(config.url)
        .bearer_auth(access_token)
        .header("x-ms-version", AZURE_STORAGE_API_VERSION)
        .send()
        .await
        .map_err(|_| BillingConfigError::new("private artifact is unavailable"))?;
    if !response.status().is_success() {
        return Err(BillingConfigError::new(format!(
            "private artifact returned HTTP {}",
            response.status().as_u16()
        )));
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_PRIVATE_ARTIFACT_BYTES as u64)
    {
        return Err(BillingConfigError::new("private artifact is too large"));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| BillingConfigError::new("private artifact could not be read"))?
    {
        if bytes.len().saturating_add(chunk.len()) > MAX_PRIVATE_ARTIFACT_BYTES {
            return Err(BillingConfigError::new("private artifact is too large"));
        }
        bytes.extend_from_slice(&chunk);
    }
    if hex::encode(Sha256::digest(&bytes)) != config.sha256 {
        return Err(BillingConfigError::new(
            "private artifact integrity check failed",
        ));
    }
    let body = String::from_utf8(bytes)
        .map_err(|_| BillingConfigError::new("private artifact is not UTF-8 Markdown"))?;
    Ok((body, config.filename))
}

fn temporarily_unavailable_response() -> Response {
    let mut response = (
        StatusCode::SERVICE_UNAVAILABLE,
        "The purchased guide is temporarily unavailable. Your purchase and certificate remain safe. Return to the dashboard and try again in 30 seconds; contact officialrullst@gmail.com if the problem continues.",
    )
        .into_response();
    response.headers_mut().insert(
        rullst::server::header::RETRY_AFTER,
        rullst::server::HeaderValue::from_static("30"),
    );
    response.headers_mut().insert(
        rullst::server::header::CACHE_CONTROL,
        rullst::server::HeaderValue::from_static("private, no-store"),
    );
    response
}

/// End-to-end readiness probe for the protected production workflow. It reads
/// and verifies the private artifact through the Container App's own managed
/// identity, but never returns the purchased content.
pub async fn live_artifact_readiness(headers: HeaderMap) -> Response {
    let mut response =
        if !crate::controllers::billing_controller::protected_operation_authorized(&headers) {
            StatusCode::UNAUTHORIZED.into_response()
        } else {
            match retrieve_live_artifact().await {
                Ok(_) => (StatusCode::OK, "ready").into_response(),
                Err(error) => {
                    eprintln!("Private artifact readiness failed: {error}");
                    StatusCode::SERVICE_UNAVAILABLE.into_response()
                }
            }
        };
    response.headers_mut().insert(
        rullst::server::header::CACHE_CONTROL,
        rullst::server::HeaderValue::from_static("private, no-store"),
    );
    response
}

pub async fn download_paid_artifact(user_id: i32) -> Response {
    match account_has_stripe_report(user_id).await {
        Ok(true) => {}
        Ok(false) => return StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            eprintln!("Artifact entitlement lookup failed: {error}");
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    }

    let live =
        std::env::var("PAYMENTS_MODE").is_ok_and(|value| value.trim().eq_ignore_ascii_case("live"));
    if !live {
        return attachment_response(
            SANDBOX_REPORT.to_owned(),
            "rullst-stripe-gateway-field-report-v1.md",
        );
    }
    match retrieve_live_artifact().await {
        Ok((body, filename)) => attachment_response(body, &filename),
        Err(error) => {
            eprintln!("Private artifact delivery failed: {error}");
            temporarily_unavailable_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AZURE_STORAGE_API_VERSION, MAX_PRIVATE_ARTIFACT_BYTES, temporarily_unavailable_response,
        trusted_identity_endpoint,
    };
    use reqwest::Url;
    use rullst::server::StatusCode;

    #[test]
    fn private_artifact_is_deliberately_bounded() {
        assert_eq!(MAX_PRIVATE_ARTIFACT_BYTES, 2 * 1024 * 1024);
        assert_eq!(AZURE_STORAGE_API_VERSION, "2023-11-03");
    }

    #[test]
    fn unavailable_download_is_explained_without_losing_purchase_context() {
        let response = temporarily_unavailable_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            response.headers()[rullst::server::header::RETRY_AFTER],
            "30"
        );
    }

    #[test]
    fn managed_identity_endpoint_must_be_local_or_link_local() {
        assert!(trusted_identity_endpoint(
            &Url::parse("http://127.0.0.1:42356/msi/token").unwrap()
        ));
        assert!(trusted_identity_endpoint(
            &Url::parse("http://169.254.129.6:42356/msi/token").unwrap()
        ));
        assert!(!trusted_identity_endpoint(
            &Url::parse("https://attacker.example/token").unwrap()
        ));
    }
}
