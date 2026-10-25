use rullst::{Server, routes};

pub mod controllers;
pub mod migrations;
pub mod models;
pub mod pages;
pub mod showcase;

#[rullst::runtime::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rullst::artisan!(crate::migrations::get_migrations());

    #[cfg(debug_assertions)]
    {
        rullst::runtime::spawn(async {
            if let Err(error) = rullst::studio::run_studio(5555).await {
                eprintln!("Rullst Studio could not start: {error}");
            }
        });
        println!("📊 Rullst Studio running on http://127.0.0.1:5555");
    }

    const HTMX_JS: &str = include_str!("../static/htmx-1.9.12.min.js");

    async fn htmx_handler() -> rullst::server::Response {
        use rullst::server::IntoResponse;
        use rullst::server::header;
        (
            rullst::server::StatusCode::OK,
            [
                (
                    header::CONTENT_TYPE,
                    "application/javascript; charset=utf-8",
                ),
                (header::CACHE_CONTROL, "public, max-age=604800"),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            ],
            HTMX_JS,
        )
            .into_response()
    }

    const CRAB_PNG: &[u8] = include_bytes!("../static/crab.png");

    async fn crab_png_handler() -> rullst::server::Response {
        use rullst::server::IntoResponse;
        use rullst::server::header;
        (
            rullst::server::StatusCode::OK,
            [
                (header::CONTENT_TYPE, "image/png"),
                (header::CACHE_CONTROL, "public, max-age=604800"),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            ],
            CRAB_PNG,
        )
            .into_response()
    }

    async fn nexus_mobile_patch(
        req: rullst::server::Request,
        next: rullst::server::Next,
    ) -> rullst::server::Response {
        use rullst::server::IntoResponse;
        let res = next.run(req).await;
        let (mut parts, body) = res.into_parts();
        let is_html = parts
            .headers
            .get(rullst::server::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|ct| ct.contains("text/html"))
            .unwrap_or(false);
        if !is_html {
            return rullst::server::Response::from_parts(parts, body);
        }
        let Ok(bytes) = axum::body::to_bytes(body, 2 * 1024 * 1024).await else {
            return (
                rullst::server::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to buffer nexus body",
            )
                .into_response();
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
            parts.headers.remove(rullst::server::header::CONTENT_LENGTH);
            return rullst::server::Response::from_parts(parts, axum::body::Body::from(patched));
        }
        rullst::server::Response::from_parts(parts, axum::body::Body::from(bytes))
    }

    // The same explicit administrator policy protects Nexus, Studio and their AI.
    let nexus_user = std::env::var("NEXUS_ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let nexus_pass = std::env::var("NEXUS_ADMIN_PASSWORD")?;
    let nexus_auth = rullst::nexus::NexusAuthPolicy::basic(nexus_user, nexus_pass)?;

    let nexus = rullst::nexus::Nexus::new()
        .with_auth_policy(nexus_auth.clone())
        .with_brand("Portfolio CMS Admin")
        .register::<models::profile::Profile>()
        .register::<models::project::Project>()
        .register::<models::experience::Experience>()
        .register::<models::skill::Skill>()
        .try_build()?
        .layer(rullst::server::from_fn(nexus_mobile_patch));

    let studio_router = rullst::studio::data_browser::router();

    use blueprint_ai::admin::{Blueprint, Surface, integrate};
    let nexus = integrate(nexus, &nexus_auth, Blueprint::Portfolio, Surface::Nexus)?;
    let studio_router = integrate(
        studio_router,
        &nexus_auth,
        Blueprint::Portfolio,
        Surface::Studio,
    )?;

    // 2. Initialize Database & Run Migrations on container boot
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:///app/db.sqlite?mode=rwc".to_string());
    println!("📦 Connecting to database: {db_url}");
    match rullst::db::Orm::init(&db_url).await {
        Ok(_) => {
            println!("🚀 Running migrations on boot...");
            for migration in crate::migrations::get_migrations() {
                if let Err(err) = migration.up().await {
                    eprintln!("⚠️ Migration error: {err}");
                }
            }
            println!("✅ Database migrations applied successfully!");
        }
        Err(err) => {
            eprintln!("❌ Database connection error: {err}");
        }
    }

    // 3. Router with Trusted TLS termination for cloud ingress (Azure Container Apps / Envoy)
    let router = routes![
        get("/" => controllers::portfolio_controller::index),
        get("/privacy" => showcase::privacy),
        get("/cookies" => showcase::cookies),
        get("/static/showcase.css" => showcase::css),
        get("/static/showcase.js" => showcase::js),
        get("/static/tailwind.js" => showcase::tailwind),
        get("/static/rullst.png" => showcase::logo),
        get("/favicon.ico" => showcase::logo),
        get("/static/htmx.js" => htmx_handler),
        get("/static/crab.png" => crab_png_handler),
        post("/api/chat" => controllers::ai_controller::chat),
    ]
    .nest_axum("/nexus", nexus)
    .nest_axum("/studio", studio_router)
    .merge_axum(rullst::health::health_router())
    .layer(rullst::server::from_fn(showcase::shell))
    // Forms need a token in development too; the framework deduplicates this
    // middleware when its production security baseline is also installed.
    .layer(rullst::server::from_fn(rullst::security::csrf_middleware))
    .layer(rullst::server::Extension(
        rullst::nexus::NexusVerifiedTls::from_trusted_tls_termination(),
    ));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    println!(
        "🚀 AI Portfolio server starting on port {port} (Engine: Zero-Bundle HTMX, ORM: Active Record)..."
    );
    println!("⚙️  Nexus CMS: http://0.0.0.0:{port}/nexus");
    Server::new(router).run(port).await?;

    Ok(())
}
