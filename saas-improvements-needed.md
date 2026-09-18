# Rullst SaaS and Capital improvements needed

Investigated on 2026-09-16 against the published `cargo-rullst` and
`rullst-capital` **12.0.0** crate sources. Provider documentation was checked
against the public contracts available on the same date.

This report distinguishes confirmed implementation defects from deliberate
capability boundaries and recommended improvements. An exported adapter is not
automatically evidence that every live operation is implemented or accepted by
a provider account.

## Executive summary

Rullst Capital 12.0.0 exports **11 adapters**:

- 10 incoming billing adapters: Stripe, Lemon Squeezy, InfinitePay, Polar,
  Paddle, Alipay, Mercado Pago, Razorpay, Coinbase Commerce and PicPay.
- 1 outgoing payout adapter: Wise.

The crate's own live-method matrix reports a checkout request implementation
for Stripe, Lemon Squeezy, Paddle, Polar and Razorpay. Mercado Pago, Coinbase
Commerce, InfinitePay and PicPay deliberately reject live plan-only checkout.
Alipay deliberately rejects live checkout because RSA2 signing is not
implemented. Wise is not a customer checkout gateway.

Source and current-provider-contract review found that the Lemon Squeezy,
Paddle and Polar checkout implementations require corrections before they
should be used for real money. Stripe and Razorpay are therefore the only
checkout transports enabled by the audited example application, and both still
require provider sandbox evidence, approved accounts, live credentials,
server-owned prices and verified webhooks.

## Confirmed v12 defects

### SAAS-001 — Lemon Squeezy checkout hard-codes store ID `1`

**Affected package:** `rullst-capital` 12.0.0  
**Source:** `src/providers/lemonsqueezy.rs`, checkout payload construction.

The adapter accepts an API key, webhook secret and variant ID, but it always
sends the following store relationship:

```json
{
  "type": "stores",
  "id": "1"
}
```

The Lemon Squeezy checkout API requires the merchant's actual store ID. Most
accounts do not own store ID `1`, so a correctly configured API key and variant
can still produce a rejected or incorrectly bound request.

**Required correction:** accept and validate a store ID in
`LemonSqueezyProvider`, include it in the typed checkout request and bind the
returned checkout to both the requested store and variant. Add non-network
protocol fixtures plus sandbox acceptance tests.

### SAAS-002 — Paddle checkout request does not match the current transaction contract

**Affected package:** `rullst-capital` 12.0.0  
**Source:** `src/providers/paddle.rs`, `create_checkout_session()`.

The adapter posts a nested `customer: { email }` value and a top-level
`return_url` to `POST /transactions`. Current Paddle Billing transaction
creation uses transaction fields such as `customer_id`, `address_id`,
`collection_mode` and `checkout.url`; customer-email prefill belongs to the
Paddle Checkout boundary. Live accounts also require an approved default
payment-link page containing Paddle.js.

The response parser expects `data.checkout.url`, but the request does not
establish the current prerequisites that make a checkout URL usable.

**Required correction:** implement the current Paddle transaction and hosted
checkout flow, including customer handling, approved payment-link semantics,
response binding and sandbox tests. Do not treat a returned URL alone as proof
of a chargeable checkout.

### SAAS-003 — Polar checkout uses an obsolete request shape

**Affected package:** `rullst-capital` 12.0.0  
**Source:** `src/providers/polar.rs`, `create_checkout_session()`.

The adapter sends `product_price_id`, `customer_email` and `success_url` to
`POST /v1/checkouts/custom/`. The current Polar checkout contract creates a
session from one or more product IDs through the current checkouts API. It can
also accept an external customer identity and customer IP address for reliable
reconciliation and localized currency selection.

**Required correction:** migrate to the current products-based checkout
contract, bind the external application customer, forward a trusted client IP
only through a reviewed proxy boundary, and update response/webhook fixtures.

### SAAS-004 — Stripe subscription webhooks cannot reliably bind a generated checkout to a local user

