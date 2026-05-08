#!/usr/bin/env python3
"""
Cross-platform installer for tairitsu CLI + MCP server + plugin binaries.

Copies compiled binaries from target/release/ into the user's Cargo bin
directory and the plugin directory.

Usage:
    python3 scripts/install_packager.py [--source <path>] [--quick]
"""

import os
import shutil
import stat
import subprocess
import sys
import time
from pathlib import Path


def cargo_bin_dir():
    is_windows = sys.platform == "win32"
    cargo_home = os.environ.get("CARGO_HOME")
    if cargo_home:
        return Path(cargo_home) / "bin"
    if is_windows:
        return Path(os.environ.get("USERPROFILE", "")) / ".cargo" / "bin"
    return Path.home() / ".cargo" / "bin"


def install_binary(source: Path, dest: Path, label: str):
    dest.parent.mkdir(parents=True, exist_ok=True)
    try:
        tmp = dest.with_suffix(".tmp")
        if tmp.exists():
            tmp.unlink()
        shutil.copy2(str(source), str(tmp))
        os.replace(str(tmp), str(dest))
    except PermissionError:
        print(f"[ERROR] Cannot replace '{dest}' — permission denied.", file=sys.stderr)
        sys.exit(1)
    except OSError as e:
        print(f"[ERROR] Cannot replace '{dest}' — {e}", file=sys.stderr)
        sys.exit(1)
    print(f"[OK] Installed '{label}' to {dest.parent}")


def plugin_dir():
    d = os.environ.get("TAIRITSU_PLUGIN_DIR")
    if d:
        p = Path(d)
        p.mkdir(parents=True, exist_ok=True)
        return p

    if sys.platform == "linux":
        base = Path.home() / ".local" / "share"
    elif sys.platform == "darwin":
        base = Path.home() / "Library" / "Application Support"
    else:
        base = Path(os.environ.get("LOCALAPPDATA", Path.home() / ".local"))

    p = base / "tairitsu" / "plugins"
    p.mkdir(parents=True, exist_ok=True)
    return p


def main():
    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    release = project_root / "target" / "release"

    is_windows = sys.platform == "win32"
    exe = ".exe" if is_windows else ""

    quick = "--quick" in sys.argv

    if quick:
        stamp = project_root / "target" / ".tairitsu-install-stamp"
        tairitsu_bin = cargo_bin_dir() / f"tairitsu{exe}"
        if stamp.exists() and tairitsu_bin.exists():
            stamp_mtime = stamp.stat().st_mtime
            src_mtime = (release / f"tairitsu{exe}").stat().st_mtime
            if stamp_mtime >= src_mtime:
                sys.exit(0)
        if not tairitsu_bin.exists():
            print("[ERROR] tairitsu CLI not installed. Run without --quick first.")
            sys.exit(1)

    # 1) tairitsu CLI
    tairitsu_src = release / f"tairitsu{exe}"
    if tairitsu_src.exists():
        install_binary(tairitsu_src, cargo_bin_dir() / f"tairitsu{exe}", "tairitsu")
    else:
        print(f"[WARN] tairitsu binary not found: {tairitsu_src}")

    # 2) tairitsu-mcp
    mcp_src = release / f"tairitsu-mcp{exe}"
    if mcp_src.exists():
        install_binary(mcp_src, cargo_bin_dir() / f"tairitsu-mcp{exe}", "tairitsu-mcp")
    else:
        print(f"[WARN] tairitsu-mcp binary not found: {mcp_src}")

    # 3) Plugin binaries
    pdir = plugin_dir()
    plugin_src = release / f"tairitsu-plugin-debug-browser{exe}"
    if plugin_src.exists():
        dest = pdir / f"tairitsu-plugin-debug-browser{exe}"
        install_binary(plugin_src, dest, "plugin: debug-browser")
        if not is_windows:
            st = dest.stat()
            dest.chmod(st.st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    else:
        print(f"[WARN] Plugin binary not found: {plugin_src}")

    stamp = project_root / "target" / ".tairitsu-install-stamp"
    stamp.write_text(str(release.resolve()))


if __name__ == "__main__":
    main()
