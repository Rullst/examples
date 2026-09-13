use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "automation_rules")]
pub struct AutomationRule {
    pub id: i32,
    pub school_id: i32,
    pub name: String,
    pub trigger_kind: String,
    pub action_kind: String,
    pub config_json: String,
    pub enabled: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for AutomationRule {
    fn nexus_table() -> &'static str { "automation_rules" }
    fn nexus_label() -> &'static str { "Automation Rules" }
    fn nexus_icon() -> &'static str { "⚙️" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "school_id", label: "School", kind: FieldKind::ForeignKey { table: "schools", label_col: "name" }, hidden: false, readonly: true },
            FieldMeta { name: "name", label: "Name", kind: FieldKind::Text, hidden: false, readonly: false },
            FieldMeta { name: "trigger_kind", label: "Trigger", kind: FieldKind::Text, hidden: false, readonly: false },
            FieldMeta { name: "action_kind", label: "Action", kind: FieldKind::Text, hidden: false, readonly: false },
            FieldMeta { name: "config_json", label: "Versioned Configuration", kind: FieldKind::Json, hidden: false, readonly: false },
            FieldMeta { name: "enabled", label: "Enabled", kind: FieldKind::Boolean, hidden: false, readonly: false },
            FieldMeta { name: "created_at", label: "Created At", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "updated_at", label: "Updated At", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
