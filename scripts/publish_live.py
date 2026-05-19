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

    run(["npm", "config", "set", "//registry.npmjs.org/:_authToken", token])

    glue_dir = REPO_ROOT / "packages" / "npm" / "celestia-tairitsu-web-glue"

    for _ in range(2):
        run(["npm", "run", "build"], cwd=str(glue_dir))
    run(["npm", "run", "build:production"], cwd=str(glue_dir))

    run(["npm", "publish", "--access", "public"], cwd=str(glue_dir))

    for d in sorted((REPO_ROOT / "packages" / "npm").glob("glue-*/")):
        run(["npm", "publish", "--access", "public"], cwd=str(d))

    for d in sorted((REPO_ROOT / "packages" / "npm").glob("*-wasm/")):
        run(["npm", "publish", "--access", "public"], cwd=str(d))

    print("All npm packages published (LIVE)!")


if __name__ == "__main__":
    main()
