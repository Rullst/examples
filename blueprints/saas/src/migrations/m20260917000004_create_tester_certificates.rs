use rullst::db::async_trait;
use rullst::db::schema::{Migration, Schema};
use rullst::db::{Orm, sqlx};

pub struct MigrationImpl;

#[async_trait]
impl Migration for MigrationImpl {
    fn name(&self) -> &'static str {
        "m20260917000004_create_tester_certificates"
    }

    async fn up(&self) -> Result<(), rullst_orm::error::RullstError> {
        Schema::create("tester_certificates", |table| {
            table.id();
            table.integer("entitlement_id").not_null();
            table.string("public_id").not_null();
            table.string("badge_kind").not_null();
            table.string("environment").not_null();
            table.string("status").not_null();
            table.timestamps();
        })
        .await?;

        let pool = Orm::pool()?;
        sqlx::query(
            "ALTER TABLE tester_certificates ADD CONSTRAINT tester_certificates_entitlement_fk FOREIGN KEY (entitlement_id) REFERENCES entitlements(id) ON DELETE CASCADE",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "ALTER TABLE tester_certificates ADD CONSTRAINT tester_certificates_badge_kind_check CHECK (badge_kind IN ('sandbox_pioneer'))",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "ALTER TABLE tester_certificates ADD CONSTRAINT tester_certificates_environment_check CHECK (environment IN ('test'))",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "ALTER TABLE tester_certificates ADD CONSTRAINT tester_certificates_status_check CHECK (status IN ('active', 'revoked'))",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "CREATE UNIQUE INDEX tester_certificates_entitlement_unique ON tester_certificates(entitlement_id)",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "CREATE UNIQUE INDEX tester_certificates_public_id_unique ON tester_certificates(public_id)",
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn down(&self) -> Result<(), rullst_orm::error::RullstError> {
        Schema::drop_if_exists("tester_certificates").await
    }
}
