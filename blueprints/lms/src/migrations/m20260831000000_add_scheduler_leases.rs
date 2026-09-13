use rullst::db::{Orm, sqlx};
use rullst::db::async_trait;
use rullst::db::schema::{Migration, Schema};

pub struct MigrationImpl;

#[async_trait]
impl Migration for MigrationImpl {
    fn name(&self) -> &'static str { "m20260831000000_add_scheduler_leases" }

    async fn up(&self) -> Result<(), rullst_orm::error::RullstError> {
        Schema::create("scheduler_leases", |table| {
            table.id();
            table.string("lease_key").not_null();
            table.string("holder_id").not_null();
            table.string("lease_token").not_null();
            table.big_integer("heartbeat_at_epoch").not_null();
            table.big_integer("expires_at_epoch").not_null();
            table.timestamps();
        }).await?;
        let pool = Orm::pool()?;
        for statement in [
            "CREATE UNIQUE INDEX scheduler_leases_key_unique ON scheduler_leases(lease_key)",
            "CREATE INDEX scheduler_leases_expiry_idx ON scheduler_leases(expires_at_epoch)",
        ] {
            sqlx::query(sqlx::AssertSqlSafe(statement)).execute(pool).await?;
        }
        Ok(())
    }

    async fn down(&self) -> Result<(), rullst_orm::error::RullstError> {
        Schema::drop_if_exists("scheduler_leases").await
    }
}
