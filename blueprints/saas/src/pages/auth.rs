use rullst::response::Html;

pub fn login_page(csrf_token: &str, error: Option<&str>, csp_nonce: &str) -> Html<String> {
    let error_html = if let Some(err) = error {
        format!(
            "<div class=\"error\" role=\"alert\">{}</div>",
            rullst::html::escape_str(err)
        )
    } else {
        String::new()
    };

    let live_mode = crate::controllers::legal_controller::live_mode();
    let terms_label = if live_mode {
        "Purchase terms"
    } else if crate::controllers::legal_controller::production_deployment() {
        "Pre-launch terms"
    } else {
        "Sandbox terms"
    };
    let document = format!(
        "<!DOCTYPE html><html lang=\"en\" class=\"dark\"><head>\
         <meta charset=\"utf-8\" />\
         <link rel=\"icon\" type=\"image/png\" href=\"/static/rullst.png\" />\
         <title>Login &mdash; Rullst SaaS</title>\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\
         <style nonce=\"{}\">\
         * {{ box-sizing: border-box; margin: 0; padding: 0; font-family: system-ui, sans-serif; }}\
         body {{ background: #0b0f19; color: #f3f4f6; min-height: 100vh; display: flex; align-items: center; justify-content: center; }}\
         .card {{ background: rgba(15, 23, 42, 0.6); backdrop-filter: blur(12px); border: 1px solid rgba(255,255,255,0.08); border-radius: 1.5rem; padding: 2.5rem; width: 100%; max-width: 420px; text-align: center; }}\
         h1 {{ font-size: 2rem; margin-bottom: 1.5rem; font-weight: 700; }}\
         .form-group {{ margin-bottom: 1.25rem; text-align: left; }}\
         label {{ display: block; font-size: 0.85rem; color: #9ca3af; margin-bottom: 0.4rem; }}\
         input {{ width: 100%; padding: 0.75rem 1rem; border-radius: 0.5rem; background: rgba(0,0,0,0.3); border: 1px solid rgba(255,255,255,0.1); color: #fff; font-size: 0.95rem; }}\
         input:focus {{ outline: none; border-color: #10b981; }}\
         .btn-primary {{ width: 100%; padding: 0.85rem; border-radius: 0.5rem; background: #10b981; color: #000; font-weight: 700; border: none; cursor: pointer; font-size: 1rem; margin-top: 0.5rem; }}\
         .btn-primary:hover {{ background: #34d399; }}\
         .links {{ margin-top: 1.5rem; font-size: 0.85rem; line-height: 1.8; color: #9ca3af; }}\
         .links a {{ color: #10b981; text-decoration: none; }}\
         .error {{ background: rgba(239,68,68,.1); border: 1px solid rgba(239,68,68,.2); color: #f87171; padding: .75rem 1rem; border-radius: .5rem; margin-bottom: 1.5rem; font-size: .9rem; }}\
         </style></head><body>\
         <div class=\"card\"><h1>Welcome Back</h1>{}\
         <form method=\"POST\" action=\"/login\">\
         <input type=\"hidden\" name=\"_token\" value=\"{}\" />\
         <div class=\"form-group\"><label>Email</label><input type=\"email\" name=\"email\" placeholder=\"you@example.com\" required /></div>\
         <div class=\"form-group\"><label>Password</label><input type=\"password\" name=\"password\" placeholder=\"••••••••\" required /></div>\
         <button type=\"submit\" class=\"btn-primary\">Sign In</button>\
         </form>\
         <div class=\"links\">Don't have an account? <a href=\"/register\">Register</a> | <a href=\"/\">Pricing</a><br /><a href=\"/privacy\">Privacy</a> | <a href=\"/terms\">__RULLST_TERMS_LABEL__</a></div>\
         </div></body></html>",
        rullst::html::escape_str(csp_nonce),
        error_html,
        rullst::html::escape_str(csrf_token)
    );
    Html(document.replace("__RULLST_TERMS_LABEL__", terms_label))
}

