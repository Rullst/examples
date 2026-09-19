// Frontend Engine: Zero-Bundle HTMX
use rullst::html;
use crate::models::category::Category;
use crate::models::course::Course;
use crate::models::lesson::Lesson;

fn render_community_callout() -> String {
    html! {
        <section class="community-callout" aria-labelledby="community-heading">
            <div class="community-callout-mark" aria-hidden="true">"R"</div>
            <div class="community-callout-copy">
                <p class="community-callout-eyebrow">"Learn, build, contribute"</p>
                <h2 id="community-heading">"Build alongside the Rullst community"</h2>
                <p>"Meet Rullst builders, share what you are learning, and help shape the framework."</p>
            </div>
            <a href="https://discord.gg/2ntKFtsSjw" target="_blank" rel="noopener noreferrer">"Join us on Discord"</a>
        </section>
    }
}

pub fn index_page(
    categories: Vec<Category>,
    courses: Vec<Course>,
    query: &str,
    selected_category: Option<i32>,
    csp_nonce: &str,
    csrf_token: &str,
) -> String {
    let category_input = selected_category.map_or_else(String::new, |category| {
        html! {
            <input type="hidden" name="category" value={category.to_string()} />
        }
    });
    let category_links = categories
        .iter()
        .map(|category| {
            let class = if selected_category == Some(category.id) {
                "chip current"
            } else {
                "chip"
            };
            html! {
                <a class={class} href={format!("/?category={}", category.id)}>{&category.name}</a>
            }
        })
        .collect::<Vec<_>>()
        .join("");
    let course_cards = if courses.is_empty() {
        html! {
            <div class="empty" role="status">
                <h2>"No courses found"</h2>
                <p>"Try another title or clear the active category."</p>
                <a class="button secondary" href="/">"Clear filters"</a>
            </div>
        }
    } else {
        courses
            .iter()
            .map(|course| html! {
                <article class="card">
                    <div class="course-mark" aria-hidden="true">"Course"</div>
                    <div class="card-body">
                        <h2>{&course.title}</h2>
                        <p>{&course.description}</p>
                        <a class="button" href={format!("/courses/{}", course.id)}>"View course"</a>
                    </div>
                </article>
            })
            .collect::<Vec<_>>()
            .join("")
    };
    let summary = match (query.is_empty(), selected_category) {
        (true, None) => format!("{} courses available", courses.len()),
        (false, None) => format!("{} results for “{}”", courses.len(), query),
        (true, Some(_)) => format!("{} courses in the selected category", courses.len()),
        (false, Some(_)) => format!("{} filtered results for “{}”", courses.len(), query),
    };

    html! {
        <html lang="en" class="dark">
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
                <title>"Rullst Academy — Course catalog"</title>
                <link rel="manifest" href="/manifest.webmanifest" />
                <meta name="theme-color" content="#080b11" />
                <link rel="icon" type="image/x-icon" href="/favicon.ico" />
                <script src="/static/htmx.js"></script>
                <style nonce={csp_nonce}>
                    "
                    * { box-sizing: border-box; }
                    body { margin: 0; background: #080b11; color: #f8fafc; min-height: 100vh; padding: 2rem 1rem 4rem; font: 16px system-ui, sans-serif; }
                    a { color: inherit; }
                    a:focus-visible, button:focus-visible, input:focus-visible { outline: 3px solid #fbbf24; outline-offset: 3px; }
                    .container { max-width: 70rem; margin: 0 auto; }
                    .skip { position: absolute; left: -9999px; top: .5rem; padding: .75rem; background: #f8fafc; color: #111827; z-index: 5; }
                    .skip:focus { left: .5rem; }
                    header { display: flex; gap: 2rem; justify-content: space-between; align-items: flex-start; margin-bottom: 2rem; }
                    h1 { margin: 0; font-size: clamp(2rem, 7vw, 3.5rem); color: #6ee7b7; }
                    .sub, .summary { color: #cbd5e1; line-height: 1.6; }
                    .actions { display: flex; gap: .75rem; flex-wrap: wrap; }
                    .button { display: inline-block; border: 1px solid #34d399; border-radius: .65rem; background: #047857; color: #fff; padding: .75rem 1rem; text-align: center; text-decoration: none; font-weight: 700; }
                    .button:hover { background: #065f46; }
                    .secondary { border-color: #64748b; background: #1e293b; }
                    .search { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: .75rem; margin: 1.5rem 0; }
                    .search label { grid-column: 1 / -1; font-weight: 700; }
                    .search input { min-width: 0; border: 1px solid #64748b; border-radius: .65rem; background: #111827; color: #fff; padding: .8rem 1rem; font: inherit; }
                    .search button { border: 1px solid #34d399; border-radius: .65rem; background: #047857; color: #fff; padding: .8rem 1rem; font: inherit; font-weight: 700; cursor: pointer; }
                    .chips { display: flex; gap: .6rem; flex-wrap: wrap; margin: 1rem 0 2rem; }
                    .chip { border: 1px solid #64748b; border-radius: 999px; padding: .55rem .85rem; color: #e2e8f0; text-decoration: none; }
                    .chip.current { border-color: #34d399; background: #064e3b; color: #fff; }
                    .courses-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(100%, 18rem), 1fr)); gap: 1.25rem; }
                    .card { display: flex; flex-direction: column; overflow: hidden; border: 1px solid #334155; border-radius: 1rem; background: #111827; }
                    .course-mark { padding: 2rem; background: #0f3d35; color: #a7f3d0; font-weight: 800; letter-spacing: .12em; text-transform: uppercase; }
                    .card-body { display: flex; flex: 1; flex-direction: column; padding: 1.5rem; }
                    .card h2 { margin: 0 0 .75rem; font-size: 1.35rem; }
                    .card p { flex: 1; margin: 0 0 1.25rem; color: #cbd5e1; line-height: 1.6; }
                    .empty { grid-column: 1 / -1; border: 1px dashed #64748b; border-radius: 1rem; padding: 2rem; text-align: center; }
                    .community-callout { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 1.25rem; margin: 0 0 2rem; padding: 1.35rem 1.5rem; border: 1px solid rgba(88, 101, 242, .5); border-radius: 1.25rem; background: linear-gradient(135deg, rgba(88, 101, 242, .2), rgba(15, 23, 42, .96) 58%, rgba(52, 211, 153, .13)); box-shadow: 0 18px 50px rgba(0, 0, 0, .28); }
                    .community-callout-mark { display: grid; width: 3.25rem; height: 3.25rem; place-items: center; border-radius: 1rem; background: linear-gradient(145deg, #5865f2, #34d399); color: #fff; font-size: 1.4rem; font-weight: 900; box-shadow: 0 10px 28px rgba(88, 101, 242, .35); }
                    .community-callout h2, .community-callout p { margin: 0; }
                    .community-callout h2 { margin: .1rem 0 .25rem; font-size: clamp(1.2rem, 3vw, 1.55rem); }
                    .community-callout-copy > p:not(.community-callout-eyebrow) { color: #cbd5e1; line-height: 1.55; }
                    .community-callout-eyebrow { color: #a5b4fc; font-size: .75rem; font-weight: 800; letter-spacing: .12em; text-transform: uppercase; }
                    .community-callout a { min-width: max-content; padding: .8rem 1rem; border: 1px solid rgba(255, 255, 255, .16); border-radius: .8rem; background: #5865f2; color: #fff; font-weight: 800; text-align: center; text-decoration: none; transition: transform 160ms ease, background 160ms ease; }
                    .community-callout a:hover { background: #4752c4; transform: translateY(-2px); }
                    @media (max-width: 48rem) { header { flex-direction: column; } .search { grid-template-columns: 1fr; } .community-callout { grid-template-columns: auto minmax(0, 1fr); padding: 1.15rem; } .community-callout a { grid-column: 1 / -1; width: 100%; } }
                    @media (prefers-reduced-motion: reduce) { * { scroll-behavior: auto !important; } }
                    "
                </style>
            </head>
            <body>
                <a class="skip" href="#catalog-results">"Skip to course results"</a>
                <div class="container">
                    <header>
                        <div>
                            <h1>"Rullst Academy"</h1>
                            <p class="sub">"A server-rendered starter catalog with bounded search."</p>
                            <p class="summary" style="margin-top: 0.5rem; font-size: 0.875rem; color: #94a3b8;">
                                "💡 Live Blueprint Showcase: Fully functional sandbox. Feel free to sign up, log in, and test course progression."
                            </p>
                        </div>
                        <nav class="actions" aria-label="Developer tools">
                            <a class="button secondary" href="/apps" style="border-color:#10b981;color:#34d399">"📱 Apps & PWA"</a>
                            <a class="button secondary" href="/login">"Login"</a>
                            <a class="button" href="/register">"Sign Up"</a>
                            <a class="button secondary" href="/nexus" target="_blank">"🛡️ Nexus Admin"</a>
                            <a class="button secondary" href="/studio" target="_blank">"🚀 Studio Cockpit"</a>
                        </nav>
                    </header>
                    {rullst::html::RawHtml(render_community_callout())}
                    <main id="catalog-results">
                        <form class="search" method="get" action="/" role="search">
                            <label for="catalog-query">"Search course titles"</label>
                            <input id="catalog-query" type="search" name="q" value={query} maxlength="100" autocomplete="off" />
                            {rullst::html::RawHtml(category_input)}
                            <button type="submit">"Search"</button>
                        </form>
                        <nav class="chips" aria-label="Course categories">
                            <a class={if selected_category.is_none() { "chip current" } else { "chip" }} href="/">"All courses"</a>
                            {rullst::html::RawHtml(category_links)}
                        </nav>
                        <p class="summary" aria-live="polite">{summary}</p>
                        <div class="courses-grid">
                            {rullst::html::RawHtml(course_cards)}
                        </div>
                    </main>
                </div>
                {rullst::html::RawHtml(render_lms_ai_widget(csrf_token))}
                <script nonce={csp_nonce}>
                    "if ('serviceWorker' in navigator) { window.addEventListener('load', () => navigator.serviceWorker.register('/sw.js').catch(console.error)); }"
                </script>
            </body>
        </html>
    }
}

pub fn course_detail_page(
    course: Course,
    lessons: Vec<Lesson>,
    csrf_token: &str,
    csp_nonce: &str,
    is_enrolled: bool,
) -> String {
    let lesson_items = if lessons.is_empty() {
        html! {
            <li class="empty-lesson">"No published lessons are available yet."</li>
        }
    } else {
        lessons
            .iter()
            .map(|lesson| html! {
                <li>
                    <a class="lesson" href={format!("/lessons/{}/play", lesson.id)}>
                        <span>{&lesson.title}</span>
                        <small>{lesson.duration.to_string()}" minutes"</small>
                    </a>
                </li>
            })
            .collect::<Vec<_>>()
            .join("")
    };
    html! {
        <html lang="en" class="dark">
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
                <title>{&course.title}" — Rullst Academy"</title>
                <link rel="icon" type="image/x-icon" href="/favicon.ico" />
                <script src="/static/htmx.js"></script>
                <style nonce={csp_nonce}>
                    "
                    * { box-sizing: border-box; }
                    body { margin: 0; background: #080b11; color: #f8fafc; min-height: 100vh; font: 16px system-ui, sans-serif; }
                    a:focus-visible, button:focus-visible { outline: 3px solid #fbbf24; outline-offset: 3px; }
                    .layout { display: grid; grid-template-columns: minmax(18rem, 24rem) 1fr; min-height: 100vh; }
                    aside { border-right: 1px solid #334155; background: #0f172a; padding: 2rem; }
                    main { display: grid; place-items: center; padding: 2rem; }
                    h1 { line-height: 1.2; }
                    .back { color: #fdba74; font-weight: 700; }
                    .description { color: #cbd5e1; line-height: 1.6; }
                    form { margin: 1.5rem 0; }
                    button { width: 100%; border: 1px solid #34d399; border-radius: .6rem; background: #047857; color: #fff; padding: .8rem; font: inherit; font-weight: 800; cursor: pointer; }
                    ul { padding: 0; list-style: none; }
                    li { border-top: 1px solid #334155; }
                    .lesson { display: flex; gap: .75rem; justify-content: space-between; padding: 1rem 0; color: #f8fafc; text-decoration: none; }
                    .lesson:hover { color: #6ee7b7; }
                    small, .notice, .empty-lesson { color: #cbd5e1; }
                    .notice { max-width: 40rem; border: 1px solid #334155; border-radius: 1rem; background: #111827; padding: 2rem; line-height: 1.7; }
                    @media (max-width: 48rem) { .layout { grid-template-columns: 1fr; } aside { border-right: 0; border-bottom: 1px solid #334155; } }
                    "
                </style>
            </head>
            <body>
                <div class="layout">
                    <aside>
                        <a class="back" href="/">"← Back to catalog"</a>
                        <h1>{&course.title}</h1>
                        <p class="description">{&course.description}</p>
                        {if is_enrolled {
                            let first_id = lessons.first().map(|l| l.id).unwrap_or(1);
                            rullst::html::RawHtml(format!(
                                r#"<a href="/lessons/{}/play" style="display:inline-block;width:100%;text-align:center;padding:.85rem;border-radius:.5rem;background:#10b981;color:#052e16;font-weight:800;text-decoration:none;margin-bottom:1rem;box-shadow:0 4px 12px rgba(16,185,129,0.3);">▶ Resume Learning</a>"#,
                                first_id
                            ))
                        } else {
                            rullst::html::RawHtml(format!(
                                r#"<form method="post" action="/courses/{}/enroll"><input type="hidden" name="_token" value="{}" /><button type="submit">Enroll in course</button></form>"#,
                                course.id,
                                csrf_token
                            ))
                        }}
                        <h2>"Lessons"</h2>
                        <ul>{rullst::html::RawHtml(lesson_items)}</ul>
                    </aside>
                    <main>
                        {if is_enrolled {
                            rullst::html::RawHtml(r#"<section class="notice" style="border-color:#10b981;background:rgba(16,185,129,0.08);"><h2 style="color:#34d399">✅ Active Enrollment</h2><p>You are enrolled in this course! Select any lesson from the syllabus on the left to start watching.</p><p style="color:#94a3b8;font-size:0.875rem">Tip: all lessons are unlocked for this cloud showcase!</p></section>"#.to_string())
                        } else {
                            rullst::html::RawHtml(r#"<section class="notice"><h2>Protected lesson area</h2><p>Register and enroll before opening a lesson. The server derives identity from the session and verifies entitlement before returning media metadata.</p><p>Production media, captions and transcripts must be supplied by the host application; the scaffold fixtures are development-only.</p></section>"#.to_string())
                        }}
                    </main>
                </div>
                {rullst::html::RawHtml(render_lms_ai_widget(csrf_token))}
            </body>
        </html>
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LessonMediaError {
    InvalidKind,
    InvalidSource,
    MissingCaptions,
    InvalidLanguage,
    InvalidTranscript,
}

fn valid_media_source(value: &str) -> bool {
    if value.is_empty()
        || value.len() > 2_048
        || value.bytes().any(|byte| byte.is_ascii_control() || byte == b'\\')
    {
        return false;
    }
    let Ok(uri) = value.parse::<rullst::server::Uri>() else {
        return false;
    };
    match uri.scheme_str() {
        Some("https") => uri.authority().is_some(),
        None => uri.authority().is_none()
            && uri.path().starts_with('/')
            && !uri.path().starts_with("//"),
        Some(_) => false,
    }
}

fn valid_language_tag(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 35
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

pub fn lesson_player_page(
    title: &str,
    media_kind: &str,
    media_url: &str,
    captions_url: &str,
    transcript: &str,
    language_tag: &str,
    course_id: i32,
    lesson_id: i32,
    progress_percent: i32,
    csrf_token: &str,
    progress_key: &str,
    csp_nonce: &str,
) -> Result<String, LessonMediaError> {
    if !valid_media_source(media_url) {
        return Err(LessonMediaError::InvalidSource);
    }
    if !valid_language_tag(language_tag) {
        return Err(LessonMediaError::InvalidLanguage);
    }
    if transcript.is_empty() || transcript.len() > 65_536 {
        return Err(LessonMediaError::InvalidTranscript);
    }
    let is_youtube = media_url.contains("youtube.com") || media_url.contains("youtube-nocookie.com") || media_kind == "youtube";
    let media_player = if is_youtube {
        html! {
            <div style="position:relative;padding-bottom:56.25%;height:0;overflow:hidden;border-radius:1rem;border:1px solid #334155;background:#000;box-shadow:0 10px 25px -5px rgba(0,0,0,0.5);">
                <iframe
                    src={media_url}
                    title={title}
                    credentialless="true"
                    style="position:absolute;top:0;left:0;width:100%;height:100%;border:0;"
                    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
                    referrerpolicy="strict-origin-when-cross-origin"
                    allowfullscreen="true">
                </iframe>
            </div>
        }
    } else {
        match media_kind {
            "video" => {
                if !valid_media_source(captions_url) {
                    return Err(LessonMediaError::MissingCaptions);
                }
                html! {
                    <video controls="controls" preload="metadata">
                        <source src={media_url} />
                        <track kind="captions" src={captions_url} srclang={language_tag} label={language_tag} default="true" />
                        "Your browser does not support HTML video. Use the transcript below."
                    </video>
                }
            }
            "audio" => html! {
                <audio controls="controls" preload="metadata">
                    <source src={media_url} />
                    "Your browser does not support HTML audio. Use the transcript below."
                </audio>
            },
            _ => return Err(LessonMediaError::InvalidKind),
        }
    };
    Ok(html! {
        <html lang="en" class="dark">
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
                <title>{title}</title>
                <link rel="icon" type="image/x-icon" href="/favicon.ico" />
                <style nonce={csp_nonce}>
                    "
                    * { box-sizing: border-box; }
                    body { margin: 0; background: #080b11; color: #f8fafc; min-height: 100vh; padding: 2rem 1rem 4rem; font: 16px system-ui, sans-serif; }
                    a { color: #fdba74; font-weight: 700; }
                    a:focus-visible, button:focus-visible, video:focus-visible, audio:focus-visible, summary:focus-visible { outline: 3px solid #fbbf24; outline-offset: 3px; }
                    main { max-width: 60rem; margin: 0 auto; }
                    .player-card { overflow: hidden; margin-top: 1.5rem; border: 1px solid #334155; border-radius: 1rem; background: #0f172a; }
                    video { display: block; width: 100%; aspect-ratio: 16 / 9; background: #000; }
                    audio { display: block; width: calc(100% - 3rem); margin: 1.5rem; }
                    .info { padding: 1.5rem; }
                    h1 { margin-top: 0; }
                    .notice { color: #cbd5e1; line-height: 1.6; }
                    .progress { color: #6ee7b7; font-weight: 700; }
                    .progress-form { display: flex; gap: .75rem; flex-wrap: wrap; margin-top: 1.25rem; }
                    .transcript { margin-top: 1.5rem; border-top: 1px solid #334155; padding-top: 1rem; }
                    .transcript summary { cursor: pointer; font-weight: 800; }
                    .transcript p { color: #e2e8f0; line-height: 1.8; white-space: pre-wrap; }
                    button { border: 1px solid #34d399; border-radius: .55rem; background: #047857; color: #fff; padding: .7rem 1rem; font: inherit; font-weight: 800; cursor: pointer; }
                    button:hover { background: #065f46; }
                    "
                </style>
            </head>
            <body>
                <main>
                    <a href={format!("/courses/{course_id}")}>"← Back to course"</a>
                    <article class="player-card">
                        {rullst::html::RawHtml(media_player)}
                        <div class="info">
                            <h1>{title}</h1>
                            <p class="notice">"This development fixture may require an explicit media-src policy. Production applications must provide approved same-origin or signed media and verify caption/transcript quality."</p>
                            <p class="progress" role="status">"Saved progress: "{progress_percent.to_string()}"%"</p>
                            <details class="transcript">
                                <summary>"Transcript ("{language_tag}")"</summary>
                                <p>{transcript}</p>
                            </details>
                            <form class="progress-form" method="post" action={format!("/lessons/{lesson_id}/progress")}>
                                <input type="hidden" name="_token" value={csrf_token} />
                                <input type="hidden" name="idempotency_key" value={progress_key} />
                                <button type="submit" name="progress_percent" value="25">"Save 25%"</button>
                                <button type="submit" name="progress_percent" value="50">"Save 50%"</button>
                                <button type="submit" name="progress_percent" value="100">"Mark complete"</button>
                            </form>
                        </div>
                    </article>
                </main>
                {rullst::html::RawHtml(render_lms_ai_widget(csrf_token))}
            </body>
        </html>
    })
}

fn render_lms_ai_widget(csrf_token: &str) -> String {
    r##"
    <style>
    .lms-crab-launcher {
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
    .lms-crab-launcher:hover {
        transform: translateY(-4px) scale(1.05);
    }
    .lms-crab-bubble {
        background: rgba(15, 23, 42, 0.95);
        border: 1px solid rgba(52, 211, 153, 0.5);
        color: #fff;
        padding: 8px 14px;
        border-radius: 14px;
        font-size: 0.84rem;
        font-weight: 700;
        letter-spacing: 0.02em;
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5), 0 0 16px rgba(52, 211, 153, 0.25);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        display: flex;
        align-items: center;
        gap: 6px;
        position: relative;
        animation: lmsBubbleFloat 3s infinite ease-in-out;
        white-space: nowrap;
    }
    .lms-crab-bubble::after {
        content: '';
        position: absolute;
        right: -6px;
        top: 50%;
        transform: translateY(-50%) rotate(45deg);
        width: 10px;
        height: 10px;
        background: rgba(15, 23, 42, 0.95);
        border-top: 1px solid rgba(52, 211, 153, 0.5);
        border-right: 1px solid rgba(52, 211, 153, 0.5);
    }
    @keyframes lmsBubbleFloat {
        0%, 100% { transform: translateY(0); }
        50% { transform: translateY(-4px); }
    }
    .lms-bubble-sparkle { font-size: 0.95rem; }
    .lms-bubble-text {
        background: linear-gradient(135deg, #34d399, #10b981);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
    }
    .lms-crab-avatar {
        position: relative;
        width: 60px;
        height: 60px;
        flex-shrink: 0;
        filter: drop-shadow(0 8px 20px rgba(52, 211, 153, 0.4));
        animation: lmsCrabWiggle 4s infinite ease-in-out;
    }
    @keyframes lmsCrabWiggle {
        0%, 100% { transform: rotate(0deg); }
        25% { transform: rotate(-3deg) translateY(-2px); }
        75% { transform: rotate(3deg) translateY(-1px); }
    }
    .lms-crab-img { width: 100%; height: 100%; object-fit: contain; display: block; }
    .lms-crab-online {
        position: absolute;
        bottom: 2px;
        right: 2px;
        width: 12px;
        height: 12px;
        border-radius: 50%;
        background: #10b981;
        border: 2px solid #080b11;
        box-shadow: 0 0 8px #10b981;
    }
    .lms-ai-drawer {
        position: fixed;
        bottom: 84px;
        right: 24px;
        width: 420px;
        max-width: calc(100vw - 32px);
        height: 590px;
        height: min(590px, calc(100dvh - 110px));
        max-height: calc(100vh - 110px);
        background: rgba(15, 23, 42, 0.96);
        border: 1px solid rgba(52, 211, 153, 0.3);
        border-radius: 20px;
        box-shadow: 0 25px 60px -10px rgba(0, 0, 0, 0.85), 0 0 35px rgba(16, 185, 129, 0.15);
        backdrop-filter: blur(24px);
        -webkit-backdrop-filter: blur(24px);
        z-index: 9999;
        display: flex;
        flex-direction: column;
        overflow: hidden;
        animation: lmsDrawerSlideUp 0.25s cubic-bezier(0.16, 1, 0.3, 1);
    }
    @keyframes lmsDrawerSlideUp {
        from { opacity: 0; transform: translateY(20px) scale(0.97); }
        to { opacity: 1; transform: translateY(0) scale(1); }
    }
    .lms-drawer-header {
        padding: 14px 18px;
        background: rgba(15, 23, 42, 0.9);
        border-bottom: 1px solid #334155;
        display: flex;
        align-items: center;
        justify-content: space-between;
        flex-shrink: 0;
    }
    .lms-close-btn {
        background: rgba(255, 255, 255, 0.06);
        border: 1px solid #334155;
        color: #94a3b8;
        width: 36px;
        height: 36px;
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        font-size: 18px;
        line-height: 1;
    }
    .lms-close-btn:hover { color: #fff; background: rgba(255, 255, 255, 0.15); }
    .lms-chat-messages {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
        padding: 16px;
        display: flex;
        flex-direction: column;
        gap: 12px;
        overscroll-behavior: contain;
        -webkit-overflow-scrolling: touch;
    }
    .chat-bubble {
        display: flex;
        flex-direction: column;
        max-width: 90%;
        min-width: 0;
        animation: bubbleFadeIn 0.2s ease;
    }
    @keyframes bubbleFadeIn {
        from { opacity: 0; transform: translateY(6px); }
        to { opacity: 1; transform: translateY(0); }
    }
    .chat-bubble-user { align-self: flex-end; }
    .chat-bubble-assistant { align-self: flex-start; }
    .chat-bubble-sender {
        font-size: 0.7rem;
        font-weight: 600;
        color: #94a3b8;
        margin-bottom: 3px;
        padding: 0 4px;
    }
    .chat-bubble-user .chat-bubble-sender { text-align: right; color: #34d399; }
    .chat-bubble-body {
        padding: 10px 14px;
        border-radius: 12px;
        font-size: 0.86rem;
        line-height: 1.55;
        min-width: 0;
        max-width: 100%;
        overflow-wrap: anywhere;
    }
    .chat-bubble-user .chat-bubble-body {
        background: #047857;
        color: #fff;
        border-bottom-right-radius: 2px;
    }
    .chat-bubble-assistant .chat-bubble-body {
        background: #0f172a;
        border: 1px solid #334155;
        color: #f1f5f9;
        border-bottom-left-radius: 2px;
    }
    .chat-bubble-assistant.error .chat-bubble-body {
        background: rgba(239, 68, 68, 0.15);
        border-color: rgba(239, 68, 68, 0.4);
        color: #fca5a5;
    }
    .chat-bubble-body .rullst-ai-prose { min-width: 0; max-width: 100%; }
    .chat-bubble-body .rullst-ai-prose > :first-child { margin-top: 0; }
    .chat-bubble-body .rullst-ai-prose > :last-child { margin-bottom: 0; }
    .chat-bubble-body .rullst-ai-prose pre,
    .chat-bubble-body .rullst-ai-prose table {
        display: block;
        max-width: 100%;
        overflow-x: auto;
        -webkit-overflow-scrolling: touch;
    }
    .lms-badge-footer {
        margin-top: 8px;
        padding-top: 6px;
        border-top: 1px solid rgba(255, 255, 255, 0.08);
        font-size: 0.68rem;
        color: #34d399;
    }
    .lms-prompt-suggestions {
        padding: 8px 14px;
        border-top: 1px solid rgba(255, 255, 255, 0.06);
        background: rgba(11, 15, 25, 0.7);
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
        max-height: 84px;
        overflow-y: auto;
    }
    .lms-prompt-suggestions::-webkit-scrollbar { width: 3px; }
    .lms-prompt-suggestions::-webkit-scrollbar-thumb {
        background: rgba(52, 211, 153, 0.3);
        border-radius: 3px;
    }
    .lms-pill-btn {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid #334155;
        color: #cbd5e1;
        font-size: 0.72rem;
        padding: 5px 10px;
        border-radius: 20px;
        cursor: pointer;
        transition: all 0.15s ease;
        line-height: 1.2;
        display: inline-flex;
        align-items: center;
        gap: 4px;
    }
    .lms-pill-btn:hover {
        background: rgba(52, 211, 153, 0.15);
        border-color: #34d399;
        color: #34d399;
        transform: translateY(-1px);
    }
    .lms-typing-indicator {
        display: none;
        align-items: center;
        gap: 4px;
        padding: 8px 16px;
        background: rgba(15, 23, 42, 0.85);
    }
    .lms-typing-indicator.htmx-request { display: flex; }
    .lms-typing-dot {
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background: #34d399;
        animation: aiPulsingDot 1.2s infinite ease-in-out;
    }
    .lms-typing-dot:nth-child(2) { animation-delay: 0.2s; }
    .lms-typing-dot:nth-child(3) { animation-delay: 0.4s; }
    @keyframes aiPulsingDot {
        0%, 80%, 100% { transform: scale(0.6); opacity: 0.4; }
        40% { transform: scale(1.1); opacity: 1; }
    }
    .lms-ai-form {
        padding: 10px 14px;
        background: #0f172a;
        border-top: 1px solid #334155;
        display: flex;
        gap: 8px;
        align-items: center;
        flex-shrink: 0;
    }
    .lms-input {
        flex: 1;
        min-width: 0;
        background: #080b11;
        border: 1px solid #334155;
        border-radius: 10px;
        padding: 9px 13px;
        color: #fff;
        font-size: 0.84rem;
        outline: none;
    }
    .lms-input:focus { border-color: #34d399; }
    .lms-submit-btn {
        background: #047857;
        color: #fff;
        font-weight: 700;
        border: none;
        border-radius: 10px;
        padding: 9px 15px;
        cursor: pointer;
        font-size: 0.84rem;
        transition: opacity 0.15s;
    }
    .lms-submit-btn:hover { background: #065f46; }
    @media (max-width: 640px) {
        .lms-crab-launcher {
            bottom: max(16px, env(safe-area-inset-bottom));
            right: max(16px, env(safe-area-inset-right));
            gap: 8px;
        }
        .lms-crab-avatar { width: 48px; height: 48px; }
        .lms-crab-bubble { font-size: 0.76rem; padding: 6px 10px; }
        .lms-ai-drawer {
            bottom: 0 !important;
            right: 0 !important;
            left: 0 !important;
            width: 100% !important;
            max-width: 100% !important;
            height: 92vh !important;
            height: 92dvh !important;
            max-height: 100vh !important;
            max-height: calc(100dvh - env(safe-area-inset-top, 0px)) !important;
            border-radius: 20px 20px 0 0 !important;
            border-bottom: none !important;
            box-shadow: 0 -10px 40px rgba(0, 0, 0, 0.85) !important;
        }
        .lms-drawer-header { padding: 10px 12px; }
        .lms-close-btn { width: 44px; height: 44px; font-size: 24px; }
        .lms-chat-messages { padding: 12px; gap: 10px; }
        .chat-bubble { max-width: 100%; }
        .chat-bubble-body { padding: 10px 12px; font-size: 0.95rem; }
        .lms-prompt-suggestions {
            flex-wrap: nowrap;
            max-height: none;
            overflow-x: auto;
            overflow-y: hidden;
            padding: 8px 12px;
            -webkit-overflow-scrolling: touch;
        }
        .lms-pill-btn { min-height: 40px; padding: 8px 12px; flex-shrink: 0; }
        .lms-ai-form { padding: 10px 12px calc(10px + env(safe-area-inset-bottom, 0px)); gap: 8px; }
        .lms-input { min-height: 44px; font-size: 16px; padding: 10px 12px; }
        .lms-submit-btn { min-width: 72px; min-height: 44px; font-size: 0.9rem; }
    }
    @media (max-width: 380px) {
        .lms-crab-bubble { display: none; }
        .lms-drawer-header > div > div:last-child { min-width: 0; }
    }
    @media (max-height: 500px) and (orientation: landscape) {
        .lms-ai-drawer { height: 100vh !important; height: 100dvh !important; max-height: 100vh !important; max-height: 100dvh !important; border-radius: 0 !important; }
        .lms-prompt-suggestions { display: none; }
    }
    </style>

    <div id="lms-crab-launcher" class="lms-crab-launcher" onclick="toggleLmsAiDrawer()" role="button" tabindex="0" aria-label="Ask me anything about Rullst or Rust!">
        <div class="lms-crab-bubble">
            <span class="lms-bubble-sparkle">✨</span>
            <span class="lms-bubble-text">Ask me anything about Rullst or Rust!</span>
        </div>
        <div class="lms-crab-avatar">
            <img src="/static/crab.png" alt="Rullst Crab Mascot" class="lms-crab-img" />
            <span class="lms-crab-online"></span>
        </div>
    </div>

    <div id="lms-ai-drawer" class="lms-ai-drawer" style="display: none;" role="dialog" aria-label="Academic Copilot">
        <div class="lms-drawer-header">
            <div style="display: flex; align-items: center; gap: 10px;">
                <div style="width: 34px; height: 34px; border-radius: 10px; background: rgba(52, 211, 153, 0.15); border: 1px solid rgba(52, 211, 153, 0.3); display: flex; align-items: center; justify-content: center; font-size: 1.1rem;">🎓</div>
                <div>
                    <div style="font-weight: 800; font-size: 0.95rem; color: #fff;">Academic Copilot</div>
                    <div style="font-size: 0.72rem; color: #34d399; display: flex; align-items: center; gap: 5px;">
                        <span style="width: 6px; height: 6px; border-radius: 50%; background: #34d399; display: inline-block;"></span>
                        <span>AI Learning & Curriculum Assistant</span>
                    </div>
                </div>
            </div>
            <button class="lms-close-btn" onclick="toggleLmsAiDrawer()" aria-label="Close">×</button>
        </div>

        <div id="lms-chat-messages" class="lms-chat-messages">
            <div class="chat-bubble chat-bubble-assistant">
                <div class="chat-bubble-sender">Academic Copilot</div>
                <div class="chat-bubble-body">
                    Hello! I am the <strong>Academic Copilot</strong> for Rullst Academy. Ask me anything about the <strong>Rust</strong> language (ownership, concurrency, types), the <strong>Rullst</strong> web framework, or our courses!
                    <div class="lms-badge-footer">⚡ Context-Aware RAG • Protected by Rullst Guardrails</div>
                </div>
            </div>
        </div>

        <div class="lms-prompt-suggestions">
            <button type="button" class="lms-pill-btn" onclick="setLmsPrompt('What is Rullst and what makes it unique for Rust web development?')">⚡ What is Rullst?</button>
            <button type="button" class="lms-pill-btn" onclick="setLmsPrompt('How does memory safety, ownership, and lifetimes work in Rust?')">🦀 Rust Memory Safety</button>
            <button type="button" class="lms-pill-btn" onclick="setLmsPrompt('What courses and learning tracks are available in Rullst Academy?')">📚 Course Catalog</button>
            <button type="button" class="lms-pill-btn" onclick="setLmsPrompt('How does HTMX integrate with Rust SSR without JavaScript build bloat?')">🌐 HTMX + Rust SSR</button>
            <button type="button" class="lms-pill-btn" onclick="setLmsPrompt('Explain asynchronous concurrency with Tokio, Arc, and Mutex in Rust.')">🧠 Tokio & Concurrency</button>
        </div>

        <div id="lms-typing" class="lms-typing-indicator">
            <span class="lms-typing-dot"></span>
            <span class="lms-typing-dot"></span>
            <span class="lms-typing-dot"></span>
            <span style="font-size: 0.72rem; color: #a1a1aa; margin-left: 6px;">Copilot is thinking...</span>
        </div>

        <form id="lms-chat-form" class="lms-ai-form"
              hx-post="/api/lms-chat"
              hx-target="#lms-chat-messages"
              hx-swap="beforeend"
              hx-indicator="#lms-typing"
              hx-on::before-request="appendLmsUserMessage()"
              hx-on::after-request="finalizeLmsAiRequest()">
            <input type="hidden" name="_token" value="__CSRF_TOKEN__" id="lms-csrf-token" />
            <input id="lms-message-input" type="text" name="message" class="lms-input" placeholder="Ask anything about Rust, Rullst, or our courses..." autocomplete="off" required maxlength="600" />
            <button type="submit" class="lms-submit-btn">Send</button>
        </form>
    </div>

    <script>
        document.body.addEventListener('htmx:configRequest', function(evt) {
            var tokenInput = document.getElementById('lms-csrf-token');
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
            var msgs = document.getElementById('lms-chat-messages');
            if (msgs) {
                var errDiv = document.createElement('div');
                errDiv.className = 'chat-bubble chat-bubble-assistant error';
                errDiv.innerHTML = '<div class="chat-bubble-sender">Academic Copilot</div><div class="chat-bubble-body">⚠️ Could not reach the Copilot (HTTP ' + (evt.detail.xhr ? evt.detail.xhr.status : 'error') + '). Please try again.</div>';
                msgs.appendChild(errDiv);
                scrollLmsToBottom();
            }
        });

        function toggleLmsAiDrawer() {
            var drawer = document.getElementById('lms-ai-drawer');
            var launcher = document.getElementById('lms-crab-launcher');
            if (!drawer) return;
            var isOpen = drawer.style.display !== 'none';
            if (isOpen) {
                drawer.style.display = 'none';
                if (launcher) launcher.style.display = 'flex';
                document.body.style.overflow = '';
            } else {
                drawer.style.display = 'flex';
                if (launcher) launcher.style.display = 'none';
                if (window.innerWidth <= 640) document.body.style.overflow = 'hidden';
                var input = document.getElementById('lms-message-input');
                if (input) setTimeout(function() { input.focus(); }, 150);
                scrollLmsToBottom();
            }
        }

        function scrollLmsToBottom() {
            var msgs = document.getElementById('lms-chat-messages');
            if (msgs) {
                setTimeout(function() { msgs.scrollTop = msgs.scrollHeight; }, 50);
            }
        }

        function setLmsPrompt(text) {
            var input = document.getElementById('lms-message-input');
            var form = document.getElementById('lms-chat-form');
            if (input && form) {
                input.value = text;
                if (form.requestSubmit) {
                    form.requestSubmit();
                } else {
                    form.submit();
                }
            }
        }

        function escapeLmsHtml(str) {
            return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#039;');
        }

        function appendLmsUserMessage() {
            var input = document.getElementById('lms-message-input');
            if (!input || !input.value.trim()) return;
            var msg = input.value.trim();
            var msgs = document.getElementById('lms-chat-messages');
            if (msgs) {
                var bubble = document.createElement('div');
                bubble.className = 'chat-bubble chat-bubble-user';
                bubble.innerHTML = '<div class="chat-bubble-sender">You</div><div class="chat-bubble-body">' + escapeLmsHtml(msg) + '</div>';
                msgs.appendChild(bubble);
                scrollLmsToBottom();
            }
        }

        function finalizeLmsAiRequest() {
            var input = document.getElementById('lms-message-input');
            if (input) {
                input.value = '';
                input.focus();
            }
            scrollLmsToBottom();
        }

        document.addEventListener('keydown', function(e) {
            if (e.key === 'Escape') {
                var drawer = document.getElementById('lms-ai-drawer');
                if (drawer && drawer.style.display !== 'none') {
                    toggleLmsAiDrawer();
                }
            }
        });

        document.addEventListener('htmx:afterSwap', function(e) {
            if (e.detail.target && e.detail.target.id === 'lms-chat-messages') {
                scrollLmsToBottom();
            }
        });
    </script>
    "##.replace("__CSRF_TOKEN__", &rullst::html::escape_str(csrf_token))
}

#[cfg(test)]
mod community_callout_tests {
    use super::render_community_callout;

    #[test]
    fn callout_links_to_discord_safely() {
        let callout = render_community_callout();
        assert!(callout.contains("Build alongside the Rullst community"));
        assert!(callout.contains("https://discord.gg/2ntKFtsSjw"));
        assert!(callout.contains("rel=\"noopener noreferrer\""));
        assert!(callout.contains("<section class=\"community-callout\""));
    }
}
