use rullst::db::{FromRow, Orm};
use rullst::nexus::{FieldKind, FieldMeta, NexusModel};

#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "automation_executions")]
pub struct AutomationExecution {
    pub id: i32,
    pub school_id: i32,
    pub execution_key: String,
    pub rule_id: i32,
    pub source_event_key: String,
    pub actor_user_id: i32,
    pub subject_user_id: i32,
    pub action_kind: String,
    pub outcome: String,
    pub created_at: String,
    pub updated_at: String,
}

impl NexusModel for AutomationExecution {
    fn nexus_table() -> &'static str { "automation_executions" }
    fn nexus_label() -> &'static str { "Automation Executions" }
    fn nexus_icon() -> &'static str { "🧭" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "school_id", label: "School", kind: FieldKind::ForeignKey { table: "schools", label_col: "name" }, hidden: false, readonly: true },
            FieldMeta { name: "execution_key", label: "Execution Key", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "rule_id", label: "Rule", kind: FieldKind::ForeignKey { table: "automation_rules", label_col: "name" }, hidden: false, readonly: true },
            FieldMeta { name: "source_event_key", label: "Source Event", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "actor_user_id", label: "Recorded Actor", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "subject_user_id", label: "Learner", kind: FieldKind::ForeignKey { table: "users", label_col: "email" }, hidden: false, readonly: true },
            FieldMeta { name: "action_kind", label: "Action", kind: FieldKind::Text, hidden: false, readonly: true },
            FieldMeta { name: "outcome", label: "Outcome", kind: FieldKind::Text, hidden: false, readonly: true },
        ]
    }
}
