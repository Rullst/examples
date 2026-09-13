use rullst::db::{Orm, sqlx};
use rullst::db::async_trait;
use rullst::db::schema::{Migration, Schema};

pub struct MigrationImpl;

#[async_trait]
impl Migration for MigrationImpl {
    fn name(&self) -> &'static str { "m20260829000000_add_lesson_availability" }

    async fn up(&self) -> Result<(), rullst_orm::error::RullstError> {
        Schema::create("lesson_release_rules", |table| {
            table.id();
            table.integer("lesson_id").not_null();
            table.string("ruleset_version").not_null();
            table.big_integer("release_at_epoch").not_null();
            table.big_integer("expire_at_epoch").not_null();
            table.integer("prerequisite_lesson_id").not_null();
            table.integer("required_progress_percent").not_null();
            table.string("status").not_null();
            table.timestamps();
        }).await?;
        let pool = Orm::pool()?;
        for statement in [
            "CREATE UNIQUE INDEX lesson_release_rules_version_unique ON lesson_release_rules(lesson_id, ruleset_version)",
            "CREATE INDEX lesson_release_rules_active_idx ON lesson_release_rules(lesson_id, status)",
        ] {
            sqlx::query(sqlx::AssertSqlSafe(statement)).execute(pool).await?;
        }
        for fixture in [
            "INSERT INTO lesson_release_rules (lesson_id, ruleset_version, release_at_epoch, expire_at_epoch, prerequisite_lesson_id, required_progress_percent, status, created_at, updated_at) VALUES (1, 'lesson-1-v1', 0, 0, 0, 0, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            "INSERT INTO lesson_release_rules (lesson_id, ruleset_version, release_at_epoch, expire_at_epoch, prerequisite_lesson_id, required_progress_percent, status, created_at, updated_at) VALUES (2, 'lesson-2-v1', 0, 0, 1, 100, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            "INSERT INTO lesson_release_rules (lesson_id, ruleset_version, release_at_epoch, expire_at_epoch, prerequisite_lesson_id, required_progress_percent, status, created_at, updated_at) VALUES (3, 'lesson-3-v1', 0, 0, 0, 0, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            "INSERT INTO lesson_release_rules (lesson_id, ruleset_version, release_at_epoch, expire_at_epoch, prerequisite_lesson_id, required_progress_percent, status, created_at, updated_at) VALUES (4, 'lesson-4-v1', 0, 0, 0, 0, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        ] {
            sqlx::query(sqlx::AssertSqlSafe(fixture)).execute(pool).await?;
        }
        Ok(())
    }

    async fn down(&self) -> Result<(), rullst_orm::error::RullstError> {
        Schema::drop_if_exists("lesson_release_rules").await
    }
}
