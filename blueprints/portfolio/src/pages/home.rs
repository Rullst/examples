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

pub fn render(profile: &Profile, projects: &[Project], experiences: &[Experience], skills: &[Skill]) -> String {
    html! {
        <html lang="en">
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>"Rullst Developer — AI & Rust Portfolio"</title>
                <link rel="icon" type="image/png" href="https://raw.githubusercontent.com/venelouis/Rullst/main/Rullst.png" />
                <link href="https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;500;600;700;800&display=swap" rel="stylesheet" />
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
            </body>
        </html>
    }
}
