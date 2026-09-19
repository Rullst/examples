use rullst::response::Html;

fn auth_page(
    title: &str,
    action: &str,
    csrf_token: &str,
    error: Option<&str>,
    include_name: bool,
    csp_nonce: &str,
    next: &str,
) -> Html<String> {
    let error_html = error.map_or_else(String::new, |message| {
        format!(
            "<p class=\"error\">{}</p>",
            rullst::html::escape_str(message)
        )
    });
    let name_html = if include_name {
        "<label>Name<input name=\"name\" type=\"text\" maxlength=\"120\" required></label>"
    } else {
        ""
    };
    let alternate = if crate::showcase::enabled() {
        "One session opens the platform, Nexus and Studio."
    } else if include_name {
        "Already registered? <a href=\"/login\">Sign in</a>"
    } else {
        "New learner? <a href=\"/register\">Create an account</a>"
    };

    Html(format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{title} — Rullst LMS Showcase</title><link rel="icon" type="image/png" href="https://raw.githubusercontent.com/Rullst/Rullst/main/Rullst.png"><style nonce="{nonce}">
        *{{box-sizing:border-box}}body{{margin:0;min-height:100vh;display:flex;flex-direction:column;align-items:center;padding:2rem 1rem;background:#080b11;color:#f8fafc;font:16px system-ui,sans-serif}}main{{width:min(100%,430px);padding:2rem;border:1px solid #263244;border-radius:1rem;background:#0f172a}}h1{{margin-top:0}}label{{display:grid;gap:.4rem;margin:1rem 0;color:#cbd5e1}}input{{padding:.8rem;border:1px solid #475569;border-radius:.5rem;background:#111827;color:white}}button{{width:100%;padding:.85rem;border:0;border-radius:.5rem;background:#10b981;color:#052e16;font-weight:800;cursor:pointer}}a{{color:#34d399}}.error{{padding:.75rem;border-radius:.5rem;background:#450a0a;color:#fecaca}}</style></head><body><main><p>🎓 Rullst LMS Showcase</p><h1>{title}</h1>{demo_html}{error_html}<form method="post" action="{action}"><input type="hidden" name="next" value="{next}"><input type="hidden" name="_token" value="{csrf}">{name_html}<label>Email<input name="email" type="email" maxlength="254" autocomplete="email" required></label><label>Password<input name="password" type="password" maxlength="72" autocomplete="current-password" required></label><button type="submit">{title}</button></form><p>{alternate}</p><p style="font-size:.8rem">Use fictional data in the shared demo. Read our <a href="/privacy">privacy notice</a>.</p><p><a href="/">Back to catalog</a></p></main></body></html>"#,
        demo_html = if crate::showcase::enabled() {
            format!(
                "<div class='showcase-access'><strong>Platform · Nexus · Studio</strong><p>Email: <code>{}</code><br>Password: <code>{}</code></p></div>",
                crate::showcase::DEMO_EMAIL,
                crate::showcase::DEMO_PASSWORD
            )
        } else {
            String::new()
        },
        next = rullst::html::escape_str(next),
        title = rullst::html::escape_str(title),
        action = rullst::html::escape_str(action),
        csrf = rullst::html::escape_str(csrf_token),
        nonce = rullst::html::escape_str(csp_nonce),
    ))
}

pub fn login_page(
    csrf_token: &str,
    error: Option<&str>,
    csp_nonce: &str,
    next: &str,
) -> Html<String> {
    auth_page(
        "Sign in", "/login", csrf_token, error, false, csp_nonce, next,
    )
}

pub fn register_page(csrf_token: &str, error: Option<&str>, csp_nonce: &str) -> Html<String> {
    auth_page(
        "Create account",
        "/register",
        csrf_token,
        error,
        true,
        csp_nonce,
        "/dashboard",
    )
}

pub fn dashboard_page(user_name: &str, csp_nonce: &str, csrf_token: &str) -> Html<String> {
    Html(format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Learner dashboard</title><link rel="icon" type="image/png" href="https://raw.githubusercontent.com/Rullst/Rullst/main/Rullst.png"><style nonce="{nonce}">body{{background:#080b11;color:#f8fafc;font:16px system-ui;padding:3rem}}main{{max-width:760px;margin:auto}}a{{color:#34d399}}</style></head><body><main><p>🎓 Rullst LMS Showcase</p><h1>Welcome, {user}</h1><p>Your encrypted session is active. Continue to the capabilities configured by this application.</p><div class="showcase-actions"><a class="showcase-primary" href="/">Open catalog</a><a class="showcase-primary" href="/nexus">Nexus</a><a class="showcase-primary" href="/studio">Studio</a></div><form method="post" action="/logout"><input type="hidden" name="_token" value="{csrf}"><button type="submit">Sign out</button></form></main></body></html>"#,
        csrf = rullst::html::escape_str(csrf_token),
        user = rullst::html::escape_str(user_name),
        nonce = rullst::html::escape_str(csp_nonce),
    ))
}
