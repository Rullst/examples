use crate::services::activity_contract::{
    ActivitySubmissionError, SingleChoiceRequest, submit_single_choice,
};
use crate::services::learning_service::LearningError;
use crate::services::score_service::ScoreError;
use rullst::server::{Extension, IntoResponse, Json, Path, Response, StatusCode};
use rullst_security::UserContext;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SingleChoicePayload {
    pub attempt_key: String,
    pub selected_option_id: i32,
}

pub(super) fn error_response(error: ActivitySubmissionError) -> Response {
    match error {
        ActivitySubmissionError::Access(LearningError::Database(error))
        | ActivitySubmissionError::Database(error)
        | ActivitySubmissionError::Score(ScoreError::Database(error)) => {
            eprintln!("Activity submission failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
        ActivitySubmissionError::Score(ScoreError::Cache(error)) => {
            eprintln!("Activity cache update failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
        ActivitySubmissionError::NotFound => StatusCode::NOT_FOUND.into_response(),
        ActivitySubmissionError::Access(_)
        | ActivitySubmissionError::Score(ScoreError::Forbidden | ScoreError::InvalidIdentity) => {
            StatusCode::FORBIDDEN.into_response()
        }
        ActivitySubmissionError::InvalidPolicy
        | ActivitySubmissionError::Score(
            ScoreError::InvalidField(_) | ScoreError::UnsupportedSchemaVersion(_),
        ) => StatusCode::CONFLICT.into_response(),
        ActivitySubmissionError::InvalidInput(_)
        | ActivitySubmissionError::Contract(_) => {
            StatusCode::UNPROCESSABLE_ENTITY.into_response()
        }
        ActivitySubmissionError::Clock => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn submit(
    Path(activity_id): Path<i32>,
    Extension(user_id): Extension<i32>,
    Extension(context): Extension<UserContext>,
    Json(payload): Json<SingleChoicePayload>,
) -> Response {
    match submit_single_choice(
        &context,
        user_id,
        activity_id,
        SingleChoiceRequest {
            attempt_key: payload.attempt_key,
            selected_option_id: payload.selected_option_id,
        },
    )
    .await
    {
        Ok(receipt) => Json(receipt).into_response(),
        Err(error) => error_response(error),
    }
}
