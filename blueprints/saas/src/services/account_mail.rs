use crate::models::password_reset;
use rullst_mail::{
    DeliveryMode, DeliveryPipeline, MailDriver, MailFailureClass, Message, ResendDriver,
    validate_action_url, validate_email_syntax,
};
use std::sync::OnceLock;
use std::time::Duration;

const MAX_DELIVERY_ATTEMPTS: i32 = 5;

#[derive(Clone)]
struct ResendConfig {
    api_key: String,
    public_base_url: String,
    from: String,
}

#[derive(Clone)]
enum AccountMailRuntime {
    Disabled,
    Resend(ResendConfig),
}

static ACCOUNT_MAIL: OnceLock<AccountMailRuntime> = OnceLock::new();

pub fn initialize() -> Result<(), String> {
    let runtime = load_runtime()?;
    ACCOUNT_MAIL
        .set(runtime)
        .map_err(|_| "account mail was initialized more than once".to_owned())
}

pub fn enabled() -> bool {
    matches!(ACCOUNT_MAIL.get(), Some(AccountMailRuntime::Resend(_)))
}

fn runtime() -> Result<&'static AccountMailRuntime, String> {
    ACCOUNT_MAIL
        .get()
        .ok_or_else(|| "account mail has not been initialized".to_owned())
}

fn load_runtime() -> Result<AccountMailRuntime, String> {
    let mode = std::env::var("PASSWORD_RESET_MODE")
        .unwrap_or_else(|_| "disabled".to_owned())
        .trim()
        .to_ascii_lowercase();
    match mode.as_str() {
        "disabled" => Ok(AccountMailRuntime::Disabled),
        "resend" => {
            let api_key = required_env("RESEND_API_KEY")?;
            let driver = ResendDriver::try_new(api_key.clone())
                .map_err(|error| format!("invalid Resend configuration: {error}"))?;
            if driver.delivery_mode() != DeliveryMode::Real {
                return Err(
                    "RESEND_API_KEY resolves to an offline mock credential; recovery refuses to simulate delivery"
                        .to_owned(),
                );
            }
            let from = required_env("ACCOUNT_MAIL_FROM")?;
            validate_email_syntax(&from)
                .map_err(|error| format!("ACCOUNT_MAIL_FROM is invalid: {error}"))?;
            let public_base_url =
                validate_public_base_url(&required_env("ACCOUNT_PUBLIC_BASE_URL")?)?;
            Ok(AccountMailRuntime::Resend(ResendConfig {
                api_key,
                public_base_url,
                from,
            }))
        }
        _ => Err("PASSWORD_RESET_MODE must be disabled or resend".to_owned()),
    }
}

fn required_env(name: &str) -> Result<String, String> {
    let value = std::env::var(name).map_err(|_| format!("{name} is required"))?;
    let value = value.trim();
    if value.is_empty() || value.contains(['\r', '\n']) {
        return Err(format!("{name} is empty or unsafe"));
    }
    Ok(value.to_owned())
}

fn validate_public_base_url(value: &str) -> Result<String, String> {
    let url = reqwest::Url::parse(value)
        .map_err(|_| "ACCOUNT_PUBLIC_BASE_URL must be an absolute URL".to_owned())?;
    if url.username() != ""
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err(
            "ACCOUNT_PUBLIC_BASE_URL must contain only scheme and host, without credentials, path, query or fragment"
                .to_owned(),
        );
    }
    let host = url
        .host_str()
        .ok_or_else(|| "ACCOUNT_PUBLIC_BASE_URL must include a host".to_owned())?;
    let tier = std::env::var("DEPLOYMENT_TIER")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    let valid = match tier.as_str() {
        "staging" => url.scheme() == "https" && host == "saas-staging.rullst.win",
        "production" => url.scheme() == "https" && host == "saas.rullst.win",
        _ => {
            matches!(url.scheme(), "http" | "https")
                && matches!(host, "localhost" | "127.0.0.1" | "::1")
        }
    };
    if !valid {
        return Err("ACCOUNT_PUBLIC_BASE_URL does not match the deployment tier".to_owned());
    }
    Ok(value.trim_end_matches('/').to_owned())
}