**Affected packages:** `rullst-capital` and the generated SaaS blueprint 12.0.0.  
**Sources:** `rullst-capital/src/providers/stripe.rs` and generated
`src/controllers/billing_controller.rs`.

The Stripe adapter accepts only `customer.subscription.*` events. A Stripe
subscription object normally contains a provider customer ID, but not the
customer email fields read by the adapter. The generated SaaS webhook handler
then rejects events whose normalized `customer_email` is empty.

The checkout request also omits a local owner identifier from
`client_reference_id`, Checkout Session metadata and subscription metadata.
Consequently, a signed real subscription event may be valid but still cannot
be authoritatively associated with the authenticated local account.

**Required correction:** bind an opaque local billing-subject ID when creating
checkout, persist the Checkout Session ID, accept the relevant signed Checkout
and subscription lifecycle events, and reconcile customer/subscription IDs
without trusting mutable email as the sole owner key. If customer lookup is
needed, perform it server-side with bounded responses and bind it to persisted
checkout evidence.

### SAAS-005 — Generated POST checkout uses a 307 redirect

**Affected package:** `cargo-rullst` 12.0.0 SaaS blueprint.  
**Source:** generated `checkout_redirect()`.

The generated handler returns `Redirect::temporary`, which is HTTP 307. A 307
preserves the original HTTP method and body, so a browser can repeat the local
form POST against the provider checkout URL. Hosted checkout handoff should use
HTTP 303 so the external navigation becomes a GET.

**Required correction:** use `Redirect::to`/HTTP 303 after successful checkout
creation and add a route test asserting both status and `Location` semantics.

### SAAS-006 — Generated live customer-portal route calls an explicitly unsupported operation

**Affected packages:** `cargo-rullst` and `rullst-capital` 12.0.0.

The SaaS scaffold mounts `/billing/portal`, but the Capital documentation and
provider implementations state that `create_customer_portal(email,
return_url)` has no reviewed live provider-session contract and returns
`UnsupportedOperation` for live credentials.

**Required correction:** do not generate a live portal route until a
provider-specific contract is implemented. A temporary UI should direct the
operator to the provider dashboard without fabricating a customer-specific
portal URL.

### SAAS-007 — Wise live transfer payload is not a valid current transfer flow

**Affected package:** `rullst-capital` 12.0.0  
**Source:** `src/providers/wise.rs`, `create_transfer()`.

The adapter sends the recipient email as `targetAccount`, constructs
`quoteUuid` as `profile_{profile_id}`, and constructs a non-UUID
`customerTransactionId`. The current Wise transfer contract requires a real
recipient account ID, a provider-issued authenticated quote UUID and a UUID
idempotency identity. Creating a transfer also does not itself fund it.

**Required correction:** model quote, recipient, transfer and funding as
separate typed operations. Validate corridor-specific requirements, especially
BRL metadata, and never infer a quote or recipient from an email address.

### SAAS-008 — Checkout mutation has no provider-forwarded idempotency key

**Affected package:** `rullst-capital` 12.0.0.

The generic checkout method accepts email, plan ID and redirect URL but no
application attempt ID. Its reviewed HTTP implementations do not forward a
persisted idempotency key. A timeout therefore leaves the application without
safe evidence for deciding whether a repeated mutation should be sent.

**Required correction:** replace the generic checkout mutation with typed,
provider-specific requests that require a durable application attempt ID where
the provider supports idempotency. Persist the attempt before network dispatch
and reconcile uncertain outcomes instead of retrying automatically.

### SAAS-009 — The generated billing data model is not provider-scoped

**Affected package:** `cargo-rullst` 12.0.0 SaaS blueprint.

Generated `billing_customers` rows are unique only by email, and generated
subscriptions are unique only by subscription ID. Neither model stores the
provider. This prevents safe multi-gateway operation and makes provider
migration or parallel acceptance ambiguous.

**Required correction:** persist provider on customer, checkout, subscription,
payment and webhook-event records. Use provider-scoped unique constraints and
never assume IDs from different gateways share one namespace.

### SAAS-010 — Webhook persistence is not atomic

