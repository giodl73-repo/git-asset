"""Exercise the CI decision using actual temporary Git histories."""
from pathlib import Path
import subprocess
import tempfile
import unittest

from ci_scope import classify_event, readme_only


class ScopeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="git-asset-ci-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.git("init", "-q", "-b", "main")
        self.git("config", "user.name", "Synthetic fixture")
        self.git("config", "user.email", "fixture@example.invalid")
        (self.root / "README.md").write_text("initial\n", encoding="utf-8")
        (self.root / "source.txt").write_text("code\n", encoding="utf-8")
        self.base = self.commit()

    def git(self, *args):
        return subprocess.check_output(
            ["git", "-C", str(self.root), "-c", "core.hooksPath=NUL",
             "-c", "commit.gpgsign=false", *args], stderr=subprocess.DEVNULL,
        ).decode().strip()

    def commit(self):
        self.git("add", "-A")
        self.git("commit", "-qm", "synthetic change")
        return self.git("rev-parse", "HEAD")

    def edit_readme(self):
        (self.root / "README.md").write_text("updated story\n", encoding="utf-8")
        return self.commit()

    def test_readme_edit_uses_fast_path_for_push_and_pr(self):
        head = self.edit_readme()
        self.assertTrue(classify_event(self.root, "push", {"before": self.base, "after": head}))
        self.assertTrue(classify_event(self.root, "pull_request", {
            "pull_request": {"base": {"sha": self.base}, "head": {"sha": head}},
        }))

    def test_code_or_mixed_change_requires_full_validation(self):
        head = self.edit_readme()
        (self.root / "source.txt").write_text("changed code\n", encoding="utf-8")
        mixed = self.commit()
        self.assertFalse(readme_only(self.root, self.base, mixed))
        self.assertFalse(readme_only(self.root, head, mixed))

    def test_other_markdown_and_hash_inventory_are_not_readme_only(self):
        self.edit_readme()
        (self.root / "docs").mkdir()
        (self.root / "docs/README.md").write_text("other documentation\n", encoding="utf-8")
        self.assertFalse(readme_only(self.root, self.base, self.commit()))
        self.git("reset", "--hard", self.base)
        self.edit_readme()
        (self.root / "inventory.json").write_text("{}\n", encoding="utf-8")
        self.assertFalse(readme_only(self.root, self.base, self.commit()))

    def test_deletion_and_rename_require_full_validation(self):
        (self.root / "README.md").unlink()
        deleted = self.commit()
        self.assertFalse(readme_only(self.root, self.base, deleted))
        self.git("reset", "--hard", self.base)
        self.git("mv", "README.md", "OVERVIEW.md")
        self.assertFalse(readme_only(self.root, self.base, self.commit()))

    def test_mode_swap_requires_full_validation(self):
        self.edit_readme()
        self.git("update-index", "--chmod=+x", "README.md")
        self.git("commit", "-qm", "synthetic mode change")
        self.assertFalse(readme_only(self.root, self.base, self.git("rev-parse", "HEAD")))

    def test_pr_uses_merge_base_when_main_advanced(self):
        head = self.edit_readme()
        self.git("checkout", "-q", "-b", "upstream", self.base)
        (self.root / "source.txt").write_text("already qualified upstream\n", encoding="utf-8")
        advanced = self.commit()
        self.assertFalse(readme_only(self.root, advanced, head))
        self.assertTrue(classify_event(self.root, "pull_request", {
            "pull_request": {"base": {"sha": advanced}, "head": {"sha": head}},
        }))

    def test_unknown_missing_empty_and_revision_expressions_run_full(self):
        head = self.edit_readme()
        for base in (None, "HEAD~1", "0" * 40, "f" * 40):
            self.assertFalse(readme_only(self.root, base, head))
        self.assertFalse(readme_only(self.root, head, head))
        self.assertFalse(classify_event(self.root, "workflow_dispatch", {}))
        self.assertFalse(classify_event(self.root, "pull_request", {}))
        self.assertFalse(classify_event(self.root, "push", {"before": self.base}))

    def test_readme_addition_without_a_previous_readme_is_not_fast(self):
        self.git("rm", "-q", "README.md")
        without = self.commit()
        head = self.edit_readme()
        self.assertFalse(readme_only(self.root, without, head))


if __name__ == "__main__":
    unittest.main()
