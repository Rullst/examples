# 🎓 LMS Blueprint Deployment & Integration Audit Protocol

**Document Type:** Pre-Flight Verification & Release Quality Gate  
**Target Release:** Rullst `v12.0.0` Stable  
**Auditor:** Showcase & Deployment Agent (Antigravity)  
**Core Framework Verifier:** Monorepo Hardening Agent  
**Framework SST:** [`docs/src/spec.md`](../docs/src/spec.md) / [`AGENTS.md`](../AGENTS.md)

---

## 🎯 1. Purpose & Inter-Agent Alignment

Prior to publishing the `v12.0.0` final crates to `crates.io` and creating official Git release tags, both autonomous agents agreed on an empirical integration gate:
1. **Core Agent Commitment:** The core hardening agent will **not** cut Git tags or publish packages to crates.io. It will finalize monorepo test suites, verify OpenSSF Scorecard diffs, and produce a verifiable candidate Git commit SHA.
2. **Deployment Agent Commitment:** The deployment agent will materialise, compile, run, and audit the **LMS (Learning Management System)** blueprint in an isolated environment.
3. **Synthesis & GO Decision:** The resulting reproducible audit report will be cross-referenced with the core codebase to catch any end-to-end integration blindspots (CSRF, cookies, multi-role auth, migrations, template hydration) before giving the final publication **GO**.

---

## 📋 2. Mandatory Verification Matrix (The 9 Core Invariants)

The audit of the LMS blueprint must rigorously test, verify, and document each of the following 9 dimensions:

| # | Inspection Dimension | Verification Requirement | Expected Result |
|---|---|---|---|
| **1** | **Exact Generation Command** | Document the exact CLI command line, flags, options, and directory layout used. | Clean, reproducible command (`cargo rullst new lms_academy --default --blueprint lms --database sqlite`). |
| **2** | **Build & Migrations** | Verify offline build without monorepo path leaks; apply initial SQLite database migrations. | Clean `cargo check` & `cargo build`; zero compilation errors; database tables created successfully. |
| **3** | **Startup & Runtime Telemetry** | Launch the server binary; inspect startup time, stdout/stderr, and bound ports. | Instant boot (< 50ms); zero unhandled warnings; clean telemetry logs. |
| **4** | **Core Routes & Navigation** | Crawl all registered routes (catalog, course player, instructor dashboard, curriculum). | HTTP 200 OK across all public routes; clean semantic HTML/HTMX; zero visual glitches. |
| **5** | **Identity, Auth & RBAC** | Test user registration, Argon2 password hashing, login, logout, and role access control (Student vs Instructor vs Admin). | Encrypted session cookie; proper role separation; strict rejection of unauthorized actions. |
| **6** | **POST Forms & CSRF Defense** | Submit all core mutation forms (course enrollment, lesson progress, quiz submission). | Mandatory Double-Submit CSRF cookie validation; 403 Forbidden on missing/tampered token. |
| **7** | **Durability & Restart Persistence** | Create courses, enroll users, advance progress, then restart the application process. | 100% data durability in SQLite; state resumes seamlessly without data loss. |
| **8** | **Zero Hardcoded Secrets & Zero 5xx** | Static AST scan for hardcoded secrets/passwords; automated smoke fuzzing for server errors. | Zero leaked credentials; zero runtime panics (`unwrap`/`expect`); zero HTTP 500 errors. |
| **9** | **UX Truth In Advertising** | Compare UI claims (buttons, badges, feature promises) against actual runtime functionality. | If a feature is mock/locked, it must have an explicit notice; zero misleading dead links. |

---

## 🛠️ 3. Execution Plan

### Step 3.1: Isolated Scaffold Materialization
```bash
# Generate standalone LMS project using SQLite
cargo rullst new lms_academy --default --blueprint lms --database sqlite
cd lms_academy
```

### Step 3.2: Static Inspection & Linting
```bash
# Verify independent dependency graph (no local path dependencies)
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

### Step 3.3: Database Migration & Schema Validation
```bash
# Execute initial schema migration
cargo rullst migrate up
# Verify table creations: users, courses, lessons, enrollments, submissions, certificates
```

### Step 3.4: Automated Route & Security Probing
- Perform automated HTTP requests across:
  - `GET /` (LMS Landing & Course Catalog)
  - `GET /login` & `POST /login` (Authentication)
  - `POST /register` (Account creation with Argon2)
  - `GET /courses/:id` (Course details)
  - `POST /courses/:id/enroll` (Enrollment with CSRF validation)
  - `POST /courses/:id/lessons/:lesson_id/complete` (Monotonic Progress tracking)
  - `POST /logout` (Session invalidation)

### Step 3.5: Durability & Failover Verification
- Kill the running instance (`SIGINT` / `SIGTERM`).
- Re-launch the binary.
- Verify that user progress, enrolled courses, and database integrity are completely preserved.

---

## 📊 4. Deliverable Format

Upon completing the verification, the final audit results will be compiled into `docs/BLUEPRINT_LMS_AUDIT_REPORT.md` featuring:
1. CLI execution transcript and generated project tree.
2. HTTP response matrix with status codes and latency.
3. Security posture evaluation (CSRF, WAF, Session headers).
4. List of identified friction points or bugs (if any) with suggested core fixes.
5. Official **GO / NO-GO** recommendation for the `v12.0.0` crates.io release.
