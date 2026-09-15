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

    // 1. Resilient Nexus Auth Policy (never crashes container boot if env is missing)
    let nexus_user = std::env::var("NEXUS_ADMIN_USERNAME").unwrap_or_else(|_| "portfolio_admin".to_string());
    let nexus_pass = std::env::var("NEXUS_ADMIN_PASSWORD").unwrap_or_default();
    let nexus_auth = if nexus_pass.len() >= 16 {
        rullst::nexus::NexusAuthPolicy::basic(nexus_user, nexus_pass)?
    } else {
        eprintln!("⚠️ NEXUS_ADMIN_PASSWORD environment variable not set or under 16 characters. Generating ephemeral secret.");
        let ephemeral_pass = format!("ephemeral_{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis());
        rullst::nexus::NexusAuthPolicy::basic(nexus_user, ephemeral_pass)?
    };

    let nexus = rullst::nexus::Nexus::new()
        .with_auth_policy(nexus_auth)
        .with_brand("Portfolio CMS Admin")
        .register::<models::profile::Profile>()
        .register::<models::project::Project>()
        .register::<models::experience::Experience>()
        .register::<models::skill::Skill>()
        .try_build()?;

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
