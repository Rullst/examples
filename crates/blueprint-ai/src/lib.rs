//! Shared presentation and bounded, read-only AI for the deployed examples.

pub mod admin;
mod provider;
mod render;

pub use provider::{AiFailure, chat};
pub use render::{STYLES, render_markdown, render_offline_html};

pub const COPILOT_STYLES: &str = include_str!("../static/copilot.css");
pub const COPILOT_SCRIPT: &str = include_str!("../static/copilot.js");
pub const COPILOT_NOTICE: &str = r#"<p class="copilot-notice">AI replies may be incorrect or imprecise. Check the <a href="https://rullst.win/Rullst/book/" target="_blank" rel="noopener noreferrer">official documentation</a>.</p>"#;
pub const COPILOT_EXPAND: &str = r#"<button type="button" class="copilot-expand" data-copilot-expand aria-pressed="false" aria-label="Expand chat to full screen" title="Expand chat to full screen">&#x26F6; Expand</button>"#;
