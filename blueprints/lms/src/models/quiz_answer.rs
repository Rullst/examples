use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "quiz_answers")]
pub struct QuizAnswer {
    pub id: i32,
    pub attempt_id: i32,
    pub question_id: i32,
    pub option_id: i32,
    pub correct: i32,
    pub points_awarded: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for QuizAnswer {
    fn nexus_table() -> &'static str { "quiz_answers" }
    fn nexus_label() -> &'static str { "Quiz Answers" }
    fn nexus_icon() -> &'static str { "🔎" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "attempt_id", label: "Attempt", kind: FieldKind::ForeignKey { table: "quiz_attempts", label_col: "attempt_key" }, hidden: false, readonly: true },
            FieldMeta { name: "question_id", label: "Question", kind: FieldKind::ForeignKey { table: "quiz_questions", label_col: "prompt" }, hidden: false, readonly: true },
            FieldMeta { name: "option_id", label: "Selected Option", kind: FieldKind::ForeignKey { table: "quiz_options", label_col: "label" }, hidden: false, readonly: true },
            FieldMeta { name: "correct", label: "Correct", kind: FieldKind::Boolean, hidden: false, readonly: true },
            FieldMeta { name: "points_awarded", label: "Points", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "created_at", label: "Created At", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "updated_at", label: "Updated At", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
