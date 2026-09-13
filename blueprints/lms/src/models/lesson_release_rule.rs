use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "lesson_release_rules")]
pub struct LessonReleaseRule {
    pub id: i32,
    pub lesson_id: i32,
    pub ruleset_version: String,
    pub release_at_epoch: i64,
    pub expire_at_epoch: i64,
    pub prerequisite_lesson_id: i32,
    pub required_progress_percent: i32,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for LessonReleaseRule {
    fn nexus_table() -> &'static str { "lesson_release_rules" }
    fn nexus_label() -> &'static str { "Lesson Release Rules" }
    fn nexus_icon() -> &'static str { "🔐" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "lesson_id", label: "Lesson", kind: FieldKind::ForeignKey { table: "lessons", label_col: "title" }, hidden: false, readonly: true },
            FieldMeta { name: "ruleset_version", label: "Ruleset", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "release_at_epoch", label: "Release Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "expire_at_epoch", label: "Expire Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "prerequisite_lesson_id", label: "Prerequisite", kind: FieldKind::ForeignKey { table: "lessons", label_col: "title" }, hidden: false, readonly: true },
            FieldMeta { name: "required_progress_percent", label: "Required Progress %", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "status", label: "Status", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "created_at", label: "Created At", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "updated_at", label: "Updated At", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
