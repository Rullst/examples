# Rullst SaaS deployment profile

This profile records public, non-secret launch decisions. Secrets, personal tax
identifiers and provider credentials must never be committed here.

## Confirmed public configuration

| Setting | Value |
| --- | --- |
| Brand | Rullst |
| Seller model | Individual seller in Brazil |
| Support contact | `officialrullst@gmail.com` |
| Privacy contact | `officialrullst@gmail.com` |
| Production host | `https://saas.rullst.win` |
| Staging host | `https://saas-staging.rullst.win` |
| Initial currency | BRL |
| Initial offer | One-time Rullst Stripe Gateway Field Report |
| Initial amount | BRL 1.00 |
| Initial provider | Stripe |

The Stripe screenshot supplied by the operator shows an active R$1.00 product
named `Rullst Stripe`. Its `Subscriptions` column is empty, so the displayed
Price is one-time. The separate `Trials (Preview)` area is for subscription
trial periods; `No trials` does not identify Stripe live mode or sandbox mode.

The original `price_1UGVGTCWTlr9100l1ADAGmw2` belongs to the earlier product and
is not staging configuration. The operator subsequently created the active,
one-time BRL 1.00 Price `price_1UGen0CWTlr9100lrR2hLPXG` while the Stripe
Dashboard visibly showed Test Mode and stated that no real transaction would
be processed. This is the confirmed staging Price ID. The application still
retrieves it with an `sk_test_...` key and requires `livemode=false` before
creating Checkout. A `prod_...` product identifier cannot be used as a line-
item Price ID.

## Staging decision

Staging is a separate deployment of the same application with test provider
keys and a separate database. It exists to exercise redirects, signed
webhooks, idempotency, refunds, report entitlement and deletion/export flows
without moving money or corrupting production records.

Staging needs an HTTPS URL because browsers and payment-provider webhooks must
reach it. The confirmed hostname is `https://saas-staging.rullst.win`. The
staging UI should be access-limited and `noindex`; its signed webhook endpoint
must remain reachable by Stripe.

Production and staging must have separate:

- Stripe test/live keys and Price IDs;
- webhook endpoints and signing secrets;
- PostgreSQL databases;
- application/session keys;
- Container Apps secrets and logs; and
- report-entitlement records.

## Why both environments remain

`saas-staging.rullst.win` is the permanent pre-production safety boundary. It
uses Stripe sandbox objects and test payment methods, may be reset, is marked
`noindex`, and can scale to zero when idle. It is where schema migrations,
Checkout redirects, webhook retries, duplicate events, declines, refunds and
authorization changes are exercised before a release can affect money or
production records.

`saas.rullst.win` is the customer-facing production showcase boundary. It must
use a separate PostgreSQL database, live Stripe account objects, live webhook
signing secret, application key, operational alerts, backup policy and legal
notices. A separate Neon Free project is acceptable for the initial low-volume
showcase only when its no-SLA limitation and scale-to-zero startup delay are
accepted and independent private Azure backups are enabled. Test users, test
entitlements and sandbox provider identifiers must never be copied into it.

Both environments should remain. Scale-to-zero keeps staging inexpensive, but
removing it would force future upgrades to be tried against production. A
release is promoted by immutable image digest only after staging acceptance;
databases and secrets are never promoted between environments.

## Current deployment status

The Azure Container App, scale-to-zero limit, GitHub environment secrets,
environment-scoped OIDC identity, direct DNS records and managed TLS binding
for `saas-staging.rullst.win` have been provisioned. The OIDC identity has the
Container Apps Contributor role only on this staging app and has no stored
Azure client secret. The Stripe sandbox Price and signed webhook destination
are also configured.

