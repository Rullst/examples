use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "user_achievements")]
pub struct UserAchievement {
    pub id: i32,
    pub school_id: i32,
    pub user_id: i32,
    pub achievement_id: i32,
    pub source_event_key: String,
    pub awarded_by_user_id: i32,
    pub awarded_at: String,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for UserAchievement {
    fn nexus_table() -> &'static str { "user_achievements" }
    fn nexus_label() -> &'static str { "Awarded Achievements" }
    fn nexus_icon() -> &'static str { "🎖️" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "school_id", label: "School", kind: FieldKind::ForeignKey { table: "schools", label_col: "name" }, hidden: false, readonly: true },
            FieldMeta { name: "user_id", label: "Learner", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "achievement_id", label: "Achievement", kind: FieldKind::ForeignKey { table: "achievements", label_col: "name" }, hidden: false, readonly: true },
            FieldMeta { name: "source_event_key", label: "Source Event", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "awarded_by_user_id", label: "Recorded Actor", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "awarded_at", label: "Awarded At", kind: FieldKind::DateTime, hidden: false, readonly: true },
        ]
    }
}
