<div align="center">

# 🦀 Rullst Framework — Official Showcase & Blueprints

[![Rullst Version](https://img.shields.io/badge/rullst-v12.0.0-orange.svg?style=flat-square&logo=rust)](https://crates.io/crates/rullst)
[![cargo-rullst](https://img.shields.io/badge/cargo--rullst-v12.0.0-blue.svg?style=flat-square&logo=rust)](https://crates.io/crates/cargo-rullst)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg?style=flat-square)](LICENSE)
[![OCI Container Ready](https://img.shields.io/badge/container-cargo--chef-blueviolet.svg?style=flat-square&logo=podman)](Containerfile)
[![Caddy SSL](https://img.shields.io/badge/proxy-caddy_auto_https-00ADD8.svg?style=flat-square&logo=caddy)](Caddyfile)
[![Live Showcase](https://img.shields.io/badge/live_showcase-azure_container_apps-0078D4.svg?style=flat-square&logo=microsoftazure)](https://rullst-showcase.redpond-24d9228d.eastus.azurecontainerapps.io/)
[![Live LMS Academy](https://img.shields.io/badge/live_lms-azure_container_apps-047857.svg?style=flat-square&logo=microsoftazure)](https://rullst-lms.redpond-24d9228d.eastus.azurecontainerapps.io/)
[![Live Portfolio](https://img.shields.io/badge/live_portfolio-azure_container_apps-7C3AED.svg?style=flat-square&logo=microsoftazure)](https://rullst-portfolio.redpond-24d9228d.eastus.azurecontainerapps.io/)

**The Sovereign SaaS, Full-Stack & Edge/IoT Showcase built with Rullst.**

</div>

---

## 🌟 Overview

Welcome to the official showcase and blueprints repository for the **Rullst Framework (`12.0.0`)**. This repository demonstrates how to architect, build, and deploy production-grade, sovereign web applications in pure Rust without JavaScript framework lock-in.

This monorepository contains three live cloud applications:
1. **🌐 Rullst Showcase App:** Demonstrates the **5 Frontend Paradigms**, Active Record ORM, WAF security layers, Prompt Injection Shield, and Omni cross-platform capabilities.
2. **🎓 Rullst Academy (LMS Blueprint):** A full-featured, real-world educational platform with 13 SQLite migrations, course catalog, Argon2 authentication, monotonic lesson progress tracking, quizzes, certifications, and integrated **Nexus Admin CMS** + **Studio Dev Cockpit**.
3. **💼 Rullst Portfolio Blueprint:** An ultra-fast, modern developer portfolio featuring dark glassmorphic UI, HTMX SSR, project showcase, experience timeline, skills matrix, and **Nexus Admin CMS**.

---

## ☁️ Live Cloud Deployments (Azure Container Apps)

All applications run in **Microsoft Azure Container Apps** on the **Azure for Students** plan using the Serverless Consumption Tier (Scale-to-Zero):

| Application | Live Public URL | Key Features | Admin / Cockpit |
| :--- | :--- | :--- | :--- |
| **🌐 Rullst Showcase** | [showcase.rullst.win](https://showcase.rullst.win) | 5 Web Paradigms, WAF Defense, LiveView, AI Assistant, Omni Simulator | Architecture Notices & Cockpit Guide |
| **🎓 LMS Academy** | [lms.rullst.win](https://lms.rullst.win) | Courses, Real Argon2 Auth, Lesson Player, Quizzes, Certificates | 🛡️ **Nexus Admin:** [/nexus](https://rullst-lms.redpond-24d9228d.eastus.azurecontainerapps.io/nexus)<br>🚀 **Studio:** [/studio](https://rullst-lms.redpond-24d9228d.eastus.azurecontainerapps.io/studio)<br>*(Protected by HTTP Basic Auth configured via container environment variables)* |
| **💼 Portfolio** | [portfolio.rullst.win](https://rullst-portfolio.redpond-24d9228d.eastus.azurecontainerapps.io/) | Dark Glassmorphic UI, HTMX SSR, Project Showcase, Experience Timeline | 🛡️ **Nexus Admin:** [/nexus](https://rullst-portfolio.redpond-24d9228d.eastus.azurecontainerapps.io/nexus)<br>*(Protected by HTTP Basic Auth configured via container environment variables)* |

---

## 🏛️ The 5 Web Paradigms in One Binary (Blog Showcase Only / Roadmap v13)

> [!WARNING]
> **Important Scope & Architecture Disclaimer (Roadmap Rullst v13.0):**  
> This section and the "5 Web Paradigms in One Binary" demonstration exist **exclusively as an exploratory architectural preview inside the Blog Showcase example (`rullst-showcase` / `examples/blog`)**.  
> This multi-paradigm unification **does NOT yet exist natively or completely within the core Rullst framework**. It is an experimental proof-of-concept targeted for potential inclusion in the **Rullst v13.0 Roadmap**.  
> In **Rullst v12.0**, the framework focuses strictly on Sovereign Zero-Bundle SSR (`html!` macro + HTMX), Active Record ORM, RASP/WAF defense layers, and the Nexus Admin CMS.

| Paradigm | Route | Technology | Footprint | Ideal For |
| :--- | :--- | :--- | :--- | :--- |
| **⚡ Zero-Bundle SSR** | `/` | Declarative HTML5 + `html!` macro + HTMX | 0 KB JS | High-speed SEO pages, CRUD apps, Sub-millisecond TTFB |
| **🔴 LiveView UI** | `/interactive-counter` | Server-driven UI over Tokio WebSockets | 0 Client JS | Real-time dashboards, live chats, collaborative tools |
| **🏝️ Reactive Islands** | `/omni-demo` | WebAssembly micro-frontends | Isolated WASM | Complex charts, offline calculators, rich markdown |
| **🎨 Classless CSS** | `/pico-demo` | Pure semantic HTML with Pico.css v2 engine | 0 JS / 0 NPM | Clean developer tooling, accessible responsive UIs |
| **📄 Classic Layouts** | `/templates-demo` | File-based Jinja2 / Tera templates | Decoupled HTML | Designers migrating from Laravel, Rails, or Loco.rs |

---

## 🎓 The LMS Academy Blueprint (`blueprints/lms`)

> [!TIP]
> **Live Interactive Showcase (Open Sandbox):**  
> This deployment is an active demonstration of the production LMS blueprint. It is 100% real and fully functional: visitors are welcome to [Sign Up](https://rullst-lms.redpond-24d9228d.eastus.azurecontainerapps.io/register) for a test account, [Log In](https://rullst-lms.redpond-24d9228d.eastus.azurecontainerapps.io/login) with real Argon2id password encryption, explore courses, and track lesson progress.

The **LMS Blueprint** is a full production-grade application generated via `cargo rullst new --blueprint lms`:

- **Active Record Architecture:** 22 domain models covering categories, courses, modules, lessons, quizzes, assignments, rubrics, achievements, and certifications.
- **Durable Migrations:** 13 sequential SQLite migrations applied automatically on container boot.
- **Identity & Security:** Full Argon2id password hashing, encrypted cookie sessions (`SameSite=Lax`, `Secure`), double-submit CSRF protection, and WAF defense.
- **Student Experience:** Public course search with category filtering, lesson player with captions, monotonic progress tracking (no regressions), and dynamic certificate verification.
- **Control Centers:**
  - 🛡️ **Nexus Admin Panel (`/nexus`):** Auto-generated admin CRUD for all 22 models.
  - 🚀 **Studio Cockpit (`/studio`):** Live telemetry, query profiler, and security radar.
  - **Access Security:** Protected by HTTP Basic Auth configured via `NEXUS_ADMIN_USERNAME` and `NEXUS_ADMIN_PASSWORD` environment variables.
- **Pre-Flight Verification:** See the official [LMS Audit Protocol](docs/BLUEPRINT_LMS_AUDIT_PROTOCOL.md) and [LMS Audit Report](docs/BLUEPRINT_LMS_AUDIT_REPORT.md).

---

## 📱 Rullst Omni: Multi-Platform Distribution & Instant PWA

> **"One Rust Codebase. Every Platform."**  
> Rullst eliminates the cost and friction of frontend rewrites. With **Rullst Omni** (`cargo rullst make:omni`), your responsive server-rendered application is automatically packaged into lightweight, native desktop and mobile executables without delegating business logic or cryptographic trust to untrusted client code.

```mermaid
graph TD
    RustServer["🦀 Rullst Core Server (Rust / Azure Container)"]
    Web["🌐 Responsive Zero-Bundle Web"]
    PWA["⚡ Instant PWA (Service Worker / Manifest)"]
    Desktop["🖥️ Omni Desktop (Tauri 2.0 / WebView2 / WebKit)"]
    Mobile["📱 Omni Mobile (Android APK & iOS Xcode)"]

    RustServer --> Web
    Web --> PWA
    RustServer --> Desktop
    RustServer --> Mobile

    Desktop --> Win["🪟 Windows (.exe / .msi)"]
    Desktop --> Mac["🍏 macOS (.dmg)"]
    Desktop --> Linux["🐧 Linux (.AppImage / .deb)"]

    Mobile --> Android["🤖 Android (.apk)"]
    Mobile --> iOS["🍏 iOS (.zip Xcode Project)"]
```

### 1. ⚡ Instant Progressive Web App (Zero-Download)
- **Install in 1 Second:** Direct home screen / desktop shortcut installation via **Brave**, **Chrome**, **Edge**, and **Safari** (iOS & macOS).
- **Zero App Store Fees:** No 30% revenue tax, zero developer program delays ($0 vs $99/year).
- **Background Offline Sync:** Service Worker caches shell UI and media metadata for instant offline boot.
- **Interactive Guide:** Visit the live [`/apps`](https://rullst-lms.redpond-24d9228d.eastus.azurecontainerapps.io/apps) page and click **"📖 Browser Guide"** to view step-by-step installation instructions for your specific browser.

### 2. 🖥️ Standalone Native Desktop Binaries (~5 MB Footprint)
- Uses the native OS webview (**WebView2** on Windows, **WebKit** on macOS and Linux) instead of bundling a 150 MB+ Chromium/Electron runtime.
- Strict Content Security Policy (CSP) and cryptographic navigation origin pinning.

### 3. 📱 Native Mobile Packages (Android & iOS)
- **Android:** Compiles directly to installable `Rullst.LMS_1.0.0.apk` using the Android SDK/NDK toolchain.
- **iOS:** Generates a complete native Apple Xcode project (`Rullst.LMS_1.0.0_iOS_Xcode.zip`) with Swift integration, ready to open in Xcode, run on iPhone/iPad via USB, or submit to the App Store.

### 📥 Official Release Downloads (v1.0.0)

All native binaries and packages are automatically built, tested, and published to GitHub Releases:

| Platform | Package Format | Direct Download | Notes |
| :--- | :--- | :--- | :--- |
| 🪟 **Windows** | Portable `.exe` | [Download Windows Setup](https://github.com/Rullst/examples/releases/download/v1.0.0/Rullst.LMS_1.0.0_x64-setup.exe) | Standard Windows NSIS installer |
| 🪟 **Windows** | Enterprise `.msi` | [Download Windows MSI](https://github.com/Rullst/examples/releases/download/v1.0.0/Rullst.LMS_1.0.0_x64_en-US.msi) | MSI package for enterprise deployment |
| 🍏 **macOS** | Universal `.dmg` | [Download macOS DMG](https://github.com/Rullst/examples/releases/download/v1.0.0/Rullst.LMS_1.0.0_aarch64.dmg) | Apple Silicon (M1/M2/M3/M4) & Intel |
| 🐧 **Linux** | Portable `.AppImage` | [Download Linux AppImage](https://github.com/Rullst/examples/releases/download/v1.0.0/Rullst.LMS_1.0.0_amd64.AppImage) | Runs on Ubuntu, Fedora, Arch, Debian |
| 🐧 **Linux** | Debian `.deb` | [Download Debian/Ubuntu Package](https://github.com/Rullst/examples/releases/download/v1.0.0/Rullst.LMS_1.0.0_amd64.deb) | `dpkg -i` / `apt install` ready |
| 🤖 **Android** | Mobile `.apk` | [Download Android APK](https://github.com/Rullst/examples/releases/download/v1.0.0/Rullst.LMS_1.0.0.apk) | Direct install on Android phones/tablets |
| 🍏 **iOS** | Xcode Project `.zip` | [Download iOS Xcode Project](https://github.com/Rullst/examples/releases/download/v1.0.0/Rullst.LMS_1.0.0_iOS_Xcode.zip) | Unzip and open in Xcode on macOS |

### 🛠️ Developer CLI Commands (`cargo rullst make:omni`)

To scaffold native Omni wrappers for any Rullst project:

```bash
# 1. Scaffold desktop shell:
cargo rullst make:omni --platform desktop

# 2. Scaffold mobile support with backend pinning:
cargo rullst make:omni --platform android,ios \
  --backend-url https://rullst-lms.redpond-24d9228d.eastus.azurecontainerapps.io \
  --identifier dev.rullst.lms \
  --product-name "Rullst LMS"

# 3. Test locally:
cargo rullst omni desktop   # Runs native desktop window
cargo rullst omni android   # Boots Android emulator with ADB reverse tunnel
cargo rullst omni ios       # Opens in iOS Simulator via Xcode
```

---

## 📦 Blueprints Catalog (`cargo-rullst`)

This showcase serves as the companion guide to the official scaffolds generated by the `cargo-rullst` CLI tool (`v12.0.0`):

```bash
# 1. Install the official CLI
cargo install cargo-rullst --version 12.0.0

# 2. Scaffold official blueprints:
cargo rullst new my-saas      --blueprint saas       # Multi-tenant SaaS with Billing, Stripe, and Subscriptions
cargo rullst new my-erp       --blueprint erp        # Double-entry Accounting, Inventory, Ledger, and RBAC
cargo rullst new my-lms       --blueprint lms        # Courses, Lessons, Quizzes, and Certifications
cargo rullst new my-portfolio --blueprint portfolio  # Ultra-fast developer showcase with dark glassmorphic UI
cargo rullst new my-app       --blueprint blank      # Minimal, pure high-performance skeleton
```

---

## 🚀 Running Locally

### Prerequisites
- **Rust 1.96+** (Rust 2024 Edition)
- **SQLite 3**

### Quickstart (Showcase App)
```bash
# 1. Clone this repository
git clone https://github.com/Rullst/examples.git
cd examples

# 2. Copy environment template
cp .env.example .env

# 3. Run the showcase server
cargo run
```

Access the local showcase endpoints:
- 🌐 **Web Showcase:** [http://127.0.0.1:3000](http://127.0.0.1:3000)
- 🛠️ **Rullst Studio (Dev Cockpit):** [http://127.0.0.1:5555](http://127.0.0.1:5555)
- 🛡️ **Nexus Admin CMS:** [http://127.0.0.1:3000/nexus](http://127.0.0.1:3000/nexus)

### Running the LMS Blueprint Locally
```bash
cd blueprints/lms
cargo run
# Open http://127.0.0.1:3000 (LMS Catalog, Login, Registration, Nexus, and Studio)
```

### Running the Portfolio Blueprint Locally
```bash
cd blueprints/portfolio
cargo run
# Open http://127.0.0.1:3000 (Dark Glassmorphism Portfolio & Nexus CMS at /nexus)
```

---

## 🐳 Container Architecture: Podman + Caddy + `cargo-chef`

This repository includes a production-grade **`Containerfile`** designed for memory-constrained environments (such as **Azure for Students B1s** instances with 1 GB RAM).

### ⚡ Blazing-Fast Caching with `cargo-chef`
Unlike standard container builds that take 10–15 minutes to recompile all dependencies on every code change, our build uses `cargo-chef`:
1. **Planner:** Generates `recipe.json` with dependency tree.
2. **Cooker:** Pre-compiles all external crates and caches the layer.
3. **Builder:** Compiles only your application code (< 30 seconds).

### 🏃 Running with Podman / Docker Compose

```bash
# Set your domain or public IP in .env
echo "DOMAIN=blog.yourdomain.com" >> .env

# Start Rullst + Caddy with automatic HTTPS
podman-compose up -d --build
# or
docker compose up -d --build
```

---

## ☁️ Azure for Students Free Deployment Guide

This project is optimized for the **Azure for Students** free tier (750 hours/month of Linux `Standard_B1s` VM) as well as **Azure Container Apps**:

### 1. Azure VM Setup
1. In the **Azure Portal**, create a Virtual Machine:
   - **Size:** `Standard_B1s` (1 vCPU, 1 GiB RAM).
   - **OS:** Ubuntu 24.04 LTS x64.
   - **Inbound Ports:** Open `22` (SSH), `80` (HTTP), and `443` (HTTPS).

### 2. Host Provisioning (One-Liner)
SSH into your Azure VM and run:
```bash
# Install Podman and Podman Compose (Daemonless, saves ~150 MB RAM vs Docker)
sudo apt update && sudo apt install -y podman podman-compose git
```

### 3. Clone & Launch
```bash
git clone https://github.com/Rullst/examples.git /opt/rullst-examples
cd /opt/rullst-examples
cp .env.example .env

# Configure your public IP or domain
nano .env

# Launch application with Caddy SSL auto-issuance
podman-compose up -d
```
Caddy will automatically obtain a valid **Let's Encrypt / ZeroSSL** certificate for your domain as soon as the first request arrives.

---

## 🔒 Security & Repository Lifecycle Note

- **Zero-Secret Guarantee:** This repository contains **no credentials, private keys, or API tokens**. All sensitive secrets must be passed via `.env` or container environment variables.
- **Audited Blueprint:** The LMS blueprint has passed the formal pre-flight integration audit ([Protocol](docs/BLUEPRINT_LMS_AUDIT_PROTOCOL.md) | [Report](docs/BLUEPRINT_LMS_AUDIT_REPORT.md)) with a **CONDITIONAL GO** verdict.

---

## 📄 License
Distributed under the **MIT License**. Created with ❤️ by the Rullst Core Team.
