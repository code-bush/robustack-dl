# RoBustack-DL

> **"Own Your Reading. Byte by Byte."**

[![License: GPL-3.0](https://img.shields.io/badge/License-GPLv3%20%2B%20Commercial-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)

**RoBustack-DL** is a tool for digital repair, cybersecurity awareness, and resilience against mis/dis/malinformation. It allows you to archive and audit web content with integrity.

---

## Installation

### From crates.io (when published)
```bash
cargo install robustack-dl
```

### From source
```bash
git clone https://github.com/codebush-collective/robustack-dl.git
cd robustack-dl
cargo build --release
# Binary available at target/release/robustack-dl
```

**Requirements:** Rust 1.85+ (Edition 2024)

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

### Download content
```bash
robustack-dl download --url https://example.com --output ./archive
```

| Flag | Description | Default |
|------|-------------|---------|
| `-u, --url` | Target URL to download | *required* |
| `-o, --output` | Output directory | `.` |

### Audit archive integrity
```bash
robustack-dl audit --manifest ./archive/manifest.json
```

| Flag | Description | Default |
|------|-------------|---------|
| `-m, --manifest` | Path to manifest.json | `manifest.json` |

### Generate shell completions
```bash
# Bash
robustack-dl completions --shell bash > ~/.local/share/bash-completion/completions/robustack-dl

# Zsh
robustack-dl completions --shell zsh > ~/.zfunc/_robustack-dl

# Fish
robustack-dl completions --shell fish > ~/.config/fish/completions/robustack-dl.fish
```

### Global options
| Flag | Description |
|------|-------------|
| `-c, --config <PATH>` | Path to a `config.toml` override |
| `-V, --version` | Print version banner |
| `-h, --help` | Print help |

### Logging
RoBustack-DL uses structured logging via `tracing`. Control verbosity with `RUST_LOG`:
```bash
RUST_LOG=debug robustack-dl download --url https://example.com
```

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
