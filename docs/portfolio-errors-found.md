# Rullst Framework v12.0.0 — Portfolio Blueprint & Studio Assets Issue Tracker

This document records the architectural bug analysis identified in the **Rullst v12.0.0 stable release** regarding Rullst Studio assets delivery, specifically causing the Studio UI to render as unstyled raw HTML in production deployments (e.g., Azure Container Apps) and embedded sub-routers.

---

## Issue Summary

* **Component:** `rullst-studio` (v12.0.0) / `data_browser` & `assets` router
* **Affected Routes:** `/studio` and `/studio/assets/studio.css`
* **Symptom:** When visiting `/studio` on a live cloud deployment or within any application mounting Studio via `nest_axum("/studio", studio_router)`, the page loads and authenticates successfully, but renders as raw, unstyled HTML (default serif fonts, blue hyperlinks, unstyled tables) instead of the intended dark glassmorphic UI.
* **HTTP Error:** `GET /studio/assets/studio.css` returns `HTTP/1.1 404 Not Found`.

---

## Background & Evolution (v12-rc.1 vs v12.0.0)

1. **In `v12.0.0-rc.1`:**
   Studio attempted to load Tailwind via the external Play CDN:
   ```html
   <script src="https://cdn.tailwindcss.com"></script>
   ```
   Under Rullst's strict production OWASP security headers (`Cross-Origin-Embedder-Policy: require-corp`), modern browsers blocked this cross-origin script with `net::ERR_BLOCKED_BY_RESPONSE.NotSameOriginAfterDefaultedToSameOriginByCoep`.

2. **In `v12.0.0` (Stable):**
   The core team addressed the CDN issue by compiling Tailwind ahead-of-time (AOT) into a standalone 26 KB stylesheet (`assets/studio.css`), embedded in the crate binary via `include_str!("../assets/studio.css")`.
   The layout template (`rullst-studio/src/data_browser/layout.rs`) was updated to link to this static stylesheet:
   ```html
   <link href="/studio/assets/studio.css" rel="stylesheet" />
   ```

---

## Root Cause Analysis

Despite the AOT CSS compilation, the stylesheet fails to load with **`404 Not Found`** due to two architectural omissions in `rullst-studio`:

### 1. `assets::router()` is Excluded from `data_browser::router()`
* In `rullst-studio/src/lib.rs`, the asset router is registered **only** inside the private local development builder:
  ```rust
  // rullst-studio/src/lib.rs - Studio::into_router()
  pub fn into_router(self, access: LocalStudioAccess) -> Result<Router, StudioBuildError> {
      ...
      let mut router = data_browser::router_with_trace_store(self.distributed_traces)
          .nest("/studio/requests", logger::router(logger_state.clone()))
          .nest("/studio/env", env_viewer::router())
          .nest("/studio/features", feature_flags::router())
          .nest("/studio/cache", cache_router)
          .nest("/studio/assets", assets::router()) // <-- ONLY REGISTERED HERE
          .nest("/studio/er", er_diagram::router())
          ...
  }
  ```
* However, applications embedding Studio behind custom authentication or cloud boundaries (such as `blueprints/portfolio/src/main.rs`) use the public module router:
  ```rust
  let studio_router = rullst::studio::data_browser::router()
      .layer(rullst::server::from_fn(studio_auth_guard));
  ```
* In `rullst-studio/src/data_browser/mod.rs`, `router_with_trace_store()` registers endpoints for `/`, `/tables/{table}`, `/migrations`, `/ai`, `/security`, `/radar`, `/capital`, and `/traces`, but **omits `assets::router()` entirely**.

### 2. Sub-Path Strip Collision with Axum Nesting
* When an application mounts the Studio sub-router using Axum's standard nesting:
  ```rust
  routes![ ... ].nest_axum("/studio", studio_router)
  ```
  Axum automatically strips the `/studio` prefix from the incoming request path before handing it to `studio_router`.
* Therefore, when the browser encounters `<link href="/studio/assets/studio.css" rel="stylesheet" />` and makes a request to `/studio/assets/studio.css`, the request reaches `studio_router` with path:
  ```text
  /assets/studio.css
  ```
* Even in `Studio::into_router()`, the route was nested as `.nest("/studio/assets", assets::router())`. If mounted under `/studio`, it would have required `/studio/studio/assets/studio.css` to match.

---

## Live Verification & Reproduction

Tested against the live Azure Container Apps deployment (`rullst-portfolio.redpond-24d9228d.eastus.azurecontainerapps.io`):

```http
GET /studio/assets/studio.css HTTP/1.1
Host: rullst-portfolio.redpond-24d9228d.eastus.azurecontainerapps.io
User-Agent: Mozilla/5.0 ...

HTTP/1.1 404 Not Found
content-length: 0
date: Tue, 15 Sep 2026 21:31:24 GMT
```

The 404 status confirms that the stylesheet requested by `studio_layout` is not served by the application router.

---

## Recommended Permanent Fix for Future Rullst Releases

### Option A: Mount `assets::router()` in `data_browser::router_with_trace_store` (Recommended)
In `rullst-studio/src/data_browser/mod.rs`:
```rust
pub fn router_with_trace_store(
    trace_store: crate::distributed_traces::DistributedTraceStore,
) -> Router {
    Router::new()
        // Dashboard
        .route("/", axum::routing::get(handle_dashboard))
        .route("/studio", axum::routing::get(handle_dashboard))
        // Serve embedded Studio assets for both root and /studio prefixes:
        .route("/assets/studio.css", axum::routing::get(crate::assets::studio_css))
        .route("/studio/assets/studio.css", axum::routing::get(crate::assets::studio_css))
        .route("/assets/logger.js", axum::routing::get(crate::assets::logger_js))
        .route("/studio/assets/logger.js", axum::routing::get(crate::assets::logger_js))
        ...
```

### Option B: Inline Stylesheet (Zero Network Dependency — Nexus Pattern)
Follow the pattern established by `rullst-nexus` (`rullst-nexus/src/nexus/ui.rs`), which injects its CSS directly as an inline `<style>` block:
In `rullst-studio/src/data_browser/layout.rs`:
```rust
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>"Rullst Studio Control Center"</title>
    <style>
        { RawHtml(crate::assets::STUDIO_CSS) }
    </style>
</head>
```
**Benefits:**
- 0 HTTP requests for stylesheets.
- 100% immune to base path routing, reverse proxy prefix rewrites, or Axum `.nest()` stripping.
- 100% immune to COEP/CORP blocking.

---

## Application-Level Workaround (Portfolio Blueprint)

Until a new Rullst release publishes this fix to crates.io, the Portfolio application can resolve this directly in `blueprints/portfolio/src/main.rs`:

```rust
const STUDIO_CSS: &str = include_str!("../../../assets/studio.css"); // or embed precompiled studio.css

async fn studio_css_handler() -> rullst::server::Response {
    use rullst::server::header;
    (
        rullst::server::StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        STUDIO_CSS,
    ).into_response()
}

// In main.rs:
let studio_router = rullst::studio::data_browser::router()
    .route("/assets/studio.css", axum::routing::get(studio_css_handler))
    .layer(rullst::server::from_fn(studio_auth_guard));
```
