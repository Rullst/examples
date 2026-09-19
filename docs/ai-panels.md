# AI in the deployed examples

The Showcase, Portfolio and LMS share `crates/blueprint-ai`. Their public chats
render Markdown on the server using pulldown-cmark, then sanitize the result
with an explicit Ammonia allowlist. Raw HTML is escaped; scripts, images,
event handlers, styles and HTMX attributes from model output are not permitted.
Lists, headings, emphasis, links, tables and code blocks have scoped styles.

Each application's existing `/nexus/chat` and `/studio/ai` navigation now opens
an application-aware assistant. No floating launcher is injected into native
Nexus or Studio pages. The administrative interface is English-only; model
replies use only the language of the latest user question and are not repeated
in a second language unless translation was requested. The same Nexus
authentication/role/TLS policy protects both panels and all AI endpoints.
Production administrators must supply
`NEXUS_ADMIN_PASSWORD` (at least 16 characters); there is no password fallback.
Earlier versions published demo administrator passwords. Removing them from
the UI and source does not rotate existing container secrets; deployments that
still use those published values need a separate credential rotation.

Configure `GROQ_API_KEY` through the existing container secret/environment
configuration. `GROQ_MODEL` defaults to `openai/gpt-oss-120b`; `GROQ_BASE_URL`
defaults to `https://api.groq.com/openai/v1`. Credentials remain server-side.
All nine assistants use this same configuration in their own container.

## Administrative scope

Nexus assists with content workflows and drafts. Studio assists with developer
questions and diagnostics. They receive only a fixed, public description of
their blueprint and the submitted question. They do not receive records,
learner information, grades, credentials, logs or live metrics. They cannot
run SQL, execute commands, modify content or deploy changes. The UI explains
what is sent to the AI provider and asks operators not to submit private data.

The admin POST endpoint requires authentication, the framework's double-submit
CSRF cookie/header proof, a same-origin Fetch Metadata value when supplied, and
a custom request header. Input is bounded to 1,200 characters / 8 KiB encoded body.
The provider has a 30-second deadline, four concurrent calls and 30 calls per
minute per application process, shared by public and admin assistants. These
are process limits, not a distributed quota; multiple replicas multiply them.
Conversations stay in the current page, with no browser persistence or server
history. Provider failures return generic messages, never upstream error bodies.

## Build and verification

Run from the repository root:

```sh
cargo test --manifest-path crates/blueprint-ai/Cargo.toml
cargo test --all-targets
cargo test --manifest-path blueprints/portfolio/Cargo.toml
cargo test --manifest-path blueprints/lms/Cargo.toml
node --check crates/blueprint-ai/static/admin.js
docker build -f blueprints/portfolio/Dockerfile .
docker build -f blueprints/lms/Dockerfile .
```

The blueprint Docker builds now use the repository root as their context to
include the shared crate. The deployment workflows watch shared-crate changes.
Updating this code locally does not itself update the Azure applications.

The container's dependency-cooking stage must copy `crates/blueprint-ai` before
running `cargo chef cook`. This crate has its own workspace and is not included
in the application's cargo-chef recipe. Omitting it caused all three image
builds for commit `0ef2434` to fail before deployment on 2026-09-16.

Deployment now fails if Azure authentication or required application settings
are missing. `scripts/verify-deployment.py` checks settings before updating the
image, then checks the expected ready revision and the actual public URLs.
The live checks cover formatted public inference, authenticated Nexus/Studio
pages and inference, anonymous-access rejection and cross-origin rejection.
A headless Chromium check also submits both admin forms through the actual page
JavaScript, so simulated HTTP headers cannot hide a browser-only denial. The
checks make five short AI calls per application; no private records are sent.
Credentials are read only inside the deployment runner and sent over HTTPS to
the fixed blueprint hostname. Redirects are not followed. LMS credentials stay
private and are never printed. Showcase and Portfolio intentionally use the
public sandbox login documented on their home pages; their workflows keep the
Azure configuration synchronized with that non-secret demo credential.

## Framework finding: Groq provider detection in v12.0.0

In the published `rullst-nexus` 12.0.0 source,
`src/nexus/ai_chat.rs::detect_ai_provider` checks Gemini, OpenAI, Anthropic,
Ollama and `OPENAI_BASE_URL`, but not `GROQ_API_KEY`. The native chat uses this
result before calling `rullst_ai::AiClient::auto()`. The latter also does not
select Groq from that variable. As a result, configuring only the Groq key
used by these examples does not connect the native Nexus assistant.

This is a missing automatic integration, not an authentication bypass. These
examples address it using an explicitly configured compatible provider. No
published framework package or local Cargo registry source is modified.

Studio's `src/ai_playground.rs::render_ai_playground_html` deliberately renders
an unconnected integration page in v12.0.0. Its own source asks the application
to supply an authenticated, authorized, rate-limited endpoint. This is an
explicit framework limitation, not a broken provider call. This change supplies
that application integration on the existing Studio AI route.

The literal Markdown defect was in the examples' own chat handlers:
`escape_str(reply).replace("\n", "<br/>")` displayed Markdown syntax as text.
The previous hand-written Studio authentication and default-password behavior
were also in the examples, not the framework policy now used by them.
