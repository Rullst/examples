use crate::services::publication_service::{
    PublicationError, create_draft, review_version_at, submit_for_review,
};
use rullst::server::{Extension, IntoResponse, Json, Path, Response, StatusCode};
use rullst_security::UserContext;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DraftPayload {
    pub version_key: String,
    pub content: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ReviewPayload {
    pub activate_at_epoch: i64,
}

fn error_response(error: PublicationError) -> Response {
    match error {
        PublicationError::Forbidden => StatusCode::FORBIDDEN.into_response(),
        PublicationError::NotFound => StatusCode::NOT_FOUND.into_response(),
        PublicationError::InvalidField(_) => StatusCode::UNPROCESSABLE_ENTITY.into_response(),
        PublicationError::InvalidState | PublicationError::SeparationOfDuties => {
            StatusCode::CONFLICT.into_response()
        }
        PublicationError::Database(error) => {
            eprintln!("Course publication operation failed: {error}");
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

pub async fn draft(
    Path(course_id): Path<i32>,
    Extension(context): Extension<UserContext>,
    Json(payload): Json<DraftPayload>,
) -> Response {
    let content_json = match serde_json::to_string(&payload.content) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("Course publication payload serialization failed: {error}");
            return StatusCode::UNPROCESSABLE_ENTITY.into_response();
        }
    };
    match create_draft(&context, course_id, &payload.version_key, &content_json).await {
        Ok(receipt) => (StatusCode::CREATED, Json(receipt)).into_response(),
        Err(error) => error_response(error),
    }
}

pub async fn submit(
    Path(version_id): Path<i32>,
    Extension(context): Extension<UserContext>,
) -> Response {
    match submit_for_review(&context, version_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::CONFLICT.into_response(),
        Err(error) => error_response(error),
    }
}

pub async fn review(
    Path(version_id): Path<i32>,
    Extension(context): Extension<UserContext>,
    Json(payload): Json<ReviewPayload>,
) -> Response {
    let now_epoch = match unix_now() {
        Ok(value) => value,
        Err(response) => return response,
    };
    match review_version_at(&context, version_id, payload.activate_at_epoch, now_epoch).await {
        Ok(receipt) => Json(receipt).into_response(),
        Err(error) => error_response(error),
    }
}
