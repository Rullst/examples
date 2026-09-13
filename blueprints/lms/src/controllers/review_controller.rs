use crate::services::score_service::{ScoreError, due_reviews};
use rullst::server::{Extension, IntoResponse, Json, Query, Response, StatusCode};
use rullst_security::UserContext;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DueReviewQuery {
    pub limit: Option<u32>,
}

pub async fn index(
    Extension(user_id): Extension<i32>,
    Extension(context): Extension<UserContext>,
    Query(query): Query<DueReviewQuery>,
) -> Response {
    match due_reviews(&context, user_id, query.limit.unwrap_or(20)).await {
        Ok(reviews) => Json(reviews).into_response(),
        Err(ScoreError::Database(error)) => {
            eprintln!("Due review query failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
        Err(ScoreError::Cache(error)) => {
            eprintln!("Due review cache failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
        Err(ScoreError::Forbidden | ScoreError::InvalidIdentity) => {
            StatusCode::FORBIDDEN.into_response()
        }
        Err(ScoreError::InvalidField(_)) => StatusCode::UNPROCESSABLE_ENTITY.into_response(),
        Err(ScoreError::UnsupportedSchemaVersion(_)) => StatusCode::CONFLICT.into_response(),
    }
}
