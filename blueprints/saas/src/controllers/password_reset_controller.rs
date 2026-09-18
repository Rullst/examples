use crate::controllers::auth_controller::{
    get_csp_nonce, get_csrf_token, normalize_email, valid_email,
};
use crate::models::password_reset;
use crate::pages::auth;
use rullst::server::{Extension, Form, HeaderMap, IntoResponse, Response, StatusCode};
use serde::Deserialize;
use std::time::{Duration, Instant};

const GENERIC_REQUEST_MESSAGE: &str =
    "If an eligible account exists, a password-reset email will arrive shortly.";
const MINIMUM_REQUEST_DURATION: Duration = Duration::from_millis(300);

#[derive(Deserialize)]
pub struct ForgotPasswordDto {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordDto {
    pub code: String,
    pub password: String,
    pub password_confirmation: String,
}

fn sensitive_response(page: rullst::response::Html<String>) -> Response {
    let mut response = page.into_response();
    response.headers_mut().insert(
        rullst::server::header::CACHE_CONTROL,
        rullst::server::HeaderValue::from_static("no-store, max-age=0"),
    );
    response.headers_mut().insert(
        rullst::server::header::HeaderName::from_static("referrer-policy"),
        rullst::server::HeaderValue::from_static("no-referrer"),
    );
    response
}

async fn wait_for_generic_response(started: Instant) {
    if let Some(remaining) = MINIMUM_REQUEST_DURATION.checked_sub(started.elapsed()) {
        tokio::time::sleep(remaining).await;
    }
}

pub async fn forgot_password_view(
    headers: HeaderMap,
    csrf: Option<Extension<rullst::security::CsrfToken>>,
    csp_nonce: Option<Extension<rullst::security::CspNonce>>,
) -> Response {
    let token = csrf
        .as_ref()
        .map(|Extension(token)| token.as_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| get_csrf_token(&headers));
    sensitive_response(auth::forgot_password_page(
        &token,
        None,
        crate::services::account_mail::enabled(),
        get_csp_nonce(&csp_nonce),
    ))
}

pub async fn forgot_password_submit(
    headers: HeaderMap,
    csp_nonce: Option<Extension<rullst::security::CspNonce>>,
    Form(payload): Form<ForgotPasswordDto>,
) -> Response {
    let started = Instant::now();
    let token = get_csrf_token(&headers);
    let nonce = get_csp_nonce(&csp_nonce);
    let email = normalize_email(&payload.email);

    if crate::services::account_mail::enabled()
        && valid_email(&email)
        && let Err(error) = enqueue_request(&email).await
    {
        eprintln!("Password-reset request could not be queued: {error}");
    }

    wait_for_generic_response(started).await;
    sensitive_response(auth::forgot_password_page(
        &token,
        Some(GENERIC_REQUEST_MESSAGE),
        crate::services::account_mail::enabled(),
        nonce,
    ))
}

async fn enqueue_request(email: &str) -> Result<(), String> {
    let app_key = rullst::auth::get_app_key().map_err(|error| error.to_string())?;
    let request_key = password_reset::request_key(email, &app_key);
    let mut transaction = rullst::db::Orm::begin_transaction()
        .await
        .map_err(|error| error.to_string())?;

    rullst::db::sqlx::query(
        "DELETE FROM password_reset_attempts WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '24 hours'",
    )
    .execute(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;
    let per_address = rullst::db::sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM password_reset_attempts WHERE request_key = $1 AND created_at > CURRENT_TIMESTAMP - INTERVAL '1 hour'",
    )
    .bind(&request_key)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;
    let global = rullst::db::sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM password_reset_attempts WHERE created_at > CURRENT_TIMESTAMP - INTERVAL '10 minutes'",
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;
    if per_address >= 3 || global >= 100 {
        transaction
            .commit()
            .await
            .map_err(|error| error.to_string())?;
        return Ok(());
    }

    rullst::db::sqlx::query("INSERT INTO password_reset_attempts (request_key) VALUES ($1)")
        .bind(request_key)
        .execute(&mut *transaction)
        .await
        .map_err(|error| error.to_string())?;

    let user = rullst::db::sqlx::query_as::<_, (i32, Option<String>)>(
        "SELECT id, password_hash FROM users WHERE email = $1 LIMIT 1",
    )
    .bind(email)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;

    if let Some((user_id, Some(_))) = user {
        rullst::db::sqlx::query(
            r#"UPDATE password_reset_mail_outbox AS outbox
               SET status = 'cancelled', locked_at = NULL
               FROM password_reset_tokens AS token
               WHERE outbox.reset_token_id = token.id
                 AND token.user_id = $1
                 AND outbox.status IN ('pending', 'processing')"#,
        )
        .bind(user_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| error.to_string())?;
        rullst::db::sqlx::query(
            "UPDATE password_reset_tokens SET used_at = CURRENT_TIMESTAMP WHERE user_id = $1 AND used_at IS NULL",
        )
        .bind(user_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| error.to_string())?;

        let material = password_reset::generate_code(user_id, &app_key)?;
        let reset_token_id = rullst::db::sqlx::query_scalar::<_, i64>(
            r#"INSERT INTO password_reset_tokens
               (user_id, token_hash, selector, expires_at)
               VALUES ($1, $2, $3, TO_TIMESTAMP($4))
               RETURNING id"#,
        )
        .bind(user_id)
        .bind(material.token_hash)
        .bind(material.selector)
        .bind(material.expires_unix)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| error.to_string())?;
        rullst::db::sqlx::query(
            "INSERT INTO password_reset_mail_outbox (reset_token_id) VALUES ($1)",
        )
        .bind(reset_token_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| error.to_string())?;
    }

