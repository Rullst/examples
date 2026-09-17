# Global privacy and data-protection baseline

This document defines the minimum privacy-by-design contract for every public
Rullst example in this repository. It is an engineering baseline, not a legal
opinion or a claim that one build automatically complies with every law in
every country. The deployer remains responsible for identifying applicable
jurisdictions, legal bases, controller/processor roles, age rules, taxes,
consumer obligations and cross-border transfer requirements with qualified
counsel.

The baseline is designed around principles shared by the GDPR, UK GDPR, LGPD
and other major privacy regimes: purpose limitation, data minimization,
accuracy, retention limits, security, transparency and accountable handling of
data-subject requests.

## Repository-wide rules

1. Collect only fields required for an active, documented purpose. Optional
   profile and public-display fields are private by default.
2. Never store raw card, bank, wallet or authentication data. A payment
   provider owns payment-method collection; applications retain only bounded
   reconciliation references and normalized status evidence.
3. Never put names, email addresses, IP addresses, provider customer IDs,
   payment/session IDs, webhook bodies, prompts, credentials or secrets in a
   public badge, downloadable gateway report, analytics event or URL.
4. Essential session and CSRF cookies must be documented, Secure, HttpOnly
   where readable JavaScript is not required, SameSite-scoped and bounded in
   lifetime. Non-essential analytics, advertising and personalization stay off
   until the applicable consent boundary permits them.
5. AI prompts are personal data whenever a person can be identified from
   their content. Prompt collection must state the provider, purpose,
   retention, cross-border path and prohibition on entering secrets. Provider
   training or secondary use stays disabled unless separately justified and
   disclosed.
6. Every mutable tenant resource is authorized server-side. Public IDs,
   hidden fields and client-supplied tenant IDs are never authorization proof.
7. Logs use correlation IDs and bounded error classes. Sensitive values are
   redacted before serialization; production logs have a documented retention
   limit and access policy.
8. Encryption in transit is mandatory. Managed databases, backups and secret
   stores must use provider-supported encryption at rest and least-privilege
   identities.
9. A breach-response owner, security contact, evidence-preservation procedure
   and jurisdiction-aware notification workflow must exist before production.
10. Subprocessors, processing locations, transfer mechanisms and retention
    commitments are published and versioned. A provider's availability does
    not by itself establish a lawful cross-border transfer.
11. Public content may be accessible to minors, but payment accounts and
    purchase contracts default to an adult account holder or a parent/legal
    guardian acting for the minor. Do not collect an exact birth date unless a
    documented age-assurance purpose and retention policy require it. Services
    directed to children require a separate jurisdiction-specific review.

## Required user controls

Authenticated applications must provide or operationally support:

- access to the user's stored profile and entitlement data;
- structured export in a portable format;
- correction of editable identity data;
- account closure and deletion/anonymization requests;
- withdrawal of optional consent as easily as it was given;
- objection/restriction routing where applicable;
- a visible privacy contact and request-status channel; and
- appeal/human review for consequential automated decisions.

Deletion does not mean erasing records that must be retained for tax, fraud,
chargeback or legal obligations. Those records must instead be isolated,
access-restricted, purpose-limited and pseudonymized when possible. The
published retention schedule must explain each exception.

## Payment data boundary

The SaaS payment example may persist only the minimum reconciliation set:

- internal user and purchase-attempt IDs;
- provider and product SKU;
- integer minor-unit amount and ISO currency;
- normalized status and timestamps;
- provider event/session/payment references required for idempotency,
  disputes and refunds; and
- digest, schema version and outcome of processed webhook evidence.

Provider references are confidential operational data even though they are not
card numbers. They must not be displayed publicly, included in reports or
placed in application logs. Raw webhook payload retention is off by default.
If incident evidence requires a raw payload, it needs encrypted restricted
storage, an explicit purpose and a short deletion deadline.

Payment success redirects never grant reports, badges or account access. Only
a final provider event whose signature, freshness, replay identity, amount,
currency, product and local purchase subject were verified can create an
entitlement.

## Badges and purchased reports

Gateway reports are immutable static technical publications. Their bytes are
identical for every purchaser of the same report version and contain no buyer
data. Authorization checks whether an account owns an entitlement and then
serves the static artifact through a short-lived, non-public download route.

A badge records only `(user_id, badge_key, awarded_at, public_opt_in)`. Public
display is false by default. The badge never exposes amount, provider customer
identity, transaction time, country or payment method. Revoking public display
does not revoke the private entitlement.

## Blueprint risk levels

| Blueprint | Primary personal data | Required posture |
| --- | --- | --- |
| Showcase | Optional AI prompts and security telemetry | Anonymous by default; no secret-presence disclosure; short logs. |
| Portfolio | Contact/chat input and AI prompts | Explicit purpose and delivery retention; no public prompt history. |
| SaaS | Account, payment reconciliation and entitlements | Highest payment controls; provider-hosted payment collection; rights workflow. |
| LMS | Identity, learning records and potentially minors' data | Existing tenant-scoped privacy lifecycle plus age/guardian and education-law review. |

## Production acceptance gate

No example may claim global compliance merely because these controls exist.
Production approval requires a completed data map, controller identity,
privacy notice, terms, lawful-basis record, retention schedule, subprocessors,
cross-border assessment, data-subject request test, account-deletion test,
backup deletion strategy, incident exercise, accessibility review and local
legal review for the countries actually served.

Primary legal and engineering references:

- [EU General Data Protection Regulation](https://eur-lex.europa.eu/eli/reg/2016/679/oj)
- [Brazilian General Data Protection Law (LGPD)](https://www.planalto.gov.br/ccivil_03/_ato2015-2018/2018/lei/l13709.htm)
- [UK ICO data protection by design guidance](https://ico.org.uk/for-organisations/uk-gdpr-guidance-and-resources/accountability-and-governance/guide-to-accountability-and-governance/accountability-and-governance/data-protection-by-design-and-default/)
- [OWASP Application Security Verification Standard](https://owasp.org/www-project-application-security-verification-standard/)
- [PCI SSC standards and self-assessment resources](https://www.pcisecuritystandards.org/)
