# SaaS production readiness gate

The intended customer host is `https://saas.rullst.win`.
`https://saas-staging.rullst.win` remains the permanent Stripe sandbox and
pre-production environment. The similarly written `saas.rullst.com` is not a
configured host in this repository.

The production deployment is not a rename of staging. Promote only an
immutable image digest that passed staging acceptance, while keeping the
database, application key, provider credentials, webhook secret and records
strictly separate.

## Before creating the production deployment

- Complete one Stripe sandbox Checkout with an official test payment method.
- Confirm signed-webhook reconciliation, duplicate-event handling, report
  entitlement, private certificate issuance and anonymous public verification.
- Test declined, expired and delayed-payment cases and retain redacted evidence.
- Move the complete paid report and tutorial out of the public repository and
  public image into authenticated private storage with an immutable digest.
- Verify the implemented signed refund/dispute events, entitlement and
  certificate revocation, refund-request queue and scheduled reconciliation.
- Verify the authenticated JSON data export and document the monitored
  operator process for correction, closure/deletion and retention exceptions.
- Publish reviewed merchant identity, privacy, terms, refund, support,
  accessibility and cookie notices for the actual launch jurisdictions.
- Define monitoring and alerts, incident response and log retention. Run the
  implemented daily backup and complete a documented restore drill.

## Production-only infrastructure

- A separate production PostgreSQL database. Neon Free is acceptable only for
  this low-volume live showcase with explicit no-SLA expectations and the
  independent private Azure backup; it is not a contractual availability plan.
- A separate Azure Container App, managed certificate and custom-domain binding
  for `saas.rullst.win`.
- A protected `saas-production` GitHub environment with environment-scoped OIDC
  and least-privilege Azure roles.
- A unique production `APP_KEY` and private Nexus credentials.
- Stripe account activation/KYC, a live one-time Price, `sk_live_...` secret and
  a distinct live webhook endpoint/signing secret.
- Production alerts for webhook failures, signature rejection, stale purchase
  attempts, reconciliation differences, refunds and disputes.

The fail-closed production deployment is automated by
`.github/workflows/deploy-saas-production.yml`. It accepts only the full commit
SHA of the image currently running successfully in staging, resolves that tag
to an immutable registry digest and always deploys with
`PAYMENTS_MODE=disabled`. The workflow requires a pre-provisioned
`rullst-saas` Container App and these protected `saas-production` environment
secrets:

- `SAAS_PRODUCTION_DATABASE_URL`;
- `SAAS_PRODUCTION_APP_KEY`;
- `SAAS_PRODUCTION_NEXUS_USERNAME` and
  `SAAS_PRODUCTION_NEXUS_PASSWORD`; and
- production-scoped `AZURE_CLIENT_ID`, `AZURE_TENANT_ID` and
  `AZURE_SUBSCRIPTION_ID` OIDC values.

Stripe live credentials are deliberately absent from this preparation
workflow. They must be introduced only by the separately reviewed live-launch
change after the acceptance gate passes.

The protected live-launch workflow is
`.github/workflows/enable-saas-live.yml`. It requires these additional
`saas-production` environment secrets:

- `STRIPE_LIVE_SECRET_KEY` and `STRIPE_LIVE_WEBHOOK_SECRET`;
- `SAAS_PRODUCTION_RECONCILIATION_TOKEN` (32-200 random characters);
- `SAAS_PRODUCTION_MERCHANT_LEGAL_NAME` (the reviewed public legal seller
  name);
- `SAAS_PRODUCTION_MERCHANT_TAX_ID` (the CPF or CNPJ that will be published to
  customers); and
- `SAAS_PRODUCTION_MERCHANT_PHYSICAL_ADDRESS` (a legally valid physical or
  correspondence address that will be published to customers).

Do not put those values in Git, workflow inputs, issue text or chat. Add them
directly as GitHub environment secrets. The application validates Brazilian
CPF/CNPJ check digits and refuses to start live mode when the required public
seller disclosure is absent. Individual sellers should obtain qualified advice
before publishing a residential address or choosing an alternative business
correspondence address. This gate implements the disclosure baseline in
[Brazilian Decree 7,962/2013, article 2](https://www.planalto.gov.br/ccivil_03/_ato2011-2014/2013/decreto/d7962.htm)
and [Decree 10,271/2020](https://www.planalto.gov.br/ccivil_03/_ato2019-2022/2020/decreto/d10271.htm);
it is an engineering safeguard, not a substitute for legal or tax advice.

The live workflow also requires the non-secret environment variables
`SAAS_PAID_ARTIFACT_STORAGE_ACCOUNT`, `SAAS_PAID_ARTIFACT_CONTAINER`,
`SAAS_PAID_ARTIFACT_BLOB` and `SAAS_PAID_ARTIFACT_SHA256`. The production
Container App must have a managed identity with `Storage Blob Data Reader`
limited to that private container; no public URL, account key or expiring SAS
is used by the application.

It accepts the active live `price_...` ID and an exact amount choice. Use `100`
for the initial BRL 1.00 launch. Raising the price to BRL 10.00 requires a new
one-time Stripe Price and one workflow run selecting `1000`, so Price ID and
server-enforced amount change together.

The audited initial offer is deliberately fixed to BRL. Checkout leaves
eligible payment-method selection to the Stripe Dashboard but explicitly
disables Adaptive Pricing for this offer, because entitlement reconciliation
requires the session currency and amount to remain exactly equal to the
server-owned BRL Price. International cards can be eligible, but availability
depends on Stripe, the issuing bank, sanctions and the configured payment
methods; an issuer can convert BRL and add foreign-exchange or international
fees. Supporting local-currency prices later requires a separately tested
reconciliation contract rather than merely enabling a Dashboard switch.

The webhook endpoint must subscribe to all six events:

- `checkout.session.completed`;
- `checkout.session.async_payment_succeeded`;
- `checkout.session.async_payment_failed`;
- `checkout.session.expired`;
- `charge.refunded`; and
- `charge.dispute.created`.

`.github/workflows/reconcile-saas-production.yml` calls the bearer-protected
reconciliation endpoint every six hours. `.github/workflows/backup-saas-production.yml`
creates a validated custom-format PostgreSQL dump every day and uploads it to a
pre-created private Azure Blob container. Configure the non-secret environment
variables `SAAS_BACKUP_STORAGE_ACCOUNT`, `SAAS_BACKUP_CONTAINER`,
`SAAS_PRODUCTION_BACKUPS_ENABLED=true` and, only after launch,
`SAAS_LIVE_ENABLED=true`. Grant the OIDC identity only the required blob role,
set a retention/lifecycle rule and perform a restore drill before live
activation.

## Launch sequence

1. Keep production checkout fail-closed while deploying and migrating.
2. Validate health, TLS, authentication, backup/restore and private artifact
   delivery without live provider credentials.
3. Configure the private artifact and live Stripe objects; the application
   verifies the exact Price server-side before each Checkout.
4. Enable live mode only after legal/operational acceptance and a backup restore
   drill.
5. Treat the first payment as a genuine customer sale. Never use the operator's
   real card as integration test data.
6. Issue a `Founding Customer` certificate only under a published finite cohort
   rule. Keep it distinct from the test-only `Rullst Sandbox Pioneer` badge.

The application accepts `PAYMENTS_MODE=live` only when every fail-closed live
control is configured. A healthy deployment proves configuration consistency;
it does not replace Stripe sandbox tests, restore testing, legal review or a
genuine independent customer sale.
