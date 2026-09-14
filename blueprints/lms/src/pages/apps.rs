// blueprints/lms/src/pages/apps.rs - Rullst Omni Multi-Platform Showcase & Browser Guide
use axum::response::{Html, IntoResponse};
use rullst::html;

pub async fn apps_page() -> impl IntoResponse {
    Html(html! {
        <html lang="en" class="dark">
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <title>"Rullst Omni - Universal Multiplatform Applications"</title>
                <link rel="manifest" href="/manifest.webmanifest" />
                <meta name="theme-color" content="#080b11" />
                <link rel="icon" type="image/png" href="https://raw.githubusercontent.com/Rullst/Rullst/main/Rullst.png" />
                <style>
                    r#"
                    * { box-sizing: border-box; margin: 0; padding: 0; }
                    body {
                        background: #080b11;
                        color: #f8fafc;
                        font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                        line-height: 1.6;
                        min-height: 100vh;
                        padding: 1.5rem;
                    }
                    .container {
                        max-width: 1200px;
                        margin: 0 auto;
                    }
                    header {
                        display: flex;
                        justify-content: space-between;
                        align-items: center;
                        border-bottom: 1px solid #1e293b;
                        padding-bottom: 1rem;
                        margin-bottom: 2rem;
                    }
                    .brand {
                        display: flex;
                        align-items: center;
                        gap: 0.75rem;
                        text-decoration: none;
                        color: #f8fafc;
                        font-weight: 800;
                        font-size: 1.25rem;
                    }
                    .brand-badge {
                        background: rgba(16, 185, 129, 0.15);
                        color: #34d399;
                        border: 1px solid rgba(16, 185, 129, 0.3);
                        font-size: 0.75rem;
                        font-weight: 700;
                        padding: 0.2rem 0.5rem;
                        border-radius: 9999px;
                        text-transform: uppercase;
                    }
                    .back-link {
                        color: #94a3b8;
                        text-decoration: none;
                        font-size: 0.9rem;
                        font-weight: 600;
                        transition: color 0.15s;
                    }
                    .back-link:hover {
                        color: #38bdf8;
                    }
                    .hero {
                        background: linear-gradient(135deg, rgba(15, 23, 42, 0.8), rgba(30, 41, 59, 0.4));
                        border: 1px solid #1e293b;
                        border-radius: 1rem;
                        padding: 2.5rem;
                        margin-bottom: 2.5rem;
                        position: relative;
                        overflow: hidden;
                    }
                    .hero::before {
                        content: '';
                        position: absolute;
                        top: -50%;
                        right: -20%;
                        width: 400px;
                        height: 400px;
                        background: radial-gradient(circle, rgba(16, 185, 129, 0.12) 0%, transparent 70%);
                        pointer-events: none;
                    }
                    .hero h1 {
                        font-size: 2.25rem;
                        font-weight: 900;
                        margin-bottom: 0.75rem;
                        background: linear-gradient(to right, #ffffff, #94a3b8);
                        -webkit-background-clip: text;
                        -webkit-text-fill-color: transparent;
                    }
                    .hero p {
                        color: #cbd5e1;
                        font-size: 1.1rem;
                        max-width: 800px;
                        margin-bottom: 1.5rem;
                    }
                    .pwa-banner {
                        background: rgba(16, 185, 129, 0.08);
                        border: 1px solid rgba(16, 185, 129, 0.3);
                        border-radius: 0.75rem;
                        padding: 1.25rem 1.5rem;
                        display: flex;
                        justify-content: space-between;
                        align-items: center;
                        gap: 1.5rem;
                        flex-wrap: wrap;
                    }
                    .pwa-info h3 {
                        color: #34d399;
                        font-size: 1.1rem;
                        font-weight: 700;
                        margin-bottom: 0.25rem;
                    }
                    .pwa-info p {
                        color: #94a3b8;
                        font-size: 0.9rem;
                        margin-bottom: 0;
                    }
                    .banner-actions {
                        display: flex;
                        gap: 0.75rem;
                        align-items: center;
                        flex-wrap: wrap;
                    }
                    .btn-install {
                        background: #10b981;
                        color: #052e16;
                        border: none;
                        font-weight: 800;
                        padding: 0.75rem 1.5rem;
                        border-radius: 0.5rem;
                        cursor: pointer;
                        text-decoration: none;
                        display: inline-flex;
                        align-items: center;
                        gap: 0.5rem;
                        box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3);
                        transition: transform 0.15s, background 0.15s;
                    }
                    .btn-install:hover {
                        background: #059669;
                        transform: translateY(-1px);
                    }
                    .btn-instructions {
                        background: #1e293b;
                        color: #38bdf8;
                        border: 1px solid rgba(56, 189, 248, 0.3);
                        font-weight: 700;
                        padding: 0.75rem 1.25rem;
                        border-radius: 0.5rem;
                        cursor: pointer;
                        transition: background 0.15s, border-color 0.15s;
                    }
                    .btn-instructions:hover {
                        background: rgba(56, 189, 248, 0.15);
                        border-color: #38bdf8;
                    }
                    .grid {
                        display: grid;
                        grid-template-columns: 1fr 420px;
                        gap: 2.5rem;
                        align-items: start;
                    }
                    @media (max-width: 1024px) {
                        .grid {
                            grid-template-columns: 1fr;
                        }
                    }
                    .card {
                        background: #0f172a;
                        border: 1px solid #1e293b;
                        border-radius: 0.875rem;
                        padding: 1.75rem;
                        margin-bottom: 2rem;
                    }
                    .card-header {
                        display: flex;
                        justify-content: space-between;
                        align-items: center;
                        margin-bottom: 1rem;
                    }
                    .badge {
                        font-size: 0.75rem;
                        font-weight: 700;
                        text-transform: uppercase;
                        padding: 0.25rem 0.6rem;
                        border-radius: 0.375rem;
                    }
                    .badge-desktop {
                        background: rgba(56, 189, 248, 0.15);
                        color: #38bdf8;
                        border: 1px solid rgba(56, 189, 248, 0.3);
                    }
                    .badge-mobile {
                        background: rgba(16, 185, 129, 0.15);
                        color: #34d399;
                        border: 1px solid rgba(16, 185, 129, 0.3);
                    }
                    .card h2 {
                        font-size: 1.35rem;
                        color: #f8fafc;
                        margin-bottom: 0.5rem;
                    }
                    .card p {
                        color: #94a3b8;
                        font-size: 0.95rem;
                        margin-bottom: 1.25rem;
                    }
                    .code-block {
                        background: #060910;
                        border: 1px solid #1e293b;
                        border-radius: 0.5rem;
                        padding: 1rem;
                        font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
                        font-size: 0.85rem;
                        color: #38bdf8;
                        white-space: pre-wrap;
                        margin-bottom: 1.25rem;
                    }
                    .downloads-row {
                        display: flex;
                        gap: 0.75rem;
                        flex-wrap: wrap;
                    }
                    .download-btn {
                        background: #1e293b;
                        border: 1px solid #334155;
                        color: #f8fafc;
                        padding: 0.6rem 1rem;
                        border-radius: 0.5rem;
                        font-size: 0.85rem;
                        font-weight: 600;
                        text-decoration: none;
                        display: inline-flex;
                        align-items: center;
                        gap: 0.4rem;
                        transition: background 0.15s, border-color 0.15s;
                    }
                    .download-btn:hover {
                        background: #334155;
                        border-color: #38bdf8;
                    }
                    /* Browser guide card */
                    .browser-guide-card {
                        background: #0d1322;
                        border: 1px solid #1e293b;
                        border-radius: 0.875rem;
                        padding: 1.5rem;
                        margin-bottom: 2rem;
                    }
                    .browser-tabs-nav {
                        display: flex;
                        gap: 0.5rem;
                        flex-wrap: wrap;
                        border-bottom: 1px solid #1e293b;
                        padding-bottom: 0.75rem;
                        margin-bottom: 1.25rem;
                    }
                    .tab-btn {
                        background: #111827;
                        border: 1px solid #334155;
                        color: #94a3b8;
                        padding: 0.5rem 0.9rem;
                        border-radius: 0.5rem;
                        font-size: 0.85rem;
                        font-weight: 700;
                        cursor: pointer;
                        transition: all 0.15s;
                    }
                    .tab-btn:hover {
                        color: #f8fafc;
                        border-color: #475569;
                    }
                    .tab-btn.active {
                        background: rgba(56, 189, 248, 0.15);
                        border-color: #38bdf8;
                        color: #38bdf8;
                    }
                    .guide-pane {
                        display: none;
                    }
                    .guide-pane.active {
                        display: block;
                    }
                    .guide-step {
                        background: rgba(15, 23, 42, 0.7);
                        border: 1px solid #1e293b;
                        border-radius: 0.5rem;
                        padding: 1rem 1.25rem;
                        margin-bottom: 0.75rem;
                    }
                    .guide-step strong {
                        color: #38bdf8;
                    }
                    .guide-step p {
                        margin-bottom: 0;
                        color: #cbd5e1;
                        font-size: 0.9rem;
                    }
                    /* Simulator styling */
                    .sim-wrapper {
                        text-align: center;
                        position: sticky;
                        top: 2rem;
                    }
                    .sim-title {
                        font-size: 0.9rem;
                        font-weight: 700;
                        color: #38bdf8;
                        margin-bottom: 0.25rem;
                    }
                    .sim-subtitle {
                        font-size: 0.75rem;
                        color: #64748b;
                        margin-bottom: 1rem;
                    }
                    .phone-frame {
                        background: #0b0f19;
                        border: 10px solid #1e293b;
                        border-radius: 40px;
                        box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.8), 0 0 0 1px #334155;
                        overflow: hidden;
                        width: 360px;
                        height: 720px;
                        margin: 0 auto;
                        display: flex;
                        flex-direction: column;
                    }
                    .phone-notch {
                        background: #0f172a;
                        height: 28px;
                        display: flex;
                        align-items: center;
                        justify-content: space-between;
                        padding: 0 1.25rem;
                        font-size: 0.72rem;
                        color: #94a3b8;
                        border-bottom: 1px solid #1e293b;
                        user-select: none;
                    }
                    .phone-screen {
                        flex-grow: 1;
                        width: 100%;
                        border: none;
                        background: #080b11;
                    }
                    .phone-bar {
                        height: 18px;
                        background: #0f172a;
                        display: flex;
                        justify-content: center;
                        align-items: center;
                    }
                    .home-pill {
                        width: 100px;
                        height: 4px;
                        background: #475569;
                        border-radius: 9999px;
                    }
                    .sim-controls {
                        display: flex;
                        justify-content: center;
                        gap: 0.5rem;
                        margin-top: 1rem;
                    }
                    .sim-btn {
                        background: #1e293b;
                        border: 1px solid #334155;
                        color: #cbd5e1;
                        padding: 0.4rem 0.8rem;
                        border-radius: 0.375rem;
                        font-size: 0.75rem;
                        font-weight: 600;
                        cursor: pointer;
                        transition: background 0.15s;
                    }
                    .sim-btn:hover {
                        background: #334155;
                        color: #ffffff;
                    }
                    /* Modal styles */
                    .modal-backdrop {
                        position: fixed;
                        inset: 0;
                        background: rgba(0, 0, 0, 0.8);
                        backdrop-filter: blur(8px);
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        padding: 1rem;
                        z-index: 9999;
                    }
                    .modal-card {
                        background: #0f172a;
                        border: 1px solid #334155;
                        border-radius: 1rem;
                        padding: 2rem;
                        max-width: 650px;
                        width: 100%;
                        box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.9);
                    }
                    .modal-header {
                        display: flex;
                        justify-content: space-between;
                        align-items: center;
                        margin-bottom: 1rem;
                    }
                    .modal-close {
                        background: transparent;
                        border: none;
                        color: #94a3b8;
                        font-size: 1.5rem;
                        cursor: pointer;
                        padding: 0.25rem 0.5rem;
                        border-radius: 0.25rem;
                    }
                    .modal-close:hover {
                        color: #ffffff;
                    }
                    "#
                </style>
            </head>
            <body>
                <div class="container">
                    <header>
                        <a href="/" class="brand">
                            <span style="font-size:1.5rem">"⚡"</span>
                            <span>"Rullst LMS"</span>
                            <span class="brand-badge">"Rullst Omni"</span>
                        </a>
                        <a href="/" class="back-link">"← Back to Course Catalog"</a>
                    </header>

                    <section class="hero">
                        <h1>"One Rust Codebase. Every Platform."</h1>
                        <p>
                            "Experience the full power of the Rullst Framework. Rullst eliminates frontend rewrite silos by packaging your responsive web application directly into native Desktop and Mobile executables with Tauri and strict zero-trust security."
                        </p>
                        <div class="pwa-banner">
                            <div class="pwa-info">
                                <h3>"⚡ Instant Progressive Web App"</h3>
                                <p>"No store download needed. Install directly to your Home Screen or Desktop with 1 click."</p>
                            </div>
                            <div class="banner-actions">
                                <button id="pwa-install-btn" class="btn-install" type="button">
                                    "⬇️ Install Web App"
                                </button>
                                <button type="button" class="btn-instructions" onclick="openModal()">
                                    "📖 Browser Guide"
                                </button>
                            </div>
                        </div>
                    </section>

                    <div class="grid">
                        <main>
                            
                            <div class="card">
                                <div class="card-header">
                                    <span class="badge badge-desktop">"🖥️ Desktop Target"</span>
                                    <span style="font-size:0.75rem;color:#10b981;font-weight:700">"Tauri 2.0 Engine"</span>
                                </div>
                                <h2>"Standalone Native Desktop Binary"</h2>
                                <p>
                                    "Compiles a lightweight native executable (~5 MB) for Windows, macOS, and Linux using the system webview (WebView2 on Windows, WebKit on macOS/Linux) with zero Electron memory bloat."
                                </p>
                                <div class="code-block">
                                    "# 1. Scaffold Omni desktop packaging:
"
                                    "cargo rullst make:omni --platform desktop

"
                                    "# 2. Run the desktop application locally:
"
                                    "cargo rullst omni desktop"
                                </div>
                                <div class="downloads-row">
                                    <a href="https://github.com/Rullst/examples/releases/download/v1.0.0/Rullst.LMS_1.0.0_x64-setup.exe" target="_blank" class="download-btn">
                                        "🪟 Windows (.exe)"
                                    </a>
                                    <a href="https://github.com/Rullst/examples/releases/download/v1.0.0/Rullst.LMS_1.0.0_aarch64.dmg" target="_blank" class="download-btn">
                                        "🍏 macOS (.dmg)"
                                    </a>
                                    <a href="https://github.com/Rullst/examples/releases/download/v1.0.0/Rullst.LMS_1.0.0_amd64.AppImage" target="_blank" class="download-btn">
                                        "🐧 Linux (.AppImage)"
                                    </a>
                                </div>
                            </div>

                            
                            <div class="card">
                                <div class="card-header">
                                    <span class="badge badge-mobile">"📱 Mobile Target"</span>
                                    <span style="font-size:0.75rem;color:#38bdf8;font-weight:700">"Android & iOS"</span>
                                </div>
                                <h2>"Native Mobile Application Package"</h2>
                                <p>
                                    "Packages the responsive LMS interface into native Android APK and iOS shells with cryptographic origin pinning, hardware-accelerated rendering, and offline fallback."
                                </p>
                                <div class="code-block">
                                    "# 1. Scaffold mobile project:
"
                                    "cargo rullst make:omni --platform android --identifier dev.rullst.lms

"
                                    "# 2. Run in Android emulator:
"
                                    "cargo rullst omni android"
                                </div>
                                <div class="downloads-row">
                                    <a href="https://github.com/Rullst/examples/releases/download/v1.0.0/Rullst.LMS_1.0.0.apk" target="_blank" class="download-btn">
                                        "🤖 Download Android APK"
                                    </a>
                                    <a href="https://github.com/Rullst/examples/releases/download/v1.0.0/Rullst.LMS_1.0.0_iOS_Xcode.zip" target="_blank" class="download-btn">
                                        "📱 iOS (Xcode Project .zip)"
                                    </a>
                                </div>
                            </div>

                            
                            <div class="card" style="border-color:#334155;background:#090d16">
                                <h2 style="font-size:1.1rem;color:#38bdf8;margin-bottom:0.5rem">"🏛️ The Rullst Web-First Invariant"</h2>
                                <p style="font-size:0.875rem;margin-bottom:0;color:#94a3b8">
                                    "The Rullst server remains the single source of truth for business rules, authorization, session encryption, and data integrity. Native Omni shells bring system integration and push notifications without delegating trust to untrusted client code."
                                </p>
                            </div>
                        </main>

                        
                        <aside class="sim-wrapper">
                            <div class="sim-title">"📱 Live Mobile Simulator"</div>
                            <div class="sim-subtitle">"Previewing responsive view in real-time"</div>

                            <div class="phone-frame">
                                <div class="phone-notch">
                                    <span>"9:41"</span>
                                    <span style="display:flex;gap:0.35rem;align-items:center">
                                        "5G"
                                        <span>"📶"</span>
                                        <span>"🔋"</span>
                                    </span>
                                </div>
                                <iframe id="sim-frame" src="/" class="phone-screen" title="Mobile Simulator"></iframe>
                                <div class="phone-bar">
                                    <div class="home-pill"></div>
                                </div>
                            </div>

                            <div class="sim-controls">
                                <button type="button" class="sim-btn" onclick="document.getElementById('sim-frame').src = '/'">
                                    "📚 Catalog"
                                </button>
                                <button type="button" class="sim-btn" onclick="document.getElementById('sim-frame').src = '/courses/1'">
                                    "🎓 Course"
                                </button>
                                <button type="button" class="sim-btn" onclick="document.getElementById('sim-frame').src = '/lessons/1/play'">
                                    "▶️ Player"
                                </button>
                            </div>
                        </aside>
                    </div>
                </div>

                
                <div id="pwa-modal" class="modal-backdrop" style="display:none" onclick="closeModal(event)">
                    <div class="modal-card" onclick="event.stopPropagation()">
                        <div class="modal-header">
                            <h2 style="font-size:1.35rem;color:#f8fafc">"🚀 How to Install on Your Browser"</h2>
                            <button class="modal-close" onclick="closeModalDirect()">"✕"</button>
                        </div>
                        <p style="color:#94a3b8;font-size:0.9rem;margin-bottom:1.25rem">
                            "Click your current browser to see exact location of the install button:"
                        </p>
                        <div class="browser-tabs-nav" style="margin-bottom:1rem">
                            <button type="button" class="tab-btn modal-tab active" onclick="switchModalGuide('brave')">"🦁 Brave"</button>
                            <button type="button" class="tab-btn modal-tab" onclick="switchModalGuide('chrome')">"🌐 Chrome"</button>
                            <button type="button" class="tab-btn modal-tab" onclick="switchModalGuide('edge')">"🌊 Edge"</button>
                            <button type="button" class="tab-btn modal-tab" onclick="switchModalGuide('safari')">"🍏 Safari"</button>
                            <button type="button" class="tab-btn modal-tab" onclick="switchModalGuide('firefox')">"🦊 Firefox"</button>
                        </div>

                        <div id="modal-pane-brave" class="modal-pane active">
                            <div class="guide-step">
                                <strong>"Option 1 (Address Bar):"</strong>
                                <p>"Look at the far right of the address bar (next to the lion Shields icon). Click the small computer with a down arrow (🖥️⬇️) and select 'Install'."</p>
                            </div>
                            <div class="guide-step">
                                <strong>"Option 2 (Menu):"</strong>
                                <p>"Click the (≡) menu at the top-right corner of Brave > select 'Install Rullst LMS...'."</p>
                            </div>
                        </div>

                        <div id="modal-pane-chrome" class="modal-pane" style="display:none">
                            <div class="guide-step">
                                <strong>"Option 1 (Address Bar):"</strong>
                                <p>"Click the install icon (monitor with down arrow) on the right side of the address bar."</p>
                            </div>
                            <div class="guide-step">
                                <strong>"Option 2 (Menu):"</strong>
                                <p>"Click the (⋮) menu > 'Save and share' > 'Install Rullst LMS...'."</p>
                            </div>
                        </div>

                        <div id="modal-pane-edge" class="modal-pane" style="display:none">
                            <div class="guide-step">
                                <strong>"Option 1 (Address Bar):"</strong>
                                <p>"Click the 'App available' icon (three squares with plus) on the right of the address bar."</p>
                            </div>
                            <div class="guide-step">
                                <strong>"Option 2 (Menu):"</strong>
                                <p>"Click the (...) menu > 'Apps' > 'Install this site as an app'."</p>
                            </div>
                        </div>

                        <div id="modal-pane-safari" class="modal-pane" style="display:none">
                            <div class="guide-step">
                                <strong>"iPhone & iPad (iOS):"</strong>
                                <p>"Tap the Share button [↑] at the bottom of the screen > scroll down and select 'Add to Home Screen' [+]."</p>
                            </div>
                            <div class="guide-step">
                                <strong>"Mac (macOS Sonoma+):"</strong>
                                <p>"In Safari, click 'File' in the top macOS menu bar > select 'Add to Dock...'."</p>
                            </div>
                        </div>

                        <div id="modal-pane-firefox" class="modal-pane" style="display:none">
                            <div class="guide-step">
                                <strong>"Android:"</strong>
                                <p>"Tap the three dots (⋮) > tap 'Install' or 'Add to Home screen'."</p>
                            </div>
                            <div class="guide-step">
                                <strong>"Desktop (Windows/Mac):"</strong>
                                <p>"Firefox Desktop lacks native PWA window support. Please use Brave, Chrome, Edge, or download the native .exe installer!"</p>
                            </div>
                        </div>
                    </div>
                </div>

                <script>
                    r#"
                    if ('serviceWorker' in navigator) {
                        navigator.serviceWorker.register('/sw.js').catch(console.error);
                    }

                    let deferredPrompt;
                    const installBtn = document.getElementById('pwa-install-btn');

                    window.addEventListener('beforeinstallprompt', (e) => {
                        e.preventDefault();
                        deferredPrompt = e;
                        installBtn.style.display = 'inline-flex';
                    });

                    installBtn.addEventListener('click', async () => {
                        if (deferredPrompt) {
                            deferredPrompt.prompt();
                            const { outcome } = await deferredPrompt.userChoice;
                            if (outcome === 'accepted') {
                                installBtn.textContent = '✅ App Installed!';
                                installBtn.disabled = true;
                            }
                            deferredPrompt = null;
                        } else {
                            openModal();
                        }
                    });

                    function openModal() {
                        document.getElementById('pwa-modal').style.display = 'flex';
                    }

                    function closeModal(e) {
                        if (e.target.id === 'pwa-modal') {
                            document.getElementById('pwa-modal').style.display = 'none';
                        }
                    }

                    function closeModalDirect() {
                        document.getElementById('pwa-modal').style.display = 'none';
                    }

                    function switchGuide(browser) {
                        document.querySelectorAll('.browser-tabs-nav .tab-btn').forEach(btn => btn.classList.remove('active'));
                        document.querySelectorAll('.guide-pane').forEach(pane => pane.classList.remove('active'));
                        event.target.classList.add('active');
                        const targetPane = document.getElementById('guide-' + browser);
                        if (targetPane) targetPane.classList.add('active');
                    }

                    function switchModalGuide(browser) {
                        document.querySelectorAll('.modal-tab').forEach(btn => btn.classList.remove('active'));
                        document.querySelectorAll('.modal-pane').forEach(pane => pane.style.display = 'none');
                        event.target.classList.add('active');
                        const targetPane = document.getElementById('modal-pane-' + browser);
                        if (targetPane) targetPane.style.display = 'block';
                    }
                    "#
                </script>
            </body>
        </html>
    })
}
