#!/usr/bin/env python3
"""Prepare SemVer for Runa GitHub Releases (S5).

Rules (docs/COMMANDS.md § Release):
- Source of truth runtime: apps/desktop/src-tauri/tauri.conf.json
- Cargo workspace + package.json must match, else fail
- Compare to latest GitHub Release:
  - no release or repo > latest → keep repo version
  - repo == latest → bump minor X.Y.Z → X.(Y+1).0
  - repo < latest → fail (no downgrade)
- Tag vX.Y.Z must not already exist

Usage:
  python scripts/prepare_version.py --dry-run
  python scripts/prepare_version.py --apply
  python scripts/prepare_version.py --apply --write-tag  # also create annotated tag locally
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TAURI_CONF = ROOT / "apps" / "desktop" / "src-tauri" / "tauri.conf.json"
PACKAGE_JSON = ROOT / "apps" / "desktop" / "package.json"
CARGO_TOML = ROOT / "Cargo.toml"
CHANGELOG = ROOT / "docs" / "product" / "CHANGELOG.md"

SEMVER_RE = re.compile(r"^(\d+)\.(\d+)\.(\d+)$")


def die(msg: str, code: int = 1) -> None:
    print(f"error: {msg}", file=sys.stderr)
    raise SystemExit(code)


def parse_semver(raw: str) -> tuple[int, int, int]:
    m = SEMVER_RE.match(raw.strip().lstrip("v"))
    if not m:
        die(f"version SemVer invalide: {raw!r}")
    return int(m.group(1)), int(m.group(2)), int(m.group(3))


def fmt_semver(v: tuple[int, int, int]) -> str:
    return f"{v[0]}.{v[1]}.{v[2]}"


def cmp_semver(a: tuple[int, int, int], b: tuple[int, int, int]) -> int:
    return (a > b) - (a < b)


def read_tauri_version() -> str:
    data = json.loads(TAURI_CONF.read_text(encoding="utf-8"))
    ver = data.get("version")
    if not isinstance(ver, str):
        die("tauri.conf.json: champ version manquant")
    return ver


def read_package_version() -> str:
    data = json.loads(PACKAGE_JSON.read_text(encoding="utf-8"))
    ver = data.get("version")
    if not isinstance(ver, str):
        die("package.json: champ version manquant")
    return ver


def read_cargo_workspace_version() -> str:
    text = CARGO_TOML.read_text(encoding="utf-8")
    # [workspace.package] … version = "x.y.z"
    in_ws = False
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            in_ws = stripped == "[workspace.package]"
            continue
        if in_ws and stripped.startswith("version"):
            m = re.match(r'version\s*=\s*"([^"]+)"', stripped)
            if m:
                return m.group(1)
            die("Cargo.toml: version workspace illisible")
    die("Cargo.toml: [workspace.package].version introuvable")


def assert_versions_aligned() -> str:
    tauri_v = read_tauri_version()
    pkg_v = read_package_version()
    cargo_v = read_cargo_workspace_version()
    if not (tauri_v == pkg_v == cargo_v):
        die(
            "versions désalignées — "
            f"tauri={tauri_v} package.json={pkg_v} Cargo.toml={cargo_v}"
        )
    parse_semver(tauri_v)  # validate
    return tauri_v


def gh_latest_release_version() -> str | None:
    """Return latest GitHub release tag without leading v, or None if none."""
    env = os.environ.copy()
    try:
        proc = subprocess.run(
            ["gh", "release", "view", "--json", "tagName", "-q", ".tagName"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
            env=env,
        )
    except FileNotFoundError:
        die("gh CLI introuvable — installer GitHub CLI")
    if proc.returncode != 0:
        err = (proc.stderr or proc.stdout or "").strip().lower()
        # No releases yet
        if "not found" in err or "could not find" in err or "no releases" in err:
            return None
        # Empty repo / no release
        if "release not found" in err:
            return None
        die(f"gh release view échoué: {proc.stderr or proc.stdout}")
    tag = proc.stdout.strip()
    if not tag:
        return None
    return tag.lstrip("v")


def git_tag_exists(tag: str) -> bool:
    try:
        proc = subprocess.run(
            ["git", "rev-parse", "-q", "--verify", f"refs/tags/{tag}"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
    except FileNotFoundError:
        return False
    if proc.returncode == 0:
        return True
    proc2 = subprocess.run(
        ["git", "ls-remote", "--tags", "origin", tag],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    return bool(proc2.stdout.strip())


def bump_minor(v: tuple[int, int, int]) -> tuple[int, int, int]:
    major, minor, _patch = v
    return major, minor + 1, 0


def write_tauri_version(version: str) -> None:
    data = json.loads(TAURI_CONF.read_text(encoding="utf-8"))
    data["version"] = version
    TAURI_CONF.write_text(
        json.dumps(data, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


def write_package_version(version: str) -> None:
    data = json.loads(PACKAGE_JSON.read_text(encoding="utf-8"))
    data["version"] = version
    PACKAGE_JSON.write_text(
        json.dumps(data, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


def write_cargo_workspace_version(version: str) -> None:
    text = CARGO_TOML.read_text(encoding="utf-8")
    lines = text.splitlines(keepends=True)
    out: list[str] = []
    in_ws = False
    replaced = False
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            in_ws = stripped == "[workspace.package]"
            out.append(line)
            continue
        if in_ws and stripped.startswith("version") and not replaced:
            out.append(f'version = "{version}"\n')
            replaced = True
            continue
        out.append(line)
    if not replaced:
        die("impossible de réécrire Cargo.toml version")
    CARGO_TOML.write_text("".join(out), encoding="utf-8")


def promote_changelog(version: str) -> None:
    """Move ## [Unreleased] body under ## [version] — date; leave empty Unreleased."""
    if not CHANGELOG.is_file():
        print("warn: CHANGELOG.md absent — skip", file=sys.stderr)
        return
    text = CHANGELOG.read_text(encoding="utf-8")
    today = date.today().isoformat()
    header = f"## [{version}] — {today}"
    if f"## [{version}]" in text:
        print(f"CHANGELOG: section [{version}] déjà présente — skip promote")
        return
    pattern = re.compile(
        r"(## \[Unreleased\]\s*\n)(.*?)(\n## \[)",
        re.DOTALL,
    )
    m = pattern.search(text)
    if not m:
        # Unreleased at end or only section
        pattern_end = re.compile(r"(## \[Unreleased\]\s*\n)(.*)\Z", re.DOTALL)
        m2 = pattern_end.search(text)
        if not m2:
            print("warn: section [Unreleased] introuvable — skip", file=sys.stderr)
            return
        body = m2.group(2).strip()
        replacement = (
            f"## [Unreleased]\n\n### Added\n\n### Changed\n\n"
            f"{header}\n\n{body}\n"
            if body
            else f"## [Unreleased]\n\n{header}\n\n"
        )
        text = text[: m2.start()] + replacement
        CHANGELOG.write_text(text, encoding="utf-8")
        return
    body = m.group(2).strip()
    new_unreleased = "## [Unreleased]\n\n"
    if body:
        promoted = f"{new_unreleased}{header}\n\n{body}\n\n{m.group(3)}"
    else:
        promoted = f"{new_unreleased}{header}\n\n{m.group(3)}"
    text = text[: m.start()] + promoted + text[m.end() :]
    CHANGELOG.write_text(text, encoding="utf-8")


