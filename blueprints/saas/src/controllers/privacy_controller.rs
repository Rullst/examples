use crate::models::entitlement::Entitlement;
use crate::models::purchase_attempt::PurchaseAttempt;
use crate::models::refund_request::RefundRequest;
use crate::models::tester_certificate::TesterCertificate;
use crate::models::user::User;
use rullst::db::{Orm, sqlx};
use rullst::server::{Extension, IntoResponse, Response, StatusCode};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn export_account_data(Extension(user_id): Extension<i32>) -> Response {
    let pool = match Orm::pool() {
        Ok(pool) => pool,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let user = match User::find(user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let attempts = match sqlx::query_as::<_, PurchaseAttempt>(
        "SELECT * FROM purchase_attempts WHERE user_id = $1 ORDER BY id ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    {
        Ok(value) => value,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let entitlements = match sqlx::query_as::<_, Entitlement>(
        "SELECT * FROM entitlements WHERE user_id = $1 ORDER BY id ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    {
        Ok(value) => value,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let refund_requests = match sqlx::query_as::<_, RefundRequest>(
        "SELECT * FROM refund_requests WHERE user_id = $1 ORDER BY id ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    {
        Ok(value) => value,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let certificates = match sqlx::query_as::<_, TesterCertificate>(
        "SELECT c.* FROM tester_certificates c INNER JOIN entitlements e ON e.id = c.entitlement_id WHERE e.user_id = $1 ORDER BY c.id ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    {
        Ok(value) => value,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let purchase_confirmation_delivery =
        match sqlx::query_as::<_, (i64, String, i32, Option<String>, String, String)>(
            r#"SELECT outbox.id, outbox.status, outbox.attempts,
                  outbox.sent_at::text, outbox.created_at::text, outbox.updated_at::text
           FROM purchase_confirmation_mail_outbox AS outbox
           INNER JOIN entitlements AS entitlement ON entitlement.id = outbox.entitlement_id
           WHERE entitlement.user_id = $1
           ORDER BY outbox.id ASC"#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        {
            Ok(value) => value,
            Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
        };

    let exported_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    let payload = json!({
        "export_version": 1,
        "exported_at_unix": exported_at_unix,
        "account": {
            "id": user.id,
            "name": user.name,
            "email": user.email,
            "oauth_provider": user.oauth_provider,
            "oauth_id": user.oauth_id,
            "created_at": user.created_at,
            "updated_at": user.updated_at
        },
        "purchase_attempts": attempts.into_iter().map(|item| json!({
            "id": item.id,
            "provider": item.provider,
            "product_sku": item.product_sku,
            "provider_price_id": item.provider_price_id,
            "expected_amount_minor": item.expected_amount_minor,
            "expected_currency": item.expected_currency,
            "provider_session_id": item.provider_session_id,
            "provider_payment_id": item.provider_payment_id,
            "status": item.status,
            "created_at": item.created_at,
            "updated_at": item.updated_at
        })).collect::<Vec<_>>(),
        "entitlements": entitlements.into_iter().map(|item| json!({
            "id": item.id,
            "product_sku": item.product_sku,
            "provider": item.provider,
            "provider_payment_id": item.provider_payment_id,
            "artifact_version": item.artifact_version,
            "status": item.status,
            "revoked_reason": item.revoked_reason,
            "revoked_at": item.revoked_at,
            "created_at": item.created_at,
            "updated_at": item.updated_at
        })).collect::<Vec<_>>(),
        "refund_requests": refund_requests.into_iter().map(|item| json!({
            "id": item.id,
            "entitlement_id": item.entitlement_id,
            "provider": item.provider,
            "provider_payment_id": item.provider_payment_id,
            "status": item.status,
            "created_at": item.created_at,
            "updated_at": item.updated_at
        })).collect::<Vec<_>>(),
        "certificates": certificates.into_iter().map(|item| json!({
            "id": item.id,
            "entitlement_id": item.entitlement_id,
            "public_id": item.public_id,
            "badge_kind": item.badge_kind,
            "environment": item.environment,
            "status": item.status,
            "created_at": item.created_at,
            "updated_at": item.updated_at
        })).collect::<Vec<_>>(),
        "purchase_confirmation_delivery": purchase_confirmation_delivery.into_iter().map(|item| json!({
            "id": item.0,
            "status": item.1,
            "attempts": item.2,
            "sent_at": item.3,
            "created_at": item.4,
            "updated_at": item.5
        })).collect::<Vec<_>>()
    });
    let body = match serde_json::to_string_pretty(&payload) {
        Ok(body) => body,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let mut response = (StatusCode::OK, body).into_response();
    response.headers_mut().insert(
        rullst::server::header::CONTENT_TYPE,
        rullst::server::HeaderValue::from_static("application/json; charset=utf-8"),
    );
    response.headers_mut().insert(
        rullst::server::header::CONTENT_DISPOSITION,
        rullst::server::HeaderValue::from_static(
            "attachment; filename=\"rullst-account-data.json\"",
        ),
    );
    response.headers_mut().insert(
        rullst::server::header::CACHE_CONTROL,
        rullst::server::HeaderValue::from_static("private, no-store"),
    );
    response
}
