# Showcase privacy operations

The privacy notice covers `showcase.rullst.win`, not every application built with Rullst. Controller: **Rullst**. Requests: **officialrullst@gmail.com**. Public pages: `/privacy` and `/cookies`.

## Implemented controls

- Privacy links in the showcase footer, Nexus sidebar and Studio; collection notices beside publication, AI and checkout forms.
- Cloud AI is unchecked by default and enforced on the server for each submission. Unchecked submissions use local responses even if an API key exists. Public post contents are not sent in cloud prompts.
- HTML responses use `Cache-Control: no-store`; HTMX history caching is disabled. The service worker only caches an explicit same-origin static-asset allowlist and removes old `rullst-shell-*` caches, including previously cached pages.
- `Referrer-Policy: no-referrer` reduces disclosure when visitors follow external links or load external resources.
- No application analytics or advertising trackers are installed. A decorative “accept cookies” banner is not a substitute for controlling future optional scripts.
- Cache inspection shows metadata and aggregate lookup counts, without keys or cached content. The template cache expires after 60 seconds.

## Data inventory and operational decisions

| Data | Location / recipient | Current behavior | Operator action |
| --- | --- | --- | --- |
| Public post title and body | SQLite; visitors; public sandbox admins | Latest 50 posts retained by count, no fixed time limit | Process removal requests; decide whether a time-based retention policy is also required |
| Chat message | Application; optional configured cloud AI provider | No intentional database chat history; optional cloud request requires `cloud_ai=yes` | Verify active provider, contract, prompt logging, retention and transfer safeguards |
| Technical requests / IP / security events | Application, Azure and security tooling | Retention varies by deployment and provider | Set and document actual logging/backup periods and ensure diagnostic output does not unnecessarily expose visitor data |
| Checkout demo email and plan | Application; potentially external generated checkout URL | Fictional inputs encouraged | Check provider behavior before enabling any real payment integration |
| Security cookie | Browser | `rullst_csrf`, session, necessary for protected forms | Review attributes behind the production TLS proxy |
| Static cache | Browser service worker | Static allowlist only, until replacement or browser clearing | Verify update reaches existing installations |
| Privacy correspondence | Gmail | Email request workflow | Restrict access and define a justified retention period |
| Technical assets | GitHub, Google Fonts, unpkg, jsDelivr, Tailwind CDN where used | Third parties can receive connection metadata | Review necessity/contracts or self-host; account for infrastructure additions |

## Before claiming compliance or deploying new processing

Confirm the controller's complete legal identity/address and any legally required representative or DPO details. Validate legal grounds for each purpose, balancing tests where applicable, actual retention periods, processor terms and international transfer mechanisms. Update the public notice with those confirmed deployment facts. Provider retention and transfer safeguards are not established by this repository.

LGPD, EU GDPR, UK GDPR/PECR and CCPA/CPRA have different territorial scope, exemptions and applicability thresholds. Other jurisdictions may add obligations. Global availability alone does not prove every law applies, and framework use alone does not establish compliance. Obtain a qualified review for the operator and deployment.

For privacy requests, monitor the contact inbox, record receipt and applicable deadline, verify identity proportionately, locate data and recipients, respond with the outcome or applicable exception, and document completion. Do not direct people to disclose personal data in public GitHub issues or Discord. Maintain incident handling, breach assessment and notification procedures outside the public demo.

If analytics, advertising, profiling or other optional storage is introduced, implement prior controls and genuine refusal/withdrawal as required; do not activate it based on merely displaying a notice. If sale/sharing becomes applicable, implement the relevant opt-out mechanisms and Global Privacy Control handling before enabling it.

## Official references checked on September 19, 2026

- [ANPD: data-subject rights](https://www.gov.br/anpd/pt-br/assuntos/titular-de-dados)
- [ANPD: cookies guidance](https://www.gov.br/anpd/pt-br/centrais-de-conteudo/materiais-educativos-e-publicacoes/guia-orientativo-cookies-e-protecao-de-dados-pessoais.pdf)
- [European Commission: GDPR obligations and information to provide](https://commission.europa.eu/law/law-topic/data-protection/information-business-and-organisations/obligations_en)
- [ICO: cookies and similar technologies](https://ico.org.uk/for-organisations/direct-marketing-and-privacy-and-electronic-communications/guide-to-pecr/cookies-and-similar-technologies/)
- [California Attorney General: CCPA rights and notices](https://www.oag.ca.gov/privacy/ccpa)
- [California Privacy Protection Agency: adjusted applicability thresholds](https://privacy.ca.gov/laws-and-regulations/monetary-thresholds-in-the-ccpa/)
