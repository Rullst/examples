use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "academy_outbox")]
pub struct DomainEvent {
    pub id: i32,
    pub school_id: i32,
    pub event_key: String,
    pub event_kind: String,
    pub subject_user_id: i32,
    pub payload_json: String,
    pub status: String,
    pub attempts: i32,
    pub claimed_by: String,
    pub claim_key: String,
    pub last_error: String,
    pub available_at: String,
    pub available_at_epoch: i64,
    pub claim_expires_at_epoch: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for DomainEvent {
    fn nexus_table() -> &'static str { "academy_outbox" }
    fn nexus_label() -> &'static str { "Domain Event Outbox" }
    fn nexus_icon() -> &'static str { "📨" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "school_id", label: "School", kind: FieldKind::ForeignKey { table: "schools", label_col: "name" }, hidden: false, readonly: true },
            FieldMeta { name: "event_key", label: "Event Key", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "event_kind", label: "Event Kind", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "subject_user_id", label: "Learner", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "payload_json", label: "Versioned Payload", kind: FieldKind::Json, hidden: false, readonly: true },
            FieldMeta { name: "status", label: "Delivery Status", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "attempts", label: "Delivery Attempts", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "claimed_by", label: "Claimed By", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "claim_key", label: "Claim Key", kind: FieldKind::Text, hidden: true, readonly: true },
            FieldMeta { name: "last_error", label: "Last Error", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "available_at", label: "Available At", kind: FieldKind::DateTime, hidden: false, readonly: true },
            FieldMeta { name: "available_at_epoch", label: "Available Epoch", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "claim_expires_at_epoch", label: "Claim Expires Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "created_at", label: "Created At", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "updated_at", label: "Updated At", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
