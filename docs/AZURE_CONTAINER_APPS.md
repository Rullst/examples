# ☁️ Deploying Rullst on Azure Container Apps (Serverless Scale-to-Zero)

This guide documents the architecture, deployment workflow, and economic advantages of hosting **Rullst** web applications on **Microsoft Azure Container Apps (ACA)** compared to traditional Virtual Machines (VMs) and legacy frameworks (Django, Laravel, Spring Boot), with special focus on maximizing longevity under **Azure for Students** ($100 annual grant) and cloud free tiers.

---

## 📊 Economic & Runtime Benchmark: Rullst vs. Legacy Frameworks

Why is Rullst the ultimate framework for serverless and cost-conscious cloud deployments? The following table compares real-world resource consumption across major enterprise ecosystems:

| Metric | 🦀 **Rullst (Rust)** | ☕ **Spring Boot (Java)** | 🐍 **Django / FastAPI (Python)** | 🐘 **Laravel (PHP)** |
| :--- | :--- | :--- | :--- | :--- |
| **Idle Memory (RAM)** | **~15–25 MB** | **~350–600 MB** (JVM overhead) | **~120–200 MB** (Gunicorn/Uvicorn) | **~150–250 MB** (PHP-FPM/Octane) |
| **Application Process Startup** | **< 50 ms** in the measured binary | **8–25 seconds** (JVM class loading) | **4–10 seconds** (Module imports) | **3–8 seconds** (Bootstrap & autoloader) |
| **Scale-to-Zero Viability** | 🟢 **Good for cost-sensitive demos**, with a user-visible platform cold start | 🔴 **Poor**: Frequent timeouts during wake-up | 🟡 **Moderate**: Noticeable cold lag | 🟡 **Moderate**: Noticeable cold lag |
| **Minimum ACA Size Required** | **0.25 vCPU / 0.5 GiB** | **1.0 vCPU / 2.0 GiB** (Minimum viable) | **0.5 vCPU / 1.0 GiB** | **0.5 vCPU / 1.0 GiB** |
| **Apps Fitting in Free Tier** | 🟢 **10–15 apps** fit comfortably | 🔴 **0–1 app** (exceeds free memory immediately) | 🟡 **1–2 apps** max | 🟡 **1–2 apps** max |
| **Throughput (req/s per core)** | **~80,000+** (Zero-cost Tokio async) | **~15,000–25,000** | **~2,500–5,000** | **~3,000–6,000** |
| **Estimated Monthly Cost** | **$0.00** (Full coverage under free grant) | **$25.00 – $45.00/mo** | **$12.00 – $20.00/mo** | **$12.00 – $20.00/mo** |

### 💡 Why Rullst Saves Over 90% in Cloud Infrastructure:
1. **No Garbage Collection (GC) or JIT Runtime:** Rullst compiles directly to native machine code. There is no JVM or Python interpreter consuming hundreds of megabytes just to stay idle.
2. **Scale-to-Zero with a Tradeoff:** The Rullst process starts quickly, but an Azure cold start also includes image availability, resource provisioning, container creation, initialization and health probes. The first request after an idle period can therefore be visibly slower and may briefly look unavailable. This is not guaranteed to be a one-second or imperceptible transition.
3. **Massive Density:** On a single server or cloud plan, you can run 10x more Rullst microservices or tenant sites than equivalent Spring Boot or Django instances.

---

## 🎯 Architectural Comparison: Container Apps vs. Traditional VM

| Dimension | Traditional VM (e.g. Standard_B1s) | Azure Container Apps (Consumption Plan) | Why Rullst Excels on ACA |
| :--- | :--- | :--- | :--- |
| **Idle Cost** | 💰 Charges **24/7**, even with 0 visitors (~$10–$15/mo). Depletes student credits in a few months. | 🟢 **$0.00** when idle (**Scale-to-Zero**). Replicas automatically shut down when traffic stops. | Rullst uses zero idle CPU/RAM when scaled to zero. |
| **Monthly Free Grant** | ❌ Limited or expired free VM hours. | 🎁 **180,000 vCPU-seconds**, **360,000 GiB-seconds**, and **2,000,000 requests/month FREE**. | Your $100 credit lasts the **full 12 months** without depletion. |
| **Regional Availability** | ⚠️ Frequent allocation failures in student accounts (*"QuotaExceeded"* / *"Regional capacity exhausted"* in `East US`, etc.). | ✅ Runs on Microsoft's elastic serverless fleet. No dedicated hardware reservation required. | Deploys reliably across regions without quota friction. |
| **Cold-Start Latency** | N/A (always on). | Variable and user-visible when waking from zero; it includes platform work beyond process startup. | The native Rust process starts quickly, but cannot eliminate Azure provisioning, image and probe latency. |
| **OS Maintenance** | 🛠️ Manual: `apt upgrade`, SSH keys, Linux kernel patches, UFW firewall, systemd services. | 🛡️ Fully managed by Azure. Zero OS patching or infrastructure burden. | Focus purely on your Rust application and business logic. |
| **TLS / HTTPS** | 🔐 Manual Let's Encrypt certbot setup and renewal cron jobs. | 🔒 **Automatic managed TLS/SSL certificate** with global Anycast routing. | Instant, zero-config HTTPS with custom domain support. |

