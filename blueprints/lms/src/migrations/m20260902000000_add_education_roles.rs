use rullst::db::{Orm, sqlx};
use rullst::db::async_trait;
use rullst::db::schema::{Migration, Schema};

pub struct MigrationImpl;

#[async_trait]
impl Migration for MigrationImpl {
    fn name(&self) -> &'static str { "m20260902000000_add_education_roles" }
    async fn up(&self) -> Result<(), rullst_orm::error::RullstError> {
        Schema::create("role_assignments", |table| {
            table.id(); table.string("assignment_key").not_null(); table.integer("school_id").not_null();
            table.integer("user_id").not_null();
            table.string("role").not_null(); table.integer("granted_by").not_null();
            table.big_integer("valid_from_epoch").not_null(); table.big_integer("expires_at_epoch").not_null();
            table.string("status").not_null(); table.string("reason").not_null();
            table.string("revocation_key").nullable(); table.integer("revoked_by").nullable();
            table.big_integer("revoked_at_epoch").nullable(); table.string("revocation_reason").nullable();
            table.timestamps();
        }).await?;
        let pool = Orm::pool()?;
        for statement in [
            "CREATE UNIQUE INDEX role_assignments_key_unique ON role_assignments(assignment_key)",
            "CREATE UNIQUE INDEX role_assignments_revocation_key_unique ON role_assignments(revocation_key)",
            "CREATE INDEX role_assignments_active_idx ON role_assignments(school_id, user_id, status, valid_from_epoch, expires_at_epoch)",
            "CREATE INDEX role_assignments_grantor_idx ON role_assignments(school_id, granted_by, created_at)",
        ] { sqlx::query(sqlx::AssertSqlSafe(statement)).execute(pool).await?; }
        Ok(())
    }
    async fn down(&self) -> Result<(), rullst_orm::error::RullstError> {
        Schema::drop_if_exists("role_assignments").await
    }
}
