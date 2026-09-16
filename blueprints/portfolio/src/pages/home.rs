// Frontend Adapter: Zero-Bundle HTMX
use rullst::html;
use crate::models::profile::Profile;
use crate::models::project::Project;
use crate::models::experience::Experience;
use crate::models::skill::Skill;

fn cv_styles() -> String {
    r#"
    * { box-sizing: border-box; margin: 0; padding: 0; font-family: 'Outfit', sans-serif; }
    
    :root {
        --bg-color: #050505;
        --sidebar-bg: rgba(15, 15, 20, 0.7);
        --accent: #00ffcc;
        --accent-glow: rgba(0, 255, 204, 0.2);
        --text-main: #f3f4f6;
        --text-muted: #9ca3af;
        --border-color: rgba(255, 255, 255, 0.08);
        --glass-bg: rgba(25, 25, 30, 0.5);
    }

    html, body {
        overflow-x: hidden;
        width: 100%;
    }

    body {
        background: var(--bg-color);
        color: var(--text-main);
        line-height: 1.6;
        min-height: 100vh;
    }
    
    .bg-grid {
        position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: -3;
        background-image: 
            linear-gradient(to right, rgba(255,255,255,0.03) 1px, transparent 1px),
            linear-gradient(to bottom, rgba(255,255,255,0.03) 1px, transparent 1px);
        background-size: 40px 40px;
        mask-image: radial-gradient(circle at center, black, transparent 80%);
        -webkit-mask-image: radial-gradient(circle at center, black, transparent 80%);
        animation: gridMove 20s linear infinite;
        pointer-events: none;
    }
    
    @keyframes gridMove {
        0% { transform: translateY(0); }
        100% { transform: translateY(40px); }
    }

    .scanlines {
        position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: -1;
        background: linear-gradient(to bottom, rgba(255,255,255,0), rgba(255,255,255,0) 50%, rgba(0,0,0,0.15) 50%, rgba(0,0,0,0.15));
        background-size: 100% 4px; pointer-events: none;
    }

    .glow-blob { position: fixed; border-radius: 50%; filter: blur(120px); z-index: -2; animation: pulseGlow 8s infinite alternate; pointer-events: none; }
    .glow-1 { top: -10%; left: -10%; width: 50vw; height: 50vh; background: rgba(0, 255, 204, 0.08); }
    .glow-2 { bottom: -10%; right: -10%; width: 50vw; height: 50vh; background: rgba(138, 43, 226, 0.08); }
    
    @keyframes pulseGlow {
        0% { transform: scale(1); opacity: 0.8; }
        100% { transform: scale(1.1); opacity: 1; }
    }

    .layout {
        display: flex;
        min-height: 100vh;
        max-width: 1400px;
        margin: 0 auto;
        padding: 2.5rem;
        gap: 3rem;
        width: 100%;
    }
    
    .sidebar {
        width: 360px;
        flex-shrink: 0;
        position: sticky;
        top: 2rem;
        height: calc(100vh - 4rem);
        background: var(--sidebar-bg);
        border: 1px solid var(--border-color);
        border-radius: 24px;
        padding: 2.25rem;
        display: flex;
        flex-direction: column;
        gap: 1.75rem;
        backdrop-filter: blur(24px);
        -webkit-backdrop-filter: blur(24px);
        box-shadow: 0 25px 50px -12px rgba(0,0,0,0.6);
        overflow-y: auto;
    }
    
    .profile-img { width: 130px; height: auto; max-height: 120px; border-radius: 14px; margin-bottom: 0.75rem; object-fit: contain; }
    h1 { font-size: clamp(1.8rem, 4vw, 2.3rem); font-weight: 800; line-height: 1.15; margin-bottom: 0.4rem; background: linear-gradient(135deg, #fff 0%, #a1a1aa 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent; }
    h2.role { color: var(--accent); font-size: 1rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 0.75rem; }
    .summary { color: var(--text-muted); font-size: 0.92rem; line-height: 1.5; }

    .contact-info { display: flex; flex-direction: column; gap: 0.75rem; margin-top: 0.5rem; }
    .contact-item { display: flex; align-items: center; gap: 0.65rem; font-size: 0.88rem; color: var(--text-muted); word-break: break-all; }

    .skill-cat { font-size: 0.82rem; font-weight: 700; color: #fff; text-transform: uppercase; margin-bottom: 0.5rem; letter-spacing: 0.06em; }
    .tags { display: flex; flex-wrap: wrap; gap: 0.45rem; margin-bottom: 1.25rem; }
    .tag { background: rgba(255, 255, 255, 0.05); color: #e4e4e7; padding: 0.35rem 0.7rem; border-radius: 6px; font-size: 0.78rem; font-weight: 500; border: 1px solid var(--border-color); }

    .content { flex-grow: 1; display: flex; flex-direction: column; gap: 3.5rem; padding-bottom: 4rem; min-width: 0; }
    .section-title { font-size: clamp(1.6rem, 3vw, 2rem); font-weight: 800; display: flex; align-items: center; gap: 0.75rem; margin-bottom: 1.75rem; }

    .timeline { position: relative; padding-left: 2rem; }
    .timeline::before { content: ''; position: absolute; left: 0; top: 0; bottom: 0; width: 2px; background: var(--border-color); }
    
    .timeline-item { position: relative; margin-bottom: 2.75rem; }
    .timeline-item::before {
        content: ''; position: absolute; left: -2.35rem; top: 0.35rem; width: 12px; height: 12px;
        border-radius: 50%; background: var(--bg-color); border: 2px solid var(--accent);
    }
    
    .exp-period { display: inline-block; font-size: 0.82rem; color: var(--accent); background: var(--accent-glow); padding: 0.2rem 0.6rem; border-radius: 4px; font-weight: 600; margin-bottom: 0.4rem; }
    .exp-role { font-size: 1.25rem; font-weight: 700; margin-bottom: 0.2rem; }
    .exp-company { font-size: 0.95rem; color: #bbb; font-weight: 500; margin-bottom: 0.75rem; }
    .exp-desc { color: var(--text-muted); font-size: 0.95rem; line-height: 1.6; }

    .projects-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1.5rem; }
    .project-card { background: var(--glass-bg); border: 1px solid var(--border-color); border-radius: 16px; padding: 1.5rem; display: flex; flex-direction: column; transition: transform 0.2s ease, border-color 0.2s ease; }
    .project-card:hover { transform: translateY(-3px); border-color: rgba(0, 255, 204, 0.3); }
    .project-title { font-size: 1.2rem; font-weight: 700; margin-bottom: 0.5rem; }
    .project-desc { font-size: 0.92rem; color: var(--text-muted); margin-bottom: 1.25rem; flex-grow: 1; }
    .project-link { display: inline-flex; align-items: center; gap: 0.5rem; color: var(--accent); text-decoration: none; font-size: 0.88rem; font-weight: 600; }
    .project-link:hover { text-decoration: underline; }

    .cms-btn { display: inline-block; margin-top: 1rem; background: #10b981; color: #000; padding: 0.6rem 1.2rem; border-radius: 8px; font-weight: 700; text-decoration: none; font-size: 0.9rem; }
    .cms-btn:hover { background: #34d399; }

    .engine-badge { display: inline-block; background: rgba(0, 255, 204, 0.08); border: 1px solid rgba(0, 255, 204, 0.25); color: #00ffcc; font-size: 0.72rem; font-weight: 600; padding: 0.2rem 0.55rem; border-radius: 20px; margin-top: 0.4rem; margin-bottom: 0.6rem; word-break: break-word; }

    /* == Tablet & Mobile Responsive Media Queries == */
    @media (max-width: 1024px) {
        .layout { padding: 1.5rem; gap: 2rem; }
        .sidebar { width: 310px; padding: 1.75rem; }
    }

    @media (max-width: 900px) {
        .layout {
            flex-direction: column;
            padding: 1.25rem 1rem;
            gap: 2.25rem;
        }
        .sidebar {
            width: 100%;
            position: static;
            height: auto;
            padding: 1.5rem 1.25rem;
            border-radius: 20px;
            box-shadow: 0 10px 30px -5px rgba(0,0,0,0.5);
        }
        .content {
            gap: 2.5rem;
            padding-bottom: 2.5rem;
            width: 100%;
        }
        .section-title {
            font-size: 1.5rem;
            margin-bottom: 1.25rem;
        }
        .projects-grid {
            grid-template-columns: 1fr;
            gap: 1.25rem;
        }
    }

    @media (max-width: 640px) {
        .layout {
            padding: 1rem 0.75rem;
            gap: 1.75rem;
        }
        .sidebar {
            padding: 1.25rem 1rem;
            border-radius: 16px;
        }
        .profile-img {
            width: 100px;
        }
        .timeline {
            padding-left: 1.25rem;
        }
        .timeline-item {
            margin-bottom: 2rem;
        }
        .timeline-item::before {
            left: -1.62rem;
            width: 10px;
            height: 10px;
            top: 0.35rem;
        }
        .exp-role {
            font-size: 1.1rem;
        }
        .exp-desc {
            font-size: 0.88rem;
        }
        .project-card {
            padding: 1.25rem 1rem;
        }
    }

    /* == AI Career Copilot Floating Drawer & Launcher == */
    /* == Floating Crab Mascot AI Launcher == */
    .ai-crab-launcher {
        position: fixed;
        bottom: 24px;
        right: 24px;
        z-index: 999;
        display: flex;
        align-items: center;
        gap: 12px;
        cursor: pointer;
        user-select: none;
        transition: transform 0.25s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    }
    .ai-crab-launcher:hover {
        transform: translateY(-4px) scale(1.05);
    }
    .ai-crab-speech-bubble {
        background: rgba(15, 23, 42, 0.95);
        border: 1px solid rgba(0, 255, 204, 0.5);
        color: #fff;
        padding: 8px 14px;
        border-radius: 14px;
        font-size: 0.84rem;
        font-weight: 700;
        letter-spacing: 0.02em;
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5), 0 0 16px rgba(0, 255, 204, 0.25);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        display: flex;
        align-items: center;
        gap: 6px;
        position: relative;
        animation: bubbleFloat 3s infinite ease-in-out;
        white-space: nowrap;
    }
    .ai-crab-speech-bubble::after {
        content: '';
        position: absolute;
        right: -6px;
        top: 50%;
        transform: translateY(-50%) rotate(45deg);
        width: 10px;
        height: 10px;
        background: rgba(15, 23, 42, 0.95);
        border-top: 1px solid rgba(0, 255, 204, 0.5);
        border-right: 1px solid rgba(0, 255, 204, 0.5);
    }
    @keyframes bubbleFloat {
        0%, 100% { transform: translateY(0); }
        50% { transform: translateY(-4px); }
    }
    .ai-bubble-sparkle {
        font-size: 0.95rem;
    }
    .ai-bubble-text {
        background: linear-gradient(135deg, #00ffcc, #38bdf8);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
    }
    .ai-crab-avatar-wrapper {
        position: relative;
        width: 60px;
        height: 60px;
        flex-shrink: 0;
        filter: drop-shadow(0 8px 20px rgba(0, 255, 204, 0.4));
        animation: crabWiggle 4s infinite ease-in-out;
    }
    @keyframes crabWiggle {
        0%, 100% { transform: rotate(0deg); }
        25% { transform: rotate(-3deg) translateY(-2px); }
        75% { transform: rotate(3deg) translateY(-1px); }
    }
    .ai-crab-img {
        width: 100%;
        height: 100%;
        object-fit: contain;
        display: block;
    }
    .ai-crab-online-dot {
        position: absolute;
        bottom: 2px;
        right: 2px;
        width: 12px;
        height: 12px;
        border-radius: 50%;
        background: #10b981;
        border: 2px solid #0b0f19;
        box-shadow: 0 0 8px #10b981;
    }

    .ai-drawer {
        position: fixed;
        bottom: 84px;
        right: 24px;
        width: 420px;
        max-width: calc(100vw - 32px);
        height: 590px;
        max-height: calc(100vh - 110px);
        background: rgba(15, 15, 22, 0.94);
        border: 1px solid rgba(0, 255, 204, 0.25);
        border-radius: 24px;
        box-shadow: 0 25px 60px -10px rgba(0, 0, 0, 0.85), 0 0 40px rgba(0, 255, 204, 0.12);
        backdrop-filter: blur(30px);
        -webkit-backdrop-filter: blur(30px);
        display: none;
        flex-direction: column;
        z-index: 1000;
        overflow: hidden;
        animation: aiDrawerSlideUp 0.25s cubic-bezier(0.16, 1, 0.3, 1);
    }
    .ai-drawer.open {
        display: flex;
    }

    @keyframes aiDrawerSlideUp {
        from { opacity: 0; transform: translateY(20px) scale(0.96); }
        to { opacity: 1; transform: translateY(0) scale(1); }
    }

    .ai-drawer-header {
        padding: 16px 20px;
        background: rgba(20, 20, 30, 0.85);
        border-bottom: 1px solid var(--border-color);
        display: flex;
        align-items: center;
        justify-content: space-between;
    }
    .ai-drawer-title {
        font-size: 1rem;
        font-weight: 700;
        color: #fff;
        display: flex;
        align-items: center;
        gap: 8px;
    }
    .ai-drawer-subtitle {
        font-size: 0.72rem;
        color: var(--accent);
        display: block;
        margin-top: 2px;
    }
    .ai-close-btn {
        background: rgba(255,255,255,0.06);
        border: 1px solid var(--border-color);
        color: #ddd;
        width: 28px;
        height: 28px;
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        font-size: 18px;
        line-height: 1;
        transition: background 0.15s;
    }
    .ai-close-btn:hover { background: rgba(255,255,255,0.15); color: #fff; }

    .ai-chat-messages {
        flex: 1;
        overflow-y: auto;
        padding: 18px;
        display: flex;
        flex-direction: column;
        gap: 14px;
        scroll-behavior: smooth;
    }

    .chat-bubble {
        display: flex;
        flex-direction: column;
        max-width: 90%;
        animation: bubbleFadeIn 0.2s ease;
    }
    @keyframes bubbleFadeIn {
        from { opacity: 0; transform: translateY(6px); }
        to { opacity: 1; transform: translateY(0); }
    }
    .chat-bubble-user {
        align-self: flex-end;
    }
    .chat-bubble-assistant {
        align-self: flex-start;
    }
    .chat-bubble-sender {
        font-size: 0.7rem;
        font-weight: 600;
        color: var(--text-muted);
        margin-bottom: 4px;
        padding: 0 4px;
    }
    .chat-bubble-user .chat-bubble-sender {
        text-align: right;
    }
    .chat-bubble-body {
        padding: 12px 16px;
        border-radius: 16px;
        font-size: 0.88rem;
        line-height: 1.55;
    }
    .chat-bubble-user .chat-bubble-body {
        background: linear-gradient(135deg, rgba(0, 255, 204, 0.25), rgba(0, 255, 204, 0.15));
        border: 1px solid rgba(0, 255, 204, 0.4);
        color: #fff;
        border-bottom-right-radius: 4px;
    }
    .chat-bubble-assistant .chat-bubble-body {
        background: rgba(30, 30, 42, 0.9);
        border: 1px solid var(--border-color);
        color: #e4e4e7;
        border-bottom-left-radius: 4px;
    }
    .chat-bubble-assistant.error .chat-bubble-body {
        background: rgba(239, 68, 68, 0.15);
        border-color: rgba(239, 68, 68, 0.4);
        color: #fca5a5;
    }
    .ai-badge-footer {
        margin-top: 8px;
        padding-top: 6px;
        border-top: 1px solid rgba(255,255,255,0.06);
        font-size: 0.68rem;
        color: #a1a1aa;
    }

    .ai-prompt-suggestions {
        padding: 8px 14px;
        border-top: 1px solid rgba(255,255,255,0.06);
        background: rgba(10, 10, 15, 0.7);
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
        max-height: 84px;
        overflow-y: auto;
    }
    .ai-prompt-suggestions::-webkit-scrollbar {
        width: 3px;
    }
    .ai-prompt-suggestions::-webkit-scrollbar-thumb {
        background: rgba(0, 255, 204, 0.3);
        border-radius: 3px;
    }
    .ai-pill-btn {
        background: rgba(255,255,255,0.05);
        border: 1px solid var(--border-color);
        color: #d4d4d8;
        font-size: 0.73rem;
        padding: 5px 10px;
        border-radius: 20px;
        cursor: pointer;
        transition: all 0.15s ease;
        line-height: 1.2;
        display: inline-flex;
        align-items: center;
        gap: 4px;
    }
    .ai-pill-btn:hover {
        background: rgba(0, 255, 204, 0.15);
        border-color: rgba(0, 255, 204, 0.5);
        color: #00ffcc;
        transform: translateY(-1px);
    }

    .ai-form {
        padding: 14px 16px;
        background: rgba(20, 20, 30, 0.98);
        border-top: 1px solid var(--border-color);
        display: flex;
        gap: 8px;
        align-items: center;
    }
    .ai-input {
        flex: 1;
        background: rgba(10, 10, 15, 0.8);
        border: 1px solid var(--border-color);
        border-radius: 12px;
        padding: 10px 14px;
        color: #fff;
        font-size: 0.86rem;
        outline: none;
        transition: border-color 0.15s;
    }
    .ai-input:focus {
        border-color: var(--accent);
    }
    .ai-submit-btn {
        background: var(--accent);
        color: #050505;
        font-weight: 700;
        border: none;
        border-radius: 12px;
        padding: 10px 16px;
        cursor: pointer;
        font-size: 0.86rem;
        transition: opacity 0.15s;
    }
    .ai-submit-btn:hover { opacity: 0.9; }

    .ai-typing-indicator {
        display: none;
        align-items: center;
        gap: 4px;
        padding: 8px 18px;
        background: rgba(15, 15, 22, 0.8);
    }
    .ai-typing-indicator.htmx-request {
        display: flex;
    }
    .ai-typing-dot {
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background: var(--accent);
        animation: aiPulsingDot 1.2s infinite ease-in-out;
    }
    .ai-typing-dot:nth-child(2) { animation-delay: 0.2s; }
    .ai-typing-dot:nth-child(3) { animation-delay: 0.4s; }
    @keyframes aiPulsingDot {
        0%, 80%, 100% { transform: scale(0.6); opacity: 0.4; }
        40% { transform: scale(1.1); opacity: 1; }
    }

    @media (max-width: 640px) {
        .ai-crab-launcher {
            bottom: 16px;
            right: 16px;
            gap: 8px;
        }
        .ai-crab-avatar-wrapper {
            width: 48px;
            height: 48px;
        }
        .ai-crab-speech-bubble {
            font-size: 0.76rem;
            padding: 6px 10px;
        }
        .ai-drawer {
            bottom: 0 !important;
            right: 0 !important;
            left: 0 !important;
            width: 100% !important;
            max-width: 100% !important;
            height: 80vh !important;
            max-height: 85vh !important;
            border-radius: 20px 20px 0 0 !important;
            border-bottom: none !important;
            box-shadow: 0 -10px 40px rgba(0, 0, 0, 0.8) !important;
        }
    }
    "#.to_string()
}

fn render_sidebar(profile: &Profile, skills: &[Skill]) -> String {
    html! {
        <aside class="sidebar">
            <div style="text-align: center;">
                <img src={&profile.avatar_url} alt={&profile.name} class="profile-img" />
                <h1>{&profile.name}</h1>
                <h2 class="role">{&profile.title}</h2>
                <div class="engine-badge">"Rullst HTMX + Tailwind SSR profile selected"</div>
                <p class="summary">{&profile.subtitle}</p>
                
                <div style="margin-top: 1.5rem; background: rgba(0, 255, 204, 0.04); border: 1px solid rgba(0, 255, 204, 0.3); border-radius: 14px; padding: 1.25rem; text-align: left; box-shadow: 0 8px 32px rgba(0,0,0,0.37);">
                    <div style="display: flex; align-items: center; gap: 0.5rem; color: #00ffcc; font-weight: 700; font-size: 0.85rem; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 0.5rem;">
                        <span>"🛡️"</span> "Live Sandbox Access"
                    </div>
                    <p style="font-size: 0.8rem; color: #9ca3af; margin-bottom: 0.75rem; line-height: 1.4;">
                        "Public showcase mode enabled. Explore the Nexus Admin CMS or monitor real-time Studio telemetry:"
                    </p>
                    <div style="background: rgba(0, 0, 0, 0.5); border-radius: 8px; padding: 0.6rem 0.8rem; font-family: monospace; font-size: 0.82rem; color: #f3f4f6; margin-bottom: 1rem; border: 1px solid rgba(255, 255, 255, 0.1);">
                        <div style="margin-bottom: 0.25rem;"><span style="color: #9ca3af;">"User: "</span><strong style="color: #00ffcc; user-select: all;">"admin"</strong></div>
                        <div><span style="color: #9ca3af;">"Pass: "</span><strong style="color: #00ffcc; user-select: all;">"SovereignPortfolio2026!"</strong></div>
                    </div>
                    <div style="display: flex; gap: 0.5rem; flex-direction: column;">
                        <a href="/nexus" target="_blank" style="display: block; text-align: center; background: #10b981; color: #000; padding: 0.6rem 1rem; border-radius: 8px; font-weight: 700; text-decoration: none; font-size: 0.85rem;">"⚙️ Manage via Nexus CMS"</a>
                        <a href="/studio" target="_blank" style="display: block; text-align: center; background: rgba(0, 255, 204, 0.12); border: 1px solid rgba(0, 255, 204, 0.4); color: #00ffcc; padding: 0.6rem 1rem; border-radius: 8px; font-weight: 700; text-decoration: none; font-size: 0.85rem;">"🚀 Open Studio Cockpit"</a>
                    </div>
                    <div style="margin-top: 0.85rem; padding-top: 0.75rem; border-top: 1px solid rgba(255, 255, 255, 0.08); display: flex; align-items: flex-start; gap: 0.4rem;">
                        <span style="font-size: 0.85rem;">"🔄"</span>
                        <p style="font-size: 0.72rem; color: #9ca3af; line-height: 1.35;">
                            <strong style="color: #e5e7eb;">"Ephemeral Scale-to-Zero Sandbox:"</strong> " Any modifications in Nexus or Studio are non-destructive and temporary. When the container sleeps and wakes, SQLite automatically resets to pristine defaults."
                        </p>
                    </div>
                </div>
            </div>
            
            <div class="contact-info">
                <div class="contact-item">"📧 "{&profile.email}</div>
                <div class="contact-item">"🌐 "<a href={&profile.website} target="_blank" style="color: var(--accent);">{&profile.website}</a></div>
                <div class="contact-item">"💻 "<a href={&profile.github_url} target="_blank" style="color: var(--text-muted);">{&profile.github_url}</a></div>
                <div class="contact-item">"💼 "<a href={&profile.linkedin_url} target="_blank" style="color: var(--text-muted);">{&profile.linkedin_url}</a></div>
            </div>

            <div>
                <div class="skill-cat">"Technical Skills"</div>
                <div class="tags">
                    { rullst::html::RawHtml::new(skills.iter().map(|s| format!("<span class=\"tag\">{}</span>", s.name)).collect::<Vec<_>>().join("")) }
                </div>
            </div>
        </aside>
    }
}

fn render_content(projects: &[Project], experiences: &[Experience]) -> String {
    html! {
        <main class="content">
            <section>
                <h2 class="section-title">"Experience"</h2>
                <div class="timeline">
                    { rullst::html::RawHtml::new(experiences.iter().map(|e| format!(
                        "<div class=\"timeline-item\">\
                            <div class=\"exp-period\">{}</div>\
                            <h3 class=\"exp-role\">{}</h3>\
                            <div class=\"exp-company\">{}</div>\
                            <p class=\"exp-desc\">{}</p>\
                        </div>", e.period, e.role, e.company, e.description
                    )).collect::<Vec<_>>().join("")) }
                </div>
            </section>

            <section>
                <h2 class="section-title">"Projects Showcase"</h2>
                <div class="projects-grid">
                    { rullst::html::RawHtml::new(projects.iter().map(|p| format!(
                        "<div class=\"project-card\">\
                            <h3 class=\"project-title\">{}</h3>\
                            <p class=\"project-desc\">{}</p>\
                            <div class=\"tags\"><span class=\"tag\">{}</span></div>\
                            <a href=\"{}\" target=\"_blank\" class=\"project-link\">View Project &rarr;</a>\
                        </div>",
                        p.title, p.description, p.tags, p.url
                    )).collect::<Vec<_>>().join("")) }
                </div>
            </section>
        </main>
    }
}

fn render_ai_widget(csrf_token: &str) -> String {
    r##"
    <div id="ai-crab-launcher" class="ai-crab-launcher" onclick="toggleAiDrawer()" role="button" tabindex="0" aria-label="Ask me anything!">
        <div class="ai-crab-speech-bubble">
            <span class="ai-bubble-sparkle">✨</span>
            <span class="ai-bubble-text">Ask me anything!</span>
        </div>
        <div class="ai-crab-avatar-wrapper">
            <img src="/static/crab.png" alt="Rullst Crab Mascot" class="ai-crab-img" />
            <span class="ai-crab-online-dot"></span>
        </div>
    </div>

    <div id="ai-drawer" class="ai-drawer" style="display: none;" role="dialog" aria-label="AI Career Copilot">
        <div class="ai-drawer-header">
            <div class="ai-header-left" style="display: flex; align-items: center; gap: 10px;">
                <div class="ai-avatar-badge" style="width: 34px; height: 34px; border-radius: 10px; background: rgba(0, 255, 204, 0.15); border: 1px solid rgba(0, 255, 204, 0.3); display: flex; align-items: center; justify-content: center; font-size: 1.1rem;">⚡</div>
                <div>
                    <div class="ai-header-title" style="font-weight: 800; font-size: 0.95rem; color: #fff;">Career Copilot</div>
                    <div class="ai-header-sub" style="font-size: 0.72rem; color: #00ffcc; display: flex; align-items: center; gap: 5px;">
                        <span class="ai-status-indicator" style="width: 6px; height: 6px; border-radius: 50%; background: #00ffcc; display: inline-block;"></span>
                        <span>AI Architecture & Career Assistant</span>
                    </div>
                </div>
            </div>
            <button class="ai-close-btn" onclick="toggleAiDrawer()" aria-label="Close">×</button>
        </div>

        <div id="ai-chat-messages" class="ai-chat-messages">
            <div class="chat-bubble chat-bubble-assistant">
                <div class="chat-bubble-sender">Career Copilot</div>
                <div class="chat-bubble-body">
                    Olá! Sou o <strong>Career Copilot</strong> deste portfólio. Pergunte-me qualquer coisa sobre habilidades em Rust, arquitetura de sistemas, projetos ou contratação! (You can also ask in English!)
                    <div class="ai-badge-footer">⚡ Context-Aware RAG • Protected by Rullst Guardrails</div>
                </div>
            </div>
        </div>

        <div class="ai-prompt-suggestions">
            <button class="ai-pill-btn" type="button" onclick="setAiQuestion('Quais são as principais habilidades em Rust e backend de Vene?')">🦀 Habilidades Rust</button>
            <button class="ai-pill-btn" type="button" onclick="setAiQuestion('Explique a arquitetura e diferenciais do projeto LMS.')">🏛️ Arquitetura LMS</button>
            <button class="ai-pill-btn" type="button" onclick="setAiQuestion('Como o Rullst protege contra ataques de Prompt Injection?')">🛡️ Segurança IA</button>
            <button class="ai-pill-btn" type="button" onclick="setAiQuestion('Por que contratar o Vene para sistemas de alta concorrência?')">💼 Por que Contratar?</button>
            <button class="ai-pill-btn" type="button" onclick="setAiQuestion('Como posso entrar em contato com o desenvolvedor?')">📧 Contato</button>
        </div>

        <div id="ai-typing" class="ai-typing-indicator">
            <span class="ai-typing-dot"></span>
            <span class="ai-typing-dot"></span>
            <span class="ai-typing-dot"></span>
            <span style="font-size: 0.72rem; color: #a1a1aa; margin-left: 6px;">Copilot pensando...</span>
        </div>

        <form id="ai-chat-form" class="ai-form"
              hx-post="/api/chat"
              hx-target="#ai-chat-messages"
              hx-swap="beforeend"
              hx-indicator="#ai-typing"
              hx-on::before-request="appendUserMessage()"
              hx-on::after-request="finalizeAiRequest()">
            <input type="hidden" name="_token" value="__CSRF_TOKEN__" id="ai-csrf-token" />
            <input id="ai-message-input" type="text" name="message" class="ai-input" placeholder="Pergunte sobre projetos, habilidades, Rust..." autocomplete="off" required maxlength="600" />
            <button type="submit" class="ai-submit-btn">Enviar</button>
        </form>
    </div>

    <script>
        document.body.addEventListener('htmx:configRequest', function(evt) {
            var tokenInput = document.getElementById('ai-csrf-token');
            var token = tokenInput ? tokenInput.value : '';
            if (!token) {
                var match = document.cookie.match(/rullst_csrf=([^;]+)/);
                if (match) token = decodeURIComponent(match[1].trim());
            }
            if (token) {
                evt.detail.parameters['_token'] = token;
                evt.detail.headers['X-CSRF-Token'] = token;
            }
        });

        document.body.addEventListener('htmx:responseError', function(evt) {
            var msgs = document.getElementById('ai-chat-messages');
            if (msgs) {
                var errDiv = document.createElement('div');
                errDiv.className = 'chat-bubble chat-bubble-assistant error';
                errDiv.innerHTML = '<div class="chat-bubble-sender">Career Copilot</div><div class="chat-bubble-body">⚠️ Could not reach the assistant (HTTP ' + (evt.detail.xhr ? evt.detail.xhr.status : 'error') + '). Please try again.</div>';
                msgs.appendChild(errDiv);
                scrollAiToBottom();
            }
        });

        function toggleAiDrawer() {
            var drawer = document.getElementById('ai-drawer');
            var launcher = document.getElementById('ai-crab-launcher');
            if (!drawer) return;
            var isOpen = drawer.style.display !== 'none';
            if (isOpen) {
                drawer.style.display = 'none';
                if (launcher) launcher.style.display = 'flex';
            } else {
                drawer.style.display = 'flex';
                if (launcher) launcher.style.display = 'none';
                var input = document.getElementById('ai-message-input');
                if (input) setTimeout(function() { input.focus(); }, 150);
                scrollAiToBottom();
            }
        }

        function scrollAiToBottom() {
            var msgs = document.getElementById('ai-chat-messages');
            if (msgs) {
                setTimeout(function() { msgs.scrollTop = msgs.scrollHeight; }, 50);
            }
        }

        function setAiQuestion(text) {
            var input = document.getElementById('ai-message-input');
            var form = document.getElementById('ai-chat-form');
            if (input && form) {
                input.value = text;
                if (form.requestSubmit) {
                    form.requestSubmit();
                } else {
                    form.submit();
                }
            }
        }

        function escapeHtml(str) {
            return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#039;');
        }

        function appendUserMessage() {
            var input = document.getElementById('ai-message-input');
            if (!input || !input.value.trim()) return;
            var msg = input.value.trim();
            var msgs = document.getElementById('ai-chat-messages');
            if (msgs) {
                var bubble = document.createElement('div');
                bubble.className = 'chat-bubble chat-bubble-user';
                bubble.innerHTML = '<div class="chat-bubble-sender">You</div><div class="chat-bubble-body">' + escapeHtml(msg) + '</div>';
                msgs.appendChild(bubble);
                scrollAiToBottom();
            }
        }

        function finalizeAiRequest() {
            var input = document.getElementById('ai-message-input');
            if (input) {
                input.value = '';
                input.focus();
            }
            scrollAiToBottom();
        }

        document.addEventListener('keydown', function(e) {
            if (e.key === 'Escape') {
                var drawer = document.getElementById('ai-drawer');
                if (drawer && drawer.style.display !== 'none') {
                    toggleAiDrawer();
                }
            }
        });

        document.addEventListener('htmx:afterSwap', function(e) {
            if (e.detail.target && e.detail.target.id === 'ai-chat-messages') {
                scrollAiToBottom();
            }
        });
    </script>
    "##.replace("__CSRF_TOKEN__", &rullst::html::escape_str(csrf_token))
}

pub fn render(profile: &Profile, projects: &[Project], experiences: &[Experience], skills: &[Skill], csrf_token: &str) -> String {
    html! {
        <html lang="en">
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>"Rullst Developer — AI & Rust Portfolio"</title>
                <link rel="icon" type="image/png" href="https://raw.githubusercontent.com/venelouis/Rullst/main/Rullst.png" />
                <link href="https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;500;600;700;800&display=swap" rel="stylesheet" />
                <script src="/static/htmx.js"></script>
                <style>{ rullst::html::RawHtml(cv_styles()) }</style>
            </head>
            <body>
                <div class="bg-grid"></div>
                <div class="scanlines"></div>
                <div class="glow-blob glow-1"></div>
                <div class="glow-blob glow-2"></div>
                
                <div class="layout">
                    { rullst::html::RawHtml(render_sidebar(profile, skills)) }
                    { rullst::html::RawHtml(render_content(projects, experiences)) }
                </div>

                { rullst::html::RawHtml(render_ai_widget(csrf_token)) }
            </body>
        </html>
    }
}


