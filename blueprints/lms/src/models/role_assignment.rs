use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "role_assignments")]
pub struct RoleAssignment {
    pub id: i32,
    pub assignment_key: String,
    pub school_id: i32,
    pub user_id: i32,
    pub role: String,
    pub granted_by: i32,
    pub valid_from_epoch: i64,
    pub expires_at_epoch: i64,
    pub status: String,
    pub reason: String,
    pub revocation_key: Option<String>,
    pub revoked_by: Option<i32>,
    pub revoked_at_epoch: Option<i64>,
    pub revocation_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for RoleAssignment {
    fn nexus_table() -> &'static str { "role_assignments" }
    fn nexus_label() -> &'static str { "Education Role Assignments" }
    fn nexus_icon() -> &'static str { "🪪" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "assignment_key", label: "Assignment Key", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "school_id", label: "School", kind: FieldKind::ForeignKey { table: "schools", label_col: "name" }, hidden: false, readonly: true },
            FieldMeta { name: "user_id", label: "Subject", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "role", label: "Role", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "granted_by", label: "Granted By", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "valid_from_epoch", label: "Valid From", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "expires_at_epoch", label: "Expires At", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "status", label: "Status", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "reason", label: "Reason", kind: FieldKind::Textarea, hidden: false, readonly: true },
            FieldMeta { name: "revocation_key", label: "Revocation Key", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "revoked_by", label: "Revoked By", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "revoked_at_epoch", label: "Revoked At", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "revocation_reason", label: "Revocation Reason", kind: FieldKind::Textarea, hidden: false, readonly: true },
        ]
    }
}
