use pulldown_cmark::{Event, Options, Parser, html};

pub const STYLES: &str = include_str!("../static/reply.css");

/// Model output is untrusted. Escape raw HTML before parsing/sanitizing and
/// allow only presentation tags: no images, forms, styles, HTMX or scripts.
pub fn render_markdown(reply: &str) -> String {
    let bounded: String = reply.chars().take(16_000).collect();
    let parser = Parser::new_ext(
        &bounded,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH,
    )
    .map(|event| match event {
        Event::Html(text) | Event::InlineHtml(text) => Event::Text(text),
        other => other,
    });
    let mut rendered = String::new();
    html::push_html(&mut rendered, parser);
    render_offline_html(&rendered)
}

/// Sanitize the examples' HTML fallback templates too: interpolated database
/// fields are untrusted even when a model is offline.
pub fn render_offline_html(rendered: &str) -> String {
    let clean = ammonia::Builder::default()
        .tags(
            [
                "p",
                "br",
                "strong",
                "em",
                "del",
                "h1",
                "h2",
                "h3",
                "h4",
                "h5",
                "h6",
                "ul",
                "ol",
                "li",
                "blockquote",
                "pre",
                "code",
                "a",
                "hr",
                "table",
                "thead",
                "tbody",
                "tr",
                "th",
                "td",
            ]
            .into_iter()
            .collect(),
        )
        .generic_attributes(Default::default())
        .tag_attributes(
            [
                ("a", ["href", "title"].into_iter().collect()),
                ("ol", ["start"].into_iter().collect()),
            ]
            .into_iter()
            .collect(),
        )
        .url_schemes(["https", "http"].into_iter().collect())
        .link_rel(Some("noopener noreferrer nofollow"))
        .clean(rendered)
        .to_string();
    // Include scoped styles in HTMX fragments too; all three public chats use it.
    format!("<style>{STYLES}</style><div class=\"rullst-ai-prose\">{clean}</div>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_the_reported_markdown_and_code() {
        let html = render_markdown(
            "### Para que serve?\n\n- **Criar aplicações** com `Rust`.\n- Aprender.\n\n```rust\nlet x = \"<script>\";\n```\n\n| A | B |\n|---|---|\n| 1 | 2 |",
        );
        for expected in [
            "<h3>Para que serve?</h3>",
            "<strong>Criar aplicações</strong>",
            "<ul>",
            "<code>Rust</code>",
            "<pre><code>",
            "&lt;script&gt;",
            "<table>",
        ] {
            assert!(html.contains(expected), "missing {expected}: {html}");
        }
        assert!(!html.contains("### Para"));
    }

    #[test]
    fn rejects_active_html_tracking_and_unsafe_links() {
        let html = render_markdown(
            "<script>alert(1)</script>\n\n<img src=x onerror=alert(1)>\n\n[click](javascript:alert%281%29) ![track](https://evil.test/pixel)\n\n<a hx-post='/nexus/delete' onclick='alert(1)'>x</a>\n\n[safe](https://example.org)",
        );
        for forbidden in [
            "<script",
            "<img",
            "<a hx-",
            "href=\"javascript:",
            "href=\"data:",
        ] {
            assert!(!html.contains(forbidden), "unsafe markup: {forbidden}");
        }
        assert!(html.contains("href=\"https://example.org\""));
        assert!(html.contains("noopener noreferrer nofollow"));
    }

    #[test]
    fn sanitizes_database_content_in_offline_templates() {
        let html = render_offline_html(
            "<p><strong>Course</strong></p><img src=x onerror=alert(1)><script>alert(1)</script><a href='javascript:alert(1)' hx-post='/nexus/delete'>click</a>",
        );
        assert!(html.contains("<strong>Course</strong>"));
        for forbidden in ["<img", "<script", "onerror", "javascript:", "hx-post"] {
            assert!(
                !html.contains(forbidden),
                "unsafe offline content: {forbidden}"
            );
        }
    }
}
