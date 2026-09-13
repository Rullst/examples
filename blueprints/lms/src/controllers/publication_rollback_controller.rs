use crate::services::publication_rollback_service::{
    PublicationRollbackError, rollback_course_at,
};
use rullst::server::{Extension, IntoResponse, Json, Path, Response, StatusCode};
use rullst_security::UserContext;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PublicationRollbackPayload {
    pub source_version_id: i32,
    pub rollback_key: String,
    pub reason: String,
}

fn error_response(error: PublicationRollbackError) -> Response {
    match error {
        PublicationRollbackError::Forbidden => StatusCode::FORBIDDEN.into_response(),
        PublicationRollbackError::NotFound => StatusCode::NOT_FOUND.into_response(),
        PublicationRollbackError::InvalidField(_) => StatusCode::UNPROCESSABLE_ENTITY.into_response(),
        PublicationRollbackError::InvalidState
        | PublicationRollbackError::SeparationOfDuties
        | PublicationRollbackError::IdempotencyConflict => StatusCode::CONFLICT.into_response(),
        PublicationRollbackError::Database(error) => {
            eprintln!("Course rollback operation failed: {error}");
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

pub async fn rollback(
    Path(course_id): Path<i32>,
    Extension(context): Extension<UserContext>,
    Json(payload): Json<PublicationRollbackPayload>,
) -> Response {
    let now_epoch = match unix_now() {
        Ok(value) => value,
        Err(response) => return response,
    };
    match rollback_course_at(
        &context, course_id, payload.source_version_id, &payload.rollback_key,
        &payload.reason, now_epoch,
    ).await {
        Ok(receipt) => Json(receipt).into_response(),
        Err(error) => error_response(error),
    }
}
