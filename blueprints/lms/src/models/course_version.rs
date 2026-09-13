use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "course_versions")]
pub struct CourseVersion {
    pub id: i32,
    pub course_id: i32,
    pub version_key: String,
    pub revision: i32,
    pub status: String,
    pub content_json: String,
    pub authored_by: i32,
    pub reviewed_by: i32,
    pub scheduled_at_epoch: i64,
    pub published_at_epoch: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for CourseVersion {
    fn nexus_table() -> &'static str { "course_versions" }
    fn nexus_label() -> &'static str { "Course Versions" }
    fn nexus_icon() -> &'static str { "📚" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "course_id", label: "Course", kind: FieldKind::ForeignKey { table: "courses", label_col: "title" }, hidden: false, readonly: true },
            FieldMeta { name: "version_key", label: "Version Key", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "revision", label: "Revision", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "status", label: "State", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "content_json", label: "Immutable Snapshot", kind: FieldKind::Json, hidden: false, readonly: true },
            FieldMeta { name: "authored_by", label: "Author", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "reviewed_by", label: "Reviewer", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "scheduled_at_epoch", label: "Scheduled Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "published_at_epoch", label: "Published Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
        ]
    }
}
