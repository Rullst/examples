use super::activity_controller::error_response;
use crate::services::activity_contract::{MatchingPair, MatchingRequest, submit_matching};
use rullst::server::{Extension, IntoResponse, Json, Path, Response};
use rullst_security::UserContext;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchingPairPayload {
    pub left_id: i32,
    pub right_id: i32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchingPayload {
    pub attempt_key: String,
    pub pairs: Vec<MatchingPairPayload>,
}

pub async fn submit(
    Path(activity_id): Path<i32>,
    Extension(user_id): Extension<i32>,
    Extension(context): Extension<UserContext>,
    Json(payload): Json<MatchingPayload>,
) -> Response {
    let request = MatchingRequest {
        attempt_key: payload.attempt_key,
        pairs: payload
            .pairs
            .into_iter()
            .map(|pair| MatchingPair {
                left_id: pair.left_id,
                right_id: pair.right_id,
            })
            .collect(),
    };
    match submit_matching(&context, user_id, activity_id, request).await {
        Ok(receipt) => Json(receipt).into_response(),
        Err(error) => error_response(error),
    }
}