**Affected package:** `cargo-rullst` 12.0.0 SaaS blueprint.

The generated webhook handler saves the provider customer binding and the
subscription in separate operations without one database transaction. A crash
between writes can leave a bound customer without the subscription update.

**Required correction:** claim the provider event durably and update all
application billing state in one transaction. Store the provider event ID,
payload digest, processing outcome and timestamps for reconciliation.

### SAAS-011 — Generated Windows linker configuration forces an unsupported option

**Affected package:** `cargo-rullst` 12.0.0 SaaS blueprint.  
**Source:** generated `.cargo/config.toml`.

The scaffold forces MSVC linker argument `/DEBUG:FASTLINK`. During both the
generated application build and `cargo rullst db:migrate`, the installed linker
reported `LNK4315`: `/DEBUG:FASTLINK` is no longer supported and the linker
silently used `/DEBUG:FULL` instead. This creates noisy builds and makes the
claimed local linker optimization ineffective.

**Required correction:** stop emitting `/DEBUG:FASTLINK`. Let the Rust/MSVC
toolchain choose its supported debug-information mode, or gate any explicit
linker option by a tested linker-version capability check.

### SAAS-012 — Generated application ignores its Cargo lockfile

**Affected package:** `cargo-rullst` 12.0.0 SaaS blueprint.  
**Sources:** generated `.gitignore` and `Dockerfile`.

The scaffold ignores `/Cargo.lock` even though it generates a deployable binary
application. Its container build also runs `cargo build --release` without
`--locked`. Two builds from the same application commit can therefore resolve
different compatible transitive dependency releases. That is avoidable supply
chain and regression risk, especially for an application handling payments.

**Required correction:** commit the generated application lockfile and use
`cargo build --release --locked` in deployment builds. Add a generator test
that creates a fresh blueprint and verifies both properties.

### SAAS-013 — `strict-postgres` still activates unrelated SQL backends

**Affected packages:** `rullst` and `rullst-orm` 12.0.0 feature wiring.

The audited SaaS application disables default features and selects
`strict-postgres` on both `rullst` and its direct `rullst-orm` dependency.
Nevertheless, `cargo tree -e features -i sqlx-sqlite` shows that `rullst`
reactivates `rullst-orm/default`; the resulting graph includes SQLx SQLite and
MySQL support in addition to PostgreSQL. Enabling Studio also activates its
SQLite queue feature.

This does not change the application's configured runtime database, but the
named strict backend mode is not compile-time exclusive. The extra drivers and
native SQLite code increase dependency/build surface and make backend-isolation
audits misleading.

**Required correction:** declare internal ORM dependencies with
`default-features = false`, forward only the selected strict backend, and put
Studio's SQLite queue behind a separate explicit feature. Add feature-matrix
tests that reject unwanted `sqlx-sqlite` and `sqlx-mysql` activation for a
PostgreSQL-only application.

### SAAS-014 — No hosted one-time checkout contract

**Affected package:** `rullst-capital` 12.0.0.

`BillingProvider::create_checkout_session()` is described generically as
creating a checkout session, but its Stripe implementation always sends
`mode=subscription` and accepts a `plan_id`. The separate `charge()` contract
supports an immediate off-session PaymentIntent only after an application has
already obtained reusable provider customer and payment-method IDs. It cannot
collect a new customer's payment method through hosted Checkout.

Consequently, an application cannot use the v12 high-level API to offer a
clearly one-time public purchase. Reusing `create_checkout_session()` for a
low-value demonstration would silently turn that purchase into a recurring
subscription.

**Required correction:** introduce an explicit typed checkout request whose
mode distinguishes one-time payment from subscription. A one-time request must
carry a server-owned Price ID, authenticated local subject or durable attempt
ID, success and cancellation URLs, provider idempotency key and reviewed
webhook event set. Keep the existing subscription method source-compatible but
rename or deprecate ambiguous entry points. Add tests proving that payment and
subscription modes cannot be confused.

### SAAS-015 — Stripe webhook verifier rejects valid multi-signature rotation headers

