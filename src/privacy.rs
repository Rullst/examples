//! Public privacy information and notices at the point of collection.

use crate::showcase_nav::{render_shared_styles, render_showcase_footer, render_showcase_nav};
use axum::response::Html;
use rullst::html;

pub fn render_ai_choice(form_id: &str) -> String {
    html! {
        <div class="showcase-ai-privacy">
            <label>
                <input type="checkbox" name="cloud_ai" value="yes" form={form_id} autocomplete="off" />
                <span>"Use cloud AI: send my message to Groq or the configured AI provider."</span>
            </label>
            <span>"Optional. Uncheck to keep future messages local. Avoid personal or confidential data. "
                <a href="/privacy#ai" target="_blank" rel="noopener noreferrer">"Privacy details"</a>
            </span>
        </div>
    }
}

pub async fn privacy_page() -> Html<String> {
    render_page("Privacy notice", include_str!("../templates/privacy.html"))
}

pub async fn cookies_page() -> Html<String> {
    render_page(
        "Cookies & browser storage",
        include_str!("../templates/cookies.html"),
    )
}

fn render_page(title: &str, content: &str) -> Html<String> {
    Html(html! {
        <html lang="en">
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <meta name="htmx-config" content={r#"{"historyCacheSize":0}"#} />
                <title>{title} " · Rullst Showcase"</title>
                <script src="/static/htmx.js"></script>
                <style>{ rullst::html::RawHtml(render_shared_styles()) }</style>
            </head>
            <body hx-history="false">
                { rullst::html::RawHtml(render_showcase_nav("/privacy")) }
                <main class="container showcase-privacy-page">
                    <article class="card">
                        <h1>{title}</h1>
                        { rullst::html::RawHtml(content.to_string()) }
                    </article>
                </main>
                { rullst::html::RawHtml(render_showcase_footer()) }
            </body>
        </html>
    })
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn missing_cloud_choice_uses_local_answers() {
        let payload: crate::ai_demo::ShowcaseChatPayload =
            serde_json::from_str(r#"{"message":"Explain the framework"}"#).unwrap();
        let response = crate::ai_demo::chat_api(axum::Form(payload))
            .await
            .into_response();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("Cloud AI is off"));
        assert!(!body.contains("AI Connection Diagnostics"));
    }

    #[tokio::test]
    async fn public_privacy_pages_use_the_verified_contact_and_disable_html_storage() {
        use tower::ServiceExt;
        let policy =
            rullst_nexus::NexusAuthPolicy::basic("privacy-test", "test-only-long-password")
                .unwrap();
        let app = crate::router_with_nexus_auth(policy).unwrap().into_axum();
        for path in ["/privacy", "/cookies"] {
            let response = app
                .clone()
                .oneshot(
                    axum::http::Request::get(path)
                        .body(axum::body::Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), axum::http::StatusCode::OK);
            assert_eq!(response.headers()["cache-control"], "no-store");
            assert_eq!(response.headers()["referrer-policy"], "no-referrer");
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let body = String::from_utf8(body.to_vec()).unwrap();
            assert!(body.contains("officialrullst@gmail.com"));
            assert!(body.contains("hx-history=\"false\""));
        }
    }
}
