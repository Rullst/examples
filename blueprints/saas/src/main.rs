use rullst::server::{IntoResponse, Next, Request, Response, StatusCode};
use rullst::{Server, routes};

pub mod controllers;
pub mod middlewares;
pub mod migrations;
pub mod models;
pub mod pages;

fn trusts_nexus_tls_termination(value: Option<&str>) -> bool {
    value.is_some_and(|value| value.trim().eq_ignore_ascii_case("azure-container-apps"))
}

async fn healthz() -> Response {
    let database_ready = match rullst::db::Orm::pool() {
        Ok(pool) => rullst::db::sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(pool)
            .await
            .is_ok(),
        Err(_) => false,
    };
    if database_ready {
        (StatusCode::OK, "ready").into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "not ready").into_response()
    }
}

async fn staging_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    if std::env::var("DEPLOYMENT_TIER").is_ok_and(|value| value.eq_ignore_ascii_case("staging")) {
        response.headers_mut().insert(
            rullst::server::header::HeaderName::from_static("x-robots-tag"),
            rullst::server::HeaderValue::from_static("noindex, nofollow, noarchive"),
        );
    }
    response
}

#[rullst::runtime::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rullst::artisan!(crate::migrations::get_migrations());
    controllers::billing_controller::initialize_billing_provider()?;

    let nexus_auth = rullst::nexus::NexusAuthPolicy::local_development_or_basic_from_env()?;
    let nexus = rullst::nexus::Nexus::new()
        .with_auth_policy(nexus_auth)
        .with_brand("SaaS Admin")
        .register::<models::user::User>()
        .register::<models::purchase_attempt::PurchaseAttempt>()
        .register::<models::entitlement::Entitlement>()
        .register::<models::refund_request::RefundRequest>()
        .register::<models::tester_certificate::TesterCertificate>()
        .try_build()?;

    let router = routes![
        get("/" => controllers::billing_controller::pricing_view),
        get("/healthz" => healthz),
        get("/pricing" => controllers::billing_controller::pricing_view),
        get("/privacy" => controllers::legal_controller::privacy_notice),
        get("/terms" => controllers::legal_controller::sandbox_terms),
        get("/login" => controllers::auth_controller::login_view),
        post("/login" => controllers::auth_controller::login_submit),
        get("/register" => controllers::auth_controller::register_view),
        post("/register" => controllers::auth_controller::register_submit),
        get("/verify/{public_id}" => controllers::certificate_controller::verify_certificate),
        post("/logout" => controllers::auth_controller::logout),
    ];

    let router = router
        .route(
            "/dashboard",
            rullst::routing::get(controllers::auth_controller::dashboard).layer(
                rullst::server::from_fn(middlewares::auth_middleware::auth_middleware),
            ),
        )
        .route(
            "/billing/checkout",
            rullst::routing::post(controllers::billing_controller::checkout_redirect).layer(
                rullst::server::from_fn(middlewares::auth_middleware::auth_middleware),
            ),
        )
        .route(
            "/certificate",
            rullst::routing::get(controllers::certificate_controller::owner_certificate).layer(
                rullst::server::from_fn(middlewares::auth_middleware::auth_middleware),
            ),
        )
        .route(
            "/reports/stripe-gateway-field-report-v1.md",
            rullst::routing::get(controllers::billing_controller::download_stripe_report).layer(
                rullst::server::from_fn(middlewares::auth_middleware::auth_middleware),
            ),
        )
        .route(
            "/refund",
            rullst::routing::get(controllers::refund_controller::refund_view)
                .post(controllers::refund_controller::refund_submit)
                .layer(rullst::server::from_fn(
                    middlewares::auth_middleware::auth_middleware,
                )),
        )
        .route(
            "/account/data-export",
            rullst::routing::get(controllers::privacy_controller::export_account_data).layer(
                rullst::server::from_fn(middlewares::auth_middleware::auth_middleware),
            ),
        )
        .layer(rullst::server::from_fn(rullst::security::csrf_middleware))
        .route(
            "/billing/webhook",
            rullst::routing::post(controllers::billing_controller::webhook_handler),
        )
        .route(
            "/billing/reconcile",
            rullst::routing::post(controllers::billing_controller::reconciliation_handler),
        )
        .layer(rullst::server::from_fn(
            rullst::security::headers_middleware,
        ))
        .layer(rullst::server::from_fn(staging_headers))
        .nest_axum("/nexus", nexus);

    let nexus_tls_termination = std::env::var("NEXUS_TRUSTED_TLS_TERMINATION").ok();
    let router = if trusts_nexus_tls_termination(nexus_tls_termination.as_deref()) {
        router.layer(rullst::server::Extension(
            rullst::nexus::NexusVerifiedTls::from_trusted_tls_termination(),
        ))
    } else {
        router
    };

    #[cfg(debug_assertions)]
    {
        rullst::runtime::spawn(async {
            if let Err(error) = rullst::studio::run_studio(5555).await {
                eprintln!("Rullst Studio could not start: {error}");
            }
        });
        println!("📊 Rullst Studio running on http://127.0.0.1:5555");
    }
    println!("🚀 SaaS server starting on port 3000...");
    Server::new(router).run(3000).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::trusts_nexus_tls_termination;

    #[test]
    fn only_the_reviewed_azure_terminator_enables_the_nexus_tls_capability() {
        assert!(trusts_nexus_tls_termination(Some("azure-container-apps")));
        assert!(trusts_nexus_tls_termination(Some(" Azure-Container-Apps ")));
        assert!(!trusts_nexus_tls_termination(None));
        assert!(!trusts_nexus_tls_termination(Some("true")));
        assert!(!trusts_nexus_tls_termination(Some("untrusted-proxy")));
    }
}