    transaction
        .commit()
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub async fn reset_password_view(
    headers: HeaderMap,
    csrf: Option<Extension<rullst::security::CsrfToken>>,
    csp_nonce: Option<Extension<rullst::security::CspNonce>>,
) -> Response {
    let token = csrf
        .as_ref()
        .map(|Extension(token)| token.as_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| get_csrf_token(&headers));
    sensitive_response(auth::reset_password_page(
        &token,
        None,
        false,
        None,
        get_csp_nonce(&csp_nonce),
    ))
}

pub async fn reset_password_submit(
    headers: HeaderMap,
    csp_nonce: Option<Extension<rullst::security::CspNonce>>,
    Form(payload): Form<ResetPasswordDto>,
) -> Response {
    let csrf_token = get_csrf_token(&headers);
    let nonce = get_csp_nonce(&csp_nonce);
    if !password_reset::valid_code_shape(&payload.code) {
        return sensitive_response(auth::reset_password_page(
            &csrf_token,
            Some("This reset link is invalid or has expired."),
            false,
            None,
            nonce,
        ));
    }
    if payload.password != payload.password_confirmation {
        return sensitive_response(auth::reset_password_page(
            &csrf_token,
            Some("The two passwords do not match."),
            false,
            Some(&payload.code),
            nonce,
        ));
    }
    if !(12..=72).contains(&payload.password.len()) {
        return sensitive_response(auth::reset_password_page(
            &csrf_token,
            Some("Password must contain between 12 and 72 bytes."),
            false,
            Some(&payload.code),
            nonce,
        ));
    }

    let token_hash = password_reset::hash_code(&payload.code);
    let pool = match rullst::db::Orm::pool() {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("Password-reset database unavailable: {error}");
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    };
    let candidate = rullst::db::sqlx::query_scalar::<_, i32>(
        "SELECT user_id FROM password_reset_tokens WHERE token_hash = $1 AND used_at IS NULL AND expires_at > CURRENT_TIMESTAMP",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await;
    match candidate {
        Ok(Some(_)) => {}
        Ok(None) => {
            return sensitive_response(auth::reset_password_page(
                &csrf_token,
                Some("This reset link is invalid or has expired."),
                false,
                None,
                nonce,
            ));
        }
        Err(error) => {
            eprintln!("Password-reset lookup failed: {error}");
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    }

    let password_hash = match rullst::auth::hash_password_async(payload.password).await {
        Ok(hash) => hash,
        Err(error) => {
            eprintln!("Password-reset hashing failed: {error}");
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    };
    match consume_reset(&token_hash, &password_hash).await {
        Ok(true) => {
            let mut response = sensitive_response(auth::reset_password_page(
                &csrf_token,
                None,
                true,
                None,
                nonce,
            ));
            if let Ok(value) =
                rullst::server::HeaderValue::from_str(&rullst::auth::make_logout_cookie())
            {
                response
                    .headers_mut()
                    .insert(rullst::server::header::SET_COOKIE, value);
            }
            response
        }
        Ok(false) => sensitive_response(auth::reset_password_page(
            &csrf_token,
            Some("This reset link is invalid or has expired."),
            false,
            None,
            nonce,
        )),
        Err(error) => {
            eprintln!("Password-reset update failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

async fn consume_reset(token_hash: &str, password_hash: &str) -> Result<bool, String> {
    let mut transaction = rullst::db::Orm::begin_transaction()
        .await
        .map_err(|error| error.to_string())?;
    let locked = rullst::db::sqlx::query_as::<_, (i64, i32)>(
        r#"SELECT id, user_id
           FROM password_reset_tokens
           WHERE token_hash = $1
             AND used_at IS NULL
             AND expires_at > CURRENT_TIMESTAMP
           FOR UPDATE"#,
    )
    .bind(token_hash)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;
    let Some((_, user_id)) = locked else {
        transaction
            .rollback()
            .await
            .map_err(|error| error.to_string())?;
        return Ok(false);
    };

    rullst::db::sqlx::query(
        "UPDATE users SET password_hash = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
    )
    .bind(password_hash)
    .bind(user_id)
    .execute(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;
    rullst::db::sqlx::query(
        "UPDATE password_reset_tokens SET used_at = CURRENT_TIMESTAMP WHERE user_id = $1 AND used_at IS NULL",
    )
    .bind(user_id)
    .execute(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;
    rullst::db::sqlx::query(
        r#"UPDATE password_reset_mail_outbox AS outbox
           SET status = 'cancelled', locked_at = NULL
           FROM password_reset_tokens AS token
           WHERE outbox.reset_token_id = token.id
             AND token.user_id = $1
             AND outbox.status IN ('pending', 'processing')"#,
    )
    .bind(user_id)
    .execute(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;
    rullst::db::sqlx::query("DELETE FROM auth_sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| error.to_string())?;
    transaction
        .commit()
        .await
        .map_err(|error| error.to_string())?;
    Ok(true)
}
