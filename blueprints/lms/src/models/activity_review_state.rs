use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "activity_review_states")]
pub struct ActivityReviewState {
    pub id: i32,
    pub school_id: i32,
    pub subject_user_id: i32,
    pub course_id: i32,
    pub activity_id: i32,
    pub algorithm_version: String,
    pub repetitions: i32,
    pub lapses: i32,
    pub ease_milli: i32,
    pub interval_seconds: i64,
    pub due_at_epoch: i64,
    pub last_attempt_key: String,
    pub last_points: i32,
    pub last_max_score: i32,
    pub last_reviewed_at_epoch: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for ActivityReviewState {
    fn nexus_table() -> &'static str { "activity_review_states" }
    fn nexus_label() -> &'static str { "Activity Review States" }
    fn nexus_icon() -> &'static str { "🗓️" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "school_id", label: "School", kind: FieldKind::ForeignKey { table: "schools", label_col: "name" }, hidden: false, readonly: true },
            FieldMeta { name: "subject_user_id", label: "Learner", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "course_id", label: "Course", kind: FieldKind::ForeignKey { table: "courses", label_col: "title" }, hidden: false, readonly: true },
            FieldMeta { name: "activity_id", label: "Activity", kind: FieldKind::ForeignKey { table: "activities", label_col: "title" }, hidden: false, readonly: true },
            FieldMeta { name: "algorithm_version", label: "Algorithm", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "repetitions", label: "Repetitions", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "lapses", label: "Lapses", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "ease_milli", label: "Ease (‰)", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "interval_seconds", label: "Interval", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "due_at_epoch", label: "Due Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "last_attempt_key", label: "Last Attempt", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "last_points", label: "Last Points", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "last_max_score", label: "Last Maximum", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "last_reviewed_at_epoch", label: "Last Reviewed", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "created_at", label: "Created At", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "updated_at", label: "Updated At", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
