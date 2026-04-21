#!/usr/bin/env python3
"""Build Rust crates as WASM components, optimize with wasm-opt, transpile with jco.

Produces npm-ready packages under packages/npm/{crate-name}-wasm/

Usage:
    python scripts/build_wasm_packages.py                  # Build all
    python scripts/build_wasm_packages.py tairitsu-vdom    # Build one
    python scripts/build_wasm_packages.py --list           # List compilable crates
"""

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

WORKSPACE_ROOT = Path(__file__).resolve().parent.parent
NPM_DIR = WORKSPACE_ROOT / "packages" / "npm"
VERSION = "0.1.0"
SCOPE = "@celestia"

WASM_CRATES = {
    "tairitsu-vdom": {
        "description": "Tairitsu Virtual DOM — reactive VNode system",
        "path": "packages/vdom",
        "wasm_opt": True,
    },
    "tairitsu-hooks": {
        "description": "Tairitsu Hooks — useState, useEffect, useMemo, etc.",
        "path": "packages/hooks",
        "wasm_opt": True,
    },
    "tairitsu-style": {
        "description": "Tairitsu Style — type-safe CSS property builders",
        "path": "packages/style",
        "wasm_opt": True,
    },
}

HIKARI_WASM_CRATES = {
    "hikari-palette": {
        "description": "Hikari Palette — color system and design tokens",
        "path": "../hikari/packages/palette",
        "wasm_opt": True,
    },
}


def find_tool(name: str, extra_names: list = None) -> str | None:
    candidates = [name] + (extra_names or [])
    if os.name == "nt":
        candidates = [c + ".cmd" if not c.endswith(".cmd") else c for c in candidates] + candidates
    for c in candidates:
        try:
            result = subprocess.run(
                [c, "--version"] if name != "wasm-opt" else [c, "--version"],
                capture_output=True, timeout=10, encoding="utf-8", errors="replace",
            )
            if result.returncode == 0:
                return c
        except (FileNotFoundError, subprocess.TimeoutExpired):
            continue
    return None


