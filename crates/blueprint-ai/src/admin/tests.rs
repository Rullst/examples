use super::*;
use axum::extract::ConnectInfo;
use base64::Engine;
use std::net::SocketAddr;
use tower::ServiceExt;

fn app(blueprint: Blueprint, surface: Surface) -> Router {
    let policy = NexusAuthPolicy::basic("test-admin", "test-only-long-password").unwrap();
    let native = Router::new().route(
        "/",
        axum::routing::get(|| async { Html("<html><body>Native panel</body></html>") }),
    );
    Router::new()
        .nest(
            surface.prefix(),
            integrate(native, &policy, blueprint, surface).unwrap(),
        )
        .layer(axum::Extension(
            rullst_nexus::NexusVerifiedTls::from_trusted_tls_termination(),
        ))
}

fn request(
    path: &str,
    method: Method,
    auth: bool,
    origin: Option<&str>,
    custom: bool,
    body: &str,
) -> Request {
    let mut builder = Request::builder()
        .uri(path)
        .method(method)
        .header(header::HOST, "example.test");
    if auth {
        let credentials =
            base64::engine::general_purpose::STANDARD.encode("test-admin:test-only-long-password");
        builder = builder.header(header::AUTHORIZATION, format!("Basic {credentials}"));
    }
    if let Some(origin) = origin {
        builder = builder.header(header::ORIGIN, origin);
    }
    if custom {
        builder = builder.header("x-rullst-ai", "1");
    }
    let mut request = builder
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body.to_owned()))
        .unwrap();
    request.extensions_mut().insert(ConnectInfo(
        "192.0.2.11:4242".parse::<SocketAddr>().unwrap(),
    ));
    request
}

#[tokio::test]
async fn all_six_admin_assistants_are_authenticated_and_contextual() {
    for blueprint in [Blueprint::Showcase, Blueprint::Portfolio, Blueprint::Lms] {
        for surface in [Surface::Nexus, Surface::Studio] {
            let router = app(blueprint, surface);
            let path = format!("{}{}", surface.prefix(), surface.page());
            let denied = router
                .clone()
                .oneshot(request(&path, Method::GET, false, None, false, ""))
                .await
                .unwrap();
            assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
            let response = router
                .clone()
                .oneshot(request(&path, Method::GET, true, None, false, ""))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
            let body = to_bytes(response.into_body(), 65536).await.unwrap();
            let html = String::from_utf8(body.to_vec()).unwrap();
            assert!(html.contains(blueprint.name()));
            assert!(html.contains(&format!("{}/copilot/query", surface.prefix())));
            assert!(!html.contains("test-only-long-password"));
            // New endpoints fall back through the native router but must still
            // pass the same policy, including assets and direct query requests.
            for route in ["/copilot/query", "/copilot.js"] {
                let path = format!("{}{route}", surface.prefix());
                let denied = router
                    .clone()
                    .oneshot(request(
                        &path,
                        Method::POST,
                        false,
                        Some("https://example.test"),
                        true,
                        "message=Hello",
                    ))
                    .await
                    .unwrap();
                assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
            }
        }
    }
}

#[tokio::test]
async fn rejects_csrf_and_oversized_input_before_inference() {
    for surface in [Surface::Nexus, Surface::Studio] {
        let router = app(Blueprint::Lms, surface);
        let path = format!("{}/copilot/query", surface.prefix());
        for (origin, custom) in [
            (None, true),
            (Some("https://evil.test"), true),
            (Some("https://example.test.evil.test"), true),
            (Some("http://example.test"), true),
            (Some("https://example.test"), false),
        ] {
            let response = router
                .clone()
                .oneshot(request(
                    &path,
                    Method::POST,
                    true,
                    origin,
                    custom,
                    "message=Hello",
                ))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }
        for (body, expected) in [
            ("message= ".to_string(), StatusCode::BAD_REQUEST),
            (
                format!("message={}", "x".repeat(1201)),
                StatusCode::BAD_REQUEST,
            ),
            (
                format!("message={}", "x".repeat(9000)),
                StatusCode::PAYLOAD_TOO_LARGE,
            ),
        ] {
            let response = router
                .clone()
                .oneshot(request(
                    &path,
                    Method::POST,
                    true,
                    Some("https://example.test"),
                    true,
                    &body,
                ))
                .await
                .unwrap();
            assert_eq!(response.status(), expected);
        }
    }
}

#[tokio::test]
async fn protects_legacy_endpoints_and_adds_entry_points() {
    for surface in [Surface::Nexus, Surface::Studio] {
        let router = app(Blueprint::Showcase, surface);
        let page = format!("{}{}{}", surface.prefix(), surface.prefix(), surface.page());
        let response = router
            .clone()
            .oneshot(request(&page, Method::GET, true, None, false, ""))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let response = router
            .clone()
            .oneshot(request(
                surface.prefix(),
                Method::GET,
                true,
                None,
                false,
                "",
            ))
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), 65536).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("Native panel"));
        assert!(html.contains("AI assistant"));
    }
    let response = app(Blueprint::Showcase, Surface::Nexus)
        .oneshot(request(
            "/nexus/chat/query",
            Method::POST,
            true,
            None,
            false,
            "message=Hello",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[test]
fn csrf_rejects_duplicate_origins_and_cross_site_fetch() {
    let mut req = request(
        "/",
        Method::POST,
        true,
        Some("https://example.test"),
        true,
        "",
    );
    assert!(same_origin(&req));
    req.headers_mut()
        .append(header::ORIGIN, "https://evil.test".parse().unwrap());
    assert!(!same_origin(&req));
    req.headers_mut()
        .insert(header::ORIGIN, "https://example.test".parse().unwrap());
    req.headers_mut()
        .insert("sec-fetch-site", "cross-site".parse().unwrap());
    assert!(!same_origin(&req));
}

#[test]
fn htmx_navigation_keeps_styles_and_initializes_the_chat() {
    let html = page(
        AdminAi {
            blueprint: Blueprint::Portfolio,
            surface: Surface::Nexus,
        },
        true,
    );
    assert!(!html.contains("<html"));
    assert!(!html.contains("<body"));
    assert!(html.starts_with("<style>"));
    assert!(html.contains("/nexus/copilot.js"));
    assert!(html.contains("id=\"rullst-admin-ai\""));
}
