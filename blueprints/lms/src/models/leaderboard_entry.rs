use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm, serde::Serialize, serde::Deserialize)]
#[orm(table = "leaderboard_entries")]
pub struct LeaderboardEntry {
    pub id: i32,
    pub user_id: i32,
    pub course_id: i32,
    pub season_key: String,
    pub score: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for LeaderboardEntry {
    fn nexus_table() -> &'static str { "leaderboard_entries" }
    fn nexus_label() -> &'static str { "Leaderboard" }
    fn nexus_icon() -> &'static str { "🥇" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "user_id", label: "Learner", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "course_id", label: "Course", kind: FieldKind::ForeignKey { table: "courses", label_col: "title" }, hidden: false, readonly: true },
            FieldMeta { name: "season_key", label: "Season", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "score", label: "Authoritative Score", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "created_at", label: "Created At", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "updated_at", label: "Updated At", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
