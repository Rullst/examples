use crate::controllers::billing_controller::{BillingConfigError, account_has_stripe_report};
use reqwest::{Client, Url};
use rullst::server::{IntoResponse, Response, StatusCode};
use sha2::{Digest, Sha256};
use std::time::Duration;

const MAX_PRIVATE_ARTIFACT_BYTES: usize = 2 * 1024 * 1024;
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
    let trusted_host = url
        .host_str()
        .is_some_and(|host| host.ends_with(".blob.core.windows.net"));
    if url.scheme() != "https"
        || !trusted_host
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_none()
    {
        return Err(BillingConfigError::new(
            "PAID_ARTIFACT_URL must be a private HTTPS Azure Blob SAS URL",
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
    live_artifact_config().map(|_| ())
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
        .send()
        .await
        .map_err(|_| BillingConfigError::new("private artifact is unavailable"))?;
    if !response.status().is_success() {
        return Err(BillingConfigError::new("private artifact is unavailable"));
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
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MAX_PRIVATE_ARTIFACT_BYTES;

    #[test]
    fn private_artifact_is_deliberately_bounded() {
        assert_eq!(MAX_PRIVATE_ARTIFACT_BYTES, 2 * 1024 * 1024);
    }
}
