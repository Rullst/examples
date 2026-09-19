use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "tester_certificates")]
pub struct TesterCertificate {
    pub id: i32,
    pub entitlement_id: i32,
    pub public_id: String,
    pub badge_kind: String,
    pub environment: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for TesterCertificate {
    fn nexus_table() -> &'static str {
        "tester_certificates"
    }

    fn nexus_label() -> &'static str {
        "Tester Certificates"
    }

    fn nexus_icon() -> &'static str {
        "Certificate"
    }

    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            readonly("id", "ID", FieldKind::Number, true),
            readonly("entitlement_id", "Entitlement ID", FieldKind::Number, true),
            readonly(
                "public_id",
                "Public verification ID",
                FieldKind::Text,
                false,
            ),
            readonly("badge_kind", "Badge kind", FieldKind::Text, false),
            readonly("environment", "Environment", FieldKind::Text, false),
            readonly("status", "Status", FieldKind::Text, false),
            readonly("created_at", "Issued at", FieldKind::Text, false),
            readonly("updated_at", "Updated at", FieldKind::Text, true),
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
