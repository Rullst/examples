# Rullst SaaS blueprint: audited test and live checkout

This Rullst 12.0.0 example separates three concerns that the public Showcase
must not blur:

- the Showcase is a public, deterministic adapter catalogue;
- this SaaS application is the deployable integration target; and
- accepting real money is a later operational decision, not a demo feature.

The permanent **staging** environment uses Stripe Test Mode, while the separate
production environment can sell one low-value digital guide through Stripe
live mode. Both use persistent PostgreSQL, private Nexus credentials and no
production Studio process. Live mode is fail-closed unless every required
credential, legal field, private artifact and reconciliation control is valid.

## Adapter inventory

Rullst Capital 12.0.0 exports 10 incoming billing adapters and one outgoing
payout adapter. Exported does not mean production-ready.

| Adapter | Audited v12 status | Important constraint |
| --- | --- | --- |
| Stripe | One-time test and application-owned live path implemented | Test and live data, keys, webhooks, databases and Price IDs remain separate. |
| Razorpay | Report/roadmap only | Its exported v12 path is recurring and needs a separate one-time contract. |
| Lemon Squeezy | Framework fix required | The v12 adapter hard-codes store ID `1`. |
| Paddle | Framework fix required | The v12 payload does not match the current transaction contract. |
| Polar | Framework fix required | The v12 request uses the former price-based checkout shape. |
| InfinitePay | No plan-only live checkout in v12 | The operation returns `UnsupportedOperation`. |
| Mercado Pago | No plan-only live checkout in v12 | The operation and body-only live webhook verifier are unavailable. |
| Coinbase Commerce | No plan-only live checkout in v12 | The operation returns `UnsupportedOperation`. |
| PicPay | No plan-only live checkout in v12 | The operation returns `UnsupportedOperation`. |
| Alipay | No live checkout in v12 | RSA2 checkout signing and live webhook verification are disabled. |
| Wise | Payout only | Wise sends funds; it is not an incoming SaaS checkout gateway. |

The full evidence is tracked in
[`../../saas-improvements-needed.md`](../../saas-improvements-needed.md).
The one-time report products, private-by-default badges and provider rollout
order are defined in [`docs/GATEWAY_REPORTS.md`](docs/GATEWAY_REPORTS.md). All
personal-data handling is subject to the repository
[`global privacy baseline`](../../docs/GLOBAL_PRIVACY_BASELINE.md).
The confirmed public launch settings and staging split are recorded in
[`docs/DEPLOYMENT_PROFILE.md`](docs/DEPLOYMENT_PROFILE.md). The first immutable,
buyer-independent artifact is
[`reports/stripe-gateway-field-report-v1.md`](reports/stripe-gateway-field-report-v1.md).

## Payment modes

`PAYMENTS_MODE` accepts exactly:

- `disabled`: fail-closed default; no provider checkout or webhook processing;
- `test`: requires Stripe sandbox credentials and enables a one-time
  `mode=payment` Checkout with test data;
- `live`: accepts real money only with an `sk_live_...` key, explicit live
  acknowledgement, signed webhook secret, private artifact, merchant notice
  fields and a strong reconciliation token.

Test and live credentials cannot be mixed: Stripe sandbox mode requires an
`sk_test_...` key and live mode requires an `sk_live_...` key. A deployed
redirect must use HTTPS; plain HTTP is accepted only for localhost in test
mode.

The browser supplies only a fixed local offer identifier and an adult-or-
guardian purchaser attestation. The server owns the Stripe Price ID, expected
amount and currency, fetches that Price immediately before checkout, and
requires an exact active one-time Price match. A return-page redirect never
grants access. The signed webhook is replay-protected in PostgreSQL, the paid
Checkout Session and line item are re-read from Stripe, and only then is a
versioned report entitlement created. The same database transaction issues a
test-only `Rullst Sandbox Pioneer` certificate or, for the first finite live
cohort, a `Rullst Founding Customer` certificate. Full refunds and disputes
revoke the entitlement and certificate after provider verification.

The authenticated certificate page may show the account holder's name and can
be printed or saved as PDF. Its public `/verify/{public_id}` page uses a random
122-bit identifier and shows only badge type, issue date, environment and
validity. It omits the holder's name, email and all provider/payment IDs. The
identifier is not listed publicly; the holder decides whether to share it.

