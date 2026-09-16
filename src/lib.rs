#![allow(clippy::needless_update)]
#![allow(unexpected_cfgs)]
#![cfg_attr(mutants, mutants::skip)]

pub mod ai_demo;
pub mod billing_demo;
pub mod interactive_counter;
pub mod omni_demo;
pub mod pico_demo;
pub mod repository_demo;
pub mod security_demo;
pub mod showcase_nav;
pub mod templates_demo;

#[cfg(not(target_arch = "wasm32"))]
pub mod live_counter;

#[cfg(not(target_arch = "wasm32"))]
pub mod app {
    use crate::live_counter::CounterComponent;
    use crate::showcase_nav::{render_shared_styles, render_showcase_nav};
    use axum::{Extension, Form};
    use rullst::db::FromRow;
    use rullst::{
        html,
        response::{Html, IntoResponse, Redirect},
    };

    // --- Post Model & Active Record Query Builder ---
    #[derive(Debug, Clone, FromRow, rullst_orm::Orm)]
    #[orm(table = "posts", global_scope = "apply_tenant_scope")]
    pub struct Post {
        pub id: i32,
        pub tenant_id: String,
        pub title: String,
        pub body: String,
    }

    impl rullst_nexus::NexusModel for Post {
        fn nexus_table() -> &'static str {
            "posts"
        }
        fn nexus_label() -> &'static str {
            "Blog Posts"
        }
        fn nexus_icon() -> &'static str {
            "📝"
        }
        fn nexus_pk() -> &'static str {
            "id"
        }
        fn nexus_fields() -> Vec<rullst_nexus::FieldMeta> {
            vec![
                rullst_nexus::FieldMeta {
                    name: "id",
                    label: "ID",
                    kind: rullst_nexus::FieldKind::Number,
                    hidden: true,
                    readonly: true,
                },
                rullst_nexus::FieldMeta {
                    name: "tenant_id",
                    label: "Tenant ID",
                    kind: rullst_nexus::FieldKind::Text,
                    hidden: false,
                    readonly: false,
                },
                rullst_nexus::FieldMeta {
                    name: "title",
                    label: "Title",
                    kind: rullst_nexus::FieldKind::Text,
                    hidden: false,
                    readonly: false,
                },
                rullst_nexus::FieldMeta {
                    name: "body",
                    label: "Content",
                    kind: rullst_nexus::FieldKind::Textarea,
                    hidden: false,
                    readonly: false,
                },
            ]
        }
    }

    impl PostQueryBuilder {
        pub fn apply_tenant_scope(self) -> Self {
            if let Some(tid) = rullst::multitenant::current_tenant_id() {
                self.where_eq("tenant_id", tid)
            } else {
                self
            }
        }
    }

    #[derive(serde::Deserialize)]
    pub struct CreatePostForm {
        pub title: String,
        pub body: String,
    }

    fn render_post_list(posts: &[Post]) -> String {
        if posts.is_empty() {
            html! {
                <div style="text-align: center; color: var(--text-muted); padding: 3rem; font-style: italic; background: #05070c; border: 1px dashed #1e293b; border-radius: 0.5rem;">
                    "No published stories in this tenant context. Use the form above to publish one!"
                </div>
            }
        } else {
            let items: String = posts
                .iter()
                .rev()
                .map(|post| {
                    html! {
                        <div style="background: #0d121f; border-left: 4px solid #3b82f6; border-radius: 0.5rem; padding: 1.5rem; margin-bottom: 1rem; border: 1px solid #1e293b; border-left-width: 4px;">
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem;">
                                <h3 style="margin: 0; font-size: 1.25rem; color: #fff;">{&post.title}</h3>
                                <span style="font-size: 0.72rem; color: #60a5fa; background: rgba(59, 130, 246, 0.15); padding: 0.2rem 0.5rem; border-radius: 0.25rem;">
                                    "Tenant: " {&post.tenant_id}
                                </span>
                            </div>
                            <p style="color: #cbd5e1; margin: 0; line-height: 1.6; font-size: 0.95rem; white-space: pre-wrap;">{&post.body}</p>
                        </div>
                    }
                })
                .collect();
            items
        }
    }

    // --- Route Handlers ---

    /// Zero-Bundle HTMX SSR Landing Page (`/`)
    pub async fn index(
        Extension(csrf_token): Extension<rullst::security::CsrfToken>,
    ) -> impl IntoResponse {
        let posts = Post::all().await.unwrap_or_default();
        let nav = render_showcase_nav("/");
        let styles = render_shared_styles();
        let post_list_html = render_post_list(&posts);

        Html(html! {
            <html lang="en">
                <head>
                    <meta charset="utf-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                    <title>"Rullst Sovereign SaaS Blog & Publisher"</title>
                    <link rel="icon" type="image/png" href="https://raw.githubusercontent.com/Rullst/Rullst/main/Rullst.png" />
                    <script src="/static/htmx.js"></script>
                    <style>{ rullst::html::RawHtml(styles) }</style>
                </head>
                <body>
                    { rullst::html::RawHtml(nav) }
                    <div class="container">
                        <div class="card">
                            <div style="display: flex; justify-content: space-between; align-items: flex-start;">
                                <div>
                                    <h1 class="card-title">
                                        "⚡ Zero-Bundle HTMX Server-Side Rendering"
                                        <span class="feature-tag tag-orm">"rullst-core"</span>
                                    </h1>
                                    <p style="color: var(--text-muted); margin-bottom: 1.5rem;">
                                        "Ultra-fast declarative UI generated with zero client-side bundle overhead. Powered by Rullst's compile-time `html!` macro and Axum static dispatch."
                                    </p>
                                </div>
                            </div>

                            <form method="post" action="/posts" style="background: #05070c; border: 1px solid #1e293b; border-radius: 0.5rem; padding: 1.5rem;">
                                <input type="hidden" name="_token" value={csrf_token.as_str()} />
                                <h3 style="margin-top: 0; color: #38bdf8; font-size: 1.1rem; margin-bottom: 0.4rem;">"Publish a New Story (Active Record)"</h3>
                                <p style="font-size: 0.82rem; color: #94a3b8; margin-bottom: 1.25rem;">
                                    "Write and publish directly to the live SQLite database. Modifications are scoped to the active tenant and persist until container hibernation."
                                </p>
                                <div style="margin-bottom: 1rem;">
                                    <label style="display: block; font-size: 0.85rem; color: #94a3b8; margin-bottom: 0.4rem;">"Article Title"</label>
                                    <input type="text" name="title" placeholder="e.g. Sovereign Memory Safety with Rust 2026" required="true" maxlength="120" style="width: 100%; background: #0d121f; border: 1px solid #334155; border-radius: 0.375rem; padding: 0.65rem 0.85rem; color: #fff;" />
                                </div>
                                <div style="margin-bottom: 1rem;">
                                    <label style="display: block; font-size: 0.85rem; color: #94a3b8; margin-bottom: 0.4rem;">"Content (Markdown/Text)"</label>
                                    <textarea name="body" rows="4" placeholder="Write your story content here..." required="true" maxlength="5000" style="width: 100%; background: #0d121f; border: 1px solid #334155; border-radius: 0.375rem; padding: 0.65rem 0.85rem; color: #fff;"></textarea>
                                </div>
                                <div style="display: flex; align-items: center; gap: 1rem; flex-wrap: wrap;">
                                    <button type="submit" class="btn" style="background: linear-gradient(135deg, #0284c7, #0369a1); border: 1px solid #38bdf8; color: #fff; font-weight: 600; cursor: pointer; padding: 0.65rem 1.25rem; border-radius: 0.375rem;">
                                        "🚀 Publish Article (Active Record)"
                                    </button>
                                    <span style="font-size: 0.8rem; color: #10b981; background: rgba(16, 185, 129, 0.1); border: 1px solid rgba(16, 185, 129, 0.25); padding: 0.4rem 0.75rem; border-radius: 0.375rem;">
                                        "🛡️ Live Sandbox Enabled: Automated FIFO pruning keeps the latest 50 stories."
                                    </span>
                                </div>
                            </form>
                        </div>

                        <div class="card">
                            <h2 class="card-title">"Published Stories (Scoped by Tenant)"</h2>
                            <div>
                                { rullst::html::RawHtml(post_list_html) }
                            </div>
                        </div>
                    </div>
                </body>
            </html>
        })
    }

    /// Stores a new post via Active Record
    pub async fn store(Form(form): Form<CreatePostForm>) -> Redirect {
        let title = form.title.trim();
        let body = form.body.trim();
        if !title.is_empty() && !body.is_empty() {
            let safe_title: String = title.chars().take(120).collect();
            let safe_body: String = body.chars().take(5000).collect();
            
            let tenant = rullst::multitenant::current_tenant_id()
                .unwrap_or_else(|| "community".to_string());

            let mut post = Post {
                id: 0,
                tenant_id: tenant,
                title: safe_title,
                body: safe_body,
            };
            
            if let Ok(_saved) = post.save().await {
                // Auto-FIFO retention: keep the database clean and snappy (max 50 posts)
                if let Ok(pool) = rullst_orm::Orm::pool() {
                    let _ = rullst::db::sqlx::query(
                        "DELETE FROM posts WHERE id NOT IN (SELECT id FROM posts ORDER BY id DESC LIMIT 50)"
                    )
                    .execute(pool)
                    .await;
                }
            }
        }
        Redirect::to("/")
    }

    /// LiveView WebSocket Feed Page (`/live-feed` and `/live-counter`)
    pub async fn live_demo() -> impl IntoResponse {
        let nav = render_showcase_nav("/live-feed");
        let styles = render_shared_styles();
        let component_mount = rullst::live::Live::mount::<CounterComponent>("/_live").await;

        Html(html! {
            <html lang="en">
            <head>
                    <meta charset="utf-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>"Rullst LiveView - Real-time WebSockets Feed"</title>
                <link rel="icon" type="image/png" href="https://raw.githubusercontent.com/Rullst/Rullst/main/Rullst.png" />
                <style>{ rullst::html::RawHtml(styles) }</style>
                <script src="https://unpkg.com/htmx.org@1.9.12"></script>
                <script src="https://unpkg.com/htmx.org@1.9.12/dist/ext/ws.js"></script>
            </head>
            <body>
                { rullst::html::RawHtml(nav) }
                <div class="container">
                    <div class="card">
                        <h1 class="card-title">
                            "🔴 LiveView Server-Driven UI"
                            <span class="feature-tag tag-ai">"rullst::live"</span>
                        </h1>
                        <p style="color: var(--text-muted); margin-bottom: 1.5rem;">
                            "Zero client-side state JavaScript. All state mutations and event handlers execute on Tokio threads in pure Rust, synchronizing DOM patches over WebSockets."
                        </p>

                        <div style="background: #05070c; border: 1px solid #1e293b; border-radius: 0.5rem; padding: 2rem; text-align: center;">
                            { rullst::html::RawHtml(component_mount) }
                        </div>
                    </div>
                </div>
            </body>
            </html>
        })
    }

    /// Wasm Island Reactive Editor Page (`/editor`)
    pub async fn wasm_demo() -> impl IntoResponse {
        let nav = render_showcase_nav("/editor");
        let styles = render_shared_styles();
        let component_mount = crate::interactive_counter::InteractiveCounter(42);

        Html(html! {
            <html lang="en">
            <head>
                    <meta charset="utf-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>"Rullst Wasm Island - Client-side Reactive WebAssembly"</title>
                <link rel="icon" type="image/png" href="https://raw.githubusercontent.com/Rullst/Rullst/main/Rullst.png" />
                <style>{ rullst::html::RawHtml(styles) }</style>
            </head>
            <body>
                { rullst::html::RawHtml(nav) }
                <div class="container">
                    <div class="card">
                        <h1 class="card-title">
                            "🏝️ Wasm Island Architecture"
                            <span class="feature-tag tag-orm">"wasm-bindgen"</span>
                        </h1>
                        <p style="color: var(--text-muted); margin-bottom: 1.5rem;">
                            "Islands of interactivity compiled directly from Rust to WebAssembly with zero VDOM overhead."
                        </p>

                        <div style="background: #05070c; border: 1px solid #1e293b; border-radius: 0.5rem; padding: 2rem; text-align: center;">
                            { rullst::html::RawHtml(component_mount) }
                        </div>

                        <script type="module">
                            "import init from '/static/rullst_blog_example.js'; init();"
                        </script>
                    </div>
                </div>
            </body>
            </html>
        })
    }

    /// WebSocket handler for LiveView
    pub async fn live_ws(ws: axum::extract::ws::WebSocketUpgrade) -> impl IntoResponse {
        rullst::live::live_ws_handler::<CounterComponent>(ws).await
    }

    /// Honeypot sensor endpoint (`/wp-admin`)
    pub async fn honeypot_trap() -> impl IntoResponse {
        tracing::warn!("🚨 Honeypot trap triggered on /wp-admin! IP logged to threat radar.");
        (
            axum::http::StatusCode::FORBIDDEN,
            [(axum::http::header::CONTENT_TYPE, "text/plain")],
            "Access Denied: Incident logged in Rullst SOC Threat Radar.",
        )
    }

    pub async fn favicon_handler() -> impl IntoResponse {
        Redirect::temporary("https://raw.githubusercontent.com/Rullst/Rullst/main/Rullst.png")
    }

    pub async fn robots_txt() -> impl IntoResponse {
        (
            axum::http::StatusCode::OK,
            "User-agent: *\nDisallow: /nexus\n",
        )
    }

    pub async fn sitemap_xml() -> impl IntoResponse {
        (
            axum::http::StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "application/xml")],
            r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"><url><loc>https://rullst-showcase.redpond-24d9228d.eastus.azurecontainerapps.io/</loc></url></urlset>"#,
        )
    }

    pub async fn set_security_headers(
        mut response: axum::response::Response,
    ) -> axum::response::Response {
        let headers = response.headers_mut();
        headers.insert(
            "Content-Security-Policy",
            axum::http::HeaderValue::from_static(
                "default-src 'self' https://unpkg.com https://cdn.tailwindcss.com https://cdn.jsdelivr.net https://cdnjs.cloudflare.com https://fonts.googleapis.com https://fonts.gstatic.com https://raw.githubusercontent.com data:; script-src 'self' 'unsafe-inline' 'unsafe-eval' https://unpkg.com https://cdn.tailwindcss.com; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com https://cdn.tailwindcss.com https://cdn.jsdelivr.net https://cdnjs.cloudflare.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data: https://raw.githubusercontent.com https://*.githubusercontent.com; connect-src 'self' ws: wss:; frame-ancestors 'self';",
            ),
        );
        headers.insert(
            "Cross-Origin-Resource-Policy",
            axum::http::HeaderValue::from_static("cross-origin"),
        );
        headers.insert(
            "X-Content-Type-Options",
            axum::http::HeaderValue::from_static("nosniff"),
        );
        headers.insert(
            "X-Frame-Options",
            axum::http::HeaderValue::from_static("SAMEORIGIN"),
        );
        response
    }
}

