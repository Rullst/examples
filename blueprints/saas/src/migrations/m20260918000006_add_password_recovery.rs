use rullst::db::schema::Migration;
use rullst::db::{Orm, async_trait, sqlx};

pub struct MigrationImpl;

#[async_trait]
impl Migration for MigrationImpl {
    fn name(&self) -> &'static str {
        "m20260918000006_add_password_recovery"
    }

    async fn up(&self) -> Result<(), rullst_orm::error::RullstError> {
        let pool = Orm::pool()?;
        sqlx::query(
            r#"CREATE TABLE auth_sessions (
                session_hash CHAR(64) PRIMARY KEY,
                user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                expires_at TIMESTAMPTZ NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                CONSTRAINT auth_sessions_hash_length CHECK (char_length(session_hash) = 64)
            )"#,
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "CREATE INDEX auth_sessions_user_expiry_idx ON auth_sessions(user_id, expires_at)",
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"CREATE TABLE password_reset_tokens (
                id BIGSERIAL PRIMARY KEY,
                user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                token_hash CHAR(64) NOT NULL UNIQUE,
                selector CHAR(64) NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                used_at TIMESTAMPTZ NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                CONSTRAINT password_reset_token_hash_length CHECK (char_length(token_hash) = 64),
                CONSTRAINT password_reset_selector_length CHECK (char_length(selector) = 64)
            )"#,
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "CREATE INDEX password_reset_tokens_user_state_idx ON password_reset_tokens(user_id, expires_at, used_at)",
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"CREATE TABLE password_reset_attempts (
                id BIGSERIAL PRIMARY KEY,
                request_key CHAR(64) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                CONSTRAINT password_reset_request_key_length CHECK (char_length(request_key) = 64)
            )"#,
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "CREATE INDEX password_reset_attempts_key_time_idx ON password_reset_attempts(request_key, created_at)",
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"CREATE TABLE password_reset_mail_outbox (
                id BIGSERIAL PRIMARY KEY,
                reset_token_id BIGINT NOT NULL UNIQUE REFERENCES password_reset_tokens(id) ON DELETE CASCADE,
                status VARCHAR(16) NOT NULL DEFAULT 'pending',
                attempts INTEGER NOT NULL DEFAULT 0,
                available_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                locked_at TIMESTAMPTZ NULL,
                sent_at TIMESTAMPTZ NULL,
                last_error_class VARCHAR(32) NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                CONSTRAINT password_reset_mail_status_check CHECK (status IN ('pending', 'processing', 'sent', 'failed', 'cancelled')),
                CONSTRAINT password_reset_mail_attempts_check CHECK (attempts BETWEEN 0 AND 5)
            )"#,
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "CREATE INDEX password_reset_mail_ready_idx ON password_reset_mail_outbox(status, available_at)",
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn down(&self) -> Result<(), rullst_orm::error::RullstError> {
        let pool = Orm::pool()?;
        sqlx::query("DROP TABLE IF EXISTS password_reset_mail_outbox")
            .execute(pool)
            .await?;
        sqlx::query("DROP TABLE IF EXISTS password_reset_attempts")
            .execute(pool)
            .await?;
        sqlx::query("DROP TABLE IF EXISTS password_reset_tokens")
            .execute(pool)
            .await?;
        sqlx::query("DROP TABLE IF EXISTS auth_sessions")
            .execute(pool)
            .await?;
        Ok(())
    }
}
