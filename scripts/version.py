#!/usr/bin/env python3
"""
Semantic version management for Tairitsu project.

This script manages version bumps following semantic versioning,
updates CHANGELOG.md, and creates version tags.
"""

import argparse
import re
import subprocess
from pathlib import Path
from datetime import datetime

# Version file path
VERSION_FILE = Path(__file__).parent.parent / "VERSION"
CARGO_TOML = Path(__file__).parent.parent / "Cargo.toml"


def get_current_version():
    """Get current version from VERSION file."""
    if VERSION_FILE.exists():
        return VERSION_FILE.read_text().strip()
    return "0.0.0"


def set_version(version):
    """Write version to VERSION file."""
    VERSION_FILE.write_text(version + "\n")
    print(f"Version set to {version}")


def bump_version(current, bump_type):
    """Bump version based on type."""
    parts = current.lstrip('v').split('.')
    if len(parts) != 3:
        raise ValueError(f"Invalid version format: {current}")

    major, minor, patch = int(parts[0]), int(parts[1]), int(parts[2])

    if bump_type == "major":
        major += 1
        minor = 0
        patch = 0
    elif bump_type == "minor":
        minor += 1
        patch = 0
    elif bump_type == "patch":
        patch += 1
    else:
        raise ValueError(f"Invalid bump type: {bump_type}")

    return f"{major}.{minor}.{patch}"


def update_cargo_toml(version):
    """Update version in workspace Cargo.toml."""
    content = CARGO_TOML.read_text()

    # Update workspace.package.version
    pattern = r'(\[workspace\.package\]\s*\n\s*version\s*=\s*)"[^"]+"'
    replacement = f'\\1"{version}"'
    content = re.sub(pattern, replacement, content)

    CARGO_TOML.write_text(content)
    print(f"Updated {CARGO_TOML}")


def generate_changelog_version(version):
    """Add version header to CHANGELOG.md."""
    changelog = Path(__file__).parent.parent / "CHANGELOG.md"
    content = changelog.read_text()

    # Check if version already exists
    if f"## [{version}]" in content or f"## {version}" in content:
        print(f"Version {version} already in CHANGELOG.md")
        return

    # Add version header at the top
    date = datetime.now().strftime('%Y-%m-%d')
    new_header = f"# [{version}] - {date}\n\n"

    # Replace the first header with versioned one
    if content.startswith("# Changes since"):
        content = new_header + content
    elif content.startswith("# Changelog"):
        content = new_header + content

    changelog.write_text(content)
    print(f"Updated {changelog}")


def create_tag(version):
    """Create git tag for version."""
    tag = f"v{version}"
    subprocess.run(["git", "tag", "-a", tag, "-m", f"Release {tag}"], check=True)
    print(f"Created tag: {tag}")


def commit_release(version):
    """Commit release changes."""
    subprocess.run([
        "git", "add",
        str(VERSION_FILE),
        str(CARGO_TOML),
        "CHANGELOG.md"
    ], check=True)

    subprocess.run([
        "git", "commit",
        "-m", f"chore(release): {version}"
    ], check=True)

    print(f"Committed release {version}")


def main():
    parser = argparse.ArgumentParser(description="Manage Tairitsu version")
    subparsers = parser.add_subparsers(dest="command", help="Command")

    # Show current version
    subparsers.add_parser("show", help="Show current version")

    # Bump version
    bump_parser = subparsers.add_parser("bump", help="Bump version")
    bump_parser.add_argument(
        "type",
        choices=["major", "minor", "patch"],
        help="Version bump type"
    )
    bump_parser.add_argument(
        "--no-commit",
        action="store_true",
        help="Skip git commit"
    )
    bump_parser.add_argument(
        "--no-tag",
        action="store_true",
        help="Skip git tag"
    )

    # Set specific version
    set_parser = subparsers.add_parser("set", help="Set specific version")
    set_parser.add_argument("version", help="Version to set (e.g., 1.0.0)")

    args = parser.parse_args()

    if args.command == "show":
        print(get_current_version())

    elif args.command == "bump":
        current = get_current_version()
        new_version = bump_version(current, args.type)

        print(f"Bumping version: {current} -> {new_version}")

        set_version(new_version)
        update_cargo_toml(new_version)
        generate_changelog_version(new_version)

        if not args.no_commit:
            commit_release(new_version)

        if not args.no_tag:
            create_tag(new_version)

    elif args.command == "set":
        # Validate version format
        if not re.match(r'^\d+\.\d+\.\d+', args.version):
            raise ValueError(f"Invalid version format: {args.version}")

        print(f"Setting version: {args.version}")
        set_version(args.version)
        update_cargo_toml(args.version)
        generate_changelog_version(args.version)

    else:
        parser.print_help()


if __name__ == "__main__":
    main()
