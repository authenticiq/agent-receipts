# Security Policy

## Supported Versions

Until `v1.0.0`, only the latest release on the default branch is supported for security fixes.

| Version | Supported |
| --- | --- |
| `main` | Yes |
| `< latest` | No |

## Reporting a Vulnerability

Do not open public issues for suspected vulnerabilities.

Report vulnerabilities privately to `security@strata.codes` with:
- A clear description of the issue
- Steps to reproduce or a proof of concept
- The affected version, commit, or environment
- Any suggested mitigation, if known

Initial response target: 3 business days.

## Disclosure Window

This project follows a 90-day coordinated disclosure window by default. If a fix is available sooner, we will disclose sooner. If additional coordination is necessary for user safety, we may extend the window in consultation with the reporter.

## Scope

Security reports are especially valuable for:
- Signature verification bypasses
- Canonicalization mismatches that enable tampering
- Ledger inclusion or chain-verification flaws
- Key-handling or unsafe default configuration issues
- Dependency or build-chain issues with practical exploit paths