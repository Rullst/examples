use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::{Html, IntoResponse, Response},
};
pub async fn css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        format!(
            "{}{}{}",
            include_str!("../static/showcase.css"),
            include_str!("../static/nexus-responsive.css"),
            blueprint_ai::STYLES.to_owned() + blueprint_ai::COPILOT_STYLES
        ),
    )
}
pub async fn js() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        include_str!("../static/showcase.js").to_owned() + blueprint_ai::COPILOT_SCRIPT,
    )
}
pub async fn tailwind() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        include_str!("../static/tailwind.js"),
    )
}
pub async fn logo() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/png")],
        include_bytes!("../static/rullst.png").as_slice(),
    )
}
pub async fn privacy() -> Html<&'static str> {
    Html(include_str!("../templates/privacy.html"))
}
pub async fn cookies() -> Html<&'static str> {
    Html(include_str!("../templates/cookies.html"))
}
pub fn footer() -> String {
    let groups: &[(&str, &[(&str, &str)])] = &[
        (
            "Showcases",
            &[
                ("LMS", "https://lms.rullst.win"),
                ("Portfolio", "https://portfolio.rullst.win"),
                ("Framework", "https://showcase.rullst.win"),
                ("SaaS", "https://saas.rullst.win"),
            ],
        ),
        (
            "Build & learn",
            &[
                ("Website", "https://rullst.github.io"),
                ("GitHub", "https://github.com/Rullst"),
                ("Academy", "https://academy.rullst.win"),
                ("Blogspot", "https://rullst.blogspot.com"),
                ("Dev.to", "https://dev.to/venelouis"),
                ("Hashnode", "https://rullst.hashnode.dev"),
                ("Substack", "https://substack.com/@rullst"),
            ],
        ),
        (
            "Community",
            &[
                ("Discord", "https://discord.gg/2ntKFtsSjw"),
                ("Daily.dev", "https://daily.dev/squads/rullst"),
                ("Reddit", "https://www.reddit.com/r/rullst"),
                ("Telegram", "https://t.me/rullst"),
                ("Bluesky", "https://bsky.app/profile/rullst.bsky.social"),
                ("LinkedIn", "https://www.linkedin.com/company/rullst"),
            ],
        ),
        (
            "Follow Rullst",
            &[
                ("BiliBili", "https://www.bilibili.tv/en/space/1436672033"),
                ("Instagram", "https://instagram.com/rullst_official"),
                ("TikTok", "https://tiktok.com/@venelouis"),
                ("YouTube", "https://youtube.com/@Rullst_Official"),
                ("X", "https://x.com/venelouis"),
            ],
        ),
    ];
    let mut result = String::from(
        r#"<footer class="showcase-footer"><div class="showcase-footer-brand"><strong>Rullst Portfolio Showcase</strong><span>Built entirely with Rullst v12.</span><a class="showcase-discord" href="https://discord.gg/2ntKFtsSjw" rel="noreferrer">Join our Discord ↗</a></div><div class="showcase-footer-grid">"#,
    );
    for (title, links) in groups {
        result.push_str(&format!("<nav aria-label=\"{title}\"><h2>{title}</h2>"));
        for (label, url) in *links {
            result.push_str(&format!("<a href=\"{url}\" rel=\"noreferrer\">{label}</a>"));
        }
        result.push_str("</nav>");
    }
    result.push_str(r#"</div><div class="showcase-footer-bottom"><a href="/privacy">Privacy notice</a><a href="/cookies">Cookies & privacy choices</a><a href="mailto:officialrullst@gmail.com">Contact Rullst</a></div></footer>"#);
    result
}

// Touch HTML only: buffering event streams would leave tool navigation loading forever.
pub async fn shell(req: Request, next: Next) -> Response {
    let tool = req.uri().path().starts_with("/nexus") || req.uri().path().starts_with("/studio");
    let asset = req.uri().path().starts_with("/static/") || req.uri().path() == "/favicon.ico";
    let mut response = next.run(req).await;
    response
        .headers_mut()
        .insert(header::REFERRER_POLICY, "no-referrer".parse().unwrap());
    if !asset {
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    }
    let is_html = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("text/html"));
    if !is_html {
        return response;
    }
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    let (mut parts, body) = response.into_parts();
    let Ok(bytes) = axum::body::to_bytes(body, 4 * 1024 * 1024).await else {
        return axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let mut html = String::from_utf8_lossy(&bytes).into_owned();
    if html.contains("</head>") {
        if html.contains("ai-drawer") && !html.contains("src=\"/static/htmx.js\"") {
            html = html.replace(
                "</head>",
                "<script src=\"/static/htmx.js\"></script></head>",
            );
        }
        html = html
            .replace("https://unpkg.com/htmx.org@2.0.4", "/static/htmx.js")
            .replace("https://unpkg.com/htmx.org@1.9.12", "/static/htmx.js")
            .replace("https://cdn.tailwindcss.com", "/static/tailwind.js")
            .replace(
                "https://raw.githubusercontent.com/Rullst/Rullst/main/Rullst.png",
                "/favicon.ico",
            )
            .replace(
                "https://raw.githubusercontent.com/venelouis/Rullst/main/Rullst.png",
                "/favicon.ico",
            );
        // System fonts avoid a third-party connection on every admin page.
        html = html
            .split_inclusive('>')
            .filter(|tag| {
                !(tag.trim_start().starts_with("<link")
                    && (tag.contains("fonts.googleapis.com") || tag.contains("fonts.gstatic.com")))
            })
            .collect();
        let head = r#"<meta name="htmx-config" content='{"historyCacheSize":0}'><link rel="stylesheet" href="/static/showcase.css"><script src="/static/showcase.js" defer></script>"#;
        html = html.replace("</head>", &format!("{head}</head>"));
        if !tool {
            html = html.replace("</body>", &format!("{}</body>", footer()));
        }
    }
    parts.headers.remove(header::CONTENT_LENGTH);
    Response::from_parts(parts, axum::body::Body::from(html))
}
