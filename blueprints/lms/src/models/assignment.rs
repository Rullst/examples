use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "assignments")]
pub struct Assignment {
    pub id: i32,
    pub lesson_id: i32,
    pub title: String,
    pub instructions: String,
    pub ruleset_version: String,
    pub max_attempts: i32,
    pub due_at_epoch: i64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for Assignment {
    fn nexus_table() -> &'static str { "assignments" }
    fn nexus_label() -> &'static str { "Assignments" }
    fn nexus_icon() -> &'static str { "📝" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "lesson_id", label: "Lesson", kind: FieldKind::ForeignKey { table: "lessons", label_col: "title" }, hidden: false, readonly: true },
            FieldMeta { name: "title", label: "Title", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "instructions", label: "Instructions", kind: FieldKind::Textarea, hidden: false, readonly: true },
            FieldMeta { name: "ruleset_version", label: "Ruleset", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "max_attempts", label: "Max Attempts", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "due_at_epoch", label: "Due Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "status", label: "Status", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
