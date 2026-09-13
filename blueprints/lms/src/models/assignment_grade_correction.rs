use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "assignment_grade_corrections")]
pub struct AssignmentGradeCorrection {
    pub id: i32,
    pub correction_key: String,
    pub assignment_grade_id: i32,
    pub actor_user_id: i32,
    pub previous_points: i32,
    pub corrected_points: i32,
    pub max_points: i32,
    pub reason: String,
    pub scores_json: String,
    pub request_json: String,
    pub corrected_at_epoch: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for AssignmentGradeCorrection {
    fn nexus_table() -> &'static str { "assignment_grade_corrections" }
    fn nexus_label() -> &'static str { "Assignment Grade Corrections" }
    fn nexus_icon() -> &'static str { "🧾" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "correction_key", label: "Correction Key", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "assignment_grade_id", label: "Grade", kind: FieldKind::ForeignKey { table: "assignment_grades", label_col: "grading_key" }, hidden: false, readonly: true },
            FieldMeta { name: "actor_user_id", label: "Administrator", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "previous_points", label: "Before", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "corrected_points", label: "After", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "max_points", label: "Maximum", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "reason", label: "Reason", kind: FieldKind::Textarea, hidden: false, readonly: true },
            FieldMeta { name: "scores_json", label: "Criterion Scores", kind: FieldKind::Json, hidden: false, readonly: true },
            FieldMeta { name: "request_json", label: "Canonical Request", kind: FieldKind::Json, hidden: true, readonly: true },
            FieldMeta { name: "corrected_at_epoch", label: "Corrected Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
        ]
    }
}
