//! Platform sessions are the single login boundary for both embedded tools.
use axum::{
    extract::Request,
    extract::State,
    http::{HeaderValue, Method, header},
    middleware::Next,
    response::{Html, IntoResponse, Redirect, Response},
};
use base64::Engine;

#[derive(Clone)]
pub struct ToolAccess {
    authorization: HeaderValue,
    pub tool: &'static str,
}

impl ToolAccess {
    pub fn new() -> Result<(Self, rullst::nexus::NexusAuthPolicy), Box<dyn std::error::Error>> {
        let mut random = [0u8; 32];
        getrandom::fill(&mut random).map_err(|_| "Cannot initialize tool session bridge")?;
        let password = base64::engine::general_purpose::STANDARD.encode(random);
        let policy = rullst::nexus::NexusAuthPolicy::basic("platform-session", &password)?;
        let encoded = base64::engine::general_purpose::STANDARD
            .encode(format!("platform-session:{password}"));
        Ok((
            Self {
                authorization: HeaderValue::from_str(&format!("Basic {encoded}"))?,
                tool: "nexus",
            },
            policy,
        ))
    }

    pub fn studio(&self) -> Self {
        Self {
            tool: "studio",
            ..self.clone()
        }
    }
}

pub fn is_staff(email: &str) -> bool {
    std::env::var("LMS_ADMIN_EMAILS")
        .unwrap_or_default()
        .split(',')
        .any(|allowed| !allowed.trim().is_empty() && allowed.trim().eq_ignore_ascii_case(email))
}

fn demo_route_allowed(tool: &str, method: &Method, path: &str) -> bool {
    let path = path.strip_prefix(&format!("/{tool}")).unwrap_or(path);
    if matches!(path, "/copilot/query" | "/chat/query") && *method == Method::POST {
        return true;
    }
    if !matches!(*method, Method::GET | Method::HEAD) {
        return false;
    }
    let path = path
        .strip_prefix("/studio")
        .filter(|_| tool == "studio")
        .unwrap_or(path);
    if matches!(path, "" | "/") {
        return true;
    }
    if matches!(path, "/chat" | "/ai" | "/copilot" | "/copilot.js") {
        return true;
    }
    if tool == "studio" && matches!(path, "/cache" | "/assets/studio.css" | "/assets/logger.js") {
        return true;
    }
    let prefix = if tool == "nexus" {
        "/table/"
    } else {
        "/tables/"
    };
    let Some(rest) = path.strip_prefix(prefix) else {
        return false;
    };
    let (table, suffix) = rest.split_once('/').unwrap_or((rest, ""));
    matches!(table, "categories" | "courses" | "lessons") && matches!(suffix, "" | "search")
}

fn login_redirect(tool: &str, htmx: bool) -> Response {
    let target = format!("/login?next=/{tool}");
    let mut response = Redirect::to(&target).into_response();
    if htmx {
        *response.status_mut() = axum::http::StatusCode::OK;
        response
            .headers_mut()
            .insert("hx-redirect", HeaderValue::from_str(&target).unwrap());
    }
    response
}

pub async fn guard(State(access): State<ToolAccess>, mut req: Request, next: Next) -> Response {
    let htmx = req.headers().contains_key("hx-request");
    let Some(cookie) = rullst::auth::extract_session_cookie(req.headers()) else {
        return login_redirect(access.tool, htmx);
    };
    let Ok(key) = rullst::auth::get_app_key() else {
        return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let Ok(id) = rullst::auth::decrypt_session(&cookie, &key) else {
        return login_redirect(access.tool, htmx);
    };
    let user = match crate::models::user::User::find(id).await {
        Ok(Some(user)) => user,
        Ok(None) => return login_redirect(access.tool, htmx),
        Err(_) => return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let demo =
        crate::showcase::enabled() && user.id == 100 && user.email == crate::showcase::DEMO_EMAIL;
    if (!demo && !is_staff(&user.email))
        || (demo && !demo_route_allowed(access.tool, req.method(), req.uri().path()))
    {
        let message = "<section style='padding:2rem;color:#cbd5e1;background:#0f172a;font:16px system-ui'><h1>Public showcase preview</h1><p>The demo account can explore the catalog in Nexus and Studio. Personal records, AI queries and administrative changes require an authorized staff account.</p><p><a style='color:#6ee7b7' href='/'>Back to the showcase</a></p></section>";
        let mut response = (axum::http::StatusCode::FORBIDDEN, Html(message)).into_response();
        // HTMX normally ignores 403 bodies; show the restriction inside the tool.
        if htmx {
            *response.status_mut() = axum::http::StatusCode::OK;
        }
        return response;
    }
    // Ignore any client-supplied Basic identity. Nexus still validates its own
    // private, random process credential after platform authentication succeeds.
    req.headers_mut()
        .insert(header::AUTHORIZATION, access.authorization);
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn demo_access_cannot_reach_private_tables_or_mutations() {
        assert!(demo_route_allowed(
            "nexus",
            &Method::GET,
            "/table/courses/search"
        ));
        assert!(demo_route_allowed(
            "studio",
            &Method::GET,
            "/studio/tables/lessons"
        ));
        for path in [
            "/table/users",
            "/table/courses/new",
            "/table/courses/1/edit",
            "/table/../users",
            "/table/%75sers",
        ] {
            assert!(!demo_route_allowed("nexus", &Method::GET, path), "{path}");
        }
        assert!(!demo_route_allowed(
            "nexus",
            &Method::POST,
            "/table/courses"
        ));
        assert!(!demo_route_allowed(
            "studio",
            &Method::GET,
            "/studio/tables/users"
        ));
        assert!(!demo_route_allowed("studio", &Method::GET, "/api/traces"));
    }
}
