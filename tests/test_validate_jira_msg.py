"""Unit tests for scripts/validate_jira_msg.py.

Run with: python3 -m pytest tests/test_validate_jira_msg.py -v
"""

from __future__ import annotations

import sys
from pathlib import Path

# Add scripts/ to path so we can import the module.
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "scripts"))

from validate_jira_msg import validate_commit_message  # noqa: E402


# ---------------------------------------------------------------------------
# Valid commit messages — should return None (no error)
# ---------------------------------------------------------------------------

class TestValidMessages:
    """Messages that MUST pass validation."""

    def test_standard_format(self):
        assert validate_commit_message("DSO-42: Implement rate limiter") is None

    def test_high_ticket_number(self):
        assert validate_commit_message("DSO-99999: Large ticket number") is None

    def test_single_digit_ticket(self):
        assert validate_commit_message("DSO-1: Initial commit setup") is None

    def test_summary_with_special_characters(self):
        assert validate_commit_message("DSO-10: Add SHA-256 hashing (v2)") is None

    def test_summary_with_backticks(self):
        assert validate_commit_message("DSO-5: Fix `client::new()` default") is None

    def test_long_summary(self):
        msg = "DSO-123: " + "A" * 200
        assert validate_commit_message(msg) is None

    def test_multiline_only_first_line_matters(self):
        msg = "DSO-42: Valid first line\n\nThis is the body with details."
        assert validate_commit_message(msg) is None


# ---------------------------------------------------------------------------
# Invalid: Missing Jira ID
# ---------------------------------------------------------------------------

class TestMissingJiraId:
    """Messages without DSO- prefix MUST be rejected."""

    def test_plain_text(self):
        err = validate_commit_message("Fix the download bug")
        assert err is not None
        assert "DSO" in err

    def test_wrong_project_prefix(self):
        err = validate_commit_message("PROJ-42: Fix something")
        assert err is not None

    def test_no_prefix_at_all(self):
        err = validate_commit_message("Add new feature to client")
        assert err is not None

    def test_lowercase_dso(self):
        err = validate_commit_message("dso-42: Lowercase prefix")
        assert err is not None

    def test_missing_number(self):
        err = validate_commit_message("DSO-: Missing number")
        assert err is not None

    def test_missing_colon(self):
        err = validate_commit_message("DSO-42 Missing colon")
        assert err is not None

    def test_missing_space_after_colon(self):
        err = validate_commit_message("DSO-42:Missing space")
        assert err is not None


# ---------------------------------------------------------------------------
# Invalid: Lazy / placeholder summaries
# ---------------------------------------------------------------------------

class TestLazySummaries:
    """Lazy summaries MUST be rejected, even with a valid Jira ID."""

    def test_fix(self):
        err = validate_commit_message("DSO-1: fix")
        assert err is not None
        assert "Lazy" in err

    def test_fix_with_period(self):
        err = validate_commit_message("DSO-1: fix.")
        assert err is not None

    def test_work(self):
        err = validate_commit_message("DSO-1: Work")
        assert err is not None

    def test_wip(self):
        err = validate_commit_message("DSO-1: Wip")
        assert err is not None

    def test_ellipsis(self):
        err = validate_commit_message("DSO-1: ...")
        assert err is not None

    def test_update(self):
        err = validate_commit_message("DSO-1: Update")
        assert err is not None

    def test_changes(self):
        err = validate_commit_message("DSO-1: Changes")
        assert err is not None

    def test_stuff(self):
        err = validate_commit_message("DSO-1: Stuff")
        assert err is not None

    def test_test(self):
        err = validate_commit_message("DSO-1: Test")
        assert err is not None

    def test_done(self):
        err = validate_commit_message("DSO-1: Done")
        assert err is not None


# ---------------------------------------------------------------------------
# Invalid: Capitalization
# ---------------------------------------------------------------------------

class TestCapitalization:
    """Summary must start with a capital letter."""

    def test_lowercase_first_letter(self):
        err = validate_commit_message("DSO-42: implement rate limiter")
        assert err is not None

    def test_number_first_letter_is_rejected(self):
        # The regex requires [A-Z] as first char, so digits fail.
        err = validate_commit_message("DSO-42: 42 is the answer")
        assert err is not None


# ---------------------------------------------------------------------------
# Edge cases
# ---------------------------------------------------------------------------

class TestEdgeCases:
    """Boundary conditions and unusual inputs."""

    def test_empty_string(self):
        err = validate_commit_message("")
        assert err is not None
        assert "empty" in err.lower()

    def test_whitespace_only(self):
        err = validate_commit_message("   \n\n  ")
        assert err is not None

    def test_newline_only(self):
        err = validate_commit_message("\n")
        assert err is not None

    def test_very_long_valid_message(self):
        msg = "DSO-1: " + "A" * 10_000
        assert validate_commit_message(msg) is None

    def test_unicode_in_summary(self):
        # Unicode after the capital letter should be fine.
        assert validate_commit_message("DSO-1: Add café support") is None

    def test_tab_characters(self):
        err = validate_commit_message("\tDSO-42: Indented with tab")
        # After strip(), this should fail because of leading tab.
        # The regex checks the first line after strip, so DSO-42 is at position 0.
        # Actually strip() removes tabs, so this becomes "DSO-42: Indented with tab"
        assert err is None
