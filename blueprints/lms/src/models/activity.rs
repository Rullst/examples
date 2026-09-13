use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "activities")]
pub struct Activity {
    pub id: i32,
    pub lesson_id: i32,
    pub title: String,
    pub activity_kind: String,
    pub max_score: i32,
    pub ruleset_version: String,
    pub season_key: String,
    pub evidence_sha256: String,
    pub config_json: String,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for Activity {
    fn nexus_table() -> &'static str { "activities" }
    fn nexus_label() -> &'static str { "Activities" }
    fn nexus_icon() -> &'static str { "🎯" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "lesson_id", label: "Lesson", kind: FieldKind::ForeignKey { table: "lessons", label_col: "title" }, hidden: false, readonly: false },
            FieldMeta { name: "title", label: "Title", kind: FieldKind::Text, hidden: false, readonly: false },
            FieldMeta { name: "activity_kind", label: "Kind", kind: FieldKind::Enum { options: vec!["quiz", "exercise", "project", "game"] }, hidden: false, readonly: false },
            FieldMeta { name: "max_score", label: "Maximum Score", kind: FieldKind::Number, hidden: false, readonly: false },
            FieldMeta { name: "ruleset_version", label: "Ruleset", kind: FieldKind::Text, hidden: false, readonly: false },
            FieldMeta { name: "season_key", label: "Season", kind: FieldKind::Text, hidden: false, readonly: false },
            FieldMeta { name: "evidence_sha256", label: "Evidence SHA-256", kind: FieldKind::Text, hidden: false, readonly: false },
            FieldMeta { name: "config_json", label: "Versioned Configuration", kind: FieldKind::Json, hidden: false, readonly: false },
            FieldMeta { name: "created_at", label: "Created At", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "updated_at", label: "Updated At", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
