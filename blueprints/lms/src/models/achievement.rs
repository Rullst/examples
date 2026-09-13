use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "achievements")]
pub struct Achievement {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub description: String,
    pub xp_reward: i32,
    pub enabled: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for Achievement {
    fn nexus_table() -> &'static str { "achievements" }
    fn nexus_label() -> &'static str { "Achievements" }
    fn nexus_icon() -> &'static str { "🏆" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "code", label: "Stable Code", kind: FieldKind::Text, hidden: false, readonly: false },
            FieldMeta { name: "name", label: "Name", kind: FieldKind::Text, hidden: false, readonly: false },
            FieldMeta { name: "description", label: "Description", kind: FieldKind::Textarea, hidden: false, readonly: false },
            FieldMeta { name: "xp_reward", label: "XP Reward", kind: FieldKind::Number, hidden: false, readonly: false },
            FieldMeta { name: "enabled", label: "Enabled", kind: FieldKind::Boolean, hidden: false, readonly: false },
            FieldMeta { name: "created_at", label: "Created At", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "updated_at", label: "Updated At", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
