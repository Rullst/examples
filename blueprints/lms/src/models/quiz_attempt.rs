use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "quiz_attempts")]
pub struct QuizAttempt {
    pub id: i32,
    pub attempt_key: String,
    pub quiz_id: i32,
    pub actor_user_id: i32,
    pub subject_user_id: i32,
    pub ruleset_version: String,
    pub status: String,
    pub score_percent: i32,
    pub points_awarded: i32,
    pub max_points: i32,
    pub graded_at_epoch: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for QuizAttempt {
    fn nexus_table() -> &'static str { "quiz_attempts" }
    fn nexus_label() -> &'static str { "Quiz Attempts" }
    fn nexus_icon() -> &'static str { "📝" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "attempt_key", label: "Attempt Key", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "quiz_id", label: "Quiz", kind: FieldKind::ForeignKey { table: "quizzes", label_col: "title" }, hidden: false, readonly: true },
            FieldMeta { name: "actor_user_id", label: "Actor", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "subject_user_id", label: "Learner", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "ruleset_version", label: "Ruleset", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "status", label: "Status", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "score_percent", label: "Score %", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "points_awarded", label: "Points", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "max_points", label: "Maximum", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "graded_at_epoch", label: "Graded Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "created_at", label: "Created At", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "updated_at", label: "Updated At", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
