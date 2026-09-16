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
