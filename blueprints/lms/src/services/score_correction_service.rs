use crate::services::school_service;
use crate::services::score_service::{ScoreError, ScoreReceipt, invalidate_leaderboard_cache};
use rullst_security::{RbacGuard, UserContext};

fn valid_key(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':' | b'.'))
}

pub async fn correct_score(
    context: &UserContext,
    correction_key: &str,
    subject_user_id: i32,
    course_id: i32,
    season_key: &str,
    corrected_score: i32,
    reason: &str,
    ruleset_version: &str,
) -> Result<ScoreReceipt, ScoreError> {
    RbacGuard::authorize(context, "admin").map_err(|_| ScoreError::Forbidden)?;
    let actor_user_id = context
        .user_id
        .parse::<i32>()
        .map_err(|_| ScoreError::InvalidIdentity)?;
    if !valid_key(correction_key, 128)
        || !valid_key(season_key, 64)
        || !valid_key(ruleset_version, 64)
        || subject_user_id <= 0
        || course_id <= 0
        || !(0..=1_000_000).contains(&corrected_score)
        || reason.trim().is_empty()
        || reason.len() > 500
    {
        return Err(ScoreError::InvalidField("score correction"));
    }
    school_service::authorize_course(context, course_id).await
        .map_err(|error| match error {
            school_service::SchoolError::Database(error) => ScoreError::Database(error),
            _ => ScoreError::Forbidden,
        })?;
    school_service::authorize_school_membership_at(context, subject_user_id, unix_now()?).await
        .map_err(|error| match error {
            school_service::SchoolError::Database(error) => ScoreError::Database(error),
            _ => ScoreError::Forbidden,
        })?;

    let pool = rullst::db::Orm::pool()?;
    let driver = rullst::db::Orm::driver()?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| ScoreError::Database(error.into()))?;
    let select_sql = match driver {
        "postgres" => "SELECT score FROM leaderboard_entries WHERE user_id = $1 AND course_id = $2 AND season_key = $3",
        _ => "SELECT score FROM leaderboard_entries WHERE user_id = ? AND course_id = ? AND season_key = ?",
    };
    let previous_score = rullst::db::sqlx::query_scalar::<_, i32>(select_sql)
        .bind(subject_user_id)
        .bind(course_id)
        .bind(season_key)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| ScoreError::Database(error.into()))?
        .unwrap_or_default();

    let correction_sql = match driver {
        "postgres" => "INSERT INTO score_corrections (correction_key, actor_user_id, subject_user_id, course_id, season_key, previous_score, corrected_score, reason, ruleset_version, occurred_at, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) ON CONFLICT DO NOTHING",
        "mysql" => "INSERT IGNORE INTO score_corrections (correction_key, actor_user_id, subject_user_id, course_id, season_key, previous_score, corrected_score, reason, ruleset_version, occurred_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        _ => "INSERT INTO score_corrections (correction_key, actor_user_id, subject_user_id, course_id, season_key, previous_score, corrected_score, reason, ruleset_version, occurred_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) ON CONFLICT DO NOTHING",
    };
    let insertion = rullst::db::sqlx::query(correction_sql)
        .bind(correction_key)
        .bind(actor_user_id)
        .bind(subject_user_id)
        .bind(course_id)
        .bind(season_key)
        .bind(previous_score)
        .bind(corrected_score)
        .bind(reason.trim())
        .bind(ruleset_version)
        .execute(&mut *transaction)
        .await
        .map_err(|error| ScoreError::Database(error.into()))?;

    let applied = insertion.rows_affected() == 1;
    if applied {
        let leaderboard_sql = match driver {
            "postgres" => "INSERT INTO leaderboard_entries (user_id, course_id, season_key, score, created_at, updated_at) VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) ON CONFLICT (user_id, course_id, season_key) DO UPDATE SET score = EXCLUDED.score, updated_at = CURRENT_TIMESTAMP",
            "mysql" => "INSERT INTO leaderboard_entries (user_id, course_id, season_key, score, created_at, updated_at) VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) ON DUPLICATE KEY UPDATE score = VALUES(score), updated_at = CURRENT_TIMESTAMP",
            _ => "INSERT INTO leaderboard_entries (user_id, course_id, season_key, score, created_at, updated_at) VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) ON CONFLICT (user_id, course_id, season_key) DO UPDATE SET score = excluded.score, updated_at = CURRENT_TIMESTAMP",
        };
        rullst::db::sqlx::query(leaderboard_sql)
            .bind(subject_user_id)
            .bind(course_id)
            .bind(season_key)
            .bind(corrected_score)
            .execute(&mut *transaction)
            .await
            .map_err(|error| ScoreError::Database(error.into()))?;
    }

    transaction
        .commit()
        .await
        .map_err(|error| ScoreError::Database(error.into()))?;
    if applied {
        let _ = invalidate_leaderboard_cache(context, course_id, season_key).await;
    }
    Ok(ScoreReceipt {
        idempotency_key: correction_key.to_string(),
        applied,
    })
}

fn unix_now() -> Result<i64, ScoreError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| ScoreError::InvalidField("clock"))?;
    i64::try_from(elapsed.as_secs()).map_err(|_| ScoreError::InvalidField("clock"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn non_admin_correction_is_denied_before_database_access() {
        let learner = UserContext::new("7", vec!["student".to_string()]);
        let result = correct_score(
            &learner,
            "correction-1",
            7,
            1,
            "season-2026",
            100,
            "reviewed rubric",
            "rules-v1",
        )
        .await;
        assert!(matches!(result, Err(ScoreError::Forbidden)));
    }
}