pub fn register_page(csrf_token: &str, error: Option<&str>, csp_nonce: &str) -> Html<String> {
    let error_html = if let Some(err) = error {
        format!(
            "<div class=\"error\" role=\"alert\">{}</div>",
            rullst::html::escape_str(err)
        )
    } else {
        String::new()
    };

    let live_mode = crate::controllers::legal_controller::live_mode();
    let production_prelaunch =
        !live_mode && crate::controllers::legal_controller::production_deployment();
    let terms_label = if live_mode {
        "Purchase terms"
    } else if production_prelaunch {
        "Pre-launch terms"
    } else {
        "Sandbox terms"
    };
    let account_label = if live_mode {
        "production account"
    } else if production_prelaunch {
        "production pre-launch account"
    } else {
        "sandbox account"
    };
    let document = format!(
        "<!DOCTYPE html><html lang=\"en\" class=\"dark\"><head>\
         <meta charset=\"utf-8\" />\
         <link rel=\"icon\" type=\"image/png\" href=\"/static/rullst.png\" />\
         <title>Register &mdash; Rullst SaaS</title>\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\
         <style nonce=\"{}\">\
         * {{ box-sizing: border-box; margin: 0; padding: 0; font-family: system-ui, sans-serif; }}\
         body {{ background: #0b0f19; color: #f3f4f6; min-height: 100vh; display: flex; align-items: center; justify-content: center; }}\
         .card {{ background: rgba(15, 23, 42, 0.6); backdrop-filter: blur(12px); border: 1px solid rgba(255,255,255,0.08); border-radius: 1.5rem; padding: 2.5rem; width: 100%; max-width: 420px; text-align: center; }}\
         h1 {{ font-size: 2rem; margin-bottom: 1.5rem; font-weight: 700; }}\
         .form-group {{ margin-bottom: 1.25rem; text-align: left; }}\
         label {{ display: block; font-size: 0.85rem; color: #9ca3af; margin-bottom: 0.4rem; }}\
         input {{ width: 100%; padding: 0.75rem 1rem; border-radius: 0.5rem; background: rgba(0,0,0,0.3); border: 1px solid rgba(255,255,255,0.1); color: #fff; font-size: 0.95rem; }}\
         input:focus {{ outline: none; border-color: #10b981; }}\
         .btn-primary {{ width: 100%; padding: 0.85rem; border-radius: 0.5rem; background: #10b981; color: #000; font-weight: 700; border: none; cursor: pointer; font-size: 1rem; margin-top: 0.5rem; }}\
         .btn-primary:hover {{ background: #34d399; }}\
         .links {{ margin-top: 1.5rem; font-size: 0.85rem; line-height: 1.8; color: #9ca3af; }}\
         .links a {{ color: #10b981; text-decoration: none; }}\
         .error {{ background: rgba(239,68,68,.1); border: 1px solid rgba(239,68,68,.2); color: #f87171; padding: .75rem 1rem; border-radius: .5rem; margin-bottom: 1.5rem; font-size: .9rem; }}\
         </style></head><body>\
         <div class=\"card\"><h1>Create Account</h1>{}\
         <form method=\"POST\" action=\"/register\">\
         <input type=\"hidden\" name=\"_token\" value=\"{}\" />\
         <div class=\"form-group\"><label>Name</label><input type=\"text\" name=\"name\" placeholder=\"John Doe\" required /></div>\
         <div class=\"form-group\"><label>Email</label><input type=\"email\" name=\"email\" placeholder=\"you@example.com\" required /></div>\
         <div class=\"form-group\"><label>Password</label><input type=\"password\" name=\"password\" placeholder=\"••••••••\" required /></div>\
         <p class=\"links\">By creating this __RULLST_ACCOUNT_LABEL__, you acknowledge the <a href=\"/privacy\">Privacy notice</a> and <a href=\"/terms\">__RULLST_TERMS_LABEL__</a>. This is not optional marketing consent.</p>\
         <button type=\"submit\" class=\"btn-primary\">Register</button>\
         </form>\
         <div class=\"links\">Already have an account? <a href=\"/login\">Sign In</a> | <a href=\"/\">Pricing</a><br /><a href=\"/privacy\">Privacy</a> | <a href=\"/terms\">__RULLST_TERMS_LABEL__</a></div>\
         </div></body></html>",
        rullst::html::escape_str(csp_nonce),
        error_html,
        rullst::html::escape_str(csrf_token)
    );
    Html(
        document
            .replace("__RULLST_TERMS_LABEL__", terms_label)
            .replace("__RULLST_ACCOUNT_LABEL__", account_label),
    )
}

