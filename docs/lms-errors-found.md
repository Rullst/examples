# Rullst Framework — LMS & Framework Errors Tracker (Audit & Status v12.0.0)

This document records architectural nuances, edge cases, bugs, and compiler quirks identified during real-world stress-testing and deployment of Rullst applications, LMS blueprints, and CLI generators.

Each entry includes its verified status against the **Rullst v12.0.0 Stable** release.

---

## Issue Registry & v12.0.0 Status

### 1. `cargo rullst make:omni`: PDB Target Collision on Windows MSVC — [CORRIGIDO na v12.0.0]
- **Component:** `cargo-rullst` / Omni App Scaffolder template
- **Status v12.0.0:** ✅ **Corrigido**
- **Symptom:** Concurrent release builds on Windows failed with:
  ```text
  warning: output filename collision at ...\target\release\deps\rullst_omni.pdb
  = note: the bin target `rullst-omni` in package `rullst-omni` has the same output filename as the lib target `rullst_omni`
  error: failed to write ...\deps\libserde_core-....rmeta: O arquivo já está sendo usado por outro processo. (os error 32)
  ```
- **Root Cause:** When generating `tauri.conf.json` and `Cargo.toml`, the binary target defaulted to package name (`rullst-omni`) and the library target was named `[lib] name = "rullst_omni"`. On Windows MSVC, both linkers produced `rullst_omni.pdb`, causing file locking and build failure.
- **Resolution in v12.0.0:** In `cargo-rullst 12.0.0` (`src/generators/desktop/scaffold/templates.rs`), the generated Cargo manifest explicitly defines `[lib] name = "rullst_omni_lib"` and `main.rs` calls `rullst_omni_lib::run()`, completely eliminating the MSVC PDB collision.

---

### 2. `html!` Procedural Macro: Raw HTML Comments Cause Parse Failure — [CORRIGIDO na v12.0.0]
- **Component:** `rullst-macros` (`html!`)
- **Status v12.0.0:** ✅ **Corrigido**
- **Symptom:** Putting standard HTML comments inside the macro:
  ```rust
  html! {
      <!-- Dedicated Browser Installation Guide -->
      <div>...</div>
  }
  ```
  Caused compiler failure: `error: expected identifier` at `<!--`.
- **Root Cause:** The tokenizer in `rullst-macros` interpreted `<` as an HTML tag opening and expected an identifier (e.g. `div`, `span`), failing on `!--`.
- **Resolution in v12.0.0:** In `rullst-macros 12.0.0` (`src/html_parser.rs`), `starts_comment()` and `parse_comment()` were implemented. The macro parser cleanly consumes `<!-- ... -->` tokens at both document root and element children levels without compilation errors.

---

### 3. `SecureHeadersLayer`: Strict COEP Blocks External Media / CDN Assets — [CORRIGIDO na v12.0.0]
- **Component:** `rullst-security` & `rullst-core`
- **Status v12.0.0:** ✅ **Corrigido**
- **Symptom:** Video players, audio tags, and canvas elements referencing external cross-origin media (e.g., YouTube, Vimeo, public CDNs, or LMS sample videos) failed to load with `CORB / COEP (require-corp)` browser blocking.
- **Root Cause:** Default `SecureHeadersLayer` enforced `Cross-Origin-Embedder-Policy: require-corp` globally without an opt-out or credentialless mode.
- **Resolution in v12.0.0:** In `rullst-security 12.0.0` (`headers.rs`) and `rullst-core 12.0.0` (`security/headers.rs`), `pub coep: Option<String>` was introduced into `SecureHeadersConfig` and `RullstConfig`. It can now be configured in `Rullst.toml` or per-request extension (e.g. `credentialless`, `unsafe-none`), removing the rigid global restriction.

---

### 4. Studio Static Assets Resolution on Sub-Paths — [NÃO CORRIGIDO na v12.0.0 / BUG DE ROTA]
- **Component:** `rullst-studio`
- **Status v12.0.0:** ❌ **Não Corrigido (Parcialmente alterado, mas gerou bug de rota 404)**
- **Symptom:** When running Studio behind reverse proxies or mounted via `nest_axum("/studio", studio_router)`, `/studio` renders as unstyled raw HTML. Requesting `/studio/assets/studio.css` returns `HTTP 404 Not Found`.
- **Root Cause:** In v12.0.0, external CDN script loading was replaced with an embedded stylesheet `assets/studio.css`. However, `assets::router()` was only wired inside `Studio::into_router()` (local loopback dev mode) and was completely omitted from `rullst::studio::data_browser::router()` (used by production applications). Furthermore, Axum's `.nest_axum("/studio", ...)` strips the prefix, causing path mismatches.
- **Reference:** See [`docs/portfolio-errors-found.md`](./portfolio-errors-found.md) for full architectural analysis and recommended fix for v12.1.0+.

---

### 5. Password Policy Length Validation vs Demo Seeds — [CORRIGIDO nos Blueprints]
- **Component:** `rullst-auth` & Blueprint Seeds
- **Status v12.0.0:** ✅ **Corrigido nos Blueprints**
- **Symptom:** Strict 12-character minimum password policy caused seeded default development users (e.g., `password123`) to be unloggable or rejected by front-end form validation.
- **Resolution:** In Rullst v12, strong password validation is intentional for security. All official blueprint seed scripts (`lms` and `portfolio`) were updated to use production-compliant passwords (e.g., `RullstAcademy2026!`, `SovereignPortfolio2026!`).

---

### 6. `cargo rullst make:omni`: Android Release Packaging Without Keystore Signing — [NÃO CORRIGIDO / REQUER CI]
- **Component:** `cargo-rullst` / Omni Mobile Pipeline
- **Status v12.0.0:** ⚠️ **Não Corrigido no CLI (Comportamento padrão do Gradle / Requer CI)**
- **Symptom:** Running `npx tauri android build --apk` in release mode outputs `app-universal-release-unsigned.apk`. When distributed directly to physical Android devices, the OS refuses installation with `INSTALL_PARSE_FAILED_NO_CERTIFICATES`.
- **Root Cause:** Gradle does not sign release APKs unless configured with a signing keystore in `build.gradle`.
- **Workaround / Resolution:** Standard mobile distribution requires explicit release signing. Use:
  1. Generate keystore (`keytool`).
  2. Align package with `zipalign -v -p 4`.
  3. Sign with `apksigner sign --ks ...`.
  4. For development, use `npx tauri android build --debug --apk` which leverages the built-in debug keystore.

---

### 7. `cargo rullst make:omni`: Android/iOS Initialization Leaves Default Tauri Icon Instead of Application Brand Icon — [NÃO CORRIGIDO na v12.0.0]
- **Component:** `cargo-rullst` / Omni Mobile Pipeline
- **Status v12.0.0:** ❌ **Não Corrigido**
- **Symptom:** After installing the generated APK on an Android device or running on iOS, the application launcher icon displays the default blue circular Tauri logo instead of the application brand logo.
- **Root Cause:** In `cargo-rullst 12.0.0` (`src/generators/desktop/scaffold.rs`, lines 163–166), `generate_platform_icons` (`tauri icon`) runs **before** `init_mobile_target` (`tauri android init`). Because mobile initialization executes second, Tauri's stock template overwrites `res/mipmap-*` with Tauri's default sample icons.
- **Recommended Framework Fix:** In `cargo-rullst`, run `tauri icon icons/icon.svg` **after** `init_mobile_target(omni_dir, "android")` and `init_mobile_target(omni_dir, "ios")`.

---

*Last Updated: September 2026 — Verified against Rullst v12.0.0 Stable crates.*
