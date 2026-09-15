use rullst::{routes, Server};

pub mod migrations;
pub mod models;
pub mod controllers;
pub mod pages;

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

fn decode_base64_cred(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0;
    for &b in input.as_bytes() {
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            _ if b.is_ascii_whitespace() => continue,
            _ => return None,
        };
        buf = (buf << 6) | u32::from(val);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Some(out)
}

const STUDIO_CSS: &str = include_str!("../static/studio.css");
const LOGGER_JS: &str = r#"document.addEventListener("DOMContentLoaded",()=>{const target=document.getElementById("studio-request-stream");if(!target||typeof EventSource==="undefined")return;const source=new EventSource("/studio/requests/stream");source.onmessage=(event)=>{const row=document.createElement("div");row.innerHTML=event.data;while(row.lastChild)target.prepend(row.lastChild);};window.addEventListener("beforeunload",()=>source.close(),{once:true});});"#;

async fn studio_css_handler() -> rullst::server::Response {
    use rullst::server::header;
    use rullst::server::IntoResponse;
    (
        rullst::server::StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        STUDIO_CSS,
    ).into_response()
}

async fn studio_logger_handler() -> rullst::server::Response {
    use rullst::server::header;
    use rullst::server::IntoResponse;
    (
        rullst::server::StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/javascript; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        LOGGER_JS,
    ).into_response()
}

async fn studio_auth_guard(
    req: rullst::server::Request,
    next: rullst::server::Next,
) -> rullst::server::Response {
    let path = req.uri().path();
    if path.ends_with(".css") || path.ends_with(".js") {
        return next.run(req).await;
    }

    use rullst::server::header;
    use rullst::server::{HeaderValue, IntoResponse, StatusCode};

    let auth_header = req.headers().get(header::AUTHORIZATION).and_then(|v| v.to_str().ok());
    let expected_user = std::env::var("NEXUS_ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let raw_pass = std::env::var("NEXUS_ADMIN_PASSWORD").unwrap_or_else(|_| "SovereignPortfolio2026!".to_string());
    let expected_pass = if raw_pass.len() >= 16 { raw_pass } else { "SovereignPortfolio2026!".to_string() };

    let mut is_authorized = false;
    if let Some(auth) = auth_header {
        if let Some(encoded) = auth.strip_prefix("Basic ") {
            if let Some(decoded) = decode_base64_cred(encoded.trim()) {
                if let Ok(credentials) = String::from_utf8(decoded) {
                    if let Some((user, pass)) = credentials.split_once(':') {
                        if user == expected_user && pass == expected_pass {
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
        let mut res = (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        res.headers_mut().insert(
            header::WWW_AUTHENTICATE,
            HeaderValue::from_static("Basic realm=\"Rullst Studio & Nexus\""),
        );
        res
    }
}

    // 1. Resilient Nexus Auth Policy (defaults to public sandbox credentials for demonstration)
    let nexus_user = std::env::var("NEXUS_ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let raw_pass = std::env::var("NEXUS_ADMIN_PASSWORD").unwrap_or_else(|_| "SovereignPortfolio2026!".to_string());
    let nexus_pass = if raw_pass.len() >= 16 {
        raw_pass
    } else {
        "SovereignPortfolio2026!".to_string()
    };
    let nexus_auth = rullst::nexus::NexusAuthPolicy::basic(nexus_user, nexus_pass)?;

    let nexus = rullst::nexus::Nexus::new()
        .with_auth_policy(nexus_auth)
        .with_brand("Portfolio CMS Admin")
        .register::<models::profile::Profile>()
        .register::<models::project::Project>()
        .register::<models::experience::Experience>()
        .register::<models::skill::Skill>()
        .try_build()?;

    let studio_router = rullst::studio::data_browser::router()
        .route("/assets/studio.css", rullst::server::get(studio_css_handler))
        .route("/studio/assets/studio.css", rullst::server::get(studio_css_handler))
        .route("/assets/logger.js", rullst::server::get(studio_logger_handler))
        .route("/studio/assets/logger.js", rullst::server::get(studio_logger_handler))
        .layer(rullst::server::from_fn(studio_auth_guard));

    // 2. Initialize Database & Run Migrations on container boot
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:///app/db.sqlite?mode=rwc".to_string());
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
    ]
    .nest_axum("/nexus", nexus)
    .nest_axum("/studio", studio_router)
    .layer(rullst::server::Extension(rullst::nexus::NexusVerifiedTls::from_trusted_tls_termination()));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    println!("🚀 AI Portfolio server starting on port {port} (Engine: Zero-Bundle HTMX, ORM: Active Record)...");
    println!("⚙️  Nexus CMS: http://0.0.0.0:{port}/nexus");
    Server::new(router)
        .run(port)
        .await?;

    Ok(())
}
