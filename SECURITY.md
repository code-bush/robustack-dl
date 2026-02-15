# Security Policy

## Supported Versions
| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We value the security community and welcome reports of security vulnerabilities. We are committed to working with you to resolve issues quickly and responsibly.

### Communication Channel
We use **GitHub Private Vulnerability Reporting**. To report a vulnerability:
1. Navigate to the [Security tab](https://github.com/code-bush/robustack-dl/security) of the repository.
2. Under "Vulnerability reporting", click **Report a vulnerability**.
3. Provide a detailed description including steps to reproduce.

### Our Commitment
- **Acknowledgment**: We will acknowledge receipt of your report within **48 hours**.
- **Resolution**: We will provide a timeline for resolution once the vulnerability is confirmed.
- **Credit**: We will credit you in our security advisories and release notes.

## Scope

### In-Scope (What is a vulnerability)
- Remote Code Execution (RCE) in the CLI or processor.
- Path Traversal vulnerabilities during content downloading/archiving.
- Sensitive data exposure in logs or manifests.
- Improper handling of authentication tokens/cookies.
- Manifest integrity bypasses (collision attacks against SHA-256).

### Out-of-Scope (What is NOT a vulnerability)
- Issues requiring physical access to the machine.
- Denial of Service (DoS) attacks against Substack's servers (these should be reported to Substack).
- Vulnerabilities in third-party libraries (please report these to the respective maintainers, or via Snyk).
- General bugs or crashes that do not have a security impact.

## Supply Chain Security
We use structured CI/CD pipelines to ensure the integrity of our builds:
- **Snyk** for SCA (software composition analysis) and SAST (static analysis).
- **Trufflehog** for secret scanning.
- **CycloneDX SBOMs** for dependency transparency.
- **SLSA Level 3 Provenance** for build authenticity.
