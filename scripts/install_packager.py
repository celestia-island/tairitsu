#!/usr/bin/env python3
"""
Cross-platform installer for the tairitsu CLI binary.

Copies the compiled tairitsu (or tairitsu.exe on Windows) from
target/release/ into the user's Cargo bin directory.

Usage:
    python3 scripts/install_packager.py [--source <path>] [--quick]
"""

import os
import shutil
import subprocess
import sys
import time
from pathlib import Path


def main():
    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent

    is_windows = sys.platform == "win32"
    exe_name = "tairitsu.exe" if is_windows else "tairitsu"

    source = project_root / "target" / "release" / exe_name
    quick = False

    i = 1
    while i < len(sys.argv):
        arg = sys.argv[i]
        if arg == "--source" and i + 1 < len(sys.argv):
            source = Path(sys.argv[i + 1])
            i += 2
            continue
        if arg == "--quick":
            quick = True
            i += 1
            continue
        i += 1

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

    if quick:
        stamp = project_root / "target" / ".tairitsu-install-stamp"
        if stamp.exists() and dest.exists():
            stamp_mtime = stamp.stat().st_mtime
            src_mtime = source.stat().st_mtime
            if stamp_mtime >= src_mtime:
                sys.exit(0)
        if not dest.exists():
            print("[ERROR] tairitsu CLI not installed. Run without --quick first.")
            sys.exit(1)

    try:
        shutil.copy2(str(source), str(dest))
    except PermissionError:
        if is_windows and dest.exists():
            old = dest.with_suffix(".old.exe")
            if old.exists():
                try:
                    old.unlink()
                except PermissionError:
                    pass
            dest.rename(old)
            try:
                shutil.copy2(str(source), str(dest))
            except PermissionError:
                eprint = lambda msg: print(f"[ERROR] {msg}", file=sys.stderr)
                eprint(f"Cannot replace '{dest}' — it is locked by a running process.")
                eprint("Stop the daemon first:  just dev --daemon --shutdown")
                sys.exit(1)
        else:
            raise
    stamp = project_root / "target" / ".tairitsu-install-stamp"
    stamp.write_text(str(source.resolve()))
    print(f"[OK] Installed '{exe_name}' CLI to {bin_dir}")


if __name__ == "__main__":
    main()
