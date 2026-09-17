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
- Implement signed refund/dispute events, entitlement revocation, certificate
  revocation and scheduled provider reconciliation.
- Complete data export, correction, closure/deletion and retention workflows.
- Publish reviewed merchant identity, privacy, terms, refund, support,
  accessibility and cookie notices for the actual launch jurisdictions.
- Define monitoring and alerts, incident response, log retention, database
  backups, point-in-time recovery and a tested restore procedure.

## Production-only infrastructure

- A separate paid managed PostgreSQL database with a documented recovery SLA.
- A separate Azure Container App, managed certificate and custom-domain binding
  for `saas.rullst.win`.
- A protected `saas-production` GitHub environment with environment-scoped OIDC
  and least-privilege Azure roles.
- A unique production `APP_KEY` and private Nexus credentials.
- Stripe account activation/KYC, a live one-time Price, `sk_live_...` secret and
  a distinct live webhook endpoint/signing secret.
- Production alerts for webhook failures, signature rejection, stale purchase
  attempts, reconciliation differences, refunds and disputes.

## Launch sequence

1. Keep production checkout fail-closed while deploying and migrating.
2. Validate health, TLS, authentication, backup/restore and private artifact
   delivery without live provider credentials.
3. Configure the live Stripe objects and verify them server-side.
4. Enable live mode only after the application live gate is implemented and
   the legal/operational acceptance record is approved.
5. Treat the first payment as a genuine customer sale. Never use the operator's
   real card as integration test data.
6. Issue a `Founding Customer` certificate only under a published finite cohort
   rule. Keep it distinct from the test-only `Rullst Sandbox Pioneer` badge.

The current Rullst 12 application deliberately rejects `PAYMENTS_MODE=live` at
startup. Removing that guard before these items pass is not production launch;
it is bypassing the acceptance boundary.
