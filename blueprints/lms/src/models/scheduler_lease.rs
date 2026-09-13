use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "scheduler_leases")]
pub struct SchedulerLease {
    pub id: i32,
    pub lease_key: String,
    pub holder_id: String,
    pub lease_token: String,
    pub heartbeat_at_epoch: i64,
    pub expires_at_epoch: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for SchedulerLease {
    fn nexus_table() -> &'static str { "scheduler_leases" }
    fn nexus_label() -> &'static str { "Scheduler Leases" }
    fn nexus_icon() -> &'static str { "🫀" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "lease_key", label: "Lease", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "holder_id", label: "Holder", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "lease_token", label: "Token", kind: FieldKind::Text, hidden: true, readonly: true },
            FieldMeta { name: "heartbeat_at_epoch", label: "Heartbeat Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "expires_at_epoch", label: "Expires Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
        ]
    }
}