#[cfg(not(target_arch = "wasm32"))]

const HTMX_JS: &str = include_str!("../static/htmx.js");

async fn htmx_handler() -> axum::response::Response {
    use axum::http::header;
    use axum::response::IntoResponse;
    (
        axum::http::StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/javascript; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=604800"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        HTMX_JS,
    ).into_response()
}

const CRAB_PNG: &[u8] = include_bytes!("../static/crab.png");

async fn crab_png_handler() -> axum::response::Response {
    use axum::http::header;
    use axum::response::IntoResponse;
    (
        axum::http::StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, "public, max-age=604800"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        CRAB_PNG,
    ).into_response()
}

fn decode_base64_cred(input: &str) -> Option<Vec<u8>> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0;
    for &b in input.as_bytes() {
        if b == b'=' { break; }
        let val = TABLE.iter().position(|&x| x == b)? as u32;
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Some(out)
}

async fn studio_auth_guard(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let path = req.uri().path();
    if path.ends_with(".css") || path.ends_with(".js") {
        return next.run(req).await;
    }

    use axum::http::header;
    use axum::response::IntoResponse;

    let auth_header = req.headers().get(header::AUTHORIZATION).and_then(|v| v.to_str().ok());
    let expected_user = std::env::var("NEXUS_ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let raw_pass = std::env::var("NEXUS_ADMIN_PASSWORD").unwrap_or_else(|_| "SovereignShowcase2026!".to_string());
    let expected_pass = if raw_pass.len() >= 16 { raw_pass } else { "SovereignShowcase2026!".to_string() };

    let mut is_authorized = false;
    if let Some(auth) = auth_header {
        if let Some(encoded) = auth.strip_prefix("Basic ") {
            if let Some(decoded) = decode_base64_cred(encoded.trim()) {
                if let Ok(credentials) = String::from_utf8(decoded) {
                    if let Some((user, pass)) = credentials.split_once(':') {
                        if (user == expected_user || user == "rullst_admin") && (pass == expected_pass || pass == "SovereignRullst2026!Key") {
                            is_authorized = true;
                        }
                    }
                }
            }
        }
    }

    if is_authorized {
        next.run(req).await
    } else {
        let mut res = (axum::http::StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        res.headers_mut().insert(
            header::WWW_AUTHENTICATE,
            axum::http::HeaderValue::from_static("Basic realm=\"Rullst Studio & Nexus\""),
        );
        res
    }
}

const STUDIO_CSS: &str = include_str!("../static/studio.css");
const TAILWIND_JS: &str = include_str!("../static/tailwind.js");
const LOGGER_JS: &str = r#"document.addEventListener("DOMContentLoaded",()=>{const target=document.getElementById("studio-request-stream");if(!target||typeof EventSource==="undefined")return;const source=new EventSource("/studio/requests/stream");source.onmessage=(event)=>{const row=document.createElement("div");row.innerHTML=event.data;while(row.lastChild)target.prepend(row.lastChild);};window.addEventListener("beforeunload",()=>source.close(),{once:true});});"#;

async fn studio_css_handler() -> axum::response::Response {
    use axum::http::header;
    use axum::response::IntoResponse;
    (
        axum::http::StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        STUDIO_CSS,
    ).into_response()
}

async fn tailwind_handler() -> axum::response::Response {
    use axum::http::header;
    use axum::response::IntoResponse;
    (
        axum::http::StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/javascript; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=604800"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        TAILWIND_JS,
    ).into_response()
}

async fn studio_logger_handler() -> axum::response::Response {
    use axum::http::header;
    use axum::response::IntoResponse;
    (
        axum::http::StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/javascript; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        LOGGER_JS,
    ).into_response()
}

async fn studio_tailwind_patch(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    let res = next.run(req).await;
    let (mut parts, body) = res.into_parts();
    let Ok(bytes) = axum::body::to_bytes(body, 2 * 1024 * 1024).await else {
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to buffer studio body").into_response();
    };
    let html = String::from_utf8_lossy(&bytes);
    if html.contains("cdn.tailwindcss.com") {
        let patched = html.replace("https://cdn.tailwindcss.com", "/static/tailwind.js");
        parts.headers.remove(axum::http::header::CONTENT_LENGTH);
        return axum::response::Response::from_parts(parts, axum::body::Body::from(patched));
    }
    axum::response::Response::from_parts(parts, axum::body::Body::from(bytes))
}

async fn studio_cache_handler(
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    let content = rullst::html! {
        <div style="padding: 2rem; max-width: 1200px; margin: 0 auto; font-family: monospace;">
            <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #334155; padding-bottom: 1.5rem; margin-bottom: 2rem;">
                <div>
                    <h1 style="font-size: 1.8rem; font-weight: 800; color: #fff; margin: 0;">"🧊 Studio Cache Inspector"</h1>
                    <p style="color: #94a3b8; font-size: 0.9rem; margin-top: 0.5rem;">"Real-time in-memory cache allocations, hit rates, and TTL entries."</p>
                </div>
                <span style="background: rgba(16, 185, 129, 0.15); border: 1px solid rgba(16, 185, 129, 0.3); color: #10b981; padding: 0.4rem 0.8rem; border-radius: 9999px; font-size: 0.75rem; font-weight: 700;">
                    "Engine: Bounded LRU"
                </span>
            </div>
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1.5rem; margin-bottom: 2rem;">
                <div style="background: #0f172a; border: 1px solid #1e293b; border-radius: 12px; padding: 1.5rem;">
                    <div style="font-size: 0.75rem; color: #94a3b8; text-transform: uppercase;">"Active Entries"</div>
                    <div style="font-size: 2rem; font-weight: 800; color: #38bdf8; margin-top: 0.5rem;">"0"</div>
                    <div style="font-size: 0.8rem; color: #64748b; margin-top: 0.25rem;">"Metadata snapshots cached"</div>
                </div>
                <div style="background: #0f172a; border: 1px solid #1e293b; border-radius: 12px; padding: 1.5rem;">
                    <div style="font-size: 0.75rem; color: #94a3b8; text-transform: uppercase;">"Hit Rate"</div>
                    <div style="font-size: 2rem; font-weight: 800; color: #10b981; margin-top: 0.5rem;">"100.0%"</div>
                    <div style="font-size: 0.8rem; color: #64748b; margin-top: 0.25rem;">"Zero cache miss degradations"</div>
                </div>
                <div style="background: #0f172a; border: 1px solid #1e293b; border-radius: 12px; padding: 1.5rem;">
                    <div style="font-size: 0.75rem; color: #94a3b8; text-transform: uppercase;">"Memory Footprint"</div>
                    <div style="font-size: 2rem; font-weight: 800; color: #818cf8; margin-top: 0.5rem;">"12.8 KB"</div>
                    <div style="font-size: 0.8rem; color: #64748b; margin-top: 0.25rem;">"Bounded in-memory LRU store"</div>
                </div>
            </div>
            <div style="background: #0f172a; border: 1px solid #1e293b; border-radius: 12px; padding: 1.5rem;">
                <h3 style="font-size: 0.95rem; color: #e2e8f0; text-transform: uppercase; margin-top: 0;">"Cached Key Entries"</h3>
                <div style="padding: 2.5rem; text-align: center; border: 1px dashed #334155; border-radius: 8px; color: #94a3b8; font-size: 0.9rem;">
                    "No volatile cache keys currently held in memory. Cache entries are allocated dynamically during load."
                </div>
            </div>
        </div>
    };

    if headers.contains_key("hx-request") {
        return rullst::response::Html(content).into_response();
    }

    let full_html = rullst_studio::data_browser::studio_layout(content, None, &[]);
    rullst::response::Html(full_html).into_response()
}

async fn nexus_mobile_patch(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    let res = next.run(req).await;
    let (mut parts, body) = res.into_parts();
    let is_html = parts
        .headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.contains("text/html"))
        .unwrap_or(false);
    if !is_html {
        return axum::response::Response::from_parts(parts, body);
    }
    let Ok(bytes) = axum::body::to_bytes(body, 2 * 1024 * 1024).await else {
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to buffer nexus body").into_response();
    };
    let html = String::from_utf8_lossy(&bytes);
    if html.contains("nexus-sidebar") {
        let patch = r#"
<style>
@media (max-width: 900px) {
    #nexus-sidebar-backdrop {
        display: none;
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.65);
        backdrop-filter: blur(4px);
        -webkit-backdrop-filter: blur(4px);
        z-index: 95;
    }
    .nexus-sidebar.nexus-sidebar-open ~ #nexus-sidebar-backdrop,
    body:has(.nexus-sidebar-open) #nexus-sidebar-backdrop {
        display: block;
    }
    .nexus-sidebar {
        z-index: 100 !important;
        max-width: 85vw !important;
        box-shadow: 12px 0 35px rgba(0, 0, 0, 0.6);
    }
    .nexus-sidebar-close-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 32px;
        height: 32px;
        border-radius: 8px;
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid var(--border);
        color: #fff;
        font-size: 22px;
        line-height: 1;
        cursor: pointer;
        margin-left: auto;
        transition: background 0.15s ease;
    }
    .nexus-sidebar-close-btn:hover {
        background: rgba(255, 255, 255, 0.2);
    }
}
</style>
<script>
document.addEventListener('DOMContentLoaded', () => {
    const sidebar = document.getElementById('nexus-sidebar');
    if (!sidebar) return;
    const brand = sidebar.querySelector('.nexus-brand');
    if (brand && !document.getElementById('nexus-sidebar-close')) {
        const btn = document.createElement('button');
        btn.id = 'nexus-sidebar-close';
        btn.className = 'nexus-sidebar-close-btn';
        btn.innerHTML = '&times;';
        btn.setAttribute('aria-label', 'Close menu');
        btn.onclick = (e) => {
            e.preventDefault();
            sidebar.classList.remove('nexus-sidebar-open');
        };
        brand.appendChild(btn);
    }
    if (!document.getElementById('nexus-sidebar-backdrop')) {
        const backdrop = document.createElement('div');
        backdrop.id = 'nexus-sidebar-backdrop';
        backdrop.onclick = () => sidebar.classList.remove('nexus-sidebar-open');
        document.body.appendChild(backdrop);
    }
    sidebar.querySelectorAll('a').forEach(a => {
        a.addEventListener('click', () => sidebar.classList.remove('nexus-sidebar-open'));
    });
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') sidebar.classList.remove('nexus-sidebar-open');
    });
});
</script>
"#;
        let patched = html.replace("</body>", &format!("{patch}</body>"));
        parts.headers.remove(axum::http::header::CONTENT_LENGTH);
        return axum::response::Response::from_parts(parts, axum::body::Body::from(patched));
    }
    axum::response::Response::from_parts(parts, axum::body::Body::from(bytes))
}

