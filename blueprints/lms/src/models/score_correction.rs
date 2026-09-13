use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "score_corrections")]
pub struct ScoreCorrection {
    pub id: i32,
    pub correction_key: String,
    pub actor_user_id: i32,
    pub subject_user_id: i32,
    pub course_id: i32,
    pub season_key: String,
    pub previous_score: i32,
    pub corrected_score: i32,
    pub reason: String,
    pub ruleset_version: String,
    pub occurred_at: String,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for ScoreCorrection {
    fn nexus_table() -> &'static str { "score_corrections" }
    fn nexus_label() -> &'static str { "Score Corrections" }
    fn nexus_icon() -> &'static str { "🛡️" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "correction_key", label: "Correction Key", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "actor_user_id", label: "Administrator", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "subject_user_id", label: "Learner", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "course_id", label: "Course", kind: FieldKind::ForeignKey { table: "courses", label_col: "title" }, hidden: false, readonly: true },
            FieldMeta { name: "season_key", label: "Season", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "previous_score", label: "Previous Score", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "corrected_score", label: "Corrected Score", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "reason", label: "Reason", kind: FieldKind::Textarea, hidden: false, readonly: true },
            FieldMeta { name: "ruleset_version", label: "Ruleset", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "occurred_at", label: "Occurred At", kind: FieldKind::DateTime, hidden: false, readonly: true },
            FieldMeta { name: "created_at", label: "Created At", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "updated_at", label: "Updated At", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
