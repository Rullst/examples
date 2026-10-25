use rullst::db::{FromRow, Nexus, Orm, RowSnapshot};

#[derive(Debug, Clone, FromRow, Orm, Nexus)]
#[orm(table = "skills")]
pub struct Skill {
    pub id: i32,
    pub name: String,
    pub category: String,
    #[sqlx(skip)]
    pub snapshot: RowSnapshot,
}