def git_commit_bump(version: str) -> None:
    files = [
        str(TAURI_CONF.relative_to(ROOT)),
        str(PACKAGE_JSON.relative_to(ROOT)),
        str(CARGO_TOML.relative_to(ROOT)),
        str(CHANGELOG.relative_to(ROOT)),
    ]
    subprocess.run(["git", "add", *files], cwd=ROOT, check=True)
    msg = f"chore(release): bump to {version}"
    subprocess.run(["git", "commit", "-m", msg], cwd=ROOT, check=True)


def git_create_tag(version: str) -> None:
    tag = f"v{version}"
    subprocess.run(
        ["git", "tag", "-a", tag, "-m", f"Release {tag}"],
        cwd=ROOT,
        check=True,
    )


def decide_version(repo_ver: str, latest: str | None) -> tuple[str, bool]:
    """Return (target_version, did_bump)."""
    repo = parse_semver(repo_ver)
    if latest is None:
        return repo_ver, False
    last = parse_semver(latest)
    c = cmp_semver(repo, last)
    if c > 0:
        return repo_ver, False
    if c == 0:
        bumped = bump_minor(repo)
        return fmt_semver(bumped), True
    die(
        f"version dépôt {repo_ver} < dernière release {latest} — "
        "pas de downgrade (bump manuel requis)"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description="Runa prepare-version (S5)")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Affiche la décision sans écrire",
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="Écrit les fichiers et commit éventuel",
    )
    parser.add_argument(
        "--write-tag",
        action="store_true",
        help="Avec --apply : crée le tag annoté local vX.Y.Z",
    )
    parser.add_argument(
        "--latest",
        default=None,
        help="Override dernière release (tests / dry-run sans gh)",
    )
    parser.add_argument(
        "--skip-gh",
        action="store_true",
        help="Ne pas appeler gh (utiliser --latest ou aucune release)",
    )
    args = parser.parse_args()
    if not args.dry_run and not args.apply:
        die("spécifier --dry-run ou --apply")

    repo_ver = assert_versions_aligned()
    if args.skip_gh:
        latest = args.latest
    elif args.latest is not None:
        latest = args.latest
    else:
        latest = gh_latest_release_version()

    target, bumped = decide_version(repo_ver, latest)
    tag = f"v{target}"

    print(f"repo_version={repo_ver}")
    print(f"latest_release={latest or '(aucune)'}")
    print(f"target_version={target}")
    print(f"bumped={str(bumped).lower()}")
    print(f"tag={tag}")

    if git_tag_exists(tag):
        die(f"tag {tag} existe déjà — pas de republish")

    # GitHub Actions outputs
    gh_out = os.environ.get("GITHUB_OUTPUT")
    if gh_out:
        with open(gh_out, "a", encoding="utf-8") as f:
            f.write(f"version={target}\n")
            f.write(f"tag={tag}\n")
            f.write(f"bumped={str(bumped).lower()}\n")

    if args.dry_run:
        return

    if bumped:
        write_tauri_version(target)
        write_package_version(target)
        write_cargo_workspace_version(target)
        promote_changelog(target)
        assert_versions_aligned()
        git_commit_bump(target)
        print(f"commit bump → {target}")
    else:
        # Still promote changelog for a fresh version if Unreleased has content
        # and this version section is missing (first publish of a pre-bumped version).
        promote_changelog(target)

    if args.write_tag:
        git_create_tag(target)
        print(f"tag créé: {tag}")


if __name__ == "__main__":
    main()
