use rullst::db::{FromRow, Orm};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "schools")]
pub struct School {
    pub id: i32,
    pub tenant_key: String,
    pub name: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}