**Affected package:** `rullst-capital` 12.0.0.

`StripeProvider::verify_signature_at()` scans `Stripe-Signature` but stores only
one `v1` value, overwriting the previous value whenever another `v1` entry is
present. Stripe can keep multiple endpoint secrets active during a delayed
secret rotation and generates one signature for each active secret. The
framework therefore verifies only the last signature instead of accepting any
`v1` signature that matches the configured secret. A valid event can be
rejected during rotation solely because of signature order.

**Required correction:** parse every `v1` value, decode each bounded candidate
and accept when any candidate passes constant-time HMAC verification with the
configured endpoint secret. Continue to require exactly one valid timestamp
and the existing freshness window. Add regression tests with a valid signature
in the first, middle and last positions, malformed neighboring signatures and
no matching signature.

The SaaS blueprint uses an application-local multi-signature verifier as a
temporary workaround. This does not modify or fix the published framework.

## Explicit v12 capability boundaries (not defects by themselves)

- InfinitePay, Mercado Pago, Coinbase Commerce and PicPay intentionally reject
  live plan-only checkout because the generic method has no authoritative
  amount/currency/customer contract.
- Alipay live checkout and webhooks intentionally fail closed because RSA2
  signing and verification are not implemented.
- Mercado Pago live body-only webhook verification intentionally fails closed;
  its signature manifest also depends on request metadata and authoritative
  resource lookup.
- Wise is an outgoing payout adapter, not an incoming SaaS checkout gateway.
- Provider customer portals are offline fixtures only in v12.
- Rullst initializes one process-global billing provider by default. The
  explicit `WebhookMiddlewareState::production_with_provider` API can support
  provider-bound routes, but the generated SaaS blueprint does not build a
  multi-provider registry around it.

## Application finding (not a framework defect)

### APP-SAAS-001 — A paid artifact committed to the public repository is not exclusive

The initial audited example checks an authenticated entitlement before serving
the Stripe field report, but the same Markdown bytes are committed under the
public `blueprints/saas/reports` directory and are embedded in the public
container image at compile time. The HTTP authorization check protects only
that route; it cannot make already-public source bytes exclusive to purchasers.

**Required correction before live sales:** keep only a public summary in the
repository. Load the complete report and sanitized deployment tutorial through
an operator-only process into private application storage, record an immutable
SHA-256 digest and version, and stream the bytes only after authenticated
entitlement verification. Do not place purchaser data in the artifact and do
not expose a permanent public object URL.

**Correction implemented in the application:** staging retains the explicitly
public field report as a sandbox teaching artifact. Live mode instead requires
a private Azure Blob URL, Container App managed identity and pinned SHA-256
digest at startup. The authenticated delivery path obtains a short-lived token
only from the trusted local Azure identity endpoint, refuses redirects, limits
the object to 2 MiB, accepts only the configured Azure Storage HTTPS host,
verifies the digest and sends the bytes with private/no-store response headers.
The paid live guide itself remains an operator-supplied private artifact and is
not committed to this repository.

### APP-SAAS-002 — Azure Nexus mount omitted the trusted TLS capability

The first release deployment mounted the v12 Nexus Basic Auth router behind
Azure Container Apps TLS ingress but did not insert `NexusVerifiedTls`. Nexus
correctly failed closed: `GET /nexus` returned HTTP 426 instead of accepting or
challenging for Basic credentials. Adding `X-Forwarded-Proto` would not be a
valid correction because an arbitrary forwarding header is not transport
evidence.

**Correction implemented here:** the application inserts
`NexusVerifiedTls::from_trusted_tls_termination()` only when the server-owned
`NEXUS_TRUSTED_TLS_TERMINATION` value exactly identifies the reviewed Azure
Container Apps boundary. The staging workflow supplies that non-secret value.
A direct local HTTP release deployment leaves it unset and continues to fail
closed. This was an application integration omission, not a framework defect;
the v12 Nexus source and README explicitly require the capability.

### APP-SAAS-003 — A rejected retry consumes the checkout creation limit

