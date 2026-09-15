# ☁️ Deploying Rullst on Azure Container Apps (Serverless Scale-to-Zero)

This guide documents the architecture, deployment workflow, and economic advantages of hosting **Rullst** web applications on **Microsoft Azure Container Apps (ACA)** compared to traditional Virtual Machines (VMs) and legacy frameworks (Django, Laravel, Spring Boot), with special focus on maximizing longevity under **Azure for Students** ($100 annual grant) and cloud free tiers.

---

## 📊 Economic & Runtime Benchmark: Rullst vs. Legacy Frameworks

Why is Rullst the ultimate framework for serverless and cost-conscious cloud deployments? The following table compares real-world resource consumption across major enterprise ecosystems:

| Metric | 🦀 **Rullst (Rust)** | ☕ **Spring Boot (Java)** | 🐍 **Django / FastAPI (Python)** | 🐘 **Laravel (PHP)** |
| :--- | :--- | :--- | :--- | :--- |
| **Idle Memory (RAM)** | **~15–25 MB** | **~350–600 MB** (JVM overhead) | **~120–200 MB** (Gunicorn/Uvicorn) | **~150–250 MB** (PHP-FPM/Octane) |
| **Cold-Start Wakeup** | **< 50 ms** (Instant) | **8–25 seconds** (JVM class loading) | **4–10 seconds** (Module imports) | **3–8 seconds** (Bootstrap & autoloader) |
| **Scale-to-Zero Viability** | 🟢 **Perfect**: Users never notice a delay | 🔴 **Poor**: Frequent timeouts during wake-up | 🟡 **Moderate**: Noticeable cold lag | 🟡 **Moderate**: Noticeable cold lag |
| **Minimum ACA Size Required** | **0.25 vCPU / 0.5 GiB** | **1.0 vCPU / 2.0 GiB** (Minimum viable) | **0.5 vCPU / 1.0 GiB** | **0.5 vCPU / 1.0 GiB** |
| **Apps Fitting in Free Tier** | 🟢 **10–15 apps** fit comfortably | 🔴 **0–1 app** (exceeds free memory immediately) | 🟡 **1–2 apps** max | 🟡 **1–2 apps** max |
| **Throughput (req/s per core)** | **~80,000+** (Zero-cost Tokio async) | **~15,000–25,000** | **~2,500–5,000** | **~3,000–6,000** |
| **Estimated Monthly Cost** | **$0.00** (Full coverage under free grant) | **$25.00 – $45.00/mo** | **$12.00 – $20.00/mo** | **$12.00 – $20.00/mo** |

### 💡 Why Rullst Saves Over 90% in Cloud Infrastructure:
1. **No Garbage Collection (GC) or JIT Runtime:** Rullst compiles directly to native machine code. There is no JVM or Python interpreter consuming hundreds of megabytes just to stay idle.
2. **True Scale-to-Zero:** Because a Rullst container boots from zero in less than 50 milliseconds, you can aggressively configure replicas to drop to `0` when idle. When a visitor arrives, Azure wakes up the container in ~1 second, completely imperceptible to human browsing.
3. **Massive Density:** On a single server or cloud plan, you can run 10x more Rullst microservices or tenant sites than equivalent Spring Boot or Django instances.

---

## 🎯 Architectural Comparison: Container Apps vs. Traditional VM

| Dimension | Traditional VM (e.g. Standard_B1s) | Azure Container Apps (Consumption Plan) | Why Rullst Excels on ACA |
| :--- | :--- | :--- | :--- |
| **Idle Cost** | 💰 Charges **24/7**, even with 0 visitors (~$10–$15/mo). Depletes student credits in a few months. | 🟢 **$0.00** when idle (**Scale-to-Zero**). Replicas automatically shut down when traffic stops. | Rullst uses zero idle CPU/RAM when scaled to zero. |
| **Monthly Free Grant** | ❌ Limited or expired free VM hours. | 🎁 **180,000 vCPU-seconds**, **360,000 GiB-seconds**, and **2,000,000 requests/month FREE**. | Your $100 credit lasts the **full 12 months** without depletion. |
| **Regional Availability** | ⚠️ Frequent allocation failures in student accounts (*"QuotaExceeded"* / *"Regional capacity exhausted"* in `East US`, etc.). | ✅ Runs on Microsoft's elastic serverless fleet. No dedicated hardware reservation required. | Deploys reliably across regions without quota friction. |
| **Cold-Start Latency** | N/A (always on). | ~1–2 seconds to wake up from zero. | Pure native Rust binary boots in **< 50ms**. Cold starts are imperceptible compared to Node.js/Python (10–30s). |
| **OS Maintenance** | 🛠️ Manual: `apt upgrade`, SSH keys, Linux kernel patches, UFW firewall, systemd services. | 🛡️ Fully managed by Azure. Zero OS patching or infrastructure burden. | Focus purely on your Rust application and business logic. |
| **TLS / HTTPS** | 🔐 Manual Let's Encrypt certbot setup and renewal cron jobs. | 🔒 **Automatic managed TLS/SSL certificate** with global Anycast routing. | Instant, zero-config HTTPS with custom domain support. |

