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

---

## Issue 2: Orphaned `/studio/cache` Navigation Route (HTTP 404)

* **Component:** `rullst-studio` / `data_browser::layout` & `cache_inspector`
* **Affected Versions:** `v12.0.0-rc.1` and `v12.0.0` (Stable)
* **Affected Routes:** `/studio/cache` and `/cache`
* **Symptom:** Clicking the "🧊 Cache" navigation button in the Studio navbar navigates to `/studio/cache` and returns `HTTP 404 Not Found`.
* **Root Cause:**
  1. In `rullst-studio/src/data_browser/layout.rs` (lines 51–53), the Studio top navigation bar unconditionally renders a link to `/studio/cache`:
     ```html
     <a href="/studio/cache" ...>
         <span>🧊 Cache</span>
     </a>
     ```
  2. However, in `rullst-studio/src/data_browser/mod.rs`, `router_with_trace_store()` registers routes for `/`, `/tables/{table}`, `/migrations`, `/ai`, `/security`, `/radar`, `/capital`, and `/traces`, but **does NOT register `/cache` or `/studio/cache`**.
  3. The internal `cache_inspector::router` is only wired within `Studio::into_router(access)` in `lib.rs` (`.nest("/studio/cache", cache_router)`), which is locked behind the local loopback capability and never exposed when mounting `rullst::studio::data_browser::router()`.
  4. As a result, every application that embeds the public Studio data browser router experiences a broken 404 link when clicking "Cache".

### Recommended Permanent Framework Fix for Rullst v12.1.0+
In `rullst-studio/src/data_browser/mod.rs`:
Mount default inspection fallback routes or wire `cache_inspector` into `data_browser::router_with_trace_store` directly:
```rust
.route("/cache", axum::routing::get(handle_studio_cache))
.route("/studio/cache", axum::routing::get(handle_studio_cache))
```

### Blueprint Workaround (Applied in `blueprints/portfolio` & `blueprints/lms`)
Register an application-level handler using `rullst_studio::data_browser::studio_layout` to provide a dark glassmorphic Cache Inspector interface responding to both full visits and HTMX partial swaps.

---

## Issue 3: Nexus Mobile Drawer Trapping (Unclosable Sidebar on Viewports <= 900px)

* **Component:** `rullst-nexus` (v12.0.0) / `src/nexus/ui.rs`
* **Affected Versions:** `v12.0.0-rc.1` and `v12.0.0` (Stable)
* **Symptom:** When accessing Nexus CMS on mobile devices or responsive viewports <= 900px, clicking the topbar hamburger toggle button (`&#9776;`) opens the sidebar drawer by applying `.nexus-sidebar-open`. Once opened, the sidebar cannot be closed:
  - There is no close button (`×`) inside the sidebar or top brand header.
  - There is no backdrop / overlay element behind the sidebar.
  - Clicking outside the drawer does nothing because no click-away event listener exists.
  - Because the fixed sidebar (`width: 240px`, `z-index: 100`) covers the left side of the viewport, the hamburger button underneath is obscured or cannot toggle it closed.
  - The user is permanently trapped in the sidebar unless they manually reload the page.

* **Root Cause Analysis:**
  In `rullst-nexus/src/nexus/ui.rs`:
  1. The topbar toggle button is declared as:
     ```html
     <button class="nexus-topbar-toggle" onclick="document.getElementById(&quot;nexus-sidebar&quot;).classList.toggle(&quot;nexus-sidebar-open&quot;)">&#9776;</button>
     ```
  2. The mobile CSS specifies:
     ```css
     @media (max-width: 900px) {
         .nexus-sidebar { position: fixed; left: 0; top: 0; bottom: 0; transform: translateX(-100%); }
         .nexus-sidebar-open { transform: translateX(0); }
         .nexus-topbar-toggle { display: flex; }
         ...
     }
     ```
  3. No `<div class="nexus-sidebar-backdrop">` element is rendered in `render_shell()`.
  4. No dismiss event listener is registered for navigation links, escape key, or backdrop clicks.

