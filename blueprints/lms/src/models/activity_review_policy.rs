use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "activity_review_policies")]
pub struct ActivityReviewPolicy {
    pub id: i32,
    pub activity_id: i32,
    pub algorithm_version: String,
    pub passing_ratio_milli: i32,
    pub first_interval_seconds: i64,
    pub lapse_interval_seconds: i64,
    pub maximum_interval_seconds: i64,
    pub enabled: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for ActivityReviewPolicy {
    fn nexus_table() -> &'static str { "activity_review_policies" }
    fn nexus_label() -> &'static str { "Activity Review Policies" }
    fn nexus_icon() -> &'static str { "🧠" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "activity_id", label: "Activity", kind: FieldKind::ForeignKey { table: "activities", label_col: "title" }, hidden: false, readonly: true },
            FieldMeta { name: "algorithm_version", label: "Algorithm", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "passing_ratio_milli", label: "Passing Ratio (‰)", kind: FieldKind::Number, hidden: false, readonly: false },
            FieldMeta { name: "first_interval_seconds", label: "First Interval", kind: FieldKind::Number, hidden: false, readonly: false },
            FieldMeta { name: "lapse_interval_seconds", label: "Lapse Interval", kind: FieldKind::Number, hidden: false, readonly: false },
            FieldMeta { name: "maximum_interval_seconds", label: "Maximum Interval", kind: FieldKind::Number, hidden: false, readonly: false },
            FieldMeta { name: "enabled", label: "Enabled", kind: FieldKind::Boolean, hidden: false, readonly: false },
            FieldMeta { name: "created_at", label: "Created At", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "updated_at", label: "Updated At", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
