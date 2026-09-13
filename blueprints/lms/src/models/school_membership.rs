use rullst::db::{FromRow, Orm};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "school_memberships")]
pub struct SchoolMembership {
    pub id: i32,
    pub membership_key: String,
    pub school_id: i32,
    pub user_id: i32,
    pub status: String,
    pub is_default: i32,
    pub valid_from_epoch: i64,
    pub expires_at_epoch: i64,
    pub created_at: String,
    pub updated_at: String,
}