The public `/privacy` and `/terms` pages disclose the staging data boundary,
international hosting path, essential cookies, data-rights contact, minors
policy and the fact that sandbox activity is not a real purchase. Registration
links to both notices without treating the privacy notice as optional marketing
consent. Automated self-service export/deletion remains a production blocker;
staging requests are handled through the documented privacy contact.

## Persistent database

The blueprint uses Rullst's strict PostgreSQL backend. Azure Container Apps
container filesystems are ephemeral, so a SQLite file inside the container is
not the deployment database.

Use a managed PostgreSQL service and store its TLS connection string as an
Azure Container Apps secret mapped to `DATABASE_URL`:

1. **Azure Database for PostgreSQL Flexible Server** is the preferred staging
   location when the owner's active Azure for Students subscription currently
   includes enough free-service allowance or credit. Eligibility, duration,
   region and remaining credit must be confirmed in that subscription's portal
   before provisioning; do not assume the database is permanently free.
2. **Neon Free** is the no-cost fallback for staging/demo. The current staging
   workflow uses one URL for runtime and migrations, so use Neon's direct
   connection string (the host must not contain `-pooler`) with
   `sslmode=require`. An idle compute may scale to zero, but persisted database
   storage is separate from the disposable application container. A future
   high-concurrency deployment can split direct migration and pooled runtime
   URLs explicitly.

Firebase Firestore is not a drop-in option: it is a document database and would
require replacing Rullst ORM models, SQL migrations and transactional billing
persistence. Firebase Data Connect uses PostgreSQL behind another application
contract, but it is also not a direct replacement for this server's managed
`DATABASE_URL` path.

The production showcase may also use a separate Neon Free project to avoid
cash spending. That plan has no application availability or recovery SLA and
may suspend idle compute, so the site must not promise uninterrupted access.
The repository compensates with a daily logical dump to a private Azure Blob
container, but a dump is useful only after a restore drill. Upgrade the
database/storage plan before offering contractual availability or serving a
material sales volume.

## Local setup

1. Start a local PostgreSQL database named `rullst_saas` or supply another
   PostgreSQL URL.
2. Copy `.env.example` to `.env` and generate a unique `APP_KEY`.
3. Set private Nexus credentials; do not reuse the public Showcase password.
4. Leave `PAYMENTS_MODE=disabled` for the first migration and startup.
5. Run:

```console
cargo rullst db:migrate
cargo run
```

The application is available at `http://localhost:3000`. Nexus is mounted at
`/nexus`. Studio is compiled and started only in debug builds and is absent
from the release container. Do not set `NEXUS_TRUSTED_TLS_TERMINATION` for a
direct local HTTP listener: release-mode Basic Auth must fail closed unless a
reviewed deployment boundary has actually terminated TLS.

## Stripe staging setup

1. In the Stripe Dashboard account picker, select **Switch to sandbox** and
   create a sandbox named `Rullst SaaS Staging` if one does not exist.
2. In that sandbox, open **Product catalogue**, create an active **one-time**
   BRL 1.00 Price, and copy its `price_...` identifier. The product page's
   `Trials (Preview)` section is unrelated to sandbox mode and may remain
   `No trials`.
3. Create a sandbox webhook endpoint at
   `https://saas-staging.rullst.win/billing/webhook` for
   `checkout.session.completed`, `checkout.session.async_payment_succeeded`,
   `checkout.session.async_payment_failed` and `checkout.session.expired`.
4. Put the sandbox secret key, webhook signing secret and Price ID in the Azure
   secret manager. Never commit or paste them into chat.
5. Configure:

```dotenv
PAYMENTS_MODE=test
BILLING_PROVIDER=stripe
BILLING_PRICE_ID=price_1UGen0CWTlr9100lrR2hLPXG
BILLING_EXPECTED_CURRENCY=BRL
BILLING_EXPECTED_AMOUNT_MINOR=100
BILLING_REDIRECT_URL=https://saas-staging.rullst.win/dashboard
```

6. Run migrations once from a trusted deployment job, then start the release
   container.
7. Use only Stripe's documented test cards. Do **not** enter a real card in
   live mode to test the integration.
8. After the reconciled webhook completes, open the dashboard to view the
   private Sandbox Pioneer certificate and its privacy-preserving verification
   link. A Checkout success redirect by itself cannot issue the certificate.

## Stripe live setup

Live mode is a genuine sale, not an integration test. Stripe's sandbox remains
the place for operator testing; do not use the operator's own real card to
simulate a customer.

1. Create a separate production PostgreSQL project and a protected GitHub
   environment named `saas-production`.