fn reset_message(config: &ResendConfig, recipient: &str, code: &str) -> Result<Message, String> {
    if !password_reset::valid_code_shape(code) {
        return Err("password-reset code has an invalid shape".to_owned());
    }
    let action_url = format!("{}/reset-password#code={code}", config.public_base_url);
    validate_action_url(&action_url)
        .map_err(|error| format!("password-reset URL is invalid: {error}"))?;
    let escaped_url = rullst_mail::escape_html(&action_url);
    let message = Message::new()
        .to(recipient)
        .from(config.from.clone())
        .subject("Reset your Rullst SaaS password")
        .html(format!(
            "<p>We received a request to reset your Rullst SaaS password.</p><p><a href=\"{escaped_url}\">Reset password</a></p><p>This link expires in 15 minutes and works once. If you did not request it, you can ignore this email.</p>"
        ))
        .text(format!(
            "We received a request to reset your Rullst SaaS password.\n\nReset password: {action_url}\n\nThis link expires in 15 minutes and works once. If you did not request it, you can ignore this email."
        ));
    DeliveryPipeline::prepare(&message)
        .map_err(|error| format!("password-reset message was rejected: {error}"))?;
    Ok(message)
}

fn purchase_confirmation_message(
    config: &ResendConfig,
    recipient: &str,
    holder_name: &str,
    public_id: &str,
    environment: &str,
) -> Result<Message, String> {
    if public_id.len() > 80
        || !public_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err("certificate public ID has an invalid shape".to_owned());
    }
    if !matches!(environment, "live" | "test") {
        return Err("certificate environment is invalid".to_owned());
    }
    let dashboard_url = format!("{}/dashboard", config.public_base_url);
    let certificate_url = format!("{}/certificate", config.public_base_url);
    let verification_url = format!("{}/verify/{public_id}", config.public_base_url);
    let guide_url = format!(
        "{}/reports/stripe-gateway-field-report-v1.md",
        config.public_base_url
    );
    for url in [
        dashboard_url.as_str(),
        certificate_url.as_str(),
        verification_url.as_str(),
        guide_url.as_str(),
        "https://rullst.win",
        "https://github.com/Rullst/examples",
        "https://discord.gg/2ntKFtsSjw",
    ] {
        validate_action_url(url)
            .map_err(|error| format!("purchase confirmation URL is invalid: {error}"))?;
    }

    let badge = if environment == "live" {
        "Rullst Founding Customer"
    } else {
        "Rullst Sandbox Pioneer"
    };
    let boundary = if environment == "live" {
        "Stripe confirmed your real one-time purchase."
    } else {
        "Stripe Test Mode confirmed this sandbox flow; no real charge was created."
    };
    let escaped_name = rullst_mail::escape_html(holder_name);
    let escaped_dashboard = rullst_mail::escape_html(&dashboard_url);
    let escaped_certificate = rullst_mail::escape_html(&certificate_url);
    let escaped_verification = rullst_mail::escape_html(&verification_url);
    let escaped_guide = rullst_mail::escape_html(&guide_url);
    let html = format!(
        r#"<!doctype html><html><body style="margin:0;background:#070b14;color:#e5e7eb;font-family:Arial,sans-serif"><table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="background:#070b14;padding:24px"><tr><td align="center"><table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="max-width:640px;border:1px solid #334155;border-radius:18px;background:#0f172a;overflow:hidden"><tr><td style="padding:34px;background:linear-gradient(135deg,#312e81,#0f172a 55%,#064e3b)"><p style="margin:0 0 8px;color:#a5b4fc;font-size:12px;font-weight:700;letter-spacing:2px;text-transform:uppercase">Rullst purchase confirmed</p><h1 style="margin:0;color:#fff;font-size:30px">Your {badge} certificate is ready</h1></td></tr><tr><td style="padding:32px"><p style="font-size:18px;line-height:1.6">Hello <strong>{escaped_name}</strong>,</p><p style="color:#cbd5e1;line-height:1.7">{boundary} Your permanent account name is now displayed on the private certificate.</p><p style="margin:28px 0"><a href="{escaped_certificate}" style="display:inline-block;padding:14px 20px;border-radius:10px;background:#f97316;color:#fff;font-weight:700;text-decoration:none">Open your certificate</a> <a href="{escaped_guide}" style="display:inline-block;margin-left:8px;padding:14px 20px;border-radius:10px;background:#10b981;color:#052e2b;font-weight:700;text-decoration:none">Download your guide</a></p><table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="border-collapse:separate;border-spacing:0 10px"><tr><td><a href="{escaped_dashboard}" style="color:#6ee7b7">SaaS dashboard</a></td></tr><tr><td><a href="{escaped_verification}" style="color:#6ee7b7">Privacy-preserving public certificate verification</a></td></tr><tr><td><a href="https://rullst.win" style="color:#6ee7b7">Rullst website</a></td></tr><tr><td><a href="https://github.com/Rullst/examples" style="color:#6ee7b7">Rullst examples repository</a></td></tr><tr><td><a href="https://discord.gg/2ntKFtsSjw" style="color:#a5b4fc">Join the Rullst Discord community</a></td></tr></table><p style="margin-top:28px;color:#94a3b8;font-size:13px;line-height:1.6">Certificate ID: {public_id}. This transactional message confirms requested account activity; it contains no marketing tracker. If you need help, contact officialrullst@gmail.com.</p></td></tr></table></td></tr></table></body></html>"#,
        public_id = rullst_mail::escape_html(public_id),
    );
    let text = format!(
        "Hello {holder_name},\n\n{boundary}\nYour {badge} certificate is ready.\n\nCertificate: {certificate_url}\nPurchased guide: {guide_url}\nDashboard: {dashboard_url}\nPublic verification: {verification_url}\nRullst: https://rullst.win\nExamples repository: https://github.com/Rullst/examples\nDiscord: https://discord.gg/2ntKFtsSjw\n\nCertificate ID: {public_id}\nThis transactional message contains no marketing tracker. Support: officialrullst@gmail.com"
    );
    let message = Message::new()
        .to(recipient)
        .from(config.from.clone())
        .subject(format!("Your {badge} certificate is ready"))
        .html(html)
        .text(text);
    DeliveryPipeline::prepare(&message)
        .map_err(|error| format!("purchase confirmation was rejected: {error}"))?;
    Ok(message)
}