The first Stripe staging acceptance attempt created a valid Checkout Session
and returned HTTP 303. A second form submission then received HTTP 409 because
the database correctly rejected another open attempt, but that rejected request
still consumed the second slot in the process-local limiter. The next request
received HTTP 429 for ten minutes. The application also stored the provider
session ID without offering a way to resume its trusted Checkout URL.

**Correction implemented here:** resolve an authenticated account's open
attempt before consuming the new-session limiter. For a `checkout_created`
attempt, retrieve the session from Stripe, verify test mode, operation, local
attempt metadata, product, hashed opaque client reference and exact trusted
Checkout host, then redirect to the existing open session. Expired sessions are
closed before creating a replacement; completed sessions return to local
processing. Pending or unknown outcomes remain fail-closed for reconciliation.
Only a request that actually needs a new provider session reaches the limiter,
and HTTP 429 now includes `Retry-After: 600`.

This is an application checkout-orchestration defect in this repository, not a
defect in the Rullst framework rate limiter.

### APP-SAAS-004 — Strict CSP blocks the hosted Checkout handoff in Chromium

The SaaS staging application used the framework's secure default
`form-action 'self'` policy. The local checkout POST correctly returned HTTP
303 with an exact, server-validated `https://checkout.stripe.com` location,
but Chromium applies `form-action` to redirects that follow a form submission.
The browser therefore remained on the pricing page while command-line HTTP
checks appeared successful because they do not enforce CSP.

**Correction implemented here:** the SaaS application's CSP now adds only
`https://checkout.stripe.com` to `form-action`. The server continues to reject
any returned Checkout URL whose scheme, host or embedded credentials differ
from that exact destination. The pricing page also preserves visible signed-in
state and gives immediate progress feedback after a valid submission.

This is a blueprint/application policy-integration omission, not a defect in
Rullst's strict CSP default. Payment blueprints should require an explicit,
provider-specific `form-action` allowlist and a real-browser redirect test.

### APP-SAAS-005 — Pre-provisioned production ingress retained the placeholder port

The production Azure Container App was safely pre-provisioned with a neutral
Microsoft image listening on port 80. The accepted SaaS image listens on port
3000, but the initial production promotion workflow updated only the image and
environment variables. The custom domain would therefore remain routed to the
wrong target port after promotion and its health check would fail.

**Correction implemented here:** the fail-closed production promotion now
updates the existing ingress target to port 3000 before verifying application
and database readiness. This was a deployment-integration defect, not a Rullst
framework defect.

### APP-SAAS-006 — Live offer omitted required Brazilian seller disclosure

The initial production notice showed a legal seller name, country and support
email but omitted the seller's CPF/CNPJ and physical address. Brazilian
e-commerce rules require supplier registration and physical/electronic address
to be readily visible before the contract is concluded.

**Correction implemented here:** live mode now requires a checksum-valid CPF or
CNPJ and a bounded public physical address supplied only through protected
deployment secrets. Both are HTML-escaped and displayed with the legal seller
name and support email on the offer and legal notices. The application refuses
to start live Checkout when the disclosure is incomplete. The values remain
out of Git and logs, although the law requires their intentional public display
to customers. This is a production application compliance omission, not a
Rullst framework defect.

### APP-SAAS-007 — Production OIDC trust did not match GitHub's immutable subject controls

The production Entra application initially trusted only the mutable
`repo:owner/repository:environment:...` subject. GitHub emitted the repository's
enabled immutable subject containing numeric owner and repository IDs, so Azure
rejected the token before any deployment mutation. Adding a second ordinary
subject credential with the visible immutable `sub` still did not authenticate
in the observed environment.

**Correction implemented in infrastructure:** add a Microsoft Entra flexible
federated identity credential that matches the exact immutable environment
`sub` and separately requires both GitHub's `repository_id` and
`repository_owner_id` claims. The production workflow then authenticated with
a short-lived OIDC token. The mutable credential must be retired after the
immutable path is fully validated. This is cloud trust configuration, not a
Rullst framework defect.

