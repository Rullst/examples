use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "refund_requests")]
pub struct RefundRequest {
    pub id: i32,
    pub user_id: i32,
    pub entitlement_id: i32,
    pub provider: String,
    pub provider_payment_id: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for RefundRequest {
    fn nexus_table() -> &'static str {
        "refund_requests"
    }

    fn nexus_label() -> &'static str {
        "Refund requests"
    }

    fn nexus_icon() -> &'static str {
        "Refunds"
    }

    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            readonly("id", "ID", FieldKind::Number, true),
            readonly("user_id", "User ID", FieldKind::Number, false),
            readonly("entitlement_id", "Entitlement ID", FieldKind::Number, false),
            readonly("provider", "Provider", FieldKind::Text, false),
            readonly(
                "provider_payment_id",
                "Provider payment ID",
                FieldKind::Text,
                false,
            ),
            readonly("status", "Status", FieldKind::Text, false),
            readonly("created_at", "Requested at", FieldKind::Text, false),
            readonly("updated_at", "Updated at", FieldKind::Text, false),
        ]
    }
}

fn readonly(name: &'static str, label: &'static str, kind: FieldKind, hidden: bool) -> FieldMeta {
    FieldMeta {
        name,
        label,
        kind,
        hidden,
        readonly: true,
    }
}
