# Rullst Framework — Audit & Errors Found Tracker

This document records architectural nuances, edge cases, bugs, and compiler quirks identified during real-world stress-testing and deployment of Rullst applications and CLI generators.

---

## Issue Registry

### 1. `cargo rullst make:omni`: PDB Target Collision on Windows MSVC
- **Component:** `cargo-rullst` / Omni App Scaffolder template
- **Symptom:** Concurrent release builds on Windows fail with:
  ```text
  warning: output filename collision at ...\target\release\deps\rullst_omni.pdb
  = note: the bin target `rullst-omni` in package `rullst-omni` has the same output filename as the lib target `rullst_omni`
  error: failed to write ...\deps\libserde_core-....rmeta: O arquivo já está sendo usado por outro processo. (os error 32)
  ```
- **Root Cause:** When generating `tauri.conf.json` and `Cargo.toml`, the binary target defaults to package name (`rullst-omni`) and the library target is named `[lib] name = "rullst_omni"`. On Windows MSVC, both linkers produce `rullst_omni.pdb`, causing file locking and build failure.
- **Recommended Framework Fix:** In `cargo-rullst`'s Omni scaffolder template, name the library `[lib] name = "rullst_omni_lib"` and update `main.rs` to call `rullst_omni_lib::run()`. This matches the official Tauri 2.0 multi-platform standard.

---

### 2. `html!` Procedural Macro: Raw HTML Comments Cause Parse Failure
- **Component:** `rullst-macros` (`html!`)
- **Symptom:** Putting standard HTML comments inside the macro:
  ```rust
  html! {
      <!-- Dedicated Browser Installation Guide -->
      <div>...</div>
  }
  ```
  Causes compiler failure: `error: expected identifier` at `<!--`.
- **Root Cause:** The tokenizer in `rullst-macros` interprets `<` as an HTML tag opening and expects an identifier (e.g. `div`, `span`), failing on `!--`.
- **Workaround:** Use Rust comments (`//` or `/* ... */`) outside or inside the macro.
- **Recommended Framework Fix:** Update the `html!` token tree parser to recognize `<!--` through `-->` and either strip them or serialize them as HTML comments.

---

### 3. `SecureHeadersLayer`: Strict COEP Blocks External Media / CDN Assets
- **Component:** `rullst-security`
- **Symptom:** Video players, audio tags, and canvas elements referencing external cross-origin media (e.g., YouTube, Vimeo, public CDNs, or LMS sample videos) fail to load with `CORB / COEP (require-corp)` browser blocking.
- **Root Cause:** Default `SecureHeadersLayer` enforces `Cross-Origin-Embedder-Policy: require-corp` globally without an opt-out or credentialless mode.
- **Recommended Framework Fix:** Support `Cross-Origin-Embedder-Policy: credentialless` or provide route-level builder exemptions (`.disable_coep()` or `.coep(CoepPolicy::Credentialless)`).

---

### 4. Studio Static Assets Resolution on Sub-Paths
- **Component:** `rullst-studio`
- **Symptom:** When running Studio behind reverse proxies or nested prefixes, asset links like `/studio.css` fail to resolve if the server router expects them under `/studio/static/`.
- **Recommended Framework Fix:** Use absolute, normalized canonical routes or embed CSS inline for zero-network-dependency control rooms.

---

### 5. Password Policy Length Validation vs Demo Seeds
- **Component:** `rullst-auth`
- **Symptom:** Strict 12-character minimum password policy causes seeded default development users (e.g., `password123`) to be unloggable or rejected by front-end form validation.
- **Recommended Framework Fix:** Ensure default seeds always satisfy the strict 12+ character security invariant (e.g., `RullstAcademy2026!`).

---

*Last Updated: September 2026 — Recorded during Rullst Omni & Blueprint LMS live verification.*
