//! Shared presentation and bounded, read-only AI for the deployed examples.

pub mod admin;
mod provider;
mod render;

pub use provider::{AiFailure, chat};
pub use render::{STYLES, render_markdown, render_offline_html};
