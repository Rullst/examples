use crate::models::lesson_progress::LessonProgress;
use crate::pages::lms;
use crate::services::learning_service::{self, LearningError};
use crate::services::progress_service::{self, ProgressError};
use rullst::server::{Extension, Form, IntoResponse, Path, Redirect, Response, StatusCode};
use rullst_security::UserContext;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ProgressDto {
    pub progress_percent: i32,
    pub idempotency_key: String,
}

fn learning_error_response(error: LearningError) -> Response {
    let status = match &error {
        LearningError::NotFound(_) => StatusCode::NOT_FOUND,
        LearningError::Forbidden => StatusCode::FORBIDDEN,
        LearningError::NotReleased | LearningError::PrerequisiteNotMet => StatusCode::FORBIDDEN,
        LearningError::Expired => StatusCode::GONE,
        LearningError::InvalidAvailabilityPolicy => StatusCode::SERVICE_UNAVAILABLE,
        LearningError::InvalidContentVersion => StatusCode::SERVICE_UNAVAILABLE,
        LearningError::InvalidProgress => StatusCode::UNPROCESSABLE_ENTITY,
        LearningError::Clock => StatusCode::INTERNAL_SERVER_ERROR,
        LearningError::Database(_) => {
            eprintln!("Learning operation failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE
        }
    };
    (status, status.canonical_reason().unwrap_or("Learning request failed")).into_response()
}

fn progress_error_response(error: ProgressError) -> Response {
    match error {
        ProgressError::Access(error) => learning_error_response(error),
        ProgressError::InvalidField(_) => StatusCode::UNPROCESSABLE_ENTITY.into_response(),
        ProgressError::Database(error) => {
            eprintln!("Progress operation failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

pub async fn enroll(
    Path(course_id): Path<i32>,
    Extension(user_id): Extension<i32>,
    Extension(context): Extension<UserContext>,
) -> Response {
    match learning_service::enroll(user_id, &context, course_id).await {
        Ok(_) => Redirect::to(&format!("/courses/{course_id}")).into_response(),
        Err(error) => learning_error_response(error),
    }
}

pub async fn play_lesson(
    Path(lesson_id): Path<i32>,
    Extension(user_id): Extension<i32>,
    Extension(context): Extension<UserContext>,
    Extension(csrf): Extension<rullst::security::CsrfToken>,
    Extension(csp_nonce): Extension<rullst::security::CspNonce>,
) -> Response {
    let lesson = match learning_service::authorize_lesson(user_id, &context, lesson_id).await {
        Ok(lesson) => lesson,
        Err(error) => return learning_error_response(error),
    };
    let progress = match LessonProgress::for_learner(user_id, lesson_id).await {
        Ok(progress) => progress.map_or(0, |value| value.progress_percent),
        Err(error) => return learning_error_response(LearningError::Database(error)),
    };
    let progress_key = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => format!("progress-{}", elapsed.as_nanos()),
        Err(error) => {
            eprintln!("Progress clock unavailable: {error}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    match lms::lesson_player_page(
        &lesson.title,
        &lesson.media_kind,
        &lesson.media_url,
        &lesson.captions_url,
        &lesson.transcript,
        &lesson.language_tag,
        lesson.course_id,
        lesson.id,
        progress,
        csrf.as_str(),
        &progress_key,
        csp_nonce.as_str(),
    ) {
        Ok(page) => rullst::response::Html(page).into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

pub async fn record_progress(
    Path(lesson_id): Path<i32>,
    Extension(user_id): Extension<i32>,
    Extension(context): Extension<UserContext>,
    Form(payload): Form<ProgressDto>,
) -> Response {
    match progress_service::record_progress(
        &context,
        user_id,
        lesson_id,
        payload.progress_percent,
        &payload.idempotency_key,
    )
    .await
    {
        Ok(_) => Redirect::to(&format!("/lessons/{lesson_id}/play")).into_response(),
        Err(error) => progress_error_response(error),
    }
}
