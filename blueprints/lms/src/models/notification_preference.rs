use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "notification_preferences")]
pub struct NotificationPreference {
    pub id: i32,
    pub school_id: i32,
    pub user_id: i32,
    pub channel: String,
    pub enabled: i32,
    pub locale: String,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for NotificationPreference {
    fn nexus_table() -> &'static str { "notification_preferences" }
    fn nexus_label() -> &'static str { "Notification Preferences" }
    fn nexus_icon() -> &'static str { "⚙️" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "school_id", label: "School", kind: FieldKind::ForeignKey { table: "schools", label_col: "name" }, hidden: false, readonly: true },
            FieldMeta { name: "user_id", label: "Learner", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "channel", label: "Channel", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "enabled", label: "Enabled", kind: FieldKind::Boolean, hidden: false, readonly: true },
            FieldMeta { name: "locale", label: "Locale", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
