use rullst::db::{FromRow, Orm};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "course_school_scopes")]
pub struct CourseSchoolScope {
    pub id: i32,
    pub school_id: i32,
    pub course_id: i32,
    pub enrollment_policy: String,
    pub created_at: String,
    pub updated_at: String,
}
