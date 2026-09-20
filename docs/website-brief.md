# Public website brief

## Publication state

This is an implementation-ready content and product brief, not a live website. Do not publish before the owner controls the domain/brand identity, approves legal policies, and has a signed verified Windows release URL. Do not put an unsigned local installer behind a public download button.

## Audience and message

Primary audiences: students, developers, knowledge workers and power users who value local-first software. Do not market to everyone.

Problem:

> Your work is scattered across files, notes, tasks and AI chats.

Solution:

> Aether keeps your work together, remembers your context, and helps you continue.

Supporting position: a calm, local-first intelligent workspace for Windows—not a chatbot, generic dashboard or cloud collaboration suite.

## Information architecture

1. **Home:** problem, Aether promise, product screenshot/video from the real signed build, four principles—understands, remembers, organizes, acts—and one truthful release-state action.
2. **Product:** Spaces, Pulse/Continue, Search/Sources, Notes/Tasks/Vault, explicit Memory, user-approved Actions and optional AI. Show real behavior only.
3. **Privacy:** local SQLite, explicit folder authorization/context, BYOK/managed-AI distinction when applicable, backup scope, no hidden telemetry.
4. **Download:** Windows requirements, exact version, signed publisher, SHA-256, release notes, install/update/rollback guidance and known limitations.
5. **Security/support:** private vulnerability route, beta issue forms, supported version, backup-first troubleshooting and status/support expectations.
6. **Legal/commercial:** owner-approved terms, privacy notice, source/product license and pricing only after decisions are final.

## Release-state actions

Use exactly one state at a time:

- **No signed public build:** “Aether is in private testing.” No download or fake waitlist.
- **Signed Public Beta:** “Download Public Beta for Windows” linking only to the exact signed release page, with prerelease/backup warning adjacent.
- **Commercial/1.0:** plan/download actions only after entitlements, policies, pricing, support and launch gates pass.

Never label a GitHub source archive, unsigned CI artifact or local MSI/NSIS as the official Windows download.

## Proof without fabrication

Allowed proof:

- real product screenshots with neutral test data;
- verified architecture/privacy statements linked to documentation;
- signed release version/publisher/hash;
- measured, owner-approved aggregate beta evidence only when privacy thresholds and methodology are documented.

Forbidden:

- invented testimonials, company logos, user counts, retention, reviews or awards;
- final prices before owner approval;
- fake countdowns, “limited spots,” always-on urgency or dead buttons;
- claims of offline AI, sync, collaboration, universal file understanding or automatic memory that do not exist;
- collection forms without a privacy purpose, retention policy, owner and secure backend.

## Visual direction

Match Aether's calm neutral/indigo system and real product typography. Use deliberate whitespace, strong hierarchy, restrained motion and real interface imagery. Avoid purple-blue AI gradients, neon/glass effects, generic SaaS card grids, fake charts, stock people and decorative dashboards.

Meet WCAG-oriented keyboard, contrast, reduced-motion, semantic heading, alt-text and zoom expectations. Ship a static, fast, CSP-hardened site with minimal dependencies, no third-party trackers by default and no cross-origin scripts without owner/security review.

## Technical publication checklist

- owner-controlled domain, DNS, hosting and recovery access;
- HTTPS/HSTS, CSP, security headers, dependency/update ownership and rollback;
- legal entity/contact and approved privacy/terms/license pages;
- no analytics or cookies unless explicitly approved and controllable;
- signed release link/hash/version tested from a clean browser/Windows account;
- accessible 404/error/download-unavailable states;
- backup of site source and reproducible deployment;
- incident owner for compromised links, domain or download metadata.
