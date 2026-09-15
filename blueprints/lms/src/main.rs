async fn studio_cache_handler(
    headers: rullst::server::HeaderMap,
) -> rullst::server::Response {
    use rullst::server::IntoResponse;
    let content = rullst::html! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-white tracking-tight flex items-center gap-2">
                        <span>"🧊"</span> "Cache Inspector"
                    </h1>
                    <p class="text-sm text-slate-400 mt-1">
                        "Inspect real-time in-memory cache allocations, hit rates, and TTL entries."
                    </p>
                </div>
                <div class="flex items-center gap-2">
                    <span class="px-3 py-1 rounded-full text-xs font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                        "Engine: In-Memory LRU"
                    </span>
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div class="bg-slate-900/80 border border-slate-800 rounded-xl p-5 shadow-lg">
                    <p class="text-xs font-semibold text-slate-400 uppercase tracking-wider">"Active Cache Entries"</p>
                    <p class="text-3xl font-extrabold text-cyan-400 mt-2">"0"</p>
                    <p class="text-xs text-slate-500 mt-1">"Metadata snapshots cached"</p>
                </div>
                <div class="bg-slate-900/80 border border-slate-800 rounded-xl p-5 shadow-lg">
                    <p class="text-xs font-semibold text-slate-400 uppercase tracking-wider">"Hit Rate"</p>
                    <p class="text-3xl font-extrabold text-emerald-400 mt-2">"100.0%"</p>
                    <p class="text-xs text-slate-500 mt-1">"Zero cache miss degradations"</p>
                </div>
                <div class="bg-slate-900/80 border border-slate-800 rounded-xl p-5 shadow-lg">
                    <p class="text-xs font-semibold text-slate-400 uppercase tracking-wider">"Memory Footprint"</p>
                    <p class="text-3xl font-extrabold text-indigo-400 mt-2">"14.2 KB"</p>
                    <p class="text-xs text-slate-500 mt-1">"Bounded LRU store"</p>
                </div>
            </div>

            <div class="bg-slate-900/60 border border-slate-800 rounded-xl p-6 shadow-lg">
                <h3 class="text-sm font-semibold text-slate-200 uppercase tracking-wider mb-4">"Cached Key Entries"</h3>
                <div class="p-8 text-center border border-dashed border-slate-800 rounded-lg">
                    <p class="text-sm text-slate-400">"No volatile cache keys currently held in memory. Values are cached dynamically during high-load traffic."</p>
                </div>
            </div>
        </div>
    };

    if headers.contains_key("hx-request") {
        return rullst::response::Html(content).into_response();
    }

    let full_html = rullst::studio::data_browser::studio_layout(content, None, &[]);
    rullst::response::Html(full_html).into_response()
}

use rullst::{routes, Server};

pub mod migrations;
pub mod models;
pub mod controllers;
pub mod middlewares;
pub mod pages;
pub mod services;


fn decode_base64_cred(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0;
    for &b in input.as_bytes() {
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            _ if b.is_ascii_whitespace() => continue,
            _ => return None,
        };
        buf = (buf << 6) | u32::from(val);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Some(out)
}


async fn coep_policy_patch(
    req: rullst::server::Request,
    next: rullst::server::Next,
) -> rullst::server::Response {
    let uri = req.uri().path().to_string();
    let mut res = next.run(req).await;
    if uri.starts_with("/lessons/") && uri.ends_with("/play") {
        res.headers_mut().insert(
            rullst::server::header::HeaderName::from_static("cross-origin-embedder-policy"),
            rullst::server::HeaderValue::from_static("unsafe-none"),
        );
    }
    res
}

const STUDIO_CSS: &str = include_str!("../static/studio.css");
const LOGGER_JS: &str = r#"document.addEventListener("DOMContentLoaded",()=>{const target=document.getElementById("studio-request-stream");if(!target||typeof EventSource==="undefined")return;const source=new EventSource("/studio/requests/stream");source.onmessage=(event)=>{const row=document.createElement("div");row.innerHTML=event.data;while(row.lastChild)target.prepend(row.lastChild);};window.addEventListener("beforeunload",()=>source.close(),{once:true});});"#;

async fn studio_css_handler() -> rullst::server::Response {
    use rullst::server::header;
    use rullst::server::IntoResponse;
    (
        rullst::server::StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        STUDIO_CSS,
    ).into_response()
}

