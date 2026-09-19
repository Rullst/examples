# LMS showcase deployment and privacy operations

This is the public **lms.rullst.win** demonstration, not the production Academy at **academy.rullst.win**. The organization is **Rullst**; the public privacy contact is **officialrullst@gmail.com**.

## Deploy

Pushing changes under `blueprints/lms/` to `main` triggers `.github/workflows/deploy-lms.yml`. It builds and publishes an image to GHCR. The workflow requires a valid `AZURE_CREDENTIALS` secret for Azure deployment, with access to `rullst-lms` in `rullst-rg`. A successful image build alone does not prove the running site was updated; check the Azure deploy step and the active revision.

Before deploying this version, configure **APP_KEY as a unique, persistent secret** in the Container App (at least 32 random bytes encoded as a string). The former shared key baked into the Dockerfile has been removed. Rotate that known key if it is still configured; existing sessions will need to sign in again. All replicas must use the same private key. Missing or invalid keys fail startup in production. Never publish this key alongside the public demo password.

Keep `LMS_SHOWCASE_MODE=true` for this public image. The shared login is `demo@rullst.dev` / `RullstAcademy2026!`; its password is restored on startup. The same encrypted platform session opens Nexus and Studio. There is no separate Basic Auth prompt or public admin password. The framework's Nexus policy remains in place behind a random internal session bridge.

The public demo gets read-only catalog access in the tools, never user tables, request traces, arbitrary AI database queries or mutations. Personal signup is disabled in showcase mode. Existing accounts remain able to sign into the platform; being a learner does not make someone an administrator. `LMS_ADMIN_EMAILS` explicitly authorizes existing platform accounts for administrative access. Leave it empty for the public demo. Use `LMS_SHOWCASE_MODE=false` and a separate database for a private LMS deployment; review policies, branding and data flows before doing so. Disabling showcase mode does not automatically delete previously seeded accounts: disable or rotate the public demo account when converting an existing database to a private service.

Keep SQLite on a persistent volume and use a compatible replica strategy; container filesystem storage alone does not guarantee durable learner records. The deployment workflow does not provision database persistence or secrets.

## Implemented privacy controls

- English `/privacy` and `/cookies`, visible from the public pages, identify the operator, contact, data use, providers and rights.
- HTML uses `Cache-Control: no-store`. HTMX history persistence is disabled; stale HTMX history is removed.
- The service worker upgrades old `rullst-shell-*` caches and only caches named same-origin public static assets. New offline storage is opt-in and can be removed from `/cookies`.
- The cloud Copilot checkbox starts unchecked. The server requires its affirmative value before using an external provider. Cloud prompts contain the submitted message and a static instruction, with no database records, learner profile or progress. Provider errors are not shown to visitors.
- YouTube is a click-to-load placeholder using its privacy-enhanced domain after activation. The transcript is available without loading the video. External social links are ordinary links, not tracking widgets.
- Public demo tool access cannot read individual user records. No analytics, advertising pixels or cross-context advertising sharing are added by this application.

## Operator responsibilities before claiming compliance

This is a technical baseline, not certification of worldwide compliance. Determine applicability of LGPD, EU/UK GDPR and ePrivacy/PECR, CCPA/CPRA and other US state laws, Canada's PIPEDA/provincial rules, Australia's Privacy Act, Singapore PDPA and other laws relevant to actual users and processing. Check current requirements with qualified counsel.

1. Inventory actual Azure/Groq configuration, logs, backups, subprocessors and processing locations. Review contracts, lawful bases and international transfer safeguards. If `GROQ_BASE_URL` points to another provider, update the notice before enabling cloud AI.
2. Set and enforce retention periods for database records, IP/request logs, backups and AI provider data. The application does not currently promise automatic deletion on a fixed schedule. Configure the existing privacy request and retention services where appropriate; verify their workflows against real data.
3. Monitor the privacy mailbox. Verify identity proportionately, track regional response deadlines, and handle access, correction, portability, deletion, objection, consent withdrawal, authorized agents and appeals where applicable. Do not use the public shared account to verify ownership of someone else's data.
4. Publish any additional operator address, representative or DPO details required for the actual deployment. Document security incidents and notification procedures.
5. Treat the real Academy's service for children as a separate assessment, including age assurance, parental/guardian authority where required, data minimization, best interests and provider choices. A showcase banner or a cookie checkbox does not implement parental consent or satisfy COPPA/LGPD/GDPR children's requirements.

Primary references: [ANPD data-subject rights](https://www.gov.br/anpd/pt-br/assuntos/titular-de-dados), [European Commission obligations](https://commission.europa.eu/law/law-topic/data-protection/information-business-and-organisations/obligations_en), [ICO cookies](https://ico.org.uk/for-organisations/direct-marketing-and-privacy-and-electronic-communications/guide-to-pecr/cookies-and-similar-technologies/), [California CCPA](https://www.oag.ca.gov/privacy/ccpa), [FTC COPPA guidance](https://www.ftc.gov/business-guidance/resources/childrens-online-privacy-protection-rule-six-step-compliance-plan-your-business).
