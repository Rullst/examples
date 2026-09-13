use super::activity_controller::error_response;
use crate::services::activity_contract::{TypedAnswerRequest, submit_typed_answer};
use rullst::server::{Extension, IntoResponse, Json, Path, Response};
use rullst_security::UserContext;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedAnswerPayload {
    pub attempt_key: String,
    pub answer: String,
}

pub async fn submit(
    Path(activity_id): Path<i32>,
    Extension(user_id): Extension<i32>,
    Extension(context): Extension<UserContext>,
    Json(payload): Json<TypedAnswerPayload>,
) -> Response {
    let request = TypedAnswerRequest {
        attempt_key: payload.attempt_key,
        answer: payload.answer,
    };
    match submit_typed_answer(&context, user_id, activity_id, request).await {
        Ok(receipt) => Json(receipt).into_response(),
        Err(error) => error_response(error),
    }
}
