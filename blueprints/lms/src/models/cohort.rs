use rullst::db::{FromRow, Orm};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "cohorts")]
pub struct Cohort {
    pub id: i32,
    pub cohort_key: String,
    pub school_id: i32,
    pub course_id: i32,
    pub name: String,
    pub status: String,
    pub starts_at_epoch: i64,
    pub ends_at_epoch: i64,
    pub created_at: String,
    pub updated_at: String,
}
