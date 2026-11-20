"""Read-only, fail-closed README-only classification for CI; development only."""
import json
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parent.parent
SHA = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")


def git(root, *args):
    return subprocess.run(
        ["git", "-C", str(root), *args],
        stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, timeout=15, check=False,
    )


def readme_only(root, base, head):
    """False means full validation, including unavailable or ambiguous input."""
    if not all(isinstance(s, str) and SHA.fullmatch(s) and set(s) != {"0"}
               for s in (base, head)):
        return False
    try:
        # Quiet tree comparisons avoid inventories and external diff programs.
        flags = ("diff", "--quiet", "--no-ext-diff", "--no-textconv", "--no-renames")
        others = git(root, *flags, base, head, "--", ".", ":(top,exclude)README.md")
        changed = git(root, *flags, base, head, "--", "README.md")
        if others.returncode != 0 or changed.returncode != 1:
            return False
        before = git(root, "ls-tree", base, "--", "README.md")
        after = git(root, "ls-tree", head, "--", "README.md")
        # An edit to ordinary narrative only, never a deletion or symlink/mode swap.
        return all(r.returncode == 0 and
                   re.fullmatch(rb"100644 blob [0-9a-f]{40,64}\tREADME\.md\n", r.stdout)
                   for r in (before, after))
    except (OSError, subprocess.SubprocessError):
        return False


def classify_event(root, name, event):
    try:
        if name == "pull_request":
            base = event["pull_request"]["base"]["sha"]
            head = event["pull_request"]["head"]["sha"]
            if not all(isinstance(s, str) and SHA.fullmatch(s) for s in (base, head)):
                return False
            common = git(root, "merge-base", base, head)
            if common.returncode != 0:
                return False
            base = common.stdout.decode("ascii").strip()
        elif name == "push":
            base, head = event["before"], event["after"]
        else:
            return False
        return readme_only(root, base, head)
    except (KeyError, TypeError, ValueError, OSError, subprocess.SubprocessError):
        return False


def main():
    try:
        event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text(encoding="utf-8"))
        fast = classify_event(ROOT, os.environ.get("GITHUB_EVENT_NAME", ""), event)
    except (KeyError, OSError, ValueError):
        fast = False
    value = str(fast).lower()
    print("README-only fast path: " + value)
    with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8", newline="\n") as out:
        out.write("readme_only=" + value + "\n")


if __name__ == "__main__":
    main()