async fn send_reset(recipient: &str, code: &str) -> Result<(), rullst_mail::MailError> {
    let AccountMailRuntime::Resend(config) =
        runtime().map_err(rullst_mail::MailError::ConfigError)?
    else {
        return Err(rullst_mail::MailError::ConfigError(
            "password recovery is disabled".to_owned(),
        ));
    };
    let message =
        reset_message(config, recipient, code).map_err(rullst_mail::MailError::ValidationError)?;
    ResendDriver::try_new(config.api_key.clone())?
        .send(&message)
        .await
}

async fn send_purchase_confirmation(
    recipient: &str,
    holder_name: &str,
    public_id: &str,
    environment: &str,
) -> Result<(), rullst_mail::MailError> {
    let AccountMailRuntime::Resend(config) =
        runtime().map_err(rullst_mail::MailError::ConfigError)?
    else {
        return Err(rullst_mail::MailError::ConfigError(
            "account mail is disabled".to_owned(),
        ));
    };
    let message =
        purchase_confirmation_message(config, recipient, holder_name, public_id, environment)
            .map_err(rullst_mail::MailError::ValidationError)?;
    ResendDriver::try_new(config.api_key.clone())?
        .send(&message)
        .await
}

pub fn spawn_worker() {
    if !enabled() {
        return;
    }
    rullst::runtime::spawn(async {
        loop {
            match process_one().await {
                Ok(true) => continue,
                Ok(false) => tokio::time::sleep(Duration::from_secs(3)).await,
                Err(error) => {
                    eprintln!("Password-reset mail worker failed: {error}");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            }
        }
    });
}

async fn process_one() -> Result<bool, String> {
    let reset_processed = process_password_reset_one().await?;
    let purchase_processed = process_purchase_confirmation_one().await?;
    Ok(reset_processed || purchase_processed)
}

async fn process_password_reset_one() -> Result<bool, String> {
    let pool = rullst::db::Orm::pool().map_err(|error| error.to_string())?;
    rullst::db::sqlx::query(
        "DELETE FROM password_reset_attempts WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '24 hours'",
    )
    .execute(pool)
    .await
    .map_err(|error| error.to_string())?;
    rullst::db::sqlx::query(
        r#"UPDATE password_reset_mail_outbox AS outbox
           SET status = 'cancelled', locked_at = NULL
           FROM password_reset_tokens AS token
           WHERE outbox.reset_token_id = token.id
             AND outbox.status IN ('pending', 'processing')
             AND (token.used_at IS NOT NULL OR token.expires_at <= CURRENT_TIMESTAMP)"#,
    )
    .execute(pool)
    .await
    .map_err(|error| error.to_string())?;
    rullst::db::sqlx::query(
        "DELETE FROM password_reset_tokens WHERE expires_at < CURRENT_TIMESTAMP - INTERVAL '24 hours'",
    )
    .execute(pool)
    .await
    .map_err(|error| error.to_string())?;

    let job = rullst::db::sqlx::query_as::<_, (i64, i32, String, i64, String, i32)>(
        r#"WITH candidate AS (
               SELECT outbox.id
               FROM password_reset_mail_outbox AS outbox
               JOIN password_reset_tokens AS token ON token.id = outbox.reset_token_id
               WHERE outbox.attempts < 5
                 AND token.used_at IS NULL
                 AND token.expires_at > CURRENT_TIMESTAMP
                 AND (
                     (outbox.status = 'pending' AND outbox.available_at <= CURRENT_TIMESTAMP)
                     OR (outbox.status = 'processing' AND outbox.locked_at < CURRENT_TIMESTAMP - INTERVAL '5 minutes')
                 )
               ORDER BY outbox.id
               FOR UPDATE OF outbox SKIP LOCKED
               LIMIT 1
           ), claimed AS (
               UPDATE password_reset_mail_outbox AS outbox
               SET status = 'processing', attempts = attempts + 1, locked_at = CURRENT_TIMESTAMP
               FROM candidate
               WHERE outbox.id = candidate.id
               RETURNING outbox.id, outbox.reset_token_id, outbox.attempts
           )
           SELECT claimed.id,
                  token.user_id,
                  token.selector,
                  EXTRACT(EPOCH FROM token.expires_at)::BIGINT,
                  users.email,
                  claimed.attempts
           FROM claimed
           JOIN password_reset_tokens AS token ON token.id = claimed.reset_token_id
           JOIN users ON users.id = token.user_id"#,
    )
    .fetch_optional(pool)
    .await
    .map_err(|error| error.to_string())?;

    let Some((outbox_id, user_id, selector, expires_unix, recipient, attempts)) = job else {
        return Ok(false);
    };
    let app_key = rullst::auth::get_app_key().map_err(|error| error.to_string())?;
    let code = password_reset::reconstruct_code(&selector, user_id, expires_unix, &app_key)?;
    match send_reset(&recipient, &code).await {
        Ok(()) => {
            rullst::db::sqlx::query(
                "UPDATE password_reset_mail_outbox SET status = 'sent', sent_at = CURRENT_TIMESTAMP, locked_at = NULL, last_error_class = NULL WHERE id = $1",
            )
            .bind(outbox_id)
            .execute(pool)
            .await
            .map_err(|error| error.to_string())?;
        }
        Err(error) => {
            let class = error.failure_class();
            let retryable = matches!(
                class,
                MailFailureClass::Transient | MailFailureClass::RateLimited
            ) && attempts < MAX_DELIVERY_ATTEMPTS;
            let retry_seconds = error
                .retry_after()
                .map(|duration| duration.as_secs().min(900) as i64)
                .unwrap_or_else(|| 15_i64.saturating_mul(1_i64 << attempts.min(5)));
            let next_status = if retryable { "pending" } else { "failed" };
            rullst::db::sqlx::query(
                r#"UPDATE password_reset_mail_outbox
                   SET status = $2,
                       available_at = CURRENT_TIMESTAMP + ($3 * INTERVAL '1 second'),
                       locked_at = NULL,
                       last_error_class = $4
                   WHERE id = $1"#,
            )
            .bind(outbox_id)
            .bind(next_status)
            .bind(retry_seconds)
            .bind(class.as_str())
            .execute(pool)
            .await
            .map_err(|database_error| database_error.to_string())?;
            eprintln!(
                "Password-reset mail delivery did not complete: outbox_id={outbox_id}, class={}",
                class.as_str()
            );
        }
    }
    Ok(true)
}