### APP-SAAS-008 — Production promotion lacked linked-resource permissions

The production identity had `Container Apps Contributor` only on the
production Container App. The promotion gate could not read the staging app to
prove the selected image, and a later secret update could not perform
`Microsoft.App/managedEnvironments/join/action` on the linked Container Apps
environment. Both failures occurred before live payments could be enabled.

**Correction implemented in infrastructure:** grant `Reader` only on the
staging Container App and `Container Apps Operator` only on the linked managed
environment. Keep `Container Apps Contributor` scoped to the production app.
The resulting identity can verify staging, join the existing environment and
update production without receiving resource-group-wide ownership. This is an
Azure deployment integration requirement, not a Rullst framework defect.

### APP-SAAS-009 — Disabled production was presented as the Stripe sandbox

The payment page selected all environment language from `PAYMENTS_MODE` alone.
The required fail-closed production deployment uses
`PAYMENTS_MODE=disabled`, so `saas.rullst.win` incorrectly described itself as
the published staging environment, linked to “Sandbox terms” and stated that
it stayed in Stripe Test Mode.

**Correction implemented here:** treat `DEPLOYMENT_TIER` and payment mode as
separate state. Disabled production now presents a production pre-launch
status, pre-launch privacy/terms and an explicit no-payment boundary, while
staging continues to describe Stripe sandbox behavior. Live wording still
requires the separate protected activation workflow. This is an application
presentation/configuration defect, not a Rullst framework defect.

## Required SaaS blueprint updates

### P0 — Required before any real-money acceptance

1. Keep live payments off by default and require an explicit real-money
   acknowledgement in addition to live credentials.
2. Store all credentials only in Azure Container Apps secrets or another
   secret manager. Never commit `.env` or live keys.
3. Keep amount, currency, recurrence and provider product IDs server-owned.
   Verify the configured provider price before creating the low-value checkout.
4. Use an authenticated local billing subject, not a browser-supplied email or
   tenant ID.
5. Verify exact raw webhook bytes, signature, freshness and replay before any
   state mutation.
6. Persist provider event IDs and process customer/subscription/entitlement
   changes atomically and idempotently.
7. Never grant access from a success-page redirect. Grant it only from final,
   reconciled provider evidence.
8. Provide refund and dispute procedures before enabling a one-time live
   purchase. Never use a recurring plan to simulate the advertised purchase.
9. Add rate limits and duplicate-submit protection around checkout creation.
10. Return generic browser errors while logging only redacted provider failure
    classes and correlation IDs.

### P1 — Multi-gateway architecture

1. Replace the one-global-provider configuration with an application-owned
   registry keyed by provider.
2. Use a distinct signed webhook endpoint and explicit provider-bound
   middleware state for each enabled gateway.
3. Define typed capabilities such as checkout, one-time payment, subscription,
   portal, cancellation, refund, payout and reconciliation. Do not expose one
   method merely because an adapter struct exists.
4. Add provider-specific price configuration instead of one shared plan label.
5. Show unavailable adapters as unavailable; never turn missing credentials
   into a fake production success URL.
6. Separate payment gateways from payout rails in documentation and UI.

### P2 — Verification and operations

1. Retain deterministic offline tests for every adapter.
2. Add provider protocol fixtures for request and webhook schemas.
3. Run official sandbox acceptance per operation and retain redacted evidence.
4. Do not use a real payment instrument as test data. If a provider requires
   live launch verification, follow its approved process only after
   account/KYC, legal, refund, webhook, logging and reconciliation checks pass.
5. Add scheduled reconciliation against provider APIs for missed webhooks and
   uncertain network outcomes.
6. Add dashboards and alerts for rejected signatures, replay attempts, stale
   pending checkouts, failed renewals and state divergence.

## Low-value amount research and live-mode warning

- **Stripe:** its published BRL minimum is R$0.50, so a provider-owned R$1.00
  one-time Price can be evaluated after sandbox tests. Stripe is available to
  Brazilian businesses, subject to onboarding and account verification.
