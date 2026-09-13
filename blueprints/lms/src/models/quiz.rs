use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "quizzes")]
pub struct Quiz {
    pub id: i32,
    pub lesson_id: i32,
    pub activity_id: i32,
    pub title: String,
    pub passing_score: i32,
    pub max_attempts: i32,
    pub time_limit_seconds: i32,
    pub ruleset_version: String,
    pub season_key: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for Quiz {
    fn nexus_table() -> &'static str { "quizzes" }
    fn nexus_label() -> &'static str { "Quizzes" }
    fn nexus_icon() -> &'static str { "🧠" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "lesson_id", label: "Lesson", kind: FieldKind::ForeignKey { table: "lessons", label_col: "title" }, hidden: false, readonly: false },
            FieldMeta { name: "activity_id", label: "Score Activity", kind: FieldKind::ForeignKey { table: "activities", label_col: "title" }, hidden: false, readonly: false },
            FieldMeta { name: "title", label: "Title", kind: FieldKind::Text, hidden: false, readonly: false },
            FieldMeta { name: "passing_score", label: "Passing Score", kind: FieldKind::Number, hidden: false, readonly: false },
            FieldMeta { name: "max_attempts", label: "Maximum Attempts", kind: FieldKind::Number, hidden: false, readonly: false },
            FieldMeta { name: "time_limit_seconds", label: "Time Limit Seconds", kind: FieldKind::Number, hidden: false, readonly: false },
            FieldMeta { name: "ruleset_version", label: "Ruleset Version", kind: FieldKind::Text, hidden: false, readonly: false },
            FieldMeta { name: "season_key", label: "Leaderboard Season", kind: FieldKind::Text, hidden: false, readonly: false },
            FieldMeta { name: "status", label: "Status", kind: FieldKind::Enum { options: vec!["draft", "published", "archived"] }, hidden: false, readonly: false },
            FieldMeta { name: "created_at", label: "Created At", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "updated_at", label: "Updated At", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