async fn process_purchase_confirmation_one() -> Result<bool, String> {
    let pool = rullst::db::Orm::pool().map_err(|error| error.to_string())?;
    rullst::db::sqlx::query(
        r#"UPDATE purchase_confirmation_mail_outbox AS outbox
           SET status = 'cancelled', locked_at = NULL, updated_at = CURRENT_TIMESTAMP
           FROM entitlements AS entitlement
           LEFT JOIN tester_certificates AS certificate
               ON certificate.entitlement_id = entitlement.id
           WHERE outbox.entitlement_id = entitlement.id
             AND outbox.status IN ('pending', 'processing')
             AND (entitlement.status <> 'active' OR certificate.status IS DISTINCT FROM 'active')"#,
    )
    .execute(pool)
    .await
    .map_err(|error| error.to_string())?;

    let job = rullst::db::sqlx::query_as::<_, (i64, String, String, String, String, i32)>(
        r#"WITH candidate AS (
               SELECT outbox.id
               FROM purchase_confirmation_mail_outbox AS outbox
               WHERE outbox.attempts < 5
                 AND (
                     (outbox.status = 'pending' AND outbox.available_at <= CURRENT_TIMESTAMP)
                     OR (outbox.status = 'processing' AND outbox.locked_at < CURRENT_TIMESTAMP - INTERVAL '5 minutes')
                 )
               ORDER BY outbox.id
               FOR UPDATE OF outbox SKIP LOCKED
               LIMIT 1
           ), claimed AS (
               UPDATE purchase_confirmation_mail_outbox AS outbox
               SET status = 'processing', attempts = attempts + 1,
                   locked_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
               FROM candidate
               WHERE outbox.id = candidate.id
               RETURNING outbox.id, outbox.entitlement_id, outbox.attempts
           )
           SELECT claimed.id, users.email, users.name, certificate.public_id,
                  certificate.environment, claimed.attempts
           FROM claimed
           INNER JOIN entitlements AS entitlement ON entitlement.id = claimed.entitlement_id
           INNER JOIN users ON users.id = entitlement.user_id
           INNER JOIN tester_certificates AS certificate
               ON certificate.entitlement_id = entitlement.id
           WHERE entitlement.status = 'active' AND certificate.status = 'active'"#,
    )
    .fetch_optional(pool)
    .await
    .map_err(|error| error.to_string())?;

    let Some((outbox_id, recipient, holder_name, public_id, environment, attempts)) = job else {
        return Ok(false);
    };
    match send_purchase_confirmation(&recipient, &holder_name, &public_id, &environment).await {
        Ok(()) => {
            rullst::db::sqlx::query(
                "UPDATE purchase_confirmation_mail_outbox SET status = 'sent', sent_at = CURRENT_TIMESTAMP, locked_at = NULL, last_error_class = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = $1",
            )
            .bind(outbox_id)
            .execute(pool)
            .await
            .map_err(|error| error.to_string())?;
        }
        Err(error) => {
            let class = error.failure_class();
            let retryable = matches!(
                class,
                MailFailureClass::Transient | MailFailureClass::RateLimited
            ) && attempts < MAX_DELIVERY_ATTEMPTS;
            let retry_seconds = error
                .retry_after()
                .map(|duration| duration.as_secs().min(900) as i64)
                .unwrap_or_else(|| 15_i64.saturating_mul(1_i64 << attempts.min(5)));
            let next_status = if retryable { "pending" } else { "failed" };
            rullst::db::sqlx::query(
                r#"UPDATE purchase_confirmation_mail_outbox
                   SET status = $2,
                       available_at = CURRENT_TIMESTAMP + ($3 * INTERVAL '1 second'),
                       locked_at = NULL,
                       last_error_class = $4,
                       updated_at = CURRENT_TIMESTAMP
                   WHERE id = $1"#,
            )
            .bind(outbox_id)
            .bind(next_status)
            .bind(retry_seconds)
            .bind(class.as_str())
            .execute(pool)
            .await
            .map_err(|database_error| database_error.to_string())?;
            eprintln!(
                "Purchase confirmation delivery did not complete: outbox_id={outbox_id}, class={}",
                class.as_str()
            );
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> ResendConfig {
        ResendConfig {
            api_key: "re_test_fixture".to_owned(),
            public_base_url: "https://saas-staging.rullst.win".to_owned(),
            from: "security@example.com".to_owned(),
        }
    }

    #[test]
    fn mandatory_pipeline_preserves_the_code_fragment_workaround() {
        let code = format!("{}.{}", "a".repeat(64), "b".repeat(64));
        let message =
            reset_message(&config(), "person@example.com", &code).expect("build reset message");
        let prepared = DeliveryPipeline::prepare(&message).expect("prepare reset message");
        let body = prepared
            .message()
            .body_text
            .as_deref()
            .expect("plain-text body");
        assert!(body.contains(&format!("#code={code}")));
        assert!(!body.contains("[REDACTED]"));
    }

    #[test]
    fn reset_message_rejects_an_unstructured_code() {
        assert!(reset_message(&config(), "person@example.com", "short").is_err());
    }

    #[test]
    fn purchase_confirmation_contains_the_requested_links_without_tracking() {
        let message = purchase_confirmation_message(
            &config(),
            "person@example.com",
            "A Person",
            "RST-LIVE-0123456789ABCDEF0123456789ABCDEF",
            "live",
        )
        .expect("build purchase confirmation");
        let prepared = DeliveryPipeline::prepare(&message).expect("prepare purchase confirmation");
        let body = prepared.message().body_html.as_deref().expect("HTML body");
        assert!(body.contains("/certificate"));
        assert!(body.contains("/reports/stripe-gateway-field-report-v1.md"));
        assert!(body.contains("https://rullst.win"));
        assert!(body.contains("https://github.com/Rullst/examples"));
        assert!(body.contains("https://discord.gg/2ntKFtsSjw"));
        assert!(!body.contains("tracking"));
    }
}