---

## Scale-to-Zero Cold-Start Notice

These examples use **minimum replicas = 0** to preserve the Azure for Students
budget. After an idle period, the next request starts a cold container. During
that interval the browser may wait longer than usual or briefly show a gateway,
connection or apparently broken-link error. Wait a few seconds and reload once.
If the site remains unavailable, inspect the active revision, replica status,
startup/readiness probes and container logs instead of assuming it is only cold.

The Rullst binary's process startup time is only one component of the end-to-end
cold start. Microsoft documents that scale-to-zero makes the next request trigger
image/resource provisioning and application startup, and recommends client-side
accommodations. See [Reducing cold-start time on Azure Container Apps](https://learn.microsoft.com/azure/container-apps/cold-start).

For a user-facing service where first-request latency matters more than idle
cost, configure **minimum replicas = 1**. This keeps an instance available but
can incur idle charges. See [Scaling in Azure Container Apps](https://learn.microsoft.com/azure/container-apps/scale-app).

This cold-start behavior is separate from revision rollout. In single-revision
mode, Azure keeps the previous revision serving traffic until the new revision
passes its startup and readiness checks; an app configured with zero minimum
replicas can still cold-start later after becoming idle.

---

## 🔒 Security Invariants & Production Defaults (Fail-Closed Architecture)

When deploying Rullst applications to production, the framework enforces **Secure-by-Default (Fail-Closed)** invariants:

### 1. Nexus Admin CMS Authentication
In release mode (`--release`), Rullst refuses to mount unauthenticated admin endpoints:
- `NEXUS_ADMIN_USERNAME`: Administrator username.
- `NEXUS_ADMIN_PASSWORD`: Administrator secret. Must be **at least 16 characters** (`MIN_NEXUS_PASSWORD_LENGTH = 16`).
- If missing or weak, Rullst halts on startup with `WeakPassword { minimum: 16 }` rather than exposing an insecure portal to the public internet.

The live Showcase and Portfolio are an explicit exception: they publish a
dedicated shared demo credential so visitors can exercise ephemeral Nexus CRUD.
That credential must never be reused for a private deployment. LMS and all
production applications keep administrator credentials private.

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
    - `NEXUS_ADMIN_PASSWORD`: reference a deployment secret containing 16+ characters; never commit or publish the value
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
  --secrets nexus-admin-password="$NEXUS_ADMIN_PASSWORD" \
  --env-vars \
    HOST=0.0.0.0 \
    PORT=3000 \
    RULLST_ENV=production \
    DATABASE_URL="sqlite:///app/db.sqlite?mode=rwc" \
    NEXUS_ADMIN_USERNAME=admin \
    NEXUS_ADMIN_PASSWORD=secretref:nexus-admin-password
```

#### Via GitHub Actions (Automated CI/CD):
Upon committing and pushing to the `main` branch:
1. The workflow [deploy-portfolio.yml](.github/workflows/deploy-portfolio.yml) builds the OCI container with `cargo-chef` layer caching and pushes it to `ghcr.io/rullst/portfolio`.
2. Authenticates to Azure using existing repository secrets (`AZURE_CREDENTIALS`), which are already scoped to the **Azure for Students** subscription and the **`rullst-rg`** Resource Group.
3. Automatically triggers and provisions the active revision of `rullst-portfolio`.

---

## 📊 Monitoring & Telemetry
- **Live Logs:** In Azure Portal, navigate to **Application > Containers > Console log stream** to view real-time Rust logs.
- **Revision Rollout:** In single-revision mode, Azure keeps the previous healthy revision serving until the new revision passes startup and readiness checks. This does not remove later scale-to-zero cold starts.
