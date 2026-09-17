# Rullst framework findings

Investigated on 2026-09-16 against the published **12.0.0** crate sources.
This report separates framework findings from defects in this examples
repository. No framework crate or Cargo registry source was modified.

## RULLST-001 — Nexus stays offline when only Groq is configured

**Classification:** missing provider integration / configuration mismatch.
**Affected packages:** `rullst-nexus` and `rullst-ai` 12.0.0.
**Impact:** a blueprint's public assistant can use Groq successfully while the
native Nexus assistant reports offline mode in the same application.
**Evidence status:** confirmed by inspection of the published crate sources;
this report does not claim a completed authenticated reproduction on Azure.

### Cause and evidence

In `rullst-nexus/src/nexus/ai_chat.rs`, `detect_ai_provider()` checks these
environment variables:

- `GEMINI_API_KEY`
- `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`
- `OLLAMA_HOST`
- `OPENAI_BASE_URL`

It does not check `GROQ_API_KEY`. When none of the listed variables is present,
it returns `(false, "Offline Embedded Intelligence")`.

In the same file, `nexus_chat_query()` uses that boolean to decide whether to
call `rullst_ai::AiClient::auto()` or return the offline response. Consequently,
a valid Groq key alone never reaches the provider selection step.

There is a second integration gap:
`rullst-ai/src/ai/client.rs::AiClient::auto()` does not select a Groq provider
from `GROQ_API_KEY` either. Adding Groq only to Nexus's detection function
would therefore be insufficient.

The examples' public assistants were constructing
`OpenAiCompatibleProvider::try_cloud()` explicitly with the Groq endpoint,
key and model. That different configuration path explains why the public
chat worked while Nexus did not. This is not evidence of invalid credentials
or a Groq outage.

### Reproduction

1. Run a v12.0.0 application that mounts the native Nexus chat.
2. Configure a valid server-side `GROQ_API_KEY`, with `GROQ_MODEL` set to the
   intended Groq model. Leave the five variables listed above unset.
3. Confirm that an explicitly configured Groq-compatible client can answer a
   question in the application.
4. Sign in to Nexus and open `/nexus/chat`.
5. Submit a normal question through `/nexus/chat/query`.

**Actual result, as determined by the implementation:** Nexus selects its
offline response branch despite the application's configured Groq integration.

**Expected integration behavior:** Nexus should accept the application's
explicitly configured AI client, or use the same supported provider resolver
as the public assistant. Merely detecting an environment variable should not
be described as proof that the provider is reachable.

### Suggested framework correction

Prefer an explicit Nexus builder hook for an application-supplied `AiClient`,
so provider configuration is not independently inferred by each panel.
Alternatively, centralize provider selection and implement Groq consistently
in both configuration detection and `AiClient::auto()`.

Keep administrator authentication and authorization in front of inference.
Preserve guardrails, bounded input/output, timeouts, usage limits, sanitized
rendering and generic provider-error responses. Never send credentials or
private database records to the model as part of automatic panel context.

Suggested regression coverage:

- Groq-only configuration selects the intended compatible provider.
- Missing credentials produce an explicitly offline/unconfigured state.
- A configured but unreachable provider produces a generic unavailable state.
- Unauthenticated and cross-origin requests cannot trigger inference.
- Provider output containing Markdown and hostile HTML renders safely.

### Application workaround implemented here

`crates/blueprint-ai` explicitly creates the Groq-compatible client and shares
that configuration between the public chats and the application-owned Nexus
and Studio assistants. The panel routes are protected by
`NexusAuthPolicy::protect_router()`. This avoids changing a published framework
package and does not imply that RULLST-001 has been fixed upstream.

## Related observation — Studio intentionally has no connected AI client

`rullst-studio/src/ai_playground.rs::render_ai_playground_html()` deliberately
renders an informational integration page in 12.0.0. Its text says that no AI
client is connected and asks applications to expose their own authenticated,
authorized and rate-limited endpoint. The corresponding handler is
`src/data_browser/handlers/ai.rs::handle_studio_tools_ai()`.

This is an explicit framework limitation, **not a failed inference request or
an accidental provider regression**. The examples needed an application-level
integration. The new assistant supplies it at the existing `/studio/ai` URL.

## Not framework defects

- The bilingual English/Portuguese administrative copy and floating Nexus/Studio
  AI launcher were added by `crates/blueprint-ai`; they are not rendered by the
  published framework. The examples now use English-only panel chrome, inject no
  floating launcher, and instruct the model to reply only in the question's
  language (any language), without a second-language duplicate.
- The admin assistant's browser-visible `Access denied` response came from the
  examples' application-owned request boundary and error mapping, not from the
  native v12 AI implementation. Its strict Origin/Host comparison was unsuitable
  behind the deployed proxy. The integration now uses Rullst's double-submit CSRF
  middleware plus Fetch Metadata and a custom header, and deployment verification
  exercises the real JavaScript flow in headless Chromium.
