use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "activity_attempts")]
pub struct ActivityAttempt {
    pub id: i32,
    pub attempt_key: String,
    pub activity_id: i32,
    pub actor_user_id: i32,
    pub subject_user_id: i32,
    pub activity_kind: String,
    pub ruleset_version: String,
    pub state_json: String,
    pub submission_key: String,
    pub points: i32,
    pub max_score: i32,
    pub started_at_epoch: i64,
    pub finished_at_epoch: i64,
    pub evidence_sha256: String,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for ActivityAttempt {
    fn nexus_table() -> &'static str { "activity_attempts" }
    fn nexus_label() -> &'static str { "Activity Attempts" }
    fn nexus_icon() -> &'static str { "🧩" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "attempt_key", label: "Attempt", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "activity_id", label: "Activity", kind: FieldKind::ForeignKey { table: "activities", label_col: "title" }, hidden: false, readonly: true },
            FieldMeta { name: "actor_user_id", label: "Actor", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "subject_user_id", label: "Learner", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "activity_kind", label: "Kind", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "ruleset_version", label: "Ruleset", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "state_json", label: "Bounded State", kind: FieldKind::Json, hidden: true, readonly: true },
            FieldMeta { name: "submission_key", label: "Submission", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "points", label: "Points", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "max_score", label: "Maximum Score", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "started_at_epoch", label: "Started Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "finished_at_epoch", label: "Finished Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "evidence_sha256", label: "Evidence SHA-256", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "created_at", label: "Created At", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "updated_at", label: "Updated At", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
