use rullst::db::{FromRow, Orm};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "course_entitlements")]
pub struct CourseEntitlement {
    pub id: i32,
    pub entitlement_key: String,
    pub school_id: i32,
    pub user_id: i32,
    pub course_id: i32,
    pub source_kind: String,
    pub status: String,
    pub starts_at_epoch: i64,
    pub expires_at_epoch: i64,
    pub created_at: String,
    pub updated_at: String,
}