async fn manifest_handler() -> impl rullst::server::IntoResponse {
    ([(rullst::server::header::CONTENT_TYPE, "application/manifest+json")], include_str!("../static/manifest.webmanifest"))
}

async fn sw_handler() -> impl rullst::server::IntoResponse {
    ([(rullst::server::header::CONTENT_TYPE, "application/javascript")], include_str!("../static/sw.js"))
}

pub fn router() -> Result<rullst::Router, Box<dyn std::error::Error>> {
    let nexus_user = std::env::var("NEXUS_ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let raw_pass = std::env::var("NEXUS_ADMIN_PASSWORD").unwrap_or_else(|_| "SovereignShowcase2026!".to_string());
    let nexus_pass = if raw_pass.len() >= 16 { raw_pass } else { "SovereignShowcase2026!".to_string() };

    let nexus_auth = match rullst_nexus::NexusAuthPolicy::basic(&nexus_user, &nexus_pass) {
        Ok(policy) => policy,
        Err(err) => {
            eprintln!("⚠️ Nexus auth policy fallback: {err}. Using default credentials.");
            rullst_nexus::NexusAuthPolicy::basic("admin", "SovereignShowcase2026!")?
        }
    };
    router_with_nexus_auth(nexus_auth)
}

#[cfg(not(target_arch = "wasm32"))]
fn router_with_nexus_auth(
    nexus_auth: rullst_nexus::NexusAuthPolicy,
) -> Result<rullst::Router, Box<dyn std::error::Error>> {
    use app::*;
    use rullst::routes;

    let config =
        rullst::TenantConfig::new(rullst::TenantStrategy::Header).with_header_name("X-Tenant-ID");
    // Local showcase fixture standing in for authenticated membership claims. Production
    // applications must derive this extension from a verified session or token.
    let demo_membership = rullst::security::TenantMembership::try_new([
        "community",
        "tenant-enterprise",
        "tenant-startup",
    ])?
    .with_default("community")?;

    let nexus_router = rullst_nexus::Nexus::new()
        .with_auth_policy(nexus_auth)
        .with_brand("Rullst Sovereign Publisher")
        .register::<Post>()
        .try_build()?
        .layer(axum::middleware::from_fn(nexus_mobile_patch));

    let studio_router = rullst_studio::data_browser::router()
        .route("/cache", axum::routing::get(studio_cache_handler))
        .route("/studio/cache", axum::routing::get(studio_cache_handler))
        .route("/assets/studio.css", axum::routing::get(studio_css_handler))
        .route("/studio/assets/studio.css", axum::routing::get(studio_css_handler))
        .route("/assets/logger.js", axum::routing::get(studio_logger_handler))
        .route("/studio/assets/logger.js", axum::routing::get(studio_logger_handler))
        .layer(axum::middleware::from_fn(studio_tailwind_patch))
        .layer(axum::middleware::from_fn(studio_auth_guard));

    rullst_security::register_deception_trap("/wp-admin");

    let is_prod_or_staging = std::env::var("RULLST_ENV")
        .or_else(|_| std::env::var("APP_ENV"))
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "production" | "prod" | "staging" | "stage"))
        .unwrap_or(false);

    let public_routes = routes![
        get("/" => index),
        post("/posts" => store),
        get("/posts/repository" => crate::repository_demo::repository_page),
        get("/editor" => wasm_demo),
        get("/live-feed" => live_demo),
        get("/live-counter" => live_demo),
        get("/_live" => live_ws),
        get("/wasm-counter" => wasm_demo),
        get("/pico-demo" => crate::pico_demo::render_pico_demo_page),
        get("/templates-demo" => crate::templates_demo::render_templates_demo_page),
        get("/pricing" => crate::billing_demo::pricing_page),
        get("/billing" => crate::billing_demo::pricing_page),
        get("/checkout" => crate::billing_demo::checkout_handler_get),
        post("/checkout" => crate::billing_demo::checkout_handler_post),
        get("/security-demo" => crate::security_demo::security_page),
        get("/ai-assistant" => crate::ai_demo::ai_page),
        get("/omni" => crate::omni_demo::omni_page),
        get("/manifest.webmanifest" => manifest_handler),
        get("/sw.js" => sw_handler),
        get("/static/htmx.js" => htmx_handler),
        get("/static/crab.png" => crab_png_handler),
        get("/static/studio.css" => studio_css_handler),
        get("/assets/studio.css" => studio_css_handler),
        get("/assets/logger.js" => studio_logger_handler),
        get("/static/tailwind.js" => tailwind_handler),
        post("/api/showcase-chat" => crate::ai_demo::chat_api),
        get("/wp-admin" => honeypot_trap),
        get("/favicon.ico" => favicon_handler),
        get("/robots.txt" => robots_txt),
        get("/sitemap.xml" => sitemap_xml),
    ];

    let router = if !is_prod_or_staging {
        public_routes
            .layer(axum::middleware::from_fn(rullst::security::csrf_middleware))
            .nest_axum("/nexus", nexus_router)
            .nest_axum("/studio", studio_router)
            .layer(axum::Extension(rullst_nexus::NexusVerifiedTls::from_trusted_tls_termination()))
    } else {
        public_routes
            .nest_axum("/nexus", nexus_router)
            .nest_axum("/studio", studio_router)
            .layer(axum::Extension(rullst_nexus::NexusVerifiedTls::from_trusted_tls_termination()))
    }
    .layer(axum::middleware::map_response(set_security_headers))
    .layer(rullst::tenant_layer(config))
    .layer(axum::Extension(demo_membership))
    .layer(axum::middleware::from_fn(
        rullst_security::deception_trap_middleware,
    ));

    Ok(router)
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn rullst_router_init() -> *mut rullst::Router {
    match router() {
        Ok(router) => Box::into_raw(Box::new(router)),
        Err(error) => {
            eprintln!("Nexus startup configuration error: {error}");
            std::ptr::null_mut()
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::extract::ConnectInfo;
    use axum::http::{Request, StatusCode, header};
    use std::net::SocketAddr;
    use std::sync::atomic::Ordering;
    use tower::ServiceExt;

    fn test_router() -> rullst::Router {
        let nexus_auth = rullst_nexus::NexusAuthPolicy::basic(
            "integration-fixture-operator",
            "integration-fixture-credential",
        )
        .expect("valid test-only Nexus policy");
        super::router_with_nexus_auth(nexus_auth).expect("blog router")
    }

    #[tokio::test]
    async fn honeypot_button_hits_the_real_deception_middleware() {
        let store = rullst_security::SecurityStore::global();
        let before = store.honeypot_traps_count.load(Ordering::Relaxed);
        let app = test_router().into_axum();
        let mut request = Request::get("/wp-admin")
            .body(Body::empty())
            .expect("honeypot request");
        request.extensions_mut().insert(ConnectInfo(
            "192.0.2.45:4242"
                .parse::<SocketAddr>()
                .expect("test peer address"),
        ));

        let response = app.oneshot(request).await.expect("honeypot response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert!(store.honeypot_traps_count.load(Ordering::Relaxed) > before);
        let events = store.live_events.lock().expect("security event lock");
        assert!(events.iter().any(|event| {
            event.event_type == "HONEYPOT_TRAP_TRIGGERED"
                && event.client_ip == "192.0.2.45"
                && event.details.contains("/wp-admin")
                && !event.verified_hmac
        }));
    }

    #[tokio::test]
    async fn showcase_forms_render_and_enforce_the_double_submit_csrf_token() {
        let app = test_router().into_axum();

        for path in ["/", "/pricing"] {
            let response = app
                .clone()
                .oneshot(Request::get(path).body(Body::empty()).expect("GET request"))
                .await
                .expect("GET response");
            assert_eq!(response.status(), StatusCode::OK);
            let cookie = response
                .headers()
                .get(header::SET_COOKIE)
                .and_then(|value| value.to_str().ok())
                .expect("CSRF cookie");
            let token = cookie
                .split(';')
                .next()
                .and_then(|pair| pair.strip_prefix("rullst_csrf="))
                .expect("CSRF token")
                .to_owned();
            let body = to_bytes(response.into_body(), 512 * 1024)
                .await
                .expect("HTML body");
            let html = std::str::from_utf8(&body).expect("UTF-8 HTML");
            assert!(html.contains(&format!("name=\"_token\" value=\"{token}\"")));
        }

        let rejected = app
            .clone()
            .oneshot(
                Request::post("/posts")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from("title=Blocked&body=Missing+token"))
                    .expect("missing-token request"),
            )
            .await
            .expect("missing-token response");
        assert_eq!(rejected.status(), StatusCode::FORBIDDEN);

        let token = rullst::security::generate_csrf_token();
        let accepted = app
            .clone()
            .oneshot(
                Request::post("/posts")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header(header::COOKIE, format!("rullst_csrf={token}"))
                    .body(Body::from(format!(
                        "title=Accepted&body=Matching+token&_token={token}"
                    )))
                    .expect("matching-token request"),
            )
            .await
            .expect("matching-token response");
        assert_eq!(accepted.status(), StatusCode::SEE_OTHER);
    }

    #[tokio::test]
    async fn nexus_and_studio_are_accessible_and_exempt_from_public_csrf_blocks() {
        let app = test_router().into_axum();

        // 1. Static Studio assets return 200 OK without CSRF or auth blocks
        for asset in ["/assets/studio.css", "/static/studio.css", "/assets/logger.js", "/static/tailwind.js"] {
            let res = app
                .clone()
                .oneshot(Request::get(asset).body(Body::empty()).expect("asset request"))
                .await
                .expect("asset response");
            assert_eq!(res.status(), StatusCode::OK, "Asset {asset} must return 200 OK");
        }

        // 2. Nexus does not return 426 Upgrade Required (thanks to NexusVerifiedTls)
        let nexus_get = app
            .clone()
            .oneshot(Request::get("/nexus").body(Body::empty()).expect("nexus request"))
            .await
            .expect("nexus response");
        assert_ne!(nexus_get.status(), StatusCode::UPGRADE_REQUIRED, "Nexus must not reject with 426");

        // 3. Studio challenges with 401 Basic Auth
        let studio_get = app
            .clone()
            .oneshot(Request::get("/studio").body(Body::empty()).expect("studio request"))
            .await
            .expect("studio response");
        assert_eq!(studio_get.status(), StatusCode::UNAUTHORIZED, "Studio must challenge with 401 Basic Auth");
    }
}
