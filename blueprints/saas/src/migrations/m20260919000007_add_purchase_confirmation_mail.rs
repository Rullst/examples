use rullst::db::async_trait;
use rullst::db::schema::{Migration, Schema};
use rullst::db::{Orm, sqlx};

pub struct MigrationImpl;

#[async_trait]
impl Migration for MigrationImpl {
    fn name(&self) -> &'static str {
        "m20260919000007_add_purchase_confirmation_mail"
    }

    async fn up(&self) -> Result<(), rullst_orm::error::RullstError> {
        let pool = Orm::pool()?;
        sqlx::query(
            r#"CREATE TABLE purchase_confirmation_mail_outbox (
                id BIGSERIAL PRIMARY KEY,
                entitlement_id INTEGER NOT NULL REFERENCES entitlements(id) ON DELETE CASCADE,
                status VARCHAR(32) NOT NULL DEFAULT 'pending',
                attempts INTEGER NOT NULL DEFAULT 0,
                available_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                locked_at TIMESTAMP NULL,
                sent_at TIMESTAMP NULL,
                last_error_class VARCHAR(64) NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                CONSTRAINT purchase_confirmation_mail_status_check
                    CHECK (status IN ('pending', 'processing', 'sent', 'failed', 'cancelled')),
                CONSTRAINT purchase_confirmation_mail_entitlement_unique UNIQUE (entitlement_id)
            )"#,
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "CREATE INDEX purchase_confirmation_mail_pending_idx ON purchase_confirmation_mail_outbox(status, available_at, id)",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            r#"INSERT INTO purchase_confirmation_mail_outbox (entitlement_id)
               SELECT entitlement.id
               FROM entitlements AS entitlement
               INNER JOIN tester_certificates AS certificate
                   ON certificate.entitlement_id = entitlement.id
               WHERE entitlement.status = 'active'
                 AND certificate.status = 'active'
               ON CONFLICT (entitlement_id) DO NOTHING"#,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn down(&self) -> Result<(), rullst_orm::error::RullstError> {
        Schema::drop_if_exists("purchase_confirmation_mail_outbox").await
    }
}
