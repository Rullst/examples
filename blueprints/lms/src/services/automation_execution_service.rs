use crate::services::automation_service::{
    AutomationError, AutomationPlan, AutomationRuleInput, PlannedAction,
    plan_score_automations,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationExecutionReceipt {
    pub execution_key: String,
    pub execution_recorded: bool,
    pub action_applied: bool,
}

#[derive(Debug)]
pub enum AutomationExecutionError {
    InvalidClaim,
    ClaimNotHeld,
    InvalidPlan,
    AchievementNotFound,
    Plan(AutomationError),
    Database(rullst_orm::Error),
}

impl std::fmt::Display for AutomationExecutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidClaim => formatter.write_str("invalid automation claim"),
            Self::ClaimNotHeld => formatter.write_str("automation source event is not held by this claim"),
            Self::InvalidPlan => formatter.write_str("automation plan does not match current durable state"),
            Self::AchievementNotFound => formatter.write_str("automation achievement is unavailable"),
            Self::Plan(error) => write!(formatter, "automation planning failed: {error}"),
            Self::Database(error) => write!(formatter, "automation database error: {error}"),
        }
    }
}

impl std::error::Error for AutomationExecutionError {}

impl From<rullst_orm::Error> for AutomationExecutionError {
    fn from(error: rullst_orm::Error) -> Self {
        Self::Database(error)
    }
}

