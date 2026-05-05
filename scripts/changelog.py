#!/usr/bin/env python3
"""
Generate CHANGELOG.md from git commit history.

This script analyzes git commits and generates a formatted changelog
following conventional commit format.
"""

import argparse
import re
import subprocess
from datetime import datetime
from pathlib import Path
from collections import defaultdict

# Commit types in order of importance
COMMIT_TYPES = {
    "feat": "Features",
    "fix": "Bug Fixes",
    "perf": "Performance",
    "refactor": "Refactoring",
    "docs": "Documentation",
    "test": "Tests",
    "chore": "Chores",
    "style": "Style",
    "ci": "CI/CD",
    "build": "Build",
    "revert": "Reverts",
}

# Commit types to include in changelog (in order)
INCLUDED_TYPES = [
    "feat",
    "fix",
    "perf",
    "refactor",
    "docs",
    "test",
    "chore",
    "style",
    "ci",
    "build",
    "revert",
]

def get_git_tags():
    """Get all git tags sorted by version."""
    result = subprocess.run(
        ["git", "tag", "--list", "--sort=-v:refname"],
        capture_output=True,
        text=True,
        check=True,
    )
    tags = result.stdout.strip().split('\n') if result.stdout.strip() else []
    return tags


def get_commits_since_tag(tag=None):
    """Get commits since a tag (or all commits if no tag)."""
    if tag:
        result = subprocess.run(
            ["git", "log", f"{tag}..HEAD", "--pretty=format:%H|%s|%an"],
            capture_output=True,
            text=True,
            check=True,
        )
    else:
        result = subprocess.run(
            ["git", "log", "--pretty=format:%H|%s|%an", "-100"],
            capture_output=True,
            text=True,
            check=True,
        )
    commits = []
    for line in result.stdout.strip().split('\n'):
        if line:
            parts = line.split('|', 2)
            if len(parts) >= 3:
                hash_val = parts[0]
                subject = parts[1]
                author = parts[2]
                commits.append({
                    "hash": hash_val,
                    "subject": subject,
                    "body": "",
                    "author": author,
                    "author_email": "",
                })
            elif len(parts) == 2:
                commits.append({
                    "hash": parts[0],
                    "subject": parts[1],
                    "body": "",
                    "author": "Unknown",
                    "author_email": "",
                })
    return commits


def parse_commit_type(subject):
    """Parse commit type from conventional commit format."""
    match = re.match(r'^(\w+)(\([^)]+\))?!?:\s+', subject)
    if match:
        return match.group(1)
    return None


def parse_commit_scope(subject):
    """Parse commit scope from conventional commit format."""
    match = re.match(r'^\w+\(([^)]+)\)!?:\s+', subject)
    if match:
        return match.group(1)
    return None


def parse_commit_breaking(subject):
    """Check if commit is a breaking change."""
    return '!' in subject


def parse_commit_body(body):
    """Parse structured information from commit body."""
    info = {
        "breaking": [],
        "closes": [],
        "issues": [],
    }
    if not body:
        return info

    for line in body.split('\n'):
        line = line.strip()
        if line.startswith("BREAKING CHANGE:"):
            info["breaking"].append(line.replace("BREAKING CHANGE:", "").strip())
        elif line.startswith("Closes:"):
            issues = re.findall(r'#(\d+)', line)
            info["closes"].extend(issues)
        elif line.startswith("Closes "):
            issues = re.findall(r'#(\d+)', line)
            info["closes"].extend(issues)
        elif line.startswith("Fixes:"):
            issues = re.findall(r'#(\d+)', line)
            info["issues"].extend(issues)
        elif line.startswith("Fixes "):
            issues = re.findall(r'#(\d+)', line)
            info["issues"].extend(issues)
    return info


def generate_changelog(since_tag=None):
    """Generate changelog markdown from git commits."""
    commits = get_commits_since_tag(since_tag)

    # Group commits by type and scope
    sections = defaultdict(lambda: {"commits": []})

    for commit in commits:
        commit_type = parse_commit_type(commit["subject"])
        if commit_type not in INCLUDED_TYPES:
            continue

        scope = parse_commit_scope(commit["subject"])
        breaking = parse_commit_breaking(commit["subject"])
        body_info = parse_commit_body(commit["body"])

        # Format commit message
        message = commit["subject"]
        if scope:
            message = f"**{scope}**: {message.split(':', 1)[1] if ':' in message else message}"
        if breaking:
            message = f"{message} **BREAKING**"

        sections[commit_type]["commits"].append({
            "hash": commit["hash"][:8],
            "message": message,
            "author": commit["author"],
            "breaking": breaking,
            "closes": body_info["closes"],
        })

    # Generate markdown
    md = []

    # Header
    if since_tag:
        md.append(f"# Changes since {since_tag}")
    else:
        md.append("# Changelog")
    md.append("")
    md.append(f"Generated on {datetime.now().strftime('%Y-%m-%d')}")
    md.append("")

    # Sections
    for commit_type in INCLUDED_TYPES:
        section = sections.get(commit_type)
        if not section or not section["commits"]:
            continue

        # Section title
        type_name = COMMIT_TYPES.get(commit_type, commit_type.capitalize())
        md.append(f"## {type_name}")
        md.append("")

        # Commits
        for commit in section["commits"]:
            hash_short = commit["hash"]
            message = commit["message"]
            author = commit["author"]

            md.append(f"- {message} (`{hash_short}`)")
            if commit["breaking"]:
                md.append("")
                md.append("  **BREAKING CHANGE**")
                md.append("")
            if commit["closes"]:
                closes = ", ".join([f"#{i}" for i in commit["closes"]])
                md.append(f"  Closes {closes}")
            md.append("")

        md.append("")

    # Footer
    md.append("---")
    md.append("")
    md.append("The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),")
    md.append("and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).")

    return "\n".join(md)


def write_changelog(output_path):
    """Generate and write changelog to file."""
    # Try to find the latest version tag
    tags = get_git_tags()
    since_tag = tags[0] if tags else None

    md = generate_changelog(since_tag)

    output_path.write_text(md)
    print(f"Changelog written to {output_path}")


def main():
    parser = argparse.ArgumentParser(description="Generate CHANGELOG.md from git history")
    parser.add_argument(
        "--output",
        "-o",
        default="CHANGELOG.md",
        help="Output file path (default: CHANGELOG.md)",
    )
    args = parser.parse_args()

    output_path = Path(args.output)
    write_changelog(output_path)


if __name__ == "__main__":
    main()
