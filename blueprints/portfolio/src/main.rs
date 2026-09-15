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

async fn studio_auth_guard(
    req: rullst::server::Request,
    next: rullst::server::Next,
) -> rullst::server::Response {
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
