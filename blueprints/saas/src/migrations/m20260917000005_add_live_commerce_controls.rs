use rullst::db::async_trait;
use rullst::db::schema::{Migration, Schema};
use rullst::db::{Orm, sqlx};

pub struct MigrationImpl;

#[async_trait]
impl Migration for MigrationImpl {
    fn name(&self) -> &'static str {
        "m20260917000005_add_live_commerce_controls"
    }

    async fn up(&self) -> Result<(), rullst_orm::error::RullstError> {
        let pool = Orm::pool()?;
        sqlx::query("ALTER TABLE entitlements ADD COLUMN revoked_reason VARCHAR(255) NULL")
            .execute(pool)
            .await?;
        sqlx::query("ALTER TABLE entitlements ADD COLUMN revoked_at TIMESTAMP NULL")
            .execute(pool)
            .await?;
        sqlx::query("ALTER TABLE entitlements ADD COLUMN last_reconciled_at TIMESTAMP NULL")
            .execute(pool)
            .await?;
        sqlx::query("ALTER TABLE purchase_attempts ADD COLUMN last_reconciled_at TIMESTAMP NULL")
            .execute(pool)
            .await?;
        sqlx::query(
            "ALTER TABLE tester_certificates DROP CONSTRAINT tester_certificates_badge_kind_check",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "ALTER TABLE tester_certificates ADD CONSTRAINT tester_certificates_badge_kind_check CHECK (badge_kind IN ('sandbox_pioneer', 'founding_customer'))",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "ALTER TABLE tester_certificates DROP CONSTRAINT tester_certificates_environment_check",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "ALTER TABLE tester_certificates ADD CONSTRAINT tester_certificates_environment_check CHECK (environment IN ('test', 'live'))",
        )
        .execute(pool)
        .await?;

        Schema::create("refund_requests", |table| {
            table.id();
            table.integer("user_id").not_null();
            table.integer("entitlement_id").not_null();
            table.string("provider").not_null();
            table.string("provider_payment_id").not_null();
            table.string("status").not_null();
            table.timestamps();
        })
        .await?;
        sqlx::query(
            "ALTER TABLE refund_requests ADD CONSTRAINT refund_requests_user_fk FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "ALTER TABLE refund_requests ADD CONSTRAINT refund_requests_entitlement_fk FOREIGN KEY (entitlement_id) REFERENCES entitlements(id) ON DELETE CASCADE",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "ALTER TABLE refund_requests ADD CONSTRAINT refund_requests_status_check CHECK (status IN ('requested', 'processing', 'completed', 'declined', 'cancelled'))",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "CREATE UNIQUE INDEX refund_requests_open_entitlement_unique ON refund_requests(entitlement_id) WHERE status IN ('requested', 'processing')",
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn down(&self) -> Result<(), rullst_orm::error::RullstError> {
        Schema::drop_if_exists("refund_requests").await?;
        let pool = Orm::pool()?;
        sqlx::query(
            "ALTER TABLE tester_certificates DROP CONSTRAINT tester_certificates_badge_kind_check",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "ALTER TABLE tester_certificates ADD CONSTRAINT tester_certificates_badge_kind_check CHECK (badge_kind IN ('sandbox_pioneer'))",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "ALTER TABLE tester_certificates DROP CONSTRAINT tester_certificates_environment_check",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "ALTER TABLE tester_certificates ADD CONSTRAINT tester_certificates_environment_check CHECK (environment IN ('test'))",
        )
        .execute(pool)
        .await?;
        sqlx::query("ALTER TABLE entitlements DROP COLUMN revoked_at")
            .execute(pool)
            .await?;
        sqlx::query("ALTER TABLE entitlements DROP COLUMN revoked_reason")
            .execute(pool)
            .await?;
        sqlx::query("ALTER TABLE entitlements DROP COLUMN last_reconciled_at")
            .execute(pool)
            .await?;
        sqlx::query("ALTER TABLE purchase_attempts DROP COLUMN last_reconciled_at")
            .execute(pool)
            .await?;
        Ok(())
    }
}
