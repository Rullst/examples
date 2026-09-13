use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "assignment_grades")]
pub struct AssignmentGrade {
    pub id: i32,
    pub grading_key: String,
    pub assignment_id: i32,
    pub submission_id: i32,
    pub grader_user_id: i32,
    pub subject_user_id: i32,
    pub points_awarded: i32,
    pub max_points: i32,
    pub feedback: String,
    pub ruleset_version: String,
    pub request_json: String,
    pub graded_at_epoch: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for AssignmentGrade {
    fn nexus_table() -> &'static str { "assignment_grades" }
    fn nexus_label() -> &'static str { "Assignment Grades" }
    fn nexus_icon() -> &'static str { "✅" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "grading_key", label: "Grading Key", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "assignment_id", label: "Assignment", kind: FieldKind::ForeignKey { table: "assignments", label_col: "title" }, hidden: false, readonly: true },
            FieldMeta { name: "submission_id", label: "Submission", kind: FieldKind::ForeignKey { table: "assignment_submissions", label_col: "submission_key" }, hidden: false, readonly: true },
            FieldMeta { name: "grader_user_id", label: "Evaluator", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "subject_user_id", label: "Learner", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "points_awarded", label: "Points", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "max_points", label: "Maximum", kind: FieldKind::Number, hidden: false, readonly: true },
            FieldMeta { name: "feedback", label: "Feedback", kind: FieldKind::Textarea, hidden: false, readonly: true },
            FieldMeta { name: "ruleset_version", label: "Ruleset", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "request_json", label: "Canonical Request", kind: FieldKind::Json, hidden: true, readonly: true },
            FieldMeta { name: "graded_at_epoch", label: "Graded Epoch", kind: FieldKind::Number, hidden: false, readonly: true },
        ]
    }
}
