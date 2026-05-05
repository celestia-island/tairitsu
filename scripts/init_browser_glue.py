#!/usr/bin/env python3
"""
Cross-platform Node.js dependency installer for packages/browser-glue.

Detects pnpm > yarn > npm, skips if node_modules is already populated.

Usage:
    python3 scripts/init_browser_glue.py [--force]
"""

import os
import shutil
import subprocess
import sys
from pathlib import Path


def find_package_manager() -> str | None:
    for pm in ("pnpm", "yarn", "npm"):
        if shutil.which(pm):
            return pm
    return None


def main():
    force = "--force" in sys.argv

    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    glue_dir = project_root / "packages" / "browser-glue"
    nm_dir = glue_dir / "node_modules"

    if not glue_dir.exists():
        print(f"[ERROR] Directory not found: {glue_dir}")
        sys.exit(1)

    if not force and nm_dir.is_dir() and any(nm_dir.iterdir()):
        print(f"  [OK] Node deps ready  ({nm_dir})")
        return

    pm = find_package_manager()
    if not pm:
        print("[ERROR] No package manager found (install pnpm, yarn, or npm)")
        sys.exit(1)

    print(f"  ->  {pm} install  ({glue_dir})")
    result = subprocess.run(
        [pm, "install"],
        cwd=str(glue_dir),
    )
    if result.returncode != 0:
        print(f"[ERROR] '{pm} install' failed with exit code {result.returncode}")
        sys.exit(result.returncode)

    print(f"  [OK] Node deps installed via {pm}")


if __name__ == "__main__":
    main()
