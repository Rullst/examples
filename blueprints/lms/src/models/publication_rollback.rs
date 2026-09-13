use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "course_publication_rollbacks")]
pub struct PublicationRollback {
    pub id: i32,
    pub rollback_key: String,
    pub course_id: i32,
    pub source_version_id: i32,
    pub replaced_version_id: i32,
    pub result_version_id: i32,
    pub actor_user_id: i32,
    pub reason: String,
    pub occurred_at_epoch: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for PublicationRollback {
    fn nexus_table() -> &'static str { "course_publication_rollbacks" }
    fn nexus_label() -> &'static str { "Publication Rollbacks" }
    fn nexus_icon() -> &'static str { "↩️" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "rollback_key", label: "Rollback Key", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "course_id", label: "Course", kind: FieldKind::ForeignKey { table: "courses", label_col: "title" }, hidden: false, readonly: true },
            FieldMeta { name: "source_version_id", label: "Restored Source", kind: FieldKind::ForeignKey { table: "course_versions", label_col: "version_key" }, hidden: false, readonly: true },
            FieldMeta { name: "replaced_version_id", label: "Replaced Version", kind: FieldKind::ForeignKey { table: "course_versions", label_col: "version_key" }, hidden: false, readonly: true },
            FieldMeta { name: "result_version_id", label: "New Published Version", kind: FieldKind::ForeignKey { table: "course_versions", label_col: "version_key" }, hidden: false, readonly: true },
            FieldMeta { name: "actor_user_id", label: "Administrator", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "reason", label: "Reason", kind: FieldKind::Textarea, hidden: false, readonly: true },
            FieldMeta { name: "occurred_at_epoch", label: "Occurred Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
        ]
    }
}