- Missing `blueprint-ai` during the container dependency-cooking stage: all
  three GitHub image builds for commit `0ef2434` failed on 2026-09-16 with
  `failed to read /app/crates/blueprint-ai/Cargo.toml`. Tests passed, but Azure
  deployment never ran. This is an examples-level packaging regression, not
  a framework defect; the standalone path dependency must be copied before
  `cargo chef cook`. This explains why the online sites still showed old UI.
- Literal `**bold**` and `### headings` in public chat: the examples escaped
  model output and replaced newlines with `<br/>`, without parsing Markdown.
- Public display of upstream provider error details: implemented by the
  examples' chat handlers.
- Hand-written Studio authentication and fallback administrator passwords:
  implemented by the examples, not by the Nexus policy now used for the panels.
- Unescaped database fields in offline LMS/Portfolio chat templates: an
  examples-level HTML-injection risk. All offline fragments now pass through
  the shared sanitizer; the LMS controller also escapes catalog text.

Deployment and verification status are reported separately; this document
records source findings and must not be read as proof that Azure is updated.

## RULLST-003 — Shield blocks legitimate machine clients by default

**Classification:** default WAF false positive / operational availability.
**Affected package:** `rullst-core` 12.0.0.
**Evidence status:** confirmed from the published source and an HTTPS
reproduction against the deployed SaaS staging application on 2026-09-17.

### Cause and evidence

`rullst-core/src/config.rs::default_user_agent_blocklist()` includes generic
HTTP clients such as `curl`, `wget`, `python-requests` and `go-http-client`.
`rullst-core/src/security/waf.rs::waf_middleware()` applies substring matching
to the `User-Agent` header before routing the request. A match returns HTTP 403
with `Access Denied: Suspicious User-Agent blocked by Rullst Shield WAF.`

The production-mode SaaS application exposes `GET /healthz` for deployment and
database-readiness verification. Its GitHub Actions deployment used `curl` and
received this framework-generated 403 even though the Azure revision was
provisioned, healthy and running. The WAF therefore caused the deployment to
be reported as unavailable after a successful application start.

Blocking a generic client identifier is also not a reliable bot boundary: an
abusive client can send a different `User-Agent`, while legitimate probes,
API consumers and command-line diagnostics are denied by default.

### Reproduction

1. Start a v12.0.0 server with the default security configuration and a public
   `GET /healthz` route.
2. Send `curl https://application.example/healthz`.
3. Observe HTTP 403 and the Rullst Shield suspicious-agent response.
4. Repeat the same request with a non-blocklisted `User-Agent` and observe that
   the request reaches the route.

**Expected behavior:** default security policy should not classify standard
HTTP libraries as malicious solely from a forgeable header. Applications must
be able to expose narrowly scoped health and API routes to authenticated or
rate-limited machine clients without disabling unrelated WAF protections.

### Suggested framework correction

Remove generic HTTP libraries from the default blocklist. Keep known crawler
policy configurable, and use route-aware authentication, authorization, rate
limits, request validation and behavioral controls for machine traffic. If a
route exemption mechanism is introduced, require exact paths and methods and
do not exempt request-body or signature validation for sensitive endpoints.

Suggested regression coverage:

- default `curl`, `wget`, Python and Go clients can reach a benign GET route;
- configured crawler entries are still rejected case-insensitively;
- health-route access does not weaken CSRF, webhook-signature or body checks;
- an application override can add and remove exact blocklist entries.

### Application workaround implemented here

The SaaS deployment probe now sends the explicit
`Rullst-SaaS-Deployment-Healthcheck/1.0` identifier. This restores deployment
verification without disabling Shield, but it does not fix the overly broad
framework default or turn `User-Agent` matching into an authentication control.

## RULLST-002 — SaaS/Capital live-payment contract defects

The v12.0.0 SaaS generator and Capital adapters have confirmed payment defects
that should not be collapsed into a broad claim of "11 supported gateways."
They include a hard-coded Lemon Squeezy store ID, stale Paddle and Polar
checkout contracts, insufficient Stripe checkout-to-local-user binding, an
HTTP 307 checkout handoff, a generated live portal route backed only by an
unsupported operation, an invalid Wise transfer flow, missing checkout
idempotency, provider-less generated billing records and non-atomic generated
webhook persistence. The generated Windows Cargo configuration also forces the
now-unsupported `/DEBUG:FASTLINK` option and triggers linker warning LNK4315.
The deployable binary scaffold also ignores `Cargo.lock` and builds its
container without `--locked`, so dependency resolution is not reproducible.

The complete evidence, capability boundaries, low-value test constraints and
acceptance criteria are maintained in
[`saas-improvements-needed.md`](saas-improvements-needed.md), findings
**SAAS-001** through **SAAS-015**. This now also records that the v12
`strict-postgres` feature graph still activates the default ORM backend set,
including SQLite and MySQL, instead of remaining backend-exclusive, and that
the hosted Stripe checkout API has no one-time purchase mode. It also records
that the Stripe signature verifier checks only the last `v1` value, which can
reject a valid event while endpoint secrets overlap during rotation. The
hardened example under `blueprints/saas` contains application workarounds and
does not mean the published framework defects have been fixed upstream.
