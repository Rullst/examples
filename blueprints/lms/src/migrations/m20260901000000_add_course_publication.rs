use rullst::db::{Orm, sqlx};
use rullst::db::async_trait;
use rullst::db::schema::{Migration, Schema};

pub struct MigrationImpl;

#[async_trait]
impl Migration for MigrationImpl {
    fn name(&self) -> &'static str { "m20260901000000_add_course_publication" }

    async fn up(&self) -> Result<(), rullst_orm::error::RullstError> {
        Schema::create("course_versions", |table| {
            table.id(); table.integer("course_id").not_null(); table.string("version_key").not_null();
            table.integer("revision").not_null(); table.string("status").not_null();
            table.string("content_json").not_null(); table.integer("authored_by").not_null();
            table.integer("reviewed_by").not_null(); table.big_integer("scheduled_at_epoch").not_null();
            table.big_integer("published_at_epoch").not_null(); table.timestamps();
        }).await?;
        Schema::create("enrollment_content_versions", |table| {
            table.id(); table.integer("enrollment_id").not_null();
            table.integer("course_version_id").not_null(); table.timestamps();
        }).await?;
        let pool = Orm::pool()?;
        for statement in [
            "CREATE UNIQUE INDEX course_versions_key_unique ON course_versions(version_key)",
            "CREATE UNIQUE INDEX course_versions_revision_unique ON course_versions(course_id, revision)",
            "CREATE INDEX course_versions_status_idx ON course_versions(course_id, status, scheduled_at_epoch)",
            "CREATE UNIQUE INDEX enrollment_content_version_unique ON enrollment_content_versions(enrollment_id)",
        ] { sqlx::query(sqlx::AssertSqlSafe(statement)).execute(pool).await?; }
        for fixture in [
            "INSERT INTO course_versions (id, course_id, version_key, revision, status, content_json, authored_by, reviewed_by, scheduled_at_epoch, published_at_epoch, created_at, updated_at) VALUES (1, 1, 'course-1-v1', 1, 'published', '{\"schema_version\":1,\"lesson_ids\":[1,2],\"completion\":{\"schema_version\":1,\"ruleset_version\":\"course-1-completion-v1\",\"required_lesson_ids\":[1,2],\"required_progress_percent\":100}}', 1, 2, 0, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            "INSERT INTO course_versions (id, course_id, version_key, revision, status, content_json, authored_by, reviewed_by, scheduled_at_epoch, published_at_epoch, created_at, updated_at) VALUES (2, 2, 'course-2-v1', 1, 'published', '{\"schema_version\":1,\"lesson_ids\":[3,4],\"completion\":{\"schema_version\":1,\"ruleset_version\":\"course-2-completion-v1\",\"required_lesson_ids\":[3,4],\"required_progress_percent\":100}}', 1, 2, 0, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        ] { sqlx::query(sqlx::AssertSqlSafe(fixture)).execute(pool).await?; }
        Ok(())
    }

    async fn down(&self) -> Result<(), rullst_orm::error::RullstError> {
        Schema::drop_if_exists("enrollment_content_versions").await?;
        Schema::drop_if_exists("course_versions").await
    }
}