2. Create an active Stripe **live**, one-time BRL Price. Start with BRL 1.00
   (`100` minor units). To move to BRL 10.00, create a new Price and rerun the
   live workflow with the new Price ID and `1000`; never edit only one side.
3. Configure the live webhook endpoint
   `https://saas.rullst.win/billing/webhook` for
   `checkout.session.completed`, `checkout.session.async_payment_succeeded`,
   `checkout.session.async_payment_failed`, `checkout.session.expired`,
   `charge.refunded` and `charge.dispute.created`.
4. Upload the paid Markdown guide to a private Azure Blob container. Assign the
   Container App managed identity `Storage Blob Data Reader` on that container,
   configure the query-free Blob URL and calculate the exact file SHA-256.
5. Run `Prepare SaaS Production (Checkout Disabled)` with the full staging
   commit SHA. Verify TLS, private Nexus, backup and restore while checkout is
   still disabled.
6. Run `Enable SaaS Live Checkout` with the reviewed live Price ID and exact
   amount. Startup rejects mixed test/live objects and incomplete settings.
7. Keep the scheduled reconciliation and daily backup workflows enabled.

An authenticated buyer may submit a refund request during the published
window. The operator reviews it in private Nexus and creates the full refund in
Stripe. The signed `charge.refunded` webhook, or scheduled provider
reconciliation if that webhook is delayed, marks the request complete and
revokes guide and certificate access. A dispute revokes access immediately;
restoration after a won dispute requires manual review.

## Manual Azure staging workflow

`.github/workflows/deploy-saas-staging.yml` never deploys on an ordinary push.
Run it manually against the protected `saas-staging` GitHub environment only
after configuring these secrets:

- `AZURE_CLIENT_ID`, `AZURE_TENANT_ID` and `AZURE_SUBSCRIPTION_ID` for the
  environment-scoped GitHub OIDC identity (no stored Azure client secret);
- `SAAS_STAGING_DATABASE_URL`;
- `SAAS_STAGING_APP_KEY` (at least 32 random characters);
- `SAAS_STAGING_NEXUS_USERNAME` and `SAAS_STAGING_NEXUS_PASSWORD`;
- `STRIPE_TEST_SECRET_KEY`; and
- `STRIPE_TEST_WEBHOOK_SECRET`.

The target Container App `rullst-saas-staging` must already exist in
`rullst-rg`. The staging workflow constrains it to one replica, runs migrations
before server startup, explicitly asserts the reviewed Azure Container Apps TLS
terminator for Nexus and checks `/healthz`. The marker is a deployment trust
assertion and must not be copied to a direct HTTP deployment. Select `disabled`
for the first revision. Select `test` only after the custom domain is healthy
and the Stripe sandbox webhook points to
`https://saas-staging.rullst.win/billing/webhook`.

## Publishing the link in Showcase

After the SaaS staging URL is healthy, set the GitHub Actions repository
variable `SAAS_BLUEPRINT_URL` to its HTTPS URL and redeploy Showcase. The
Showcase renders the callout only when this value is a valid HTTPS URL, so an
unfinished deployment never creates a broken public link. Its wording clearly
labels the destination as Stripe Test Mode with no real charge.

## Paid tutorial boundary

The downloadable product may include a detailed, sanitized implementation
tutorial in addition to the gateway field report. The complete paid bytes must
live in private application storage and be streamed only after authentication
and entitlement checks. Committing those bytes to this public repository, or
embedding them in a public GHCR image, would make the route paywall cosmetic.

The repository may retain a public summary, schema and loader mechanism. The
private artifact record should contain a version, media type, immutable
SHA-256 digest, status and body. It must contain no purchaser data, secrets,
merchant IDs or copied provider documentation.

## Remaining operational boundary

The application now contains signed refund/dispute handling, scheduled
reconciliation, private paid-artifact delivery and backup automation. The
operator must still provision production resources, test a backup restore,
configure monitoring/alerts, review the actual public seller identity and
document the operator process for correction/closure/deletion requests. The
authenticated dashboard already provides a private JSON data export. Rullst 12.0.0 still
lacks a typed one-time Capital contract, so this Stripe integration remains
application-owned and must not be presented as proof that every exported
gateway is live-ready.

The live `Founding Customer` cohort is capped by
`FOUNDING_CUSTOMER_LIMIT` (100 by default). Sandbox certificates are never
upgraded or relabelled as customer purchases.
