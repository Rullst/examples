# Privacy operations for the SaaS showcase

The public brand and privacy contact are Rullst and officialrullst@gmail.com.
The live notice retains the configured legal seller identification; the brand
name must not replace legally required seller details.

The site now publishes a versioned `/privacy` notice, a `/cookies` inventory,
authenticated account export, a monitored-contact route for correction and
deletion requests, and community/showcase links without embedded social widgets.
Only essential session and CSRF cookies are used by this application. Public
pages load first-party assets; Stripe receives checkout data after the visitor
chooses to proceed. A “showcase” label does not make live purchases test payments.

## Requests and retention

- Verify account ownership proportionately before disclosing or changing data.
  Never request passwords, reset codes, or unnecessary identification documents.
- Use `/account/data-export` for the authenticated application export. It does
  not disclose password hashes or reset tokens. Assess separately any provider
  records or security data covered by a valid access request.
- Process correction requests even when an account name is not editable in the
  user interface. Coordinate certificate corrections and fraud controls.
- For closure/deletion, identify relevant account, session, certificate,
  purchase, mail, provider, and backup records. Revoke access as appropriate and
  explain restricted retention needed for taxes, disputes, fraud investigation,
  or another applicable legal obligation. Deletion and refund are distinct.
- Document request deadlines, actions taken, reasons for exceptions, and any
  applicable appeal or regulator complaint route.
- Keep concrete retention schedules for application records, Azure/Neon logs,
  backups, Stripe records, and Resend delivery records. Browser cookie lifetimes
  are not server retention schedules. Existing reset-token cleanup is separate.

## Jurisdictions and providers

The implementation is a privacy baseline, not universal legal certification.
Assess LGPD, GDPR/UK GDPR, CCPA/CPRA, PIPEDA, the Australian Privacy Act, Singapore PDPA, and other regional laws based on the actual
audience and operation. Record legal bases, legitimate-interest assessments,
processor agreements, international transfer safeguards, and incident response.
Confirm the need for local representatives, child-specific safeguards, and any
additional regional notices with qualified counsel. Do not enable optional
analytics/advertising without updating controls and notices first.

Useful primary guidance: [ANPD data-subject rights](https://www.gov.br/anpd/pt-br/assuntos/titular-de-dados),
[European Commission obligations](https://commission.europa.eu/law/law-topic/data-protection/information-business-and-organisations/obligations_en),
[California Attorney General CCPA guidance](https://www.oag.ca.gov/privacy/ccpa),
[Canadian OPC access guidance](https://www.priv.gc.ca/en/privacy-topics/accessing-personal-information/obligations-for-organizations/02_05_d_54_ati_02/),
[Australian OAIC rights guidance](https://www.oaic.gov.au/privacy/privacy-legislation/the-privacy-act/rights-and-responsibilities),
and [Singapore PDPC individual rights](https://www.pdpc.gov.sg/overview-of-pdpa/data-protection/individual/individuals-overview).

## Release boundary

This update changes notices and presentation, not prices, checkout modes,
payment credentials, entitlements, or database schemas. Run existing SaaS tests
and acceptance in staging, then promote the accepted image by immutable digest
while preserving production settings. Do not use the initial “Checkout Disabled”
provisioning workflow to update an already live service.
