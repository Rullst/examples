use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "certificates")]
pub struct Certificate {
    pub id: i32,
    pub certificate_key: String,
    pub completion_id: i32,
    pub status: String,
    pub issued_at_epoch: i64,
    pub revocation_key: Option<String>,
    pub revoked_by: i32,
    pub revoked_at_epoch: i64,
    pub revocation_reason: String,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for Certificate {
    fn nexus_table() -> &'static str { "certificates" }
    fn nexus_label() -> &'static str { "Certificates" }
    fn nexus_icon() -> &'static str { "📜" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "certificate_key", label: "Public Key", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "completion_id", label: "Completion", kind: FieldKind::ForeignKey { table: "course_completions", label_col: "completion_key" }, hidden: false, readonly: true },
            FieldMeta { name: "status", label: "Status", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "issued_at_epoch", label: "Issued Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "revocation_key", label: "Revocation Key", kind: FieldKind::Text, hidden: true, readonly: true },
            FieldMeta { name: "revoked_by", label: "Revoked By", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "revoked_at_epoch", label: "Revoked Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "revocation_reason", label: "Revocation Reason", kind: FieldKind::Textarea, hidden: false, readonly: true },
        ]
    }
}
