use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "assignment_submissions")]
pub struct AssignmentSubmission {
    pub id: i32,
    pub submission_key: String,
    pub assignment_id: i32,
    pub actor_user_id: i32,
    pub subject_user_id: i32,
    pub attempt_number: i32,
    pub content_text: String,
    pub ruleset_version: String,
    pub status: String,
    pub submitted_at_epoch: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for AssignmentSubmission {
    fn nexus_table() -> &'static str { "assignment_submissions" }
    fn nexus_label() -> &'static str { "Assignment Submissions" }
    fn nexus_icon() -> &'static str { "📨" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "submission_key", label: "Submission Key", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "assignment_id", label: "Assignment", kind: FieldKind::ForeignKey { table: "assignments", label_col: "title" }, hidden: false, readonly: true },
            FieldMeta { name: "actor_user_id", label: "Actor", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "subject_user_id", label: "Learner", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "attempt_number", label: "Attempt", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "content_text", label: "Submission", kind: FieldKind::Textarea, hidden: true, readonly: true },
            FieldMeta { name: "ruleset_version", label: "Ruleset", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "status", label: "Status", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "submitted_at_epoch", label: "Submitted Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
        ]
    }
}
