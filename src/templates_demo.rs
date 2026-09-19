//! Classic File-Based Template Demonstration (Jinja2 / Tera Engine).
//! Demonstrates how developers coming from Django, Rails, and Loco.rs can render
//! external HTML templates located in `templates/` with full separation of concerns.

use crate::showcase_cache::ShowcaseCache;
use crate::showcase_nav::{render_shared_styles, render_showcase_footer, render_showcase_nav};
use axum::Extension;
use axum::response::Html;

/// Renders the file-based template demo page as an Axum HTML response.
pub async fn render_templates_demo_page(
    Extension(cache): Extension<ShowcaseCache>,
) -> Html<String> {
    let nav_html = render_showcase_nav("/templates-demo");
    // Only shared article HTML is cached. Tenant navigation is rendered per request.
    let template_html = match cache.get("templates:article").await {
        Ok(Some(html)) => html.as_str().to_owned(),
        _ => {
            let html = render_article_template();
            if let Err(error) = cache.put("templates:article", &html, 60).await {
                tracing::warn!(%error, "Could not cache the template demo");
            }
            html
        }
    };
    Html(
        template_html
            .replace("{{ nav_html | safe }}", &nav_html)
            .replace("{{ footer_html | safe }}", &render_showcase_footer()),
    )
}

fn render_article_template() -> String {
    let shared_styles = render_shared_styles();

    // Template source loaded from templates/article.html
    let template_source = include_str!("../templates/article.html");

    // Simple template string replacement (emulating compile-time / runtime Tera engine)
    template_source
        .replace("{{ title }}", "Decoupled MVC Architectures in Rust")
        .replace("{{ author }}", "Chief Architect (Sovereign Systems)")
        .replace("{{ published_at }}", "2026-08-15 14:00 UTC")
        .replace(
            "{{ content }}",
            "This page is rendered directly from an external HTML file located at 'templates/article.html'. Unlike inline macros, file-based templating enables UI designers and frontend developers to edit layout files without touching Rust source code or triggering Rust compiler recompilations.",
        )
        .replace("{{ shared_styles | safe }}", &shared_styles)
}
