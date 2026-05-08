#!/usr/bin/env python3
"""
Cross-platform dev installer for tairitsu CLI + MCP server + plugin binaries.

For CLI binaries (tairitsu, tairitsu-mcp): atomically copies to ~/.cargo/bin/
(because symlinks may not work on all platforms for cargo-installed tools).

For plugin binaries: creates symlinks from target/release/ into the plugin
directory so that rebuilds are reflected immediately. The production
tairitsu-mcp will replace these with downloaded binaries on first run
(unless --disable virtual-browser is set).

Usage:
    python3 scripts/install_packager.py [--quick]
"""

import os
import shutil
import stat
import sys
from pathlib import Path


def cargo_bin_dir():
    is_windows = sys.platform == "win32"
    cargo_home = os.environ.get("CARGO_HOME")
    if cargo_home:
        return Path(cargo_home) / "bin"
    if is_windows:
        return Path(os.environ.get("USERPROFILE", "")) / ".cargo" / "bin"
    return Path.home() / ".cargo" / "bin"


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
    print(f"[OK] Installed '{label}' → {dest}")


def symlink_plugin(source: Path, dest: Path, label: str):
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.is_symlink():
        target = os.readlink(str(dest))
        if dest.resolve() == source.resolve():
            print(f"[OK] Symlink '{label}' already points to {source.name}")
            return
        dest.unlink()
    elif dest.exists():
        dest.unlink()
    try:
        os.symlink(str(source), str(dest))
    except OSError as e:
        print(f"[WARN] Symlink failed ({e}), falling back to copy")
        shutil.copy2(str(source), str(dest))
        if sys.platform != "win32":
            st = dest.stat()
            dest.chmod(st.st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    print(f"[OK] Symlink '{label}' → {source}")


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

    # 1) tairitsu CLI — copy (needs to be standalone binary in PATH)
    tairitsu_src = release / f"tairitsu{exe}"
    if tairitsu_src.exists():
        install_binary(tairitsu_src, cargo_bin_dir() / f"tairitsu{exe}", "tairitsu")
    else:
        print(f"[WARN] tairitsu not found: {tairitsu_src}")

    # 2) tairitsu-mcp — copy
    mcp_src = release / f"tairitsu-mcp{exe}"
    if mcp_src.exists():
        install_binary(mcp_src, cargo_bin_dir() / f"tairitsu-mcp{exe}", "tairitsu-mcp")
    else:
        print(f"[WARN] tairitsu-mcp not found: {mcp_src}")

    # 3) Plugin binaries — symlink (dev builds update automatically;
    #    production tairitsu-mcp will replace with downloaded binary)
    pdir = plugin_dir()
    plugin_src = release / f"tairitsu-plugin-debug-browser{exe}"
    if plugin_src.exists():
        dest = pdir / f"tairitsu-plugin-debug-browser{exe}"
        symlink_plugin(plugin_src, dest, "plugin: debug-browser")
    else:
        print(f"[WARN] Plugin not found: {plugin_src}")

    stamp = project_root / "target" / ".tairitsu-install-stamp"
    stamp.write_text(str(release.resolve()))


if __name__ == "__main__":
    main()