pub fn dashboard_page(
    user_name: &str,
    csrf_token: &str,
    csp_nonce: &str,
    has_stripe_report: bool,
    certificate_public_id: Option<&str>,
    live_mode: bool,
    refund_status: Option<&str>,
) -> Html<String> {
    let production_prelaunch =
        !live_mode && crate::controllers::legal_controller::production_deployment();
    let nonce = rullst::html::escape_str(csp_nonce);
    let user_name = rullst::html::escape_str(user_name);
    let csrf_token = rullst::html::escape_str(csrf_token);
    let report_action = if has_stripe_report {
        if live_mode {
            "<a class=\"btn-report\" href=\"/reports/stripe-gateway-field-report-v1.md\">Download purchased guide</a>"
        } else {
            "<a class=\"btn-report\" href=\"/reports/stripe-gateway-field-report-v1.md\">Download Stripe report</a>"
        }
    } else if live_mode {
        "<a class=\"btn-report\" href=\"/pricing\">Open one-time checkout</a>"
    } else if production_prelaunch {
        "<a class=\"btn-report\" href=\"/pricing\">View production launch status</a>"
    } else {
        "<a class=\"btn-report\" href=\"/pricing\">Open sandbox checkout</a>"
    };
    let certificate_action = certificate_public_id.map_or_else(
        || {
            if has_stripe_report {
                "<p class=\"muted small\">Certificate issuance is still being reconciled.</p>"
                    .to_owned()
            } else if production_prelaunch {
                "<p class=\"muted small\">Production Checkout is disabled; no purchase certificate can be issued yet.</p>"
                    .to_owned()
            } else {
                "<p class=\"muted small\">Complete the verified sandbox checkout to receive a privacy-preserving tester certificate.</p>"
                    .to_owned()
            }
        },
        |public_id| {
            format!(
                "<a class=\"btn-report btn-certificate\" href=\"/certificate\">View {} certificate</a><p class=\"muted small\">Public verification ID: {}</p>",
                if public_id.starts_with("RST-LIVE-") { "Founding Customer" } else { "Sandbox Pioneer" },
                rullst::html::escape_str(public_id)
            )
        },
    );
    let refund_action = if live_mode && has_stripe_report {
        match refund_status {
            Some("requested" | "processing") => {
                "<a class=\"btn-report btn-refund\" href=\"/refund\">Refund requested</a>"
                    .to_owned()
            }
            Some("completed") => {
                "<p class=\"muted small\">Stripe confirmed the refund.</p>".to_owned()
            }
            _ => {
                "<a class=\"btn-report btn-refund\" href=\"/refund\">Request refund</a>".to_owned()
            }
        }
    } else {
        String::new()
    };
    let terms_label = if live_mode {
        "Purchase terms"
    } else if production_prelaunch {
        "Pre-launch terms"
    } else {
        "Sandbox terms"
    };
    let entitlement_description = if live_mode {
        "Access is granted only after a verified live Stripe event. Refunds and disputes revoke access after provider confirmation."
    } else if production_prelaunch {
        "Real-money Checkout is disabled during production readiness validation."
    } else {
        "Access is granted only after a verified, replay-protected provider test event."
    };
    Html(r#"<!DOCTYPE html><html lang="en" class="dark"><head>
         <meta charset="utf-8" />
         <link rel="icon" type="image/png" href="/static/rullst.png" />
         <title>Dashboard — Rullst SaaS</title>
         <meta name="viewport" content="width=device-width, initial-scale=1.0" />
         <style nonce="__RULLST_CSP_NONCE__">
         * { box-sizing: border-box; margin: 0; padding: 0; font-family: system-ui, sans-serif; }
         body { background: #0b0f19; color: #f3f4f6; min-height: 100vh; padding: 2rem; }
         .topbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: 3rem; max-width: 1200px; margin: 0 auto 3rem auto; }
         .topbar-actions { display: flex; align-items: center; gap: 0.75rem; }
         .logo { font-size: 1.5rem; font-weight: 800; color: #10b981; }
         .container { max-width: 1200px; margin: 0 auto; }
         .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1.5rem; margin-top: 2rem; }
         .card { background: rgba(15, 23, 42, 0.6); backdrop-filter: blur(12px); border: 1px solid rgba(255,255,255,0.08); border-radius: 1rem; padding: 2rem; }
         .btn-logout { background: #ef4444; color: white; padding: 0.5rem 1rem; border-radius: 0.5rem; font-weight: 600; font-size: 0.9rem; border: 0; cursor: pointer; }
         .logout-form { display: inline; }
         .btn-nexus { background: #1e293b; color: white; padding: 0.5rem 1rem; border-radius: 0.5rem; text-decoration: none; font-weight: 600; font-size: 0.9rem; border: 1px solid #374151; }
         .btn-report { display: inline-block; margin-top: 1rem; background: #10b981; color: #03120c; padding: 0.65rem 0.9rem; border-radius: 0.5rem; text-decoration: none; font-weight: 750; }
         .btn-certificate { margin-left: 0.5rem; background: #f97316; color: #fff; }
         .muted { color: #9ca3af; margin-top: 0.5rem; }
         .small { font-size: 0.85rem; }
         .metric { font-size: 1.5rem; font-weight: 700; }
         .card h3 { margin-bottom: 0.5rem; }
         .subscription { color: #10b981; }
         .performance { color: #38bdf8; }
         .security { color: #a855f7; }
         @media (max-width: 700px) { body { padding: 1rem; } .topbar { align-items: stretch; flex-direction: column; gap: 1rem; margin-bottom: 2rem; } .topbar-actions { align-items: stretch; flex-direction: column; } .btn-nexus, .btn-logout, .btn-report { display: block; width: 100%; text-align: center; } .btn-certificate { margin-left: 0; } .logout-form { display: block; } .card { padding: 1.25rem; } }
         </style></head><body>
         <div class="topbar">
           <div class="logo">⚡ Rullst SaaS Dashboard</div>
           <div class="topbar-actions">
             <a href="/privacy" class="btn-nexus">Privacy</a>
             <a href="/account/data-export" class="btn-nexus">Export my data</a>
             <a href="/terms" class="btn-nexus">__RULLST_TERMS_LABEL__</a>
             <a href="/nexus" class="btn-nexus">⚙️ Nexus CMS</a>
             <form method="post" action="/logout" class="logout-form"><input type="hidden" name="_token" value="__RULLST_CSRF_TOKEN__" /><button type="submit" class="btn-logout">Logout</button></form>
           </div>
         </div>
         <div class="container">
           <h1>Welcome, __RULLST_USER_NAME__</h1>
           <p class="muted">This starter authenticates passwords with Argon2id and stores the user ID in an encrypted session cookie.</p>
           <div class="grid">
             <div class="card">
               <h3 class="subscription">💳 Stripe report</h3>
               <p class="metric">Webhook-derived access</p>
               <p class="muted small">__RULLST_ENTITLEMENT_DESCRIPTION__</p>
               __RULLST_REPORT_ACTION__
               __RULLST_CERTIFICATE_ACTION__
               __RULLST_REFUND_ACTION__
             </div>
             <div class="card">
               <h3 class="performance">⚡ Performance</h3>
               <p class="metric">Server-rendered UI</p>
               <p class="muted small">Measure latency in your own deployment; this starter makes no universal timing claim.</p>
             </div>
             <div class="card">
               <h3 class="security">🛡️ Security Guard</h3>
               <p class="metric">CSRF + secure headers</p>
               <p class="muted small">Production still requires TLS termination, secret management and provider sandbox validation.</p>
             </div>
           </div>
          </div></body></html>"#
        .replace("__RULLST_CSP_NONCE__", &nonce)
        .replace("__RULLST_CSRF_TOKEN__", &csrf_token)
        .replace("__RULLST_REPORT_ACTION__", report_action)
        .replace("__RULLST_CERTIFICATE_ACTION__", &certificate_action)
        .replace("__RULLST_REFUND_ACTION__", &refund_action)
        .replace("__RULLST_TERMS_LABEL__", terms_label)
        .replace(
            "__RULLST_ENTITLEMENT_DESCRIPTION__",
            entitlement_description,
        )
        .replace("__RULLST_USER_NAME__", &user_name))
}
