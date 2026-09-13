use rullst::db::{Orm, FromRow};
use rullst::nexus::{NexusModel, FieldMeta, FieldKind};
#[derive(Debug, Clone, FromRow, Orm)]
#[orm(table = "lessons")]
pub struct Lesson {
    pub id: i32,
    pub course_id: i32,
    pub module_id: i32,
    pub title: String,
    pub media_kind: String,
    pub media_url: String,
    pub captions_url: String,
    pub transcript: String,
    pub language_tag: String,
    pub duration: i32,
}
impl NexusModel for Lesson {
    fn nexus_table() -> &'static str { "lessons" }
    fn nexus_label() -> &'static str { "Lessons" }
    fn nexus_icon() -> &'static str { "▶️" }
    fn nexus_fields() -> Vec<FieldMeta> {
        vec![
            FieldMeta { name: "id", label: "ID", kind: FieldKind::Number, hidden: true, readonly: true },
            FieldMeta { name: "course_id", label: "Course", kind: FieldKind::ForeignKey { table: "courses", label_col: "title" }, hidden: false, readonly: false },
            FieldMeta { name: "module_id", label: "Module", kind: FieldKind::ForeignKey { table: "course_modules", label_col: "title" }, hidden: false, readonly: false },
            FieldMeta { name: "title", label: "Title", kind: FieldKind::Text, hidden: false, readonly: false },
            FieldMeta { name: "media_kind", label: "Media Kind", kind: FieldKind::Text, hidden: false, readonly: false },
            FieldMeta { name: "media_url", label: "Media URL", kind: FieldKind::Url, hidden: false, readonly: false },
            FieldMeta { name: "captions_url", label: "Captions URL", kind: FieldKind::Url, hidden: false, readonly: false },
            FieldMeta { name: "transcript", label: "Transcript", kind: FieldKind::Textarea, hidden: false, readonly: false },
            FieldMeta { name: "language_tag", label: "Language", kind: FieldKind::Text, hidden: false, readonly: false },
            FieldMeta { name: "duration", label: "Duration (mins)", kind: FieldKind::Number, hidden: false, readonly: false },
        ]
    }
}
