use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "rubric_scores")]
pub struct RubricScore {
    pub id: i32,
    pub assignment_grade_id: i32,
    pub criterion_id: i32,
    pub points_awarded: i32,
    pub feedback: String,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for RubricScore {
    fn nexus_table() -> &'static str { "rubric_scores" }
    fn nexus_label() -> &'static str { "Rubric Scores" }
    fn nexus_icon() -> &'static str { "📊" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "assignment_grade_id", label: "Grade", kind: FieldKind::ForeignKey { table: "assignment_grades", label_col: "grading_key" }, hidden: false, readonly: true },
            FieldMeta { name: "criterion_id", label: "Criterion", kind: FieldKind::ForeignKey { table: "rubric_criteria", label_col: "label" }, hidden: false, readonly: true },
            FieldMeta { name: "points_awarded", label: "Points", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "feedback", label: "Feedback", kind: FieldKind::Textarea, hidden: false, readonly: true },
        ]
    }
}
