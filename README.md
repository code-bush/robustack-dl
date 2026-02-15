# RoBustack-DL

> **"Own Your Reading. Byte by Byte."**

[![License: GPL-3.0](https://img.shields.io/badge/License-GPLv3%20%2B%20Commercial-blue.svg)](LICENSE)
[![CI](https://github.com/code-bush/robustack-dl/actions/workflows/ci.yml/badge.svg)](https://github.com/code-bush/robustack-dl/actions/workflows/ci.yml)
[![Snyk Security](https://snyk.io/test/github/code-bush/robustack-dl/badge.svg)](https://snyk.io/test/github/code-bush/robustack-dl)
[![SLSA 3](https://slsa.dev/images/gh-badge-level3.svg)](https://slsa.dev)
[![Rust](https://img.shields.io/badge/Rust-1.88%2B-orange.svg)](https://www.rust-lang.org/)

**RoBustack-DL** is a tool for digital repair, cybersecurity awareness, and resilience against mis/dis/malinformation. It allows you to archive and audit web content with integrity.

---

## Installation

### Pre-built Binaries (Linux)
Download the latest binary from the [Releases](https://github.com/codebush/robustack-dl/releases) page.

```bash
chmod +x robustack-dl
sudo mv robustack-dl /usr/local/bin/
```

### From Source

**Prerequisites**: [Rust 1.85+](https://www.rust-lang.org/tools/install)

#### Linux (Debian/Ubuntu)
```bash
# Install build dependencies
sudo apt-get update && sudo apt-get install -y build-essential pkg-config libssl-dev

# Install via cargo
cargo install robustack-dl

# OR build from source
git clone https://github.com/codebush/robustack-dl.git
cd robustack-dl
cargo build --release
```

#### macOS
```bash
# Install OpenSSL (if not already present)
brew install openssl

# Install via cargo
cargo install robustack-dl
```

#### Windows
Ensure you have the [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) installed.

```powershell
# Install via cargo
cargo install robustack-dl
```

---

## Usage

### Version
```bash
robustack-dl -V
```
```
RoBustack-DL v1.0.0
"Own Your Reading. Byte by Byte."
[GPLv3 + Commercial License]
```

---

### List posts
```bash
# List the last 10 posts
robustack-dl list --url https://example.substack.com --limit 10
```

#### List flags

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--url <URL>` | `-u` | Substack URL to list | *required* |
| `--limit <N>` | | Max number of posts to list | *all* |

---

### Download content
```bash
# Basic download (HTML format, current directory)
robustack-dl download --url https://example.substack.com

# Download as Markdown with images
robustack-dl download \
  --url https://example.substack.com \
  --output ./archive \
  --format md \
  --download-images \
  --image-quality high

# Full-featured download with all options
robustack-dl \
  --verbose \
  --rate 5 \
  --after 2024-01-01 \
  --before 2025-12-31 \
  --cookie-name substack.sid \
  --cookie-val "$SUBSTACK_TOKEN" \
  download \
    --url https://example.substack.com \
    --output ./archive \
    --format md \
    --download-images \
    --download-files \
    --file-extensions "pdf,epub" \
    --add-source-url \
    --create-archive

# Dry run — preview what would be downloaded
robustack-dl download --url https://example.substack.com --dry-run
```

#### Download flags

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--url <URL>` | `-u` | Substack URL to download | *required* |
| `--output <DIR>` | `-o` | Output directory | `.` |
| `--format <FMT>` | `-f` | Output format: `html`, `md`, `txt` | `html` |
| `--dry-run` | `-n` | Preview mode — no files written | `false` |
| `--download-images` | | Download images locally | `false` |
| `--images-dir <DIR>` | | Image subdirectory name | `images` |
| `--image-quality <Q>` | | Image quality: `high`, `medium`, `low` | `high` |
| `--download-files` | | Download file attachments locally | `false` |
| `--files-dir <DIR>` | | Attachment subdirectory name | `files` |
| `--file-extensions <LIST>` | | Comma-separated extension allowlist | *(all)* |
| `--add-source-url` | | Append source URL to each file | `false` |
| `--create-archive` | | Generate an archive index page | `false` |
| `--limit <N>` | | Max number of posts to download | *all* |

---

### Audit archive integrity
```bash
robustack-dl audit --manifest ./archive/manifest.json
```

The audit command verifies every file in the archive against its SHA-256 hash in the manifest. Reports:
- ✅ Files that match their expected hash
- ❌ Hash mismatches (possible corruption)
- ⚠️ Missing files

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--manifest <PATH>` | `-m` | Path to `manifest.json` | `manifest.json` |

---

### Generate shell completions
```bash
# Bash
robustack-dl completions --shell bash > ~/.local/share/bash-completion/completions/robustack-dl

# Zsh
robustack-dl completions --shell zsh > ~/.zfunc/_robustack-dl

# Fish
robustack-dl completions --shell fish > ~/.config/fish/completions/robustack-dl.fish
```

---

### Global flags

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--verbose` | `-v` | Enable debug-level logging | `false` |
| `--proxy <URL>` | `-x` | HTTP/SOCKS5 proxy URL | *none* |
| `--rate <N>` | `-r` | Max requests per second | `2` |
| `--limit <N>` | | Global limit on posts processed | *all* |
| `--after <DATE>` | | Only process posts after this date | *none* |
| `--before <DATE>` | | Only process posts before this date | *none* |
| `--cookie-name <NAME>` | | Cookie name for Substack auth | *none* |
| `--cookie-val <VALUE>` | | Cookie value (use env var `ROBUSTACK_COOKIE_VAL`) | *none* |
| `--config <PATH>` | `-c` | Path to `config.toml` override | *none* |
| `--version` | `-V` | Print version banner | |
| `--help` | `-h` | Print help | |

> **Security tip:** Use the `ROBUSTACK_COOKIE_VAL` environment variable instead of passing the cookie value on the command line, to avoid leaking it in shell history.

### Logging
RoBustack-DL uses structured logging via `tracing`. Control verbosity with `--verbose` or `RUST_LOG`:
```bash
# Via flag
robustack-dl --verbose download --url https://example.com

# Via environment variable (fine-grained control)
RUST_LOG=debug robustack-dl download --url https://example.com
```

---

## Architecture

RoBustack-DL follows **clean architecture** with unidirectional dependency flow:

```
Presentation (CLI)  →  Application (Handlers)  →  Domain (Integrity, Processor)
                                                        ↑
                                            Infrastructure (HttpClient)
```

| Layer | Modules | Responsibility |
|-------|---------|----------------|
| **Presentation** | `cli/` | Argument parsing via `clap` — no business logic |
| **Application** | `handlers/`, `config/` | Orchestration; receives `AppConfig` + `&dyn HttpClient` |
| **Domain** | `integrity/`, `processor/` | Pure business logic: hashing, manifests, content transforms |
| **Infrastructure** | `client/` | `HttpClient` trait + `ReqwestClient` impl (swappable) |

### Idempotency
Every download is **idempotent**: content is SHA-256 hashed, stored under content-addressed filenames, and tracked in `manifest.json`. Re-running the same command produces zero new I/O.

---

## Testing

```bash
# Run all tests (unit + integration + non-functional)
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test cli_integration

# Run non-functional tests (performance, binary size, provenance)
cargo test --test nonfunctional

# Run with verbose output
cargo test -- --nocapture
```

### Test categories
| Suite | Count | What it covers |
|-------|-------|----------------|
| Unit tests | 63 | CLI parsing, AppConfig, HttpClient, Manifest, Processor |
| Integration tests | 30 | End-to-end binary: flags, exit codes, error messages |
| Non-functional tests | 8 | Startup time, binary size, provenance headers, no unsafe/println |

---

## Mission
We believe in **Digital Repair**. Just as we repair physical objects, we must also curate and repair our digital environments. RoBustack-DL empowers users to own their reading data, verify its integrity, and preserve it against decay or censorship.

---

## Contributing
See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, pre-commit hooks, Jira commit conventions (`DSO-` prefix), and coding standards.

## Security
See [SECURITY.md](SECURITY.md) for our vulnerability disclosure policy.

## License
Dual-Licensed: **GPLv3 + Commercial**. See [LICENSE](LICENSE) for details.
