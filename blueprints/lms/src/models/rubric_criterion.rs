use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "rubric_criteria")]
pub struct RubricCriterion {
    pub id: i32,
    pub assignment_id: i32,
    pub criterion_key: String,
    pub label: String,
    pub max_points: i32,
    pub position: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for RubricCriterion {
    fn nexus_table() -> &'static str { "rubric_criteria" }
    fn nexus_label() -> &'static str { "Rubric Criteria" }
    fn nexus_icon() -> &'static str { "📏" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "assignment_id", label: "Assignment", kind: FieldKind::ForeignKey { table: "assignments", label_col: "title" }, hidden: false, readonly: true },
            FieldMeta { name: "criterion_key", label: "Criterion Key", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "label", label: "Label", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "max_points", label: "Max Points", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "position", label: "Position", kind: FieldKind::Number, hidden: false, readonly: true },
        ]
    }
}
