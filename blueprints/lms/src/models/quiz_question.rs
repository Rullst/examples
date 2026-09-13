use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "quiz_questions")]
pub struct QuizQuestion {
    pub id: i32,
    pub quiz_id: i32,
    pub prompt: String,
    pub position: i32,
    pub points: i32,
    pub enabled: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for QuizQuestion {
    fn nexus_table() -> &'static str { "quiz_questions" }
    fn nexus_label() -> &'static str { "Quiz Questions" }
    fn nexus_icon() -> &'static str { "❓" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "quiz_id", label: "Quiz", kind: FieldKind::ForeignKey { table: "quizzes", label_col: "title" }, hidden: false, readonly: false },
            FieldMeta { name: "prompt", label: "Prompt", kind: FieldKind::Textarea, hidden: false, readonly: false },
            FieldMeta { name: "position", label: "Position", kind: FieldKind::Number, hidden: false, readonly: false },
            FieldMeta { name: "points", label: "Points", kind: FieldKind::Number, hidden: false, readonly: false },
            FieldMeta { name: "enabled", label: "Enabled", kind: FieldKind::Boolean, hidden: false, readonly: false },
            FieldMeta { name: "created_at", label: "Created At", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "updated_at", label: "Updated At", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
