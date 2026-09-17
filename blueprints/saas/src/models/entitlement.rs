use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "entitlements")]
pub struct Entitlement {
    pub id: i32,
    pub user_id: i32,
    pub product_sku: String,
    pub provider: String,
    pub provider_payment_id: String,
    pub artifact_version: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for Entitlement {
    fn nexus_table() -> &'static str {
        "entitlements"
    }

    fn nexus_label() -> &'static str {
        "Entitlements"
    }

    fn nexus_icon() -> &'static str {
        "Access"
    }

    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            readonly("id", "ID", FieldKind::Number, true),
            readonly("user_id", "User ID", FieldKind::Number, false),
            readonly("product_sku", "Product SKU", FieldKind::Text, false),
            readonly("provider", "Provider", FieldKind::Text, false),
            readonly(
                "provider_payment_id",
                "Provider payment ID",
                FieldKind::Text,
                false,
            ),
            readonly(
                "artifact_version",
                "Artifact version",
                FieldKind::Text,
                false,
            ),
            readonly("status", "Status", FieldKind::Text, false),
            readonly("created_at", "Created at", FieldKind::Text, false),
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
