#!/usr/bin/env python3
"""
Cross-platform installer for the tairitsu CLI binary.

Copies the compiled tairitsu (or tairitsu.exe on Windows) from
target/release/ into the user's Cargo bin directory.

Usage:
    python3 scripts/install_packager.py [--source <path>]
"""

import os
import shutil
import sys
from pathlib import Path


def main():
    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent

    is_windows = sys.platform == "win32"
    exe_name = "tairitsu.exe" if is_windows else "tairitsu"

    source = project_root / "target" / "release" / exe_name

    for i, arg in enumerate(sys.argv):
        if arg == "--source" and i + 1 < len(sys.argv):
            source = Path(sys.argv[i + 1])

    if not source.exists():
        print(f"[ERROR] Binary not found: {source}")
        print("  Run 'cargo build --release --package tairitsu-packager' first.")
        sys.exit(1)

    cargo_home = os.environ.get("CARGO_HOME")
    if cargo_home:
        bin_dir = Path(cargo_home) / "bin"
    elif is_windows:
        userprofile = os.environ.get("USERPROFILE", "")
        bin_dir = Path(userprofile) / ".cargo" / "bin"
    else:
        bin_dir = Path.home() / ".cargo" / "bin"

    bin_dir.mkdir(parents=True, exist_ok=True)
    dest = bin_dir / exe_name

    shutil.copy2(str(source), str(dest))
    print(f"[OK] Installed '{exe_name}' CLI to {bin_dir}")


if __name__ == "__main__":
    main()
