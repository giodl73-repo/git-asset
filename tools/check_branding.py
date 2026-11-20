"""Audit product naming and frozen existing-data identifiers; development only."""
from pathlib import Path
from collections import Counter
import argparse
import io
import json
import re
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parent.parent
BASELINE = "4ce4da04cd92947a49b8f6c33c0f37ccfe8751b9"
CRATES = {"tig-core": "core", "tig-store": "store", "tig-assets": "model", "tig-git": "git-asset"}
# Existing schema/hash/ref spellings are data compatibility, not product names.
IDENTITIES = re.compile(
    r"urn:tig:[A-Za-z0-9_.:-]+|\btig\.[A-Za-z0-9_.:-]*|TIG\\0|"
    r"refs/tig/proposals\b|\btig-(?:delivery-policy|revision-history|delivery-history|selected-revision)\b|"
    r'''(?:(?<=[/"'`\\\s])|^)\.[tT][iI][gG](?![A-Za-z0-9_.-])''', re.M)
NOT_IDENTITIES = {"tig.git", "tig.yml", "tig.path", "tig.args", "tig.direction", "tig.concurrent", "tig."}
LEGACY = re.compile(
    r"urn:tig:[A-Za-z0-9_.:-]+|\btig\.[A-Za-z0-9_.:-]*|TIG\\0|"
    r"refs/tig/proposals\b|\btig-(?:delivery-policy|revision-history|delivery-history|selected-revision)\b|"
    r"\.tig-restore-|\.tig\b", re.I)
BRAND = re.compile(r"(?<![A-Za-z0-9])tig(?=[^a-z]|$)", re.I)
CAMEL = re.compile(r"\b[A-Za-z_]*Tig[A-Z][A-Za-z0-9_]*")


def renamed(name):
    parts = name.split("/")
    if len(parts) > 1 and parts[0] == "crates" and parts[1] in CRATES:
        parts[1] = CRATES[parts[1]]
    return "/".join(p.replace("tig-", "git-asset-").replace("tig_", "git_asset_").replace("TIG", "GIT-ASSET") for p in parts)


def tokens(text):
    return [v for v in IDENTITIES.findall(text) if v not in NOT_IDENTITIES]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", default=BASELINE)
    args = parser.parse_args()
    if not re.fullmatch(r"[0-9a-f]{40}", args.baseline):
        parser.error("baseline must be an immutable full commit SHA")
    names = set()
    for command in (["git", "ls-files", "-z"], ["git", "ls-files", "--others", "--exclude-standard", "-z"]):
        names.update(subprocess.check_output(command, cwd=ROOT).decode().split("\0"))
    pins = json.loads((ROOT / "tools/vendor/pins.json").read_text(encoding="utf-8"))
    upstream = {entry["path"] for entry in pins["files"]}
    allowed = upstream | {"docs/COMPATIBILITY.md", "tools/check_branding.py"}
    errors, current = [], {}
    for name in sorted(names - {""}):
        path = ROOT / name
        if not path.is_file():
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            errors.append(f"non-text artifact in source inventory: {name}")
            continue
        current[name] = text
        if name in allowed:
            continue
        if BRAND.search(name) or CAMEL.search(name):
            errors.append(f"old product in path: {name}")
        for number, line in enumerate(text.splitlines(), 1):
            if name == ".github/workflows/validate.yml" and ("$baselineRoot/crates/" in line or '$baselineBuild "debug/' in line):
                continue  # Historical artifacts must match their pinned commits.
            cleaned = LEGACY.sub(lambda m: m[0] if m[0].lower() in NOT_IDENTITIES - {"tig."} else "", line)
            old_url = re.search(r"github.com/giodl73-repo/tig(?:$|[./\s])", line)
            if old_url or BRAND.search(cleaned) or CAMEL.search(cleaned):
                errors.append(f"old branding outside compatibility at {name}:{number}")
    archive = subprocess.check_output(["git", "archive", "--format=tar", args.baseline], cwd=ROOT)
    checked = 0
    with tarfile.open(fileobj=io.BytesIO(archive)) as tree:
        for member in tree:
            if not member.isfile() or member.name in upstream:
                continue
            # Runtime/fixture identifiers survive in their renamed files. Prose
            # may move into bounded review chapters without changing stored data.
            if not member.name.endswith((".rs", ".json", ".js", ".cjs", ".mjs", ".py", ".ps1", ".yml")):
                continue
            expected = tokens(tree.extractfile(member).read().decode("utf-8"))
            if not expected:
                continue
            target = renamed(member.name)
            if Counter(expected) - Counter(tokens(current.get(target, ""))):
                errors.append(f"frozen existing-data identifier changed: {target}")
            checked += len(expected)
    if errors:
        for error in errors[:80]:
            print(error)
        print(f"FAIL: {len(errors)} rename/compatibility findings")
        return 1
    print(f"PASS git-asset branding; {checked} frozen runtime/fixture identifier occurrences preserved")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
