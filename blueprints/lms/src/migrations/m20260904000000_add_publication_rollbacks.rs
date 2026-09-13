use rullst::db::{Orm, sqlx};
use rullst::db::async_trait;
use rullst::db::schema::{Migration, Schema};

pub struct MigrationImpl;

#[async_trait]
impl Migration for MigrationImpl {
    fn name(&self) -> &'static str { "m20260904000000_add_publication_rollbacks" }

    async fn up(&self) -> Result<(), rullst_orm::error::RullstError> {
        Schema::create("course_publication_rollbacks", |table| {
            table.id(); table.string("rollback_key").not_null(); table.integer("course_id").not_null();
            table.integer("source_version_id").not_null(); table.integer("replaced_version_id").not_null();
            table.integer("result_version_id").not_null(); table.integer("actor_user_id").not_null();
            table.string("reason").not_null(); table.big_integer("occurred_at_epoch").not_null();
            table.timestamps();
        }).await?;
        let pool = Orm::pool()?;
        for statement in [
            "CREATE UNIQUE INDEX course_publication_rollbacks_key_unique ON course_publication_rollbacks(rollback_key)",
            "CREATE UNIQUE INDEX course_publication_rollbacks_result_unique ON course_publication_rollbacks(result_version_id)",
            "CREATE INDEX course_publication_rollbacks_course_time_idx ON course_publication_rollbacks(course_id, occurred_at_epoch)",
        ] { sqlx::query(sqlx::AssertSqlSafe(statement)).execute(pool).await?; }
        Ok(())
    }

    async fn down(&self) -> Result<(), rullst_orm::error::RullstError> {
        Schema::drop_if_exists("course_publication_rollbacks").await
    }
}
