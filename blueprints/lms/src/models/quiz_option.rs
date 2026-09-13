use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "quiz_options")]
pub struct QuizOption {
    pub id: i32,
    pub question_id: i32,
    pub label: String,
    pub position: i32,
    pub is_correct: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for QuizOption {
    fn nexus_table() -> &'static str { "quiz_options" }
    fn nexus_label() -> &'static str { "Quiz Options" }
    fn nexus_icon() -> &'static str { "☑️" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "question_id", label: "Question", kind: FieldKind::ForeignKey { table: "quiz_questions", label_col: "prompt" }, hidden: false, readonly: false },
            FieldMeta { name: "label", label: "Label", kind: FieldKind::Text, hidden: false, readonly: false },
            FieldMeta { name: "position", label: "Position", kind: FieldKind::Number, hidden: false, readonly: false },
            FieldMeta { name: "is_correct", label: "Correct", kind: FieldKind::Boolean, hidden: false, readonly: false },
            FieldMeta { name: "created_at", label: "Created At", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "updated_at", label: "Updated At", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
