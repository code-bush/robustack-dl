# Security Policy

## Supported Versions
| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability
Please report vulnerabilities to the CodeBush Collective via our secure contact channel (to be defined) or open a draft security advisory on GitHub.

## Supply Chain Security
We use structured CI/CD pipelines to ensure the integrity of our builds:
- **Snyk** for vulnerability scanning.
- **Gitleaks** for secret detection.
- **Trufflehog** for deep secret scanning.
- **CycloneDX SBOMs** + **GitHub Artifact Attestations**.
