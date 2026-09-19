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
        r#"<footer class="showcase-footer"><div class="showcase-footer-brand"><strong>Rullst SaaS Showcase</strong><span>Built entirely with Rullst v12.</span><a class="showcase-discord" href="https://discord.gg/2ntKFtsSjw" rel="noreferrer">Join our Discord ↗</a></div><div class="showcase-footer-grid">"#,
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
