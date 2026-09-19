# Portfolio showcase operations

The public portfolio belongs to Rullst. Privacy contact: officialrullst@gmail.com.
The `/privacy` and `/cookies` notices describe this site's behavior; they are not
a certification of compliance with every global privacy law.

- Default scripts, fonts, and images are first party. There are no application
  analytics or advertising trackers. CMS editors should not add trackers or
  remote assets without reviewing the notice and consent requirements.
- Local assistant replies are the default. Only `cloud_ai=yes` enables Groq.
  The public assistant sends bounded public portfolio context; the shared admin
  assistant sends a static blueprint description, never private database records.
- Chat history and cloud preference stay in page memory. HTML/API responses use
  `no-store`; HTMX history storage is disabled. The CSRF cookie is essential.
- This is a shared public sandbox. Enter fictional data only. Its SQLite data is
  ephemeral in the existing container deployment, and edits may be public.
  Private deployments must replace the public Basic credentials and provision a
  persistent database as appropriate.
- The content revision migration updates the original profile and project
  examples once, including existing databases. It preserves unrelated records
  and later CMS edits. Do not remove its revision marker to reset content.

Before using the blueprint for personal data, Rullst must confirm the applicable
laws and legal bases, processor agreements, cross-border safeguards, hosting and
Groq retention settings, deletion/request procedures, and incident response.
Resolve privacy requests through the contact above, verify identity
proportionately, and meet applicable local response deadlines. Delete data from
each relevant system; clearing browser storage or restarting a container does
not delete provider logs or email. Keep a record of operational retention and
legal basis assessments. Child-directed services need a separate review.

The current privacy baseline addresses notice, minimization, essential storage,
optional external AI, and a rights contact. Applicability of LGPD, GDPR/UK GDPR,
CCPA/CPRA and other regional laws depends on the actual operation and audience.

## Publication

A push to `main` changing `blueprints/portfolio/**` triggers
`.github/workflows/deploy-portfolio.yml`: tests, OCI image build, Azure deploy,
and live verification. A successful push starts this process; successful
deployment and live checks establish that the change is serving traffic.
