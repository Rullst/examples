# Rullst Stripe Gateway Field Report

**Artifact:** `gateway-report-stripe`  
**Version:** 1  
**Framework scope:** Rullst Capital 12.0.0  
**Privacy classification:** public technical content; no purchaser data

## What this report covers

This field report explains how a Rullst application can place Stripe Hosted
Checkout behind a server-owned offer, process signed evidence and grant a local
entitlement without handling card details. It records the exact limitations
found in Rullst Capital 12.0.0 so an exported adapter is not mistaken for a
complete production contract.

The report bytes are identical for every purchaser. It contains no name,
email, IP address, country, Stripe customer/session/payment identifier,
transaction timestamp, card detail or webhook payload.

## Intended one-time flow

1. An authenticated account asks to purchase the fixed
   `gateway-report-stripe` offer.
2. The server creates a durable purchase attempt with an application-owned
   idempotency key.
3. The server reads the configured Stripe Price and requires the expected
   active, one-time BRL amount.
4. The server creates Stripe Checkout with `mode=payment`, a server-owned Price
   ID and an opaque purchase-attempt reference.
5. The browser follows an HTTP 303 redirect to Stripe. Card data never reaches
   the Rullst application.
6. A success-page redirect displays only a pending/reconciled status and never
   grants the report.
7. The signed Stripe event is checked for signature freshness and replay,
   expected mode, final payment status, amount, currency, Price, event/session
   identity and local purchase attempt.
8. One database transaction records the accepted event, marks the purchase and
   creates the report entitlement.
9. The same transaction issues an environment-specific certificate. Its public
   verification URL uses a random identifier and omits the holder's identity
   and every provider/payment identifier; the holder decides whether to share
   it.

## Rullst v12 findings

- `StripeProvider::create_checkout_session()` hard-codes
  `mode=subscription`; it cannot safely represent this one-time product.
- The generic checkout method has no durable attempt/idempotency parameter and
  does not bind an opaque local billing subject.
- The normalized Stripe webhook contract accepts subscription lifecycle
  objects, not the one-time Checkout event needed by this offer.
- Direct `charge()` uses an idempotent off-session PaymentIntent, but requires
  provider customer and reusable payment-method IDs already collected. It is
  not a public hosted-payment collection flow.
- A local one-time Checkout boundary is therefore required until an upstream
  typed API is released and audited.

These findings are recorded as SAAS-004, SAAS-008 and SAAS-014 in the
repository framework report.

## Security and privacy checklist

- Keep secret keys and webhook secrets in Azure Container Apps secrets.
- Use Stripe test keys and test payment methods in staging.
- Never use a real card as integration test data in live mode.
- Keep amount, currency, Price ID and artifact version on the server.
- Enforce CSRF, authenticated ownership, rate limits and duplicate-submit
  protection before creating Checkout.
- Persist provider references as confidential reconciliation data; do not log
  or expose them in report URLs.
- Reject stale, unsigned, replayed, mismatched or uninteresting events.
- Grant only from final provider evidence, never from the return URL.
- Publish refund, privacy, support and seller information before launch.
- Test access/export/deletion and retention exceptions before production.

## Operational acceptance evidence

The gateway is not ready merely because this report exists. The operator must
retain redacted evidence for:

- a test-mode one-time Checkout and cancellation;
- a correctly signed completed event and replay rejection;
- an amount/currency/Price mismatch rejection;
- one entitlement despite duplicate event delivery;
- refund and dispute handling;
- report authorization and cross-account denial;
- data export, badge opt-in withdrawal and account deletion; and
- database backup/restore and missed-event reconciliation.

## Primary references

- [Stripe Checkout quickstart](https://docs.stripe.com/checkout/quickstart)
- [Stripe Checkout Sessions API](https://docs.stripe.com/api/checkout/sessions)
- [Stripe webhook signatures](https://docs.stripe.com/webhooks/signature)
- [Stripe test environments and payment methods](https://docs.stripe.com/testing)
- [Stripe supported currencies and minimums](https://docs.stripe.com/currencies)
- [Rullst SaaS/Capital findings](../../../saas-improvements-needed.md)
- [Repository privacy baseline](../../../docs/GLOBAL_PRIVACY_BASELINE.md)
