use crate::services::role_service::{
    RoleError, grant_role, revoke_role_at,
};
use rullst::server::{Extension, IntoResponse, Json, Path, Response, StatusCode};
use rullst_security::UserContext;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GrantPayload {
    pub assignment_key: String,
    pub role: String,
    pub valid_from_epoch: i64,
    pub expires_at_epoch: i64,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct RevokePayload {
    pub revocation_key: String,
    pub reason: String,
}

fn error_response(error: RoleError) -> Response {
    match error {
        RoleError::Forbidden => StatusCode::FORBIDDEN.into_response(),
        RoleError::NotFound => StatusCode::NOT_FOUND.into_response(),
        RoleError::InvalidField(_) => StatusCode::UNPROCESSABLE_ENTITY.into_response(),
        RoleError::IdempotencyConflict => StatusCode::CONFLICT.into_response(),
        RoleError::Database(error) => {
            eprintln!("Education role operation failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

fn unix_now() -> Result<i64, Response> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|elapsed| i64::try_from(elapsed.as_secs()).ok())
        .ok_or_else(|| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

pub async fn grant(
    Path(user_id): Path<i32>,
    Extension(context): Extension<UserContext>,
    Json(payload): Json<GrantPayload>,
) -> Response {
    match grant_role(
        &context,
        &payload.assignment_key,
        user_id,
        &payload.role,
        payload.valid_from_epoch,
        payload.expires_at_epoch,
        &payload.reason,
    )
    .await
    {
        Ok(receipt) if receipt.applied => {
            (StatusCode::CREATED, Json(receipt)).into_response()
        }
        Ok(receipt) => Json(receipt).into_response(),
        Err(error) => error_response(error),
    }
}

pub async fn revoke(
    Path(assignment_key): Path<String>,
    Extension(context): Extension<UserContext>,
    Json(payload): Json<RevokePayload>,
) -> Response {
    let observed_at_epoch = match unix_now() {
        Ok(value) => value,
        Err(response) => return response,
    };
    match revoke_role_at(
        &context,
        &payload.revocation_key,
        &assignment_key,
        observed_at_epoch,
        &payload.reason,
    )
    .await
    {
        Ok(receipt) => Json(receipt).into_response(),
        Err(error) => error_response(error),
    }
}