/// Applies only a plan rederived from the exact claimed outbox event and the
/// currently enabled durable rule. Execution and award are one transaction.
pub async fn apply_claimed_plan(
    plan: &AutomationPlan,
    claim_key: &str,
) -> Result<AutomationExecutionReceipt, AutomationExecutionError> {
    if !valid_key(claim_key, 128) {
        return Err(AutomationExecutionError::InvalidClaim);
    }
    let pool = rullst::db::Orm::pool()?;
    let driver = rullst::db::Orm::driver()?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| AutomationExecutionError::Database(error.into()))?;

    let event_sql = match driver {
        "postgres" => "SELECT school_id, event_kind, payload_json FROM academy_outbox WHERE event_key = $1 AND status = $2 AND claim_key = $3",
        _ => "SELECT school_id, event_kind, payload_json FROM academy_outbox WHERE event_key = ? AND status = ? AND claim_key = ?",
    };
    let event = rullst::db::sqlx::query_as::<_, (i32, String, String)>(event_sql)
        .bind(&plan.source_event_key)
        .bind("processing")
        .bind(claim_key)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| AutomationExecutionError::Database(error.into()))?;
    let Some((school_id, event_kind, payload_json)) = event else {
        return Err(AutomationExecutionError::ClaimNotHeld);
    };
    if school_id <= 0 { return Err(AutomationExecutionError::InvalidClaim); }

    let rule_sql = match driver {
        "postgres" => "SELECT id, school_id, enabled, trigger_kind, action_kind, config_json FROM automation_rules WHERE id = $1",
        _ => "SELECT id, school_id, enabled, trigger_kind, action_kind, config_json FROM automation_rules WHERE id = ?",
    };
    let rule = rullst::db::sqlx::query_as::<_, (i32, i32, i32, String, String, String)>(rule_sql)
        .bind(plan.rule_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| AutomationExecutionError::Database(error.into()))?;
    let Some((id, rule_school_id, enabled, trigger_kind, action_kind, config_json)) = rule else {
        return Err(AutomationExecutionError::InvalidPlan);
    };
    if rule_school_id != school_id { return Err(AutomationExecutionError::InvalidPlan); }
    let expected = plan_score_automations(
        &plan.source_event_key,
        &event_kind,
        &payload_json,
        &[AutomationRuleInput {
            id,
            enabled: enabled == 1,
            trigger_kind,
            action_kind,
            config_json,
        }],
    )
    .map_err(AutomationExecutionError::Plan)?;
    if expected.as_slice() != std::slice::from_ref(plan) {
        return Err(AutomationExecutionError::InvalidPlan);
    }

    let (subject_user_id, achievement_code) = match &plan.action {
        PlannedAction::AwardAchievement {
            subject_user_id,
            achievement_code,
        } => (*subject_user_id, achievement_code),
    };
    let insert_execution_sql = match driver {
        "postgres" => "INSERT INTO automation_executions (school_id, execution_key, rule_id, source_event_key, actor_user_id, subject_user_id, action_kind, outcome, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) ON CONFLICT DO NOTHING",
        "mysql" => "INSERT IGNORE INTO automation_executions (school_id, execution_key, rule_id, source_event_key, actor_user_id, subject_user_id, action_kind, outcome, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        _ => "INSERT INTO automation_executions (school_id, execution_key, rule_id, source_event_key, actor_user_id, subject_user_id, action_kind, outcome, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) ON CONFLICT DO NOTHING",
    };
    let inserted = rullst::db::sqlx::query(insert_execution_sql)
        .bind(school_id)
        .bind(&plan.execution_key)
        .bind(plan.rule_id)
        .bind(&plan.source_event_key)
        .bind(plan.actor_user_id)
        .bind(subject_user_id)
        .bind("award_achievement")
        .bind("started")
        .execute(&mut *transaction)
        .await
        .map_err(|error| AutomationExecutionError::Database(error.into()))?
        .rows_affected()
        == 1;
    if !inserted {
        transaction
            .commit()
            .await
            .map_err(|error| AutomationExecutionError::Database(error.into()))?;
        return Ok(AutomationExecutionReceipt {
            execution_key: plan.execution_key.clone(),
            execution_recorded: false,
            action_applied: false,
        });
    }

    let achievement_sql = match driver {
        "postgres" => "SELECT id FROM achievements WHERE code = $1 AND enabled = $2",
        _ => "SELECT id FROM achievements WHERE code = ? AND enabled = ?",
    };
    let achievement_id = rullst::db::sqlx::query_scalar::<_, i32>(achievement_sql)
        .bind(achievement_code)
        .bind(1_i32)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| AutomationExecutionError::Database(error.into()))?
        .ok_or(AutomationExecutionError::AchievementNotFound)?;
    let award_sql = match driver {
        "postgres" => "INSERT INTO user_achievements (school_id, user_id, achievement_id, source_event_key, awarded_by_user_id, awarded_at, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) ON CONFLICT DO NOTHING",
        "mysql" => "INSERT IGNORE INTO user_achievements (school_id, user_id, achievement_id, source_event_key, awarded_by_user_id, awarded_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        _ => "INSERT INTO user_achievements (school_id, user_id, achievement_id, source_event_key, awarded_by_user_id, awarded_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) ON CONFLICT DO NOTHING",
    };
    let action_applied = rullst::db::sqlx::query(award_sql)
        .bind(school_id)
        .bind(subject_user_id)
        .bind(achievement_id)
        .bind(&plan.source_event_key)
        .bind(plan.actor_user_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| AutomationExecutionError::Database(error.into()))?
        .rows_affected()
        == 1;
    if action_applied {
        let achievement_event_key = format!("achievement:{}", plan.execution_key);
        let achievement_payload = serde_json::json!({
            "schema_version": 1,
            "actor_user_id": plan.actor_user_id,
            "subject_user_id": subject_user_id,
            "achievement_code": achievement_code,
            "execution_key": &plan.execution_key,
        })
        .to_string();
        let outbox_sql = match driver {
            "postgres" => "INSERT INTO academy_outbox (school_id, event_key, event_kind, subject_user_id, payload_json, status, attempts, claimed_by, claim_key, last_error, available_at, available_at_epoch, claim_expires_at_epoch, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, 0, $7, $8, $9, CURRENT_TIMESTAMP, 0, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            _ => "INSERT INTO academy_outbox (school_id, event_key, event_kind, subject_user_id, payload_json, status, attempts, claimed_by, claim_key, last_error, available_at, available_at_epoch, claim_expires_at_epoch, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, 0, ?, ?, ?, CURRENT_TIMESTAMP, 0, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        };
        rullst::db::sqlx::query(outbox_sql)
            .bind(school_id)
            .bind(achievement_event_key)
            .bind("achievement_awarded")
            .bind(subject_user_id)
            .bind(achievement_payload)
            .bind("pending")
            .bind("")
            .bind("")
            .bind("")
            .execute(&mut *transaction)
            .await
            .map_err(|error| AutomationExecutionError::Database(error.into()))?;
    }
    let outcome = if action_applied { "applied" } else { "already_awarded" };
    let outcome_sql = match driver {
        "postgres" => "UPDATE automation_executions SET outcome = $1, updated_at = CURRENT_TIMESTAMP WHERE execution_key = $2",
        _ => "UPDATE automation_executions SET outcome = ?, updated_at = CURRENT_TIMESTAMP WHERE execution_key = ?",
    };
    rullst::db::sqlx::query(outcome_sql)
        .bind(outcome)
        .bind(&plan.execution_key)
        .execute(&mut *transaction)
        .await
        .map_err(|error| AutomationExecutionError::Database(error.into()))?;
    transaction
        .commit()
        .await
        .map_err(|error| AutomationExecutionError::Database(error.into()))?;
    Ok(AutomationExecutionReceipt {
        execution_key: plan.execution_key.clone(),
        execution_recorded: true,
        action_applied,
    })
}

fn valid_key(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':' | b'.'))
}
