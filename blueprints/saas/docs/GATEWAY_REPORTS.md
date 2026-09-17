# Gateway reports, badges and rollout contract

The SaaS blueprint sells one-time access to original technical field reports.
It does not sell a fake "test payment." Each billing provider has an
independent product SKU and entitlement. Initial launch pricing may be BRL 1.00;
moving to BRL 10.00 creates a new provider Price/version and never rewrites a
historical purchase.

## Artifact contract

Each report contains:

- a provider capability and account-prerequisite summary;
- the Rullst v12 request, webhook and reconciliation boundary;
- documented framework defects or unsupported operations;
- a secure deployment and secret-management checklist;
- sandbox and production acceptance criteria;
- refund/dispute and monitoring considerations; and
- links to current official provider documentation.

The purchased Stripe edition also includes a sanitized implementation tutorial
covering provider sandbox setup, PostgreSQL, GitHub OIDC, Azure Container Apps,
DNS/TLS, signed webhooks, deployment validation and production promotion.

Reports are static, versioned and contain no purchaser data. They must not
include credentials, real webhook payloads, personal information, merchant
account identifiers or copied provider documentation. A content digest binds
each entitlement to an exact report version.

The complete paid bytes are not source-controlled in the public examples
repository and are not embedded in a public container image. They are loaded
into private application storage through an operator-only process and streamed
only after the authenticated entitlement check. The repository copy is a
public summary and must not be described as exclusive paid content.

## Products and badges

| Provider | Product SKU | Private badge | v12 real-payment status |
| --- | --- | --- | --- |
| Stripe | `gateway-report-stripe` | `stripe-field-tester` | Application-owned one-time sandbox path implemented because v12 hard-codes subscription mode; live remains gated. |
| Razorpay | `gateway-report-razorpay` | `razorpay-field-tester` | Subscription path exists; one-time/account eligibility needs separate provider review. |
| Lemon Squeezy | `gateway-report-lemonsqueezy` | `lemonsqueezy-field-tester` | Blocked by hard-coded store ID until framework correction. |
| Paddle | `gateway-report-paddle` | `paddle-field-tester` | Blocked by stale transaction contract. |
| Polar | `gateway-report-polar` | `polar-field-tester` | Blocked by stale checkout contract. |
| InfinitePay | `gateway-report-infinitepay` | `infinitepay-field-tester` | v12 live plan-only checkout unsupported. |
| Mercado Pago | `gateway-report-mercadopago` | `mercadopago-field-tester` | v12 live checkout and webhook boundary unsupported. |
| Coinbase Commerce | `gateway-report-coinbase` | `coinbase-field-tester` | v12 live plan-only checkout unsupported. |
| PicPay | `gateway-report-picpay` | `picpay-field-tester` | v12 live plan-only checkout unsupported. |
| Alipay | `gateway-report-alipay` | `alipay-field-tester` | Live RSA2 checkout/webhook support absent in v12. |
| Wise | `payout-report-wise` | `wise-payout-lab` | Payout education only. Wise cannot be presented as a customer payment button. |

Badges are private by default. A separately recorded opt-in is required before
public profile display. A badge reveals neither price nor transaction details.

## Rollout order

1. The current sandbox milestone implements purchase attempts, provider-event
   replay protection and report entitlements. Move the complete report and
   tutorial bytes to private storage before any live sale.
2. Complete Stripe sandbox acceptance, then add signed refund/dispute handling,
   scheduled reconciliation, the private badge and privacy lifecycle controls.
3. Complete one provider-approved live launch with a genuine product purchase;
   never use a real card as integration test data.
4. Fix or replace each blocked adapter one at a time. Retain protocol fixtures
   and sandbox evidence before exposing its button.
5. Expose a provider only when the merchant account is approved for the
   operator's country, the exact currency/amount is accepted, and live
   webhook/refund/reconciliation evidence exists.

The UI is capability-driven. Unavailable providers remain visible as audited
roadmap entries, not clickable payment controls. This prevents an exported
adapter or configured environment variable from being mistaken for a live
gateway.
