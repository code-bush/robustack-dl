# Contributing to RoBustack-DL

## Funding
This project is part of the CodeBush Collective. Funds support cybersecurity awareness and Repair Cafés NI.
Please support us via our [Open Collective](https://opencollective.com/codebush-collective), hosted by The Social Change Nest.

## Succession Policy
**Bus Factor**: If the primary maintainer is inactive for **180 days**, ownership transfers to the **Designated Successor** (Placeholder) or will be decided by a community vote by the CodeBush Collective.

---

## Local Development Setup

### Prerequisites
| Tool | Purpose | Install |
|------|---------|---------|
| Rust 1.93.1+ | Compiler | `rustup update stable` |
| `pre-commit` | Git hook manager | `pip install pre-commit` |
| `gitleaks` | Secret scanning | `brew install gitleaks` or [GitHub releases](https://github.com/gitleaks/gitleaks/releases) |
| `cargo-deny` | License/advisory audit | `cargo install cargo-deny` |
| GPG key | Commit signing | `gpg --full-generate-key` |

### Initialize the Defensive Layer
```bash
# Clone and enter the repo
git clone https://github.com/codebush-collective/robustack-dl.git
cd robustack-dl

# Install pre-commit hooks (pre-commit + commit-msg stages)
pre-commit install --hook-type pre-commit --hook-type commit-msg

# Enable GPG signing
git config --local commit.gpgsign true
git config --local user.signingkey <YOUR-GPG-KEY-ID>
```

### Commit Message Format
Every commit **must** reference a Jira ticket from the `DSO` project:

```
DSO-<number>: <Capitalized summary>
```

**Examples:**
```
DSO-42: Implement rate limiter with 20% jitter
DSO-107: Fix SHA-256 manifest generation for large archives
```

**Rejected:**
```
fix                         # ❌ No Jira ID, lazy summary
DSO-42: fix                 # ❌ Lazy summary
DSO-42: implement thing     # ❌ Summary not capitalized
Updated the thing           # ❌ No Jira ID
```

### What the Hooks Enforce

| Hook | Stage | What It Checks |
|------|-------|---------------|
| `jira-commit-msg` | `commit-msg` | DSO- prefix, capitalized summary, no lazy messages |
| `cargo-fmt` | `pre-commit` | Code formatting (`cargo fmt --check`) |
| `cargo-clippy` | `pre-commit` | Lints: pedantic + nursery, deny warnings |
| `cargo-test-doc` | `pre-commit` | Doc examples compile and pass |
| `gitleaks` | `pre-commit` | High-entropy tokens, private keys |
| `cargo-deny` | `pre-commit` | Unauthorized licenses, vulnerable deps |
| `ai-provenance-header` | `pre-commit` | AI Provenance & HITL metadata block in `.rs` files |
| `gpg-signing-check` | `pre-commit` | Warns if `commit.gpgsign` is not `true` |

---

## Coding Standards
We adhere to "Boring Rust":
- **No `unsafe`**.
- **No `unwrap()` or `expect()`**. Use explicit error handling.
- **Explicit Error Handling**: Use `thiserror` for library code and `anyhow` for applications.
- **Type-Driven Design**: Leverage the type system to make invalid states unrepresentable.
- **Clippy is Mandatory**: All code must pass `cargo clippy -- -D warnings -W clippy::pedantic -W clippy::nursery`.
- **Observability**: Use `tracing` for logging, NO `println!`.
- **Secrets**: No unmasked secrets in memory. Use `secrecy` crate.
- **AI Provenance**: Every `.rs` file must include the AI Provenance header (see `.github/file_header.txt`).