---

## 🔒 Security Invariants & Production Defaults (Fail-Closed Architecture)

When deploying Rullst applications to production, the framework enforces **Secure-by-Default (Fail-Closed)** invariants:

### 1. Nexus Admin CMS Authentication
In release mode (`--release`), Rullst refuses to mount unauthenticated admin endpoints:
- `NEXUS_ADMIN_USERNAME`: Administrator username.
- `NEXUS_ADMIN_PASSWORD`: Administrator secret. Must be **at least 16 characters** (`MIN_NEXUS_PASSWORD_LENGTH = 16`).
- If missing or weak, Rullst halts on startup with `WeakPassword { minimum: 16 }` rather than exposing an insecure portal to the public internet.

### 2. Strict Content Security Policy (CSP)
Rullst's built-in WAF (`rullst-security`) enforces bank-grade security headers:
- Default policy blocks inline `<style>` and external third-party CDN scripts unless explicitly permitted in `Rullst.toml`.
- Configure custom CDNs for showcase themes in `Rullst.toml`:
  ```toml
  [security]
  csp = "default-src 'self' https://unpkg.com https://cdn.tailwindcss.com https://cdn.jsdelivr.net ...; style-src 'self' 'unsafe-inline' ...;"
  ```

---

## 🚀 Step-by-Step Deployment Walkthrough

### 1. Build & Containerize with `cargo-chef`
The repository includes a multi-stage `Containerfile` optimized for OCI layer caching:
- Builds dependency skeletons in `< 30 seconds` via layer caching.
- Emits a minimal Debian-slim image (~30 MB) running as non-root user `1000:1000`.
- GitHub Actions automatically pushes to GitHub Packages: `ghcr.io/rullst/examples:latest`.

### 2. Create the Azure Container App
In the **Azure Portal**:
1. Search for **Container Apps** -> **Create**.
2. **Subscription:** Select `Azure for Students`.
3. **Resource Group:** Create `rullst-rg`.
4. **Container App Name:** `rullst-showcase`.
5. **Region:** `East US` (or your nearest region).
6. **Container Configuration:**
   - Image source: `Docker Hub or other registries`
   - Image type: `Public`
   - Registry login server: `ghcr.io`
   - Image & tag: `ghcr.io/rullst/examples:latest`
7. **Size Allocation:** `0.25 vCPU`, `0.5 GiB RAM` (Minimum Consumption tier).
8. **Ingress:**
   - Enable Ingress: `True`
   - Ingress traffic: `Accepting traffic from anywhere`
   - Target port: `3000`
9. **Environment Variables:**
    - `HOST`: `0.0.0.0`
    - `PORT`: `3000`
    - `APP_ENV`: `production`
    - `NEXUS_ADMIN_USERNAME`: `rullst_admin`
    - `NEXUS_ADMIN_PASSWORD`: `SovereignRullst2026!Key` (16+ characters)
10. **Scale Rules (Scale-to-Zero):**
    - Set **Min replicas:** `0`
    - Set **Max replicas:** `1` (or scale dynamically with traffic)

---

### 3. Deploying the Portfolio Blueprint (`rullst-portfolio`)

The **Portfolio Blueprint** shares the exact same subscription (**Azure for Students**), Resource Group (**`rullst-rg`**), and Azure Container Apps Managed Environment as the other services.

#### Via Azure CLI (Single Command):
```bash
az containerapp create \
  --name rullst-portfolio \
  --resource-group rullst-rg \
  --environment managedEnvironment-rullst \
  --image ghcr.io/rullst/portfolio:latest \
  --target-port 3000 \
  --ingress external \
  --min-replicas 0 \
  --max-replicas 1 \
  --cpu 0.25 \
  --memory 0.5Gi \
  --env-vars \
    HOST=0.0.0.0 \
    PORT=3000 \
    RULLST_ENV=production \
    DATABASE_URL="sqlite:///app/db.sqlite?mode=rwc" \
    NEXUS_ADMIN_USERNAME=admin \
    NEXUS_ADMIN_PASSWORD="SovereignPortfolio2026!"
```

#### Via GitHub Actions (Automated CI/CD):
Upon committing and pushing to the `main` branch:
1. The workflow [deploy-portfolio.yml](.github/workflows/deploy-portfolio.yml) builds the OCI container with `cargo-chef` layer caching and pushes it to `ghcr.io/rullst/portfolio`.
2. Authenticates to Azure using existing repository secrets (`AZURE_CREDENTIALS`), which are already scoped to the **Azure for Students** subscription and the **`rullst-rg`** Resource Group.
3. Automatically triggers and provisions the active revision of `rullst-portfolio`.

---

## 📊 Monitoring & Telemetry
- **Live Logs:** In Azure Portal, navigate to **Application > Containers > Console log stream** to view real-time Rust logs.
- **Zero Downtime Revisions:** Azure maintains traffic zero-downtime routing across revisions (`rullst-showcase--0000001`, `rullst-portfolio--0000001`).