use rullst::db::{FromRow, Orm};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "cohort_memberships")]
pub struct CohortMembership {
    pub id: i32,
    pub cohort_id: i32,
    pub school_membership_id: i32,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}