### Recommended Permanent Framework Fix for Rullst v12.1.0+
In `rullst-nexus/src/nexus/ui.rs`:
1. In `render_sidebar()` or `render_shell()`:
   - Add a dismiss button inside `.nexus-brand`:
     ```html
     <button class="nexus-sidebar-close" onclick="document.getElementById('nexus-sidebar').classList.remove('nexus-sidebar-open')">&times;</button>
     ```
   - Render a backdrop right after the sidebar:
     ```html
     <div class="nexus-sidebar-backdrop" id="nexus-sidebar-backdrop" onclick="document.getElementById('nexus-sidebar').classList.remove('nexus-sidebar-open')"></div>
     ```
2. In `NEXUS_CSS`:
   ```css
   @media (max-width: 900px) {
       .nexus-sidebar-backdrop {
           display: none;
           position: fixed;
           inset: 0;
           background: rgba(0, 0, 0, 0.65);
           backdrop-filter: blur(4px);
           z-index: 95;
       }
       .nexus-sidebar.nexus-sidebar-open ~ .nexus-sidebar-backdrop {
           display: block;
       }
       .nexus-sidebar { z-index: 100; }
   }
   ```
3. Auto-close the sidebar when any navigation link inside the sidebar is clicked, or when `Escape` is pressed.

### Blueprint Workaround (Applied in `blueprints/portfolio` & `blueprints/lms`)
An Axum middleware layer `nexus_mobile_patch` is attached to `nexus.layer(...)` that buffers HTML responses from Nexus, dynamically injecting:
- `#nexus-sidebar-backdrop` with backdrop blur and touch dismiss.
- `#nexus-sidebar-close` ("×") inside `.nexus-brand`.
- Event listeners for link clicks, click-outside, and Escape key dismissal.

---

## Issue 4: Portfolio Blueprint Layout Missing Responsive Mobile Breakpoints

* **Component:** `blueprints/portfolio` / `src/pages/home.rs`
* **Symptom:** On mobile browsers (screen widths <= 900px / 360–420px phones), `portfolio.rullst.win` exhibited severe layout degradation:
  - The sidebar remained pinned with a rigid `width: 350px` and `position: sticky`.
  - The main container `.layout` maintained `display: flex; gap: 3rem;`, forcing the sidebar and project cards side-by-side.
  - The content overflowed off-screen horizontally, requiring horizontal scrolling and breaking readability.
  - Large display headings (`h1`) wrapped awkwardly or overflowed smaller viewports.

* **Root Cause:**
  `cv_styles()` was written with desktop-first fixed measurements and zero `@media` query breakpoints:
  ```css
  .layout { display: flex; min-height: 100vh; max-width: 1400px; margin: 0 auto; padding: 2rem; gap: 3rem; }
  .sidebar { width: 350px; flex-shrink: 0; position: sticky; top: 2rem; height: calc(100vh - 4rem); ... }
  ```

* **Blueprint Resolution:**
  Implemented fluid, mobile-first responsive media queries:
  1. Added `overflow-x: hidden; width: 100%;` to `html, body`.
  2. Fluid heading scale: `font-size: clamp(1.8rem, 4vw, 2.3rem);`.
  3. `@media (max-width: 900px)`:
     - `.layout` switches to `flex-direction: column; padding: 1.25rem 1rem; gap: 2.25rem;`.
     - `.sidebar` becomes fluid `width: 100%; position: static; height: auto;`.
     - `.projects-grid` drops to a single responsive column (`grid-template-columns: 1fr;`).
  4. `@media (max-width: 640px)`:
     - Tighter gutter padding (`1rem 0.75rem`), scaled profile avatar (`100px`), and compact timeline connectors.

---

## Issue 5: Zero-Bundle HTMX Form Submissions Silently Blocked by Framework CSRF Baseline (HTTP 403 "CSRF token cookie missing")