def build_wasm_crate(crate_name: str, crate_info: dict):
    print(f"\n  Building {crate_name}...")

    crate_path = WORKSPACE_ROOT / crate_info["path"]
    if not crate_path.exists():
        print(f"    SKIP: {crate_path} not found")
        return False

    wasm_stem = crate_name.replace("-", "_")
    pkg_dir = NPM_DIR / f"{crate_name}-wasm"
    dist_dir = pkg_dir / "dist"
    dist_dir.mkdir(parents=True, exist_ok=True)

    # Step 1: cargo build --target wasm32-wasip2 --release
    print(f"    [1/4] cargo build --target wasm32-wasip2 --release ...")
    result = subprocess.run(
        ["cargo", "build", "--target", "wasm32-wasip2", "--release", "--lib", "-p", crate_name],
        cwd=str(WORKSPACE_ROOT),
        capture_output=True, text=True, encoding="utf-8", errors="replace",
        timeout=600,
    )
    if result.returncode != 0:
        print(f"    FAIL: cargo build failed")
        print(f"    {result.stderr[-500:]}")
        return False

    wasm_path = WORKSPACE_ROOT / "target" / "wasm32-wasip2" / "release" / f"{wasm_stem}.wasm"
    if not wasm_path.exists():
        print(f"    FAIL: {wasm_path} not found after build")
        return False

    wasm_size = wasm_path.stat().st_size
    print(f"    Built: {wasm_size:,} bytes")

    # Step 2: wasm-opt
    if crate_info.get("wasm_opt"):
        wasm_opt = find_tool("wasm-opt")
        if wasm_opt:
            optimized = dist_dir / f"{wasm_stem}.wasm"
            print(f"    [2/4] wasm-opt -Oz ...")
            result = subprocess.run(
                [wasm_opt, "-Oz", "-o", str(optimized), str(wasm_path)],
                capture_output=True, text=True, encoding="utf-8", errors="replace",
                timeout=120,
            )
            if result.returncode == 0:
                opt_size = optimized.stat().st_size
                reduction = (1 - opt_size / wasm_size) * 100
                print(f"    Optimized: {wasm_size:,} -> {opt_size:,} bytes ({reduction:.1f}% reduction)")
                wasm_path = optimized
            else:
                print(f"    wasm-opt failed, using unoptimized")
                shutil.copy2(wasm_path, dist_dir / f"{wasm_stem}.wasm")
        else:
            print(f"    [2/4] wasm-opt not found, skipping optimization")
            shutil.copy2(wasm_path, dist_dir / f"{wasm_stem}.wasm")
    else:
        shutil.copy2(wasm_path, dist_dir / f"{wasm_stem}.wasm")

    # Step 3: jco transpile
    print(f"    [3/4] jco transpile ...")
    jco = find_tool("jco", ["npx.cmd jco", "npx jco"])
    wrapper_dir = dist_dir / "wrapper"
    wrapper_dir.mkdir(exist_ok=True)

    if jco:
        jco_cmd = jco.split() if " " in jco else [jco]
        result = subprocess.run(
            jco_cmd + [
                "transpile",
                str(dist_dir / f"{wasm_stem}.wasm"),
                "-o", str(wrapper_dir),
            ],
            capture_output=True, text=True, encoding="utf-8", errors="replace",
            timeout=60,
        )
        if result.returncode == 0:
            print(f"    jco transpile OK")
        else:
            print(f"    jco transpile failed: {result.stderr[-200:]}")

    # Step 4: Generate package.json
    print(f"    [4/4] Generating npm package ...")
    pkg_json = {
        "name": f"{SCOPE}/{crate_name}-wasm",
        "version": VERSION,
        "description": crate_info["description"],
        "license": "MIT OR Apache-2.0",
        "type": "module",
        "main": f"./dist/wrapper/{wasm_stem}.js",
        "files": [
            f"dist/{wasm_stem}.wasm",
            "dist/wrapper/**/*.js",
            "README.md",
        ],
        "repository": {
            "type": "git",
            "url": "https://github.com/celestia-org/tairitsu.git",
            "directory": f"packages/npm/{crate_name}-wasm",
        },
        "publishConfig": {
            "access": "public",
            "registry": "https://registry.npmjs.org/",
        },
    }
    (pkg_dir / "package.json").write_text(
        json.dumps(pkg_json, indent=2) + "\n", encoding="utf-8"
    )

    final_wasm = dist_dir / f"{wasm_stem}.wasm"
    print(f"    DONE: {crate_name}-wasm ({final_wasm.stat().st_size:,} bytes)")
    return True


def main():
    args = sys.argv[1:]

    if "--list" in args:
        print("Compilable WASM crates:")
        all_crates = {**WASM_CRATES, **HIKARI_WASM_CRATES}
        for name, info in all_crates.items():
            exists = (WORKSPACE_ROOT / info["path"]).exists()
            print(f"  {'OK' if exists else 'MISSING'} {name}: {info['description']}")
        return

    target_crates = [a for a in args if not a.startswith("--")]
    if not target_crates:
        target_crates = list(WASM_CRATES.keys())

    print("=" * 60)
    print(f"Building WASM component packages: {', '.join(target_crates)}")
    print("=" * 60)

    all_crates = {**WASM_CRATES, **HIKARI_WASM_CRATES}
    built = 0
    failed = []

    for crate_name in target_crates:
        if crate_name not in all_crates:
            print(f"\n  Unknown crate: {crate_name}")
            failed.append(crate_name)
            continue
        if build_wasm_crate(crate_name, all_crates[crate_name]):
            built += 1
        else:
            failed.append(crate_name)

    print(f"\n{'=' * 60}")
    print(f"Built: {built}, Failed: {len(failed)}")
    if failed:
        print(f"Failed: {', '.join(failed)}")
    print(f"{'=' * 60}")


if __name__ == "__main__":
    main()