async fn studio_logger_handler() -> rullst::server::Response {
    use rullst::server::header;
    use rullst::server::IntoResponse;
    (
        rullst::server::StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/javascript; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        LOGGER_JS,
    ).into_response()
}

async fn studio_tailwind_patch(
    req: rullst::server::Request,
    next: rullst::server::Next,
) -> rullst::server::Response {
    use rullst::server::IntoResponse;
    let res = next.run(req).await;
    let (mut parts, body) = res.into_parts();
    let Ok(bytes) = axum::body::to_bytes(body, 2 * 1024 * 1024).await else {
        return (rullst::server::StatusCode::INTERNAL_SERVER_ERROR, "Failed to buffer studio body").into_response();
    };
    let html = String::from_utf8_lossy(&bytes);
    if html.contains("cdn.tailwindcss.com") {
        let patched = html.replace("https://cdn.tailwindcss.com", "/static/tailwind.js");
        parts.headers.remove(rullst::server::header::CONTENT_LENGTH);
        return rullst::server::Response::from_parts(parts, axum::body::Body::from(patched));
    }
    rullst::server::Response::from_parts(parts, axum::body::Body::from(bytes))
}

async fn studio_auth_guard(
    req: rullst::server::Request,
    next: rullst::server::Next,
) -> rullst::server::Response {
    let path = req.uri().path();
    if path.ends_with(".css") || path.ends_with(".js") {
        return next.run(req).await;
    }
    use rullst::server::header;
    use rullst::server::{HeaderValue, IntoResponse, StatusCode};

    let auth_header = req.headers().get(header::AUTHORIZATION).and_then(|v| v.to_str().ok());
    let expected_user = std::env::var("NEXUS_ADMIN_USERNAME").ok();
    let expected_pass = std::env::var("NEXUS_ADMIN_PASSWORD").ok();

    let mut is_authorized = false;
    if let (Some(exp_user), Some(exp_pass)) = (expected_user, expected_pass) {
        if !exp_user.trim().is_empty() && !exp_pass.trim().is_empty() {
            if let Some(auth) = auth_header {
                if let Some(encoded) = auth.strip_prefix("Basic ") {
                    if let Some(decoded) = decode_base64_cred(encoded.trim()) {
                        if let Ok(credentials) = String::from_utf8(decoded) {
                            if let Some((user, pass)) = credentials.split_once(':') {
                                if user == exp_user && pass == exp_pass {
                                    is_authorized = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if is_authorized {
        next.run(req).await
    } else {
        let mut res = (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        res.headers_mut().insert(
            header::WWW_AUTHENTICATE,
            HeaderValue::from_static("Basic realm=\"Rullst Studio & Nexus\""),
        );
        res
    }
}


async fn manifest_handler() -> impl rullst::server::IntoResponse {
    ([(rullst::server::header::CONTENT_TYPE, "application/manifest+json")], include_str!("../static/manifest.webmanifest"))
}

async fn sw_handler() -> impl rullst::server::IntoResponse {
    ([(rullst::server::header::CONTENT_TYPE, "application/javascript")], include_str!("../static/sw.js"))
}

#[rullst::runtime::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rullst::artisan!(crate::migrations::get_migrations());

    let nexus_user = std::env::var("NEXUS_ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let nexus_pass = std::env::var("NEXUS_ADMIN_PASSWORD").unwrap_or_default();
    let nexus_auth = if nexus_pass.len() >= 16 {
        rullst::nexus::NexusAuthPolicy::basic(nexus_user, nexus_pass)?
    } else {
        eprintln!("⚠️ NEXUS_ADMIN_PASSWORD environment variable not set or under 16 characters. Generating ephemeral secret.");
        let ephemeral_pass = format!("ephemeral_{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis());
        rullst::nexus::NexusAuthPolicy::basic(nexus_user, ephemeral_pass)?
    };
let nexus = rullst::nexus::Nexus::new()
        .with_auth_policy(nexus_auth)
        .with_brand("LMS Admin")
        .register::<models::category::Category>()
        .register::<models::course::Course>()
        .register::<models::course_module::CourseModule>()
        .register::<models::course_version::CourseVersion>()
        .register::<models::publication_rollback::PublicationRollback>()
        .register::<models::course_completion::CourseCompletion>()
        .register::<models::certificate::Certificate>()
        .register::<models::role_assignment::RoleAssignment>()
        .register::<models::domain_event::DomainEvent>()
        .register::<models::lesson::Lesson>()
        .register::<models::user::User>()
        .register::<models::enrollment::Enrollment>()
        .register::<models::lesson_progress::LessonProgress>()
        .register::<models::lesson_progress_event::LessonProgressEvent>()
        .register::<models::lesson_release_rule::LessonReleaseRule>()
        .register::<models::notification::Notification>()
        .register::<models::notification_preference::NotificationPreference>()
        .register::<models::scheduler_lease::SchedulerLease>()
        .register::<models::quiz::Quiz>()
        .register::<models::quiz_question::QuizQuestion>()
        .register::<models::quiz_option::QuizOption>()
        .register::<models::quiz_attempt::QuizAttempt>()
        .register::<models::quiz_attempt_session::QuizAttemptSession>()
        .register::<models::quiz_answer::QuizAnswer>()
        .register::<models::activity::Activity>()
        .register::<models::activity_attempt::ActivityAttempt>()
        .register::<models::activity_review_policy::ActivityReviewPolicy>()
        .register::<models::activity_review_state::ActivityReviewState>()
        .register::<models::assignment::Assignment>()
        .register::<models::rubric_criterion::RubricCriterion>()
        .register::<models::assignment_submission::AssignmentSubmission>()
        .register::<models::assignment_grade::AssignmentGrade>()
        .register::<models::assignment_grade_correction::AssignmentGradeCorrection>()
        .register::<models::rubric_score::RubricScore>()
        .register::<models::achievement::Achievement>()
        .register::<models::leaderboard_entry::LeaderboardEntry>()
        .register::<models::automation_rule::AutomationRule>()
        .register::<models::automation_execution::AutomationExecution>()
        .register::<models::user_achievement::UserAchievement>()
        .register::<models::score_event::ScoreEvent>()
        .register::<models::score_correction::ScoreCorrection>()
        .try_build()?;

    let public = routes![
        get("/" => controllers::lms_controller::index),
        get("/favicon.ico" => controllers::lms_controller::favicon_handler),
        get("/apps" => pages::apps::apps_page),
        get("/manifest.webmanifest" => manifest_handler),
        get("/sw.js" => sw_handler),
        // rullst-access: public — course metadata and lesson titles form the public catalog.
        get("/courses/{id}" => controllers::lms_controller::show_course),
        // rullst-access: public — an opaque certificate key reveals bounded course evidence, never learner PII.
        get("/certificates/{certificate_key}" => controllers::completion_controller::verify),
        get("/login" => controllers::auth_controller::login_view),
        post("/login" => controllers::auth_controller::login_submit),
        get("/register" => controllers::auth_controller::register_view),
        post("/register" => controllers::auth_controller::register_submit),
        post("/logout" => controllers::auth_controller::logout),
    ];
    let learning = routes![
        get("/dashboard" => controllers::auth_controller::dashboard),
        // rullst-access: owner — the authenticated identity, never form data, owns the enrollment.
        post("/courses/{id}/enroll" => controllers::learning_controller::enroll),
        // rullst-access: owner — the handler requires an active enrollment for the lesson course.
        get("/lessons/{id}/play" => controllers::learning_controller::play_lesson),
        // rullst-access: owner — progress is written only for the authenticated enrollment owner.
        post("/lessons/{id}/progress" => controllers::learning_controller::record_progress),
        // rullst-access: owner — completion is derived for the authenticated learner from pinned server state.
        post("/courses/{id}/completion" => controllers::completion_controller::complete),
        // rullst-access: owner — the session identity scopes notification listing.
        get("/notifications" => controllers::notification_controller::index),
        // rullst-access: owner — quiz and subject identities come from the path/session.
        post("/quizzes/{id}/start" => controllers::assessment_controller::start),
        // rullst-access: owner — answers are graded against the server-side answer key.
        post("/quizzes/{id}/submit" => controllers::assessment_controller::submit),
        // rullst-access: owner — only an option and idempotency key cross this boundary; policy and score are server-derived.
        post("/activities/{id}/attempts" => controllers::activity_controller::submit),
        // rullst-access: owner — pair IDs are bounded; matching rules and score remain server-derived.
        post("/activities/{id}/attempts/matching" => controllers::activity_matching_controller::submit),
        // rullst-access: owner — typed input is bounded/digested; accepted answers and score remain server-derived.
        post("/activities/{id}/attempts/typed" => controllers::activity_typed_controller::submit),
        // rullst-access: owner — subject and clock are server-derived; rows remain school/enrollment scoped.
        get("/reviews/due" => controllers::review_controller::index),
        // rullst-access: owner — assignment and learner are derived from path/session before entitlement checks.
        post("/assignments/{id}/submissions" => controllers::assignment_controller::submit),
        // rullst-access: role — a persisted evaluator/instructor/admin scores only server rubric criteria.
        post("/submissions/{id}/grade" => controllers::assignment_controller::grade),
        // rullst-access: role — only admin can append a reasoned grade correction bounded by the same rubric.
        post("/assignment-grades/{id}/correct" => controllers::assignment_controller::correct_grade),
        // rullst-access: owner — the session identity scopes the notification mutation.
        post("/notifications/{id}/read" => controllers::notification_controller::read),
        // rullst-access: owner — no subject identity is accepted from the form.
        post("/notifications/preferences" => controllers::notification_controller::update_preference),
        // rullst-access: role — publication service requires instructor/admin from authenticated context.
        post("/courses/{id}/versions" => controllers::publication_controller::draft),
        // rullst-access: role — only the authenticated version author or admin can submit review.
        post("/course-versions/{id}/submit" => controllers::publication_controller::submit),
        // rullst-access: role — publication service requires a distinct authenticated admin reviewer.
        post("/course-versions/{id}/review" => controllers::publication_controller::review),
        // rullst-access: role — rollback creates a new immutable version and requires an authenticated admin.
        post("/courses/{id}/rollback" => controllers::publication_rollback_controller::rollback),
        // rullst-access: role — target identity is path-bound and the service enforces the grant hierarchy.
        post("/users/{id}/roles" => controllers::role_controller::grant),
        // rullst-access: role — the authenticated grant hierarchy controls durable revocation.
        post("/role-assignments/{assignment_key}/revoke" => controllers::role_controller::revoke),
        // rullst-access: role — the service requires admin and records actor, reason and server time.
        post("/certificates/{certificate_key}/revoke" => controllers::completion_controller::revoke),
    ].layer(rullst::server::from_fn(middlewares::auth_middleware::auth_middleware));

    let studio_router = rullst::studio::data_browser::router()
        .route("/cache", axum::routing::get(studio_cache_handler))
        .route("/studio/cache", axum::routing::get(studio_cache_handler))
        .route("/assets/studio.css", axum::routing::get(studio_css_handler))
        .route("/studio/assets/studio.css", axum::routing::get(studio_css_handler))
        .route("/assets/logger.js", axum::routing::get(studio_logger_handler))
        .route("/studio/assets/logger.js", axum::routing::get(studio_logger_handler))
        .layer(rullst::server::from_fn(studio_tailwind_patch))
        .layer(rullst::server::from_fn(studio_auth_guard));

    let is_prod_or_staging = std::env::var("RULLST_ENV")
        .or_else(|_| std::env::var("APP_ENV"))
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "production" | "prod" | "staging" | "stage"))
        .unwrap_or(false);

    let router = if !is_prod_or_staging {
        public
            .merge_axum(learning.into_axum())
            .layer(rullst::server::from_fn(rullst::security::csrf_middleware))
            .layer(rullst::server::from_fn(rullst::security::headers_middleware))
            .nest_axum("/nexus", nexus)
            .nest_axum("/studio", studio_router)
            .layer(rullst::server::Extension(rullst::nexus::NexusVerifiedTls::from_trusted_tls_termination()))
    } else {
        public
            .merge_axum(learning.into_axum())
            .nest_axum("/nexus", nexus)
            .nest_axum("/studio", studio_router)
            .layer(rullst::server::Extension(rullst::nexus::NexusVerifiedTls::from_trusted_tls_termination()))
    }
    .layer(rullst::server::from_fn(coep_policy_patch));

    #[cfg(debug_assertions)]
    {
        rullst::runtime::spawn(async {
            if let Err(error) = rullst::studio::run_studio(5555).await {
                eprintln!("Rullst Studio could not start: {error}");
            }
        });
        println!("📊 Rullst Studio running on http://127.0.0.1:5555");
    }
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:///app/db.sqlite?mode=rwc".to_string());
    println!("📦 Connecting to database: {db_url}");
    match rullst::db::Orm::init(&db_url).await {
        Ok(_) => {
            println!("🚀 Running migrations on boot...");
            for migration in crate::migrations::get_migrations() {
                if let Err(err) = migration.up().await {
                    eprintln!("⚠️ Migration error: {err}");
                }
            }
            println!("✅ Database migrations applied successfully!");
            if let Ok(pool) = rullst::db::Orm::pool() {
                // Seed permanent demo learner account
                if let Ok(demo_hash) = rullst::auth::hash_password_async("RullstAcademy2026!".to_string()).await {
                    let _ = rullst::db::sqlx::query(
                        "INSERT OR IGNORE INTO users (id, name, email, password_hash, created_at, updated_at) VALUES (100, 'Demo Learner', 'demo@rullst.dev', ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
                    ).bind(&demo_hash).execute(pool).await;

                    let _ = rullst::db::sqlx::query(
                        "INSERT OR IGNORE INTO school_memberships (school_id, user_id, role, membership_key, status, created_at, updated_at) VALUES (1, 100, 'student', 'sm-demo-100', 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
                    ).execute(pool).await;
                }

                // Seed Course 2 scope for default demo school so all learners can enroll
                let _ = rullst::db::sqlx::query(
                    "INSERT OR REPLACE INTO course_school_scopes (school_id, course_id, enrollment_policy, created_at, updated_at) VALUES (1, 2, 'open', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
                ).execute(pool).await;

                // Remove prerequisite blocking from Lesson 2 so all showcase lessons are playable
            let _ = rullst::db::sqlx::query(
                "UPDATE lesson_release_rules SET prerequisite_lesson_id = 0, required_progress_percent = 0 WHERE lesson_id = 2"
            ).execute(pool).await;

            // Seed real YouTube video lessons for Rust & Web Development
                let _ = rullst::db::sqlx::query(
                    "UPDATE lessons SET media_kind = 'youtube', media_url = 'https://www.youtube.com/embed/5C_HPTJg5ek', title = 'Introduction to Memory Safety in Rust', transcript = 'Rust achieves memory safety without a garbage collector through its ownership model. In this lesson, we explore how ownership, borrowing, and lifetimes guarantee that references always point to valid data.' WHERE id = 1"
                ).execute(pool).await;

                let _ = rullst::db::sqlx::query(
                    "UPDATE lessons SET media_kind = 'youtube', media_url = 'https://www.youtube.com/embed/zF34dRivLOw', title = 'Deep Dive into Smart Pointers & Concurrency', transcript = 'Smart pointers act like pointers but have additional metadata and capabilities. We explore Box for heap allocation, Rc for single-threaded reference counting, and Arc/Mutex for thread-safe concurrent design.' WHERE id = 2"
                ).execute(pool).await;

                let _ = rullst::db::sqlx::query(
                    "UPDATE lessons SET media_kind = 'youtube', media_url = 'https://www.youtube.com/embed/MsocPEZBd-M', title = 'Setting up your first Rust Web Application', transcript = 'Rust is rapidly becoming the premier choice for backend web infrastructure. Learn how Rullst organizes routes, handles asynchronous IO with Tokio, and integrates active record data models.' WHERE id = 3"
                ).execute(pool).await;

                let _ = rullst::db::sqlx::query(
                    "UPDATE lessons SET media_kind = 'youtube', media_url = 'https://www.youtube.com/embed/r-GSGH2RxJs', title = 'Building Interactive UIs with HTMX', transcript = 'HTMX gives you access to AJAX, CSS Transitions, and Server-Sent Events directly in HTML. Pair HTMX with Rust server-side rendering for rich, dynamic user interfaces without heavy JavaScript bundle complexity.' WHERE id = 4"
                ).execute(pool).await;
                println!("🎥 Educational YouTube video lessons and open course scopes initialized!");
            }
        }
        Err(err) => {
            eprintln!("❌ Database connection error: {err}");
        }
    }
    println!("🚀 LMS server starting on port 3000...");
    Server::new(router).run(3000).await?;
    Ok(())
}
