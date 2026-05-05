#!/usr/bin/env python3
"""
WIT Package Version Management

Handles version bumping and changelog generation for WIT packages.
"""

from __future__ import annotations

import argparse
import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional


# WIT package version (bump this when breaking changes occur)
WIT_VERSION = "0.2.0"

# Version history file path
VERSION_HISTORY_PATH = Path("packages/browser-worlds/WIT_VERSIONS.json")


def load_version_history() -> Dict:
    """Load version history from JSON file."""
    if VERSION_HISTORY_PATH.exists():
        with open(VERSION_HISTORY_PATH, "r") as f:
            return json.load(f)
    return {
        "current_version": WIT_VERSION,
        "history": []
    }


def save_version_history(history: Dict) -> None:
    """Save version history to JSON file."""
    VERSION_HISTORY_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(VERSION_HISTORY_PATH, "w") as f:
        json.dump(history, f, indent=2)


def get_webref_commit() -> Optional[str]:
    """Get the latest w3c/webref commit hash from cache."""
    cache_info = Path("target/tairitsu-wit/webidl-cache/_webref_info.json")
    if cache_info.exists():
        with open(cache_info, "r") as f:
            data = json.load(f)
            return data.get("commit", data.get("last_commit"))
    return None


def detect_wit_changes() -> List[str]:
    """Detect which WIT files have changed since last generation."""
    changes = []
    wit_dir = Path("packages/browser-worlds/wit/generated")

    if not wit_dir.exists():
        return changes

    # Compare with git to detect changes
    import subprocess
    try:
        result = subprocess.run(
            ["git", "diff", "--name-only", "HEAD", str(wit_dir)],
            capture_output=True,
            text=True,
            check=False
        )
        if result.returncode == 0:
            changed_files = result.stdout.strip().split("\n")
            changes = [f for f in changed_files if f.endswith(".wit")]
    except Exception:
        pass

    return changes


def generate_changelog_entry(version: str, changes: List[str]) -> Dict:
    """Generate a changelog entry for a version."""
    return {
        "version": version,
        "date": datetime.now(timezone.utc).isoformat(),
        "webref_commit": get_webref_commit(),
        "changes": changes,
        "domains_affected": sorted(set(
            Path(f).stem for f in changes if f
        ))
    }


def bump_version(part: str = "patch") -> str:
    """Bump version by part (major, minor, patch)."""
    current = WIT_VERSION
    major, minor, patch = current.split(".")
    patch = int(patch)

    if part == "major":
        return f"{int(major) + 1}.0.0"
    elif part == "minor":
        return f"{major}.{int(minor) + 1}.0"
    else:  # patch
        return f"{major}.{minor}.{patch + 1}"


def update_wit_version(new_version: str) -> None:
    """Update the WIT version in generate_browser_wit.py"""
    script_path = Path("scripts/generate_browser_wit.py")
    if not script_path.exists():
        return

    content = script_path.read_text()
    # Update WIT_VERSION constant
    content = re.sub(
        r'WIT_VERSION = "[^"]*"',
        f'WIT_VERSION = "{new_version}"',
        content
    )
    script_path.write_text(content)


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        "command",
        choices=["bump", "changelog", "status"],
        help="Command to run"
    )
    parser.add_argument(
        "--part",
        choices=["major", "minor", "patch"],
        default="patch",
        help="Version part to bump (default: patch)"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be done without making changes"
    )
    args = parser.parse_args()

    if args.command == "status":
        history = load_version_history()
        print(f"Current WIT version: {history['current_version']}")
        webref = get_webref_commit()
        if webref:
            print(f"w3c/webref commit: {webref}")
        changes = detect_wit_changes()
        if changes:
            print(f"\nUncommitted changes ({len(changes)} files):")
            for f in changes:
                print(f"  - {f}")
        else:
            print("\nNo uncommitted WIT changes.")
        return

    if args.command == "bump":
        new_version = bump_version(args.part)
        print(f"Bumping version: {WIT_VERSION} -> {new_version}")

        if args.dry_run:
            print("[DRY RUN] Would update version in generate_browser_wit.py")
            return

        update_wit_version(new_version)

        history = load_version_history()
        entry = {
            "version": new_version,
            "date": datetime.now(timezone.utc).isoformat(),
            "bump_type": args.part,
            "previous_version": WIT_VERSION,
            "webref_commit": get_webref_commit(),
        }
        history["history"].append(entry)
        history["current_version"] = new_version
        save_version_history(history)

        print(f"Updated to version {new_version}")
        print(f"Run 'just wit-gen' to regenerate WIT files with new version")

    elif args.command == "changelog":
        history = load_version_history()
        print(f"# WIT Package Changelog")
        print(f"\nCurrent version: {history['current_version']}")
        print(f"\n## Version History\n")

        for entry in reversed(history["history"][-10:]):
            print(f"### {entry['version']} ({entry['date']})")
            if "bump_type" in entry:
                print(f"Type: {entry['bump_type']}")
            if "webref_commit" in entry:
                print(f"w3c/webref: {entry['webref_commit'][:8]}")
            if "changes" in entry:
                print(f"Changed domains: {', '.join(entry['domains_affected'])}")
            print()


if __name__ == "__main__":
    main()
