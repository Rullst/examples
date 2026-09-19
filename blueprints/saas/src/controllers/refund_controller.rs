use crate::pages::refund;
use rullst::db::{FromRow, Orm, sqlx};
use rullst::server::{Extension, Form, IntoResponse, Response, StatusCode};
use serde::Deserialize;

#[derive(Debug, FromRow)]
struct RefundableEntitlement {
    id: i32,
    provider: String,
    provider_payment_id: String,
    eligible: bool,
}

#[derive(Debug, Deserialize)]
pub struct RefundRequestForm {
    confirmation: String,
}

pub async fn refund_request_status(user_id: i32) -> Result<Option<String>, rullst_orm::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT rr.status FROM refund_requests rr INNER JOIN entitlements e ON e.id = rr.entitlement_id WHERE rr.user_id = $1 AND e.product_sku = 'gateway-report-stripe' ORDER BY rr.id DESC LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(Orm::pool()?)
    .await
    .map_err(Into::into)
}

async fn refundable_entitlement(
    user_id: i32,
) -> Result<Option<RefundableEntitlement>, rullst_orm::Error> {
    let days = i32::from(crate::controllers::legal_controller::refund_window_days());
    sqlx::query_as::<_, RefundableEntitlement>(
        "SELECT id, provider, provider_payment_id, created_at::timestamp >= CURRENT_TIMESTAMP - make_interval(days => $2) AS eligible FROM entitlements WHERE user_id = $1 AND product_sku = 'gateway-report-stripe' AND status = 'active' LIMIT 1",
    )
    .bind(user_id)
    .bind(days)
    .fetch_optional(Orm::pool()?)
    .await
    .map_err(Into::into)
}

pub async fn refund_view(
    Extension(user_id): Extension<i32>,
    csrf: Option<Extension<rullst::security::CsrfToken>>,
    csp_nonce: Option<Extension<rullst::security::CspNonce>>,
) -> Response {
    if !crate::controllers::legal_controller::live_mode() {
        return StatusCode::NOT_FOUND.into_response();
    }
    let nonce = csp_nonce
        .as_ref()
        .map(|Extension(nonce)| nonce.as_str())
        .unwrap_or_default();
    let csrf_token = csrf
        .as_ref()
        .map(|Extension(token)| token.as_str())
        .unwrap_or_default();
    match (
        refundable_entitlement(user_id).await,
        refund_request_status(user_id).await,
    ) {
        (Ok(Some(entitlement)), Ok(status)) => refund::refund_page(
            entitlement.eligible,
            status.as_deref(),
            crate::controllers::legal_controller::refund_window_days(),
            csrf_token,
            nonce,
        )
        .into_response(),
        (Ok(None), _) => StatusCode::NOT_FOUND.into_response(),
        _ => refund::refund_unavailable_page(nonce).into_response(),
    }
}

pub async fn refund_submit(
    Extension(user_id): Extension<i32>,
    Form(form): Form<RefundRequestForm>,
) -> Response {
    if !crate::controllers::legal_controller::live_mode() {
        return StatusCode::NOT_FOUND.into_response();
    }
    if form.confirmation != "request_full_refund" {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let entitlement = match refundable_entitlement(user_id).await {
        Ok(Some(entitlement)) if entitlement.eligible => entitlement,
        Ok(Some(_)) => return StatusCode::CONFLICT.into_response(),
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            eprintln!("Refund entitlement lookup failed: {error}");
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    };
    match sqlx::query(
        "INSERT INTO refund_requests (user_id, entitlement_id, provider, provider_payment_id, status) VALUES ($1, $2, $3, $4, 'requested') ON CONFLICT (entitlement_id) WHERE status IN ('requested', 'processing') DO NOTHING",
    )
    .bind(user_id)
    .bind(entitlement.id)
    .bind(entitlement.provider)
    .bind(entitlement.provider_payment_id)
    .execute(match Orm::pool() {
        Ok(pool) => pool,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    })
    .await
    {
        Ok(_) => rullst::server::Redirect::to("/refund").into_response(),
        Err(error) => {
            eprintln!("Refund request creation failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}
