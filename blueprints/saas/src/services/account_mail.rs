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
}