On 2026-09-17, the reviewed immutable image was deployed with PostgreSQL
migrations, `/healthz`, the public route and the protected Nexus boundary
working. The initial fail-closed deployment evidence is
[`35264511389`](https://github.com/Rullst/examples/actions/runs/35264511389).
The subsequent test-mode deployment and automated Chromium Checkout handoff
also passed, and the operator confirmed a completed Stripe sandbox Checkout,
the resulting private certificate and the matching Stripe test record. No
customer or payment details are retained in this public evidence.

The sandbox happy path is therefore validated. The application now implements
full-refund/dispute revocation, an operator refund queue, private tutorial
delivery, scheduled provider reconciliation, a finite Founding Customer cohort
and daily logical backup automation. Provisioning the separate production
resources, testing a restore, configuring alerts and reviewing public seller
details remain operator acceptance tasks. Test-mode readiness is not
live-money readiness.

## Payment validation policy

Integration tests use Stripe sandbox keys and documented test payment methods.
Stripe explicitly prohibits treating real payment details in live mode as test
data. Therefore, the launch plan does not make an artificial self-purchase with
the operator's real card.

BRL 1.00 is above Stripe's currently documented BRL 0.50 minimum, but minimum
amount eligibility does not replace account activation, fees, tax, refund or
consumer-law review. Production is considered operational only after sandbox
acceptance, live-key and live-webhook configuration, and the first legitimate
customer purchase. That transaction is a real sale, not a test transaction.

## Paid tutorial delivery

The purchased Stripe artifact should include a sanitized end-to-end tutorial
for sandbox setup, Neon PostgreSQL, GitHub OIDC, Azure Container Apps, DNS/TLS,
webhooks, deployment checks and production promotion. It must use placeholders
and must never contain this deployment's credentials, merchant identifiers,
subscription IDs, database URL or webhook payloads.

The complete paid tutorial must not be committed to this public repository or
embedded in a publicly downloadable container layer. Before live sales, store
the versioned bytes in a private Azure Blob container and supply the application
with a query-free Blob URL plus immutable SHA-256 digest. Give the Container App
managed identity read access only to that container. The authenticated route
obtains a short-lived Azure token from the trusted local identity endpoint,
downloads at most 2 MiB, refuses redirects and untrusted hosts, verifies the
digest and streams the bytes only after an active entitlement check.

The precise production acceptance checklist is maintained in
[`PRODUCTION_READINESS.md`](PRODUCTION_READINESS.md). A sandbox purchase may
issue a clearly labelled `Rullst Sandbox Pioneer` certificate, but it must not
be described as a real customer purchase. `Founding Customer` remains reserved
for a finite, published live-production cohort.

## Production storage prepared on 2026-09-17

Azure Storage account `rullstsaasprod` was created in East US as Standard LRS
with HTTPS-only transport, TLS 1.2 minimum, anonymous blob access disabled and
shared-key authorization disabled. Private containers `paid-artifacts` and
`database-backups` were created. Blob and container soft delete are enabled for
30 days.

The 10,470-byte private artifact
`rullst-stripe-production-guide-v1.md` was uploaded with version metadata and
SHA-256
`a6e2bd03e59597838741fa723f10755a190004c0f012cf9b8e88bcff498b066f`.
No public URL was enabled. Production access uses Azure managed identity, not
an account key or expiring SAS. Recalculate and update the recorded digest if
the private guide changes.

[`azure-storage-lifecycle.json`](azure-storage-lifecycle.json) is a reviewed
template for deleting database backups after 90 days. It has not been applied:
automatic backup deletion requires explicit operator approval after the restore
and retention policy is accepted.

## Private configuration still required

An individual seller's legal/controller identity may need to appear in the
privacy and consumer notices. Do not put a CPF, residential address or private
legal name in source control or chat. Configure legally required public seller
details through a deployment secret or reviewed content-management boundary
after qualified Brazilian consumer/privacy advice.

## Age and purchaser policy

Public technical content may be read by people under 18. A purchase must be
made by a person who is at least 18 or by a parent/legal guardian acting for a
minor. The application records only that purchaser-authority attestation; it
does not collect an exact date of birth by default.

This deliberately does not claim that every minor can enter a direct purchase
contract worldwide. Contract capacity, parental consent and children's data
rules vary by jurisdiction. A direct self-service offer to minors requires a
separate country and age-assurance review. Until the policy and tax review is
complete, the production offer should be presented as a Brazil/BRL launch and
must not claim automatic legal availability in every country.

Primary references:

- [Stripe sandboxes and test payments](https://docs.stripe.com/testing)
- [Stripe products and one-time Prices](https://docs.stripe.com/products-prices/manage-prices)
- [GDPR Article 8 and member-state age variation](https://eur-lex.europa.eu/eli/reg/2016/679/oj)
- [Brazilian ANPD study on children's and adolescents' data](https://www.gov.br/anpd/pt-br/centrais-de-conteudo/documentos-tecnicos-orientativos/estudo-preliminar-tratamento-de-dados-crianca-e-adolescente.pdf)
- [US FTC children's privacy guidance](https://www.ftc.gov/business-guidance/privacy-security/childrens-privacy)