* **Component:** `rullst-core` (v12.0.0) / `security::csrf_middleware` & `cargo-rullst` blueprint templates
* **Affected Versions:** `v12.0.0-rc.1` and `v12.0.0` (Stable)
* **Affected Routes:** `POST /api/chat` and any blueprint HTMX mutation endpoints (`POST /...`)
* **Symptom:** Submitting dynamic chat messages, prompt inputs, or any AJAX/HTMX mutation form (`hx-post="/api/chat"`) fails silently in the browser:
  - The user sees their optimistic message bubble ("You: hi") inserted into the chat view via client-side DOM scripting.
  - The Copilot never responds; the typing indicator either spins indefinitely or disappears without a reply bubble.
  - The browser Network tab shows `POST /api/chat` failing immediately with `HTTP 403 Forbidden` and response body:
    ```text
    CSRF token cookie missing
    ```
  - HTMX by default ignores HTTP 403 responses and does not swap them into `#ai-chat-messages`, giving the impression that the AI service or API key is completely unresponsive.

* **Root Cause Analysis:**
  1. **Framework Mandatory CSRF Baseline:**
     When starting a server with `Server::new(router).run(port)`, Rullst's canonical `apply_security_baseline` wraps the entire application router with `rullst::security::csrf_middleware`. This middleware automatically enforces double-submit cookie validation on all non-safe HTTP methods (`POST`, `PUT`, `DELETE`, `PATCH`).
  2. **Blueprint Scaffold CSRF Token Omission:**
     The portfolio blueprint scaffold (`blueprints/portfolio/src/controllers/portfolio_controller.rs`) did not extract `Extension<rullst::security::CsrfToken>` from incoming GET requests, and `home::render` did not receive or render any CSRF token.
  3. **HTMX Missing CSRF Header / Body Bridge:**
     The chat form (`<form hx-post="/api/chat">`) lacked a `<input type="hidden" name="_token" ... />` element and did not configure an `htmx:configRequest` listener to forward the `rullst_csrf` cookie value.
  4. **Premature Input Value Clearing in JavaScript:**
     In `home.rs`, `appendUserMessage()` executed `input.value = '';` inside the `hx-on::before-request` handler. Because this executed before HTMX serialized the form body, HTMX submitted `message=` (an empty string), compounding the failure.
  5. **Lack of Route-Level CSRF Exemption in `rullst-core`:**
     The Rullst framework currently lacks a declarative macro or router builder method (such as `.route_csrf_exempt(...)` or `#[csrf_exempt]`) to whitelist public read-only query endpoints (like AI chat widgets or webhooks) from session-based cookie CSRF enforcement.

### Recommended Permanent Framework Fix for Rullst v12.1.0+
1. **Add CSRF Route Exemption Capability:**
   Introduce an explicit exemption layer in `rullst::security` to allow developer-selected public endpoints (e.g., public AI chat, public webhooks) to bypass cookie CSRF checks while maintaining WAF and Rate-Limiting protections:
   ```rust
   router.route_csrf_exempt("/api/chat", post(chat_handler))
   ```
2. **Standardize Blueprint HTMX CSRF Scaffolding:**
   Update `cargo-rullst` blueprints to inject the standard HTMX double-submit bridge into the base layout `<head>`:
   ```javascript
   document.body.addEventListener('htmx:configRequest', function(evt) {
       var match = document.cookie.match(/rullst_csrf=([^;]+)/);
       if (match) {
           evt.detail.parameters['_token'] = decodeURIComponent(match[1].trim());
           evt.detail.headers['X-CSRF-Token'] = decodeURIComponent(match[1].trim());
       }
   });
   ```

### Blueprint Workaround (Applied in `blueprints/portfolio`)
1. **Pass CSRF Token to Home Template:** In `portfolio_controller::index`, extract `Extension(csrf_token): Extension<rullst::security::CsrfToken>` and pass `csrf_token.as_str()` into `home::render`.
2. **Hidden Form Token & HTMX Bridge:** Render `<input type="hidden" name="_token" value="{csrf_token}" id="ai-csrf-token" />` inside `#ai-chat-form` and attach an `htmx:configRequest` listener.
3. **Defer Input Clearing:** Move `input.value = ''` from `before-request` to `finalizeAiRequest` (`after-request`), ensuring HTMX successfully serializes the user message.
