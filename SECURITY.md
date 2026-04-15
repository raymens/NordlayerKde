# Security Policy

## Supported Versions

This project currently supports the latest `main` branch and the latest tagged release.

## Reporting a Vulnerability

Please do not open public issues for security vulnerabilities.

Report privately by using GitHub's "Report a vulnerability" feature in the repository security tab.

Include:

- Affected version or commit
- Reproduction steps
- Impact assessment
- Any known workaround

We will acknowledge reports as soon as possible and coordinate remediation and disclosure.

## Supply-Chain Hardening

This repository uses:

- Dependabot for Cargo and GitHub Actions updates
- CI checks for `cargo audit` and `cargo deny`
- Locked builds (`--locked`) in release/security workflows
- Release checksums (`SHA256SUMS.txt`) for published artifacts

