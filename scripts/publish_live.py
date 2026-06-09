#!/usr/bin/env python3
"""Publish npm packages live (requires NPM_TOKEN env var)."""
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent


def run(cmd, **kw):
    print(f"  → {' '.join(cmd)}")
    subprocess.check_call(cmd, **kw)


def main():
    token = os.environ.get("NPM_TOKEN")
    if not token:
        print("Error: NPM_TOKEN environment variable is not set.", file=sys.stderr)
        sys.exit(1)

    # Use a temporary .npmrc instead of overwriting the user's
    temp_npmrc = REPO_ROOT / ".npmrc.temp"
    temp_npmrc.write_text(f"//registry.npmjs.org/:_authToken={token}\n")
    temp_npmrc.chmod(0o600)

    env = os.environ.copy()
    env["NPM_CONFIG_USERCONFIG"] = str(temp_npmrc)

    glue_dir = REPO_ROOT / "packages" / "npm" / "celestia-tairitsu-web-glue"

    run(["npm", "run", "build"], cwd=str(glue_dir), env=env)
    run(["npm", "run", "build:production"], cwd=str(glue_dir), env=env)

    run(["npm", "publish", "--access", "public"], cwd=str(glue_dir), env=env)

    for d in sorted((REPO_ROOT / "packages" / "npm").glob("glue-*/")):
        run(["npm", "publish", "--access", "public"], cwd=str(d), env=env)

    for d in sorted((REPO_ROOT / "packages" / "npm").glob("*-wasm/")):
        run(["npm", "publish", "--access", "public"], cwd=str(d), env=env)

    temp_npmrc.unlink(missing_ok=True)


if __name__ == "__main__":
    main()
