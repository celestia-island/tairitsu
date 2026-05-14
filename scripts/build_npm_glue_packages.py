#!/usr/bin/env python3
"""Build CDN-hosted glue packages for Tairitsu.

Reads source modules from packages/npm/celestia-tairitsu-web-glue/src/
and compiles each domain into dist/<domain>/index.js for CDN hosting.

Also reads runtime WASM components from target/wasm32-wasip2/release/
and copies them into dist/wasm/.

Usage:
    python scripts/build_npm_glue_packages.py          # Generate + build
    python scripts/build_npm_glue_packages.py --gen     # Generate only
    python scripts/build_npm_glue_packages.py --build   # Build only (must generate first)
"""

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

WORKSPACE_ROOT = Path(__file__).resolve().parent.parent
RUNTIME_DIR = WORKSPACE_ROOT / "packages" / "browser-glue" / "src" / "runtime"
PKG_DIR = WORKSPACE_ROOT / "packages" / "npm" / "celestia-tairitsu-web-glue"
SRC_DIR = PKG_DIR / "src"
DIST_DIR = PKG_DIR / "dist"

VERSION = "0.5.0"

DOMAIN_MAP = {
    "dom": ["document", "element", "node", "nonElementParentNode", "parentNode"],
    "events": ["event", "eventTarget"],
    "css": ["cssStyleDeclaration", "elementCssInlineStyle"],
    "html": ["htmlElement"],
    "observers": ["mutationObserver", "intersectionObserver"],
    "resize-observer": ["resizeObserver"],
    "platform": [
        "performance", "animationFrame", "timer", "clipboard",
        "contentEditable", "scroll", "resize", "mediaQuery", "querySelector",
    ],
    "auth": ["webAuthentication"],
    "canvas": ["canvas"],
    "crypto": ["crypto"],
    "device": ["device"],
    "fetch": ["fetch"],
    "geolocation": ["geolocation"],
    "media": ["media"],
    "misc": ["misc"],
    "notifications": ["notifications"],
    "payments": ["payments"],
    "performance": ["performanceExt"],
    "permissions": ["permissions"],
    "storage": ["storage"],
    "svg": ["svg"],
    "url": ["url"],
    "wasm": ["wasm"],
    "webrtc": ["webrtc"],
    "websocket": ["websocket"],
    "websockets": ["websockets"],
    "workers": ["workers"],
}

STUB_DOMAINS = [
    "file-api",
    "indexed-db",
    "service-workers",
    "streams",
    "web-animations",
]


def find_esbuild():
    candidates = []
    if os.name == "nt":
        candidates.append(("npx.cmd", ["npx.cmd", "esbuild", "--version"]))
        candidates.append(("esbuild.cmd", ["esbuild.cmd", "--version"]))
    candidates.append(("npx", ["npx", "esbuild", "--version"]))
    candidates.append(("esbuild", ["esbuild", "--version"]))
    for name, cmd in candidates:
        try:
            result = subprocess.run(cmd, capture_output=True, timeout=30)
            if result.returncode == 0:
                if name in ("npx", "npx.cmd"):
                    return [name, "esbuild"]
                return [name]
        except (FileNotFoundError, subprocess.TimeoutExpired):
            continue
    raise RuntimeError("esbuild not found. Install with: npm install -g esbuild")


def build_domain(domain_name):
    """Build a single domain from src/glue-<domain>.ts into dist/<domain>/index.js."""
    src_file = SRC_DIR / f"glue-{domain_name}.ts"
    if not src_file.exists():
        print(f"  SKIP {domain_name}: no source file")
        return False

    out_dir = DIST_DIR / domain_name
    out_dir.mkdir(parents=True, exist_ok=True)

    try:
        esbuild_cmd = find_esbuild()
        result = subprocess.run(
            esbuild_cmd + [
                str(src_file),
                "--bundle",
                f"--outfile={out_dir / 'index.js'}",
                "--format=esm",
                "--platform=browser",
                "--minify",
            ],
            capture_output=True, text=True, encoding="utf-8", errors="replace",
            timeout=30,
        )
        if result.returncode == 0:
            size = (out_dir / "index.js").stat().st_size
            print(f"  OK {domain_name}: {size:,} bytes")
            return True
        else:
            print(f"  FAIL {domain_name}: {result.stderr.strip()}")
            return False
    except Exception as e:
        print(f"  FAIL {domain_name}: {e}")
        return False


def copy_wasm_components():
    """Copy pre-built WASM components from target/ into dist/wasm/."""
    wasm_src = WORKSPACE_ROOT / "target" / "wasm32-wasip2" / "release"
    wasm_dst = DIST_DIR / "wasm"

    if not wasm_src.exists():
        print("  SKIP wasm: no wasm32-wasip2/release/ directory")
        return

    wasm_files = list(wasm_src.glob("*.wasm"))
    if not wasm_files:
        print("  SKIP wasm: no .wasm files found")
        return

    wasm_dst.mkdir(parents=True, exist_ok=True)
    for f in wasm_files:
        shutil.copy2(f, wasm_dst / f.name)
        print(f"  OK wasm/{f.name}: {f.stat().st_size:,} bytes")


def main():
    args = set(sys.argv[1:])

    if "--build" not in args:
        DIST_DIR.mkdir(parents=True, exist_ok=True)

        print("Building CDN glue packages...")
        all_domains = list(DOMAIN_MAP.keys()) + STUB_DOMAINS
        ok = 0
        for domain in all_domains:
            if build_domain(domain):
                ok += 1

        print(f"\n  Built: {ok}/{len(all_domains)} domains")

        print("\nCopying WASM components...")
        copy_wasm_components()

        print("\nDone.")


if __name__ == "__main__":
    main()
