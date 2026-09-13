use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "course_completions")]
pub struct CourseCompletion {
    pub id: i32,
    pub completion_key: String,
    pub subject_user_id: i32,
    pub course_id: i32,
    pub course_version_id: i32,
    pub ruleset_version: String,
    pub completed_at_epoch: i64,
    pub evidence_json: String,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for CourseCompletion {
    fn nexus_table() -> &'static str { "course_completions" }
    fn nexus_label() -> &'static str { "Course Completions" }
    fn nexus_icon() -> &'static str { "🏁" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "completion_key", label: "Completion Key", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "subject_user_id", label: "Learner", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "course_id", label: "Course", kind: FieldKind::ForeignKey { table: "courses", label_col: "title" }, hidden: false, readonly: true },
            FieldMeta { name: "course_version_id", label: "Course Version", kind: FieldKind::ForeignKey { table: "course_versions", label_col: "version_key" }, hidden: false, readonly: true },
            FieldMeta { name: "ruleset_version", label: "Ruleset", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "completed_at_epoch", label: "Completed Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "evidence_json", label: "Evidence", kind: FieldKind::Json, hidden: false, readonly: true },
        ]
    }
}
