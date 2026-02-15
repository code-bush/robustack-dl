#!/usr/bin/env python3
"""Validate that a commit message follows the DSO Jira semantic format.

Required pattern: DSO-<number>: <Capitalized summary>
Example:          DSO-42: Implement rate limiter with 20% jitter

Exit codes:
    0 — Commit message is valid.
    1 — Commit message is rejected (with guidance printed to stderr).
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
JIRA_PATTERN = re.compile(r"^DSO-[0-9]+: [A-Z].+$")

LAZY_SUMMARIES = frozenset({
    "fix",
    "fix.",
    "work",
    "wip",
    "...",
    "update",
    "changes",
    "stuff",
    "test",
    "done",
})

# Git commit message filenames that git may pass to commit-msg hooks.
_ALLOWED_MSG_FILES = frozenset({
    "COMMIT_EDITMSG",
    "MERGE_MSG",
    "SQUASH_MSG",
    "TAG_EDITMSG",
})

# ---------------------------------------------------------------------------
# Validation
# ---------------------------------------------------------------------------


def validate_commit_message(message: str) -> str | None:
    """Return an error string if *message* is invalid, otherwise ``None``."""
    first_line = message.strip().splitlines()[0] if message.strip() else ""

    if not first_line:
        return "Commit message is empty."

    # Check for lazy / placeholder summaries.
    summary_part = first_line.split(": ", maxsplit=1)[-1].strip().lower().rstrip(".")
    if summary_part in LAZY_SUMMARIES:
        return (
            f"Lazy commit summary rejected: '{first_line}'\n"
            "  Summaries like 'fix', 'work', or '...' are not allowed.\n"
            "  Write a meaningful description of the change."
        )

    # Enforce DSO- Jira pattern.
    if not JIRA_PATTERN.match(first_line):
        return (
            f"Commit rejected: '{first_line}'\n"
            "  Every commit must reference a Jira ticket from the DSO project.\n"
            "  Required format: DSO-<number>: <Capitalized summary>\n"
            "  Example:         DSO-42: Implement rate limiter with 20% jitter"
        )

    return None


def _safe_commit_msg_path(raw_arg: str) -> Path | None:
    """Resolve *raw_arg* to a safe, validated commit message path.

    Extracts only the basename from the argument, validates it against
    a known allowlist, then constructs the path from the git directory.
    This prevents CWE-23 path traversal by never passing unsanitized
    input into ``pathlib.Path``.

    Returns ``None`` if the argument is not a recognized git commit
    message filename.
    """
    basename = os.path.basename(raw_arg)
    if basename not in _ALLOWED_MSG_FILES:
        return None

    # Hardcoded .git directory — no external input flows into Path().
    # Git hooks are always invoked from the repo root.
    safe_path = Path(".git").resolve() / basename

    if not safe_path.is_file():
        return None

    return safe_path


def main() -> int:
    """Entry point — reads the commit message file passed by git."""
    if len(sys.argv) < 2:
        print("Usage: validate_jira_msg.py <commit-msg-file>", file=sys.stderr)
        return 1

    commit_msg_path = _safe_commit_msg_path(sys.argv[1])
    if commit_msg_path is None:
        print(
            f"Error: '{sys.argv[1]}' is not a recognized git commit message file.\n"
            f"  Expected one of: {', '.join(sorted(_ALLOWED_MSG_FILES))}",
            file=sys.stderr,
        )
        return 1

    message = commit_msg_path.read_text(encoding="utf-8")
    error = validate_commit_message(message)

    if error is not None:
        print(f"\n❌ {error}\n", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
