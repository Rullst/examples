use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "purchase_attempts")]
pub struct PurchaseAttempt {
    pub id: i32,
    pub user_id: i32,
    pub provider: String,
    pub product_sku: String,
    pub provider_price_id: String,
    pub expected_amount_minor: i32,
    pub expected_currency: String,
    pub checkout_token_hash: String,
    pub provider_session_id: Option<String>,
    pub provider_payment_id: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for PurchaseAttempt {
    fn nexus_table() -> &'static str {
        "purchase_attempts"
    }

    fn nexus_label() -> &'static str {
        "Purchase attempts"
    }

    fn nexus_icon() -> &'static str {
        "Payments"
    }

    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            readonly("id", "ID", FieldKind::Number, true),
            readonly("user_id", "User ID", FieldKind::Number, false),
            readonly("provider", "Provider", FieldKind::Text, false),
            readonly("product_sku", "Product SKU", FieldKind::Text, false),
            readonly(
                "provider_price_id",
                "Provider price ID",
                FieldKind::Text,
                false,
            ),
            readonly(
                "expected_amount_minor",
                "Amount (minor unit)",
                FieldKind::Number,
                false,
            ),
            readonly("expected_currency", "Currency", FieldKind::Text, false),
            readonly(
                "checkout_token_hash",
                "Checkout token hash",
                FieldKind::Text,
                true,
            ),
            readonly(
                "provider_session_id",
                "Provider session ID",
                FieldKind::Text,
                false,
            ),
            readonly(
                "provider_payment_id",
                "Provider payment ID",
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
