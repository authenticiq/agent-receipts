# Maintenance Policy

This document defines how dependency updates, issue intake, and maintainer triage should work in this repo.

## Dependency And Security Updates

- Dependabot configuration lives in `.github/dependabot.yml`.
- Dependabot should open small, reviewable PRs on a weekly cadence.
- Dependency PRs only merge with green CI.
- Security alerts and automated security fixes should stay enabled on GitHub.
- Any update that touches verification, signing, canonicalization, or fixture integrity gets manual review even if it looks routine.

## Issues Vs Discussions

Use Issues for:
- bugs and regressions
- feature requests
- spec or contract changes
- release-blocking work

Use Discussions for:
- usage questions
- design discussion
- integration help
- examples, showcases, and implementation notes

Do not use Issues or Discussions for suspected vulnerabilities. Follow `SECURITY.md`.

## Triage Rules

- New issues and discussions should be labeled or redirected within 7 days.
- Anything that looks release-blocking should be triaged the same day when practical.
- Security-sensitive public reports should be redirected to private reporting immediately.
- If a question is filed as an issue, redirect it to Discussions and close the issue once the handoff is clear.

## Standard Labels

- `bug`: confirmed defect or regression
- `docs`: documentation work
- `enhancement`: feature or improvement request
- `question`: issue needs clarification or should move to Discussions
- `spec`: contract, schema, or design-surface change
- `security`: security-sensitive work that is safe to track publicly
- `release-blocker`: must be resolved before the next release
- `breaking-change`: intentionally incompatible public change
- `dependencies`: dependency or automation update work
- `good first issue`: suitable for a new contributor
- `help wanted`: maintainer welcomes outside help