- **Paddle:** its published minimum for BRL is R$4.00, so R$1.00 is not valid.
- **Lemon Squeezy:** BRL can be displayed, but transactions are processed in
  USD and payouts are made in USD. Fees make very small tests economically
  unsuitable, and the v12 store-ID defect blocks this adapter first.
- **Polar:** BRL is a supported product currency, but the v12 checkout request
  must be updated to the current API before live testing.
- **Razorpay:** plans require provider-owned amount/currency configuration.
  International payments and merchant availability are account-dependent and
  may require activation; do not assume a Brazilian R$1 plan will be accepted.
- **Other adapters:** no R$1 live claim should be made until a reviewed checkout
  contract, current provider documentation and sandbox evidence all exist.

Testing with the operator's own real payment instrument can create fees, tax
records, fraud signals, recurring renewals and provider-policy violations.
Stripe explicitly requires test API keys and test payment methods for
integration testing and prohibits using real payment details in live mode as
test data. The published SaaS environment must therefore remain in Stripe Test
Mode. Any later live launch must follow the provider's approved activation and
verification procedure; a low amount does not make a prohibited test safe.

## Documentation corrections

- Replace broad claims such as "11 supported gateways" with an operation-level
  matrix. State that the count is 10 billing adapters plus one payout adapter.
- Include Alipay consistently in the adapter inventory or explain why it is
  excluded from the crate README provider table.
- Mark deterministic `mock_*` URLs as offline fixtures that never prove account
  reachability or live checkout conformance.
- Publish minimum-currency and merchant-country links as provider-owned facts,
  not hard-coded evergreen Rullst promises.
- Explain that provider price IDs are authoritative: the amount shown by an
  application must be verified against the provider before redirecting a real
  customer.

## Acceptance criteria for calling a gateway “live”

A gateway should be labelled live only when all of the following are recorded:

- exact provider and operation;
- current API request/response fixture;
- official sandbox result;
- signed webhook result, including replay rejection;
- server-owned amount and currency verification;
- durable idempotency/reconciliation evidence;
- authenticated tenant/customer binding;
- cancellation/refund path;
- provider-approved live launch evidence when required, with secrets and
  personal data redacted; never a prohibited live-mode test transaction.

Compilation, a configured environment variable, a generated checkout-looking
URL or a successful redirect is not sufficient evidence.

## Primary references checked

- [Rullst v12 specification and package capabilities](https://rullst.github.io/Rullst/book/spec.html)
- [Rullst v12 CLI reference](https://rullst.github.io/Rullst/book/cli_reference.html)
- [Stripe supported currencies and minimum charge amounts](https://docs.stripe.com/currencies)
- [Stripe test environments and test payment methods](https://docs.stripe.com/testing)
- [Stripe webhook signatures and endpoint-secret rotation](https://docs.stripe.com/webhooks)
- [Stripe availability in Brazil](https://stripe.com/br/global)
- [Lemon Squeezy supported currencies](https://docs.lemonsqueezy.com/help/payments/currencies)
- [Lemon Squeezy create-checkout contract](https://docs.lemonsqueezy.com/api/checkouts/create-checkout)
- [Paddle supported currencies and minimum amounts](https://developer.paddle.com/concepts/sell/supported-currencies/)
- [Paddle create-transaction contract](https://developer.paddle.com/api-reference/transactions/create-transaction/)
- [Paddle default payment-link requirements](https://developer.paddle.com/build/transactions/default-payment-link/)
- [Polar checkout sessions](https://polar.sh/docs/features/checkout/session)
- [Polar products and currencies](https://polar.sh/docs/features/products)
- [Razorpay create-subscription contract](https://razorpay.com/docs/api/payments/subscriptions/create-subscription/)
- [Razorpay international payment requirements](https://razorpay.com/docs/payments/international-payments/)
- [Wise transfer contract](https://docs.wise.com/api-reference/transfer)
- [Cargo guidance for committing application lockfiles](https://doc.rust-lang.org/cargo/faq.html#why-have-cargolock-in-version-control)
