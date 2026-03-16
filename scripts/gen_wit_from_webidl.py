#!/usr/bin/env python3
"""
Orchestrate the WebIDL → WIT generation pipeline.

Step 1 — fetch   : Download spec files from w3c/webref → target/tairitsu-wit/webidl-cache/
Step 2 — generate: Parse WebIDL → generate WIT         → packages/browser-worlds/wit/generated/

Usage:
    python scripts/gen_wit_from_webidl.py               # fetch + generate
    python scripts/gen_wit_from_webidl.py --fetch-only  # only download WebIDL
    python scripts/gen_wit_from_webidl.py --gen-only    # only generate WIT (cache must exist)
    python scripts/gen_wit_from_webidl.py --dry-run     # show what would happen
    python scripts/gen_wit_from_webidl.py --list-sources  # show data sources
    python scripts/gen_wit_from_webidl.py --stats       # show coverage statistics
    python scripts/gen_wit_from_webidl.py --force       # re-download all specs

Individual steps can also be run directly:
    python scripts/fetch_webidl.py --help
    python scripts/generate_browser_wit.py --help
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


SCRIPTS_DIR = Path(__file__).parent
FETCH_SCRIPT = SCRIPTS_DIR / "fetch_webidl.py"
GENERATE_SCRIPT = SCRIPTS_DIR / "generate_browser_wit.py"


def log_info(message: str) -> None:
    print(f"[INFO] {message}")


def log_ok(message: str) -> None:
    print(f"[OK] {message}")


def log_warn(message: str) -> None:
    print(f"[WARN] {message}")


def log_error(message: str) -> None:
    print(f"[ERROR] {message}", file=sys.stderr)


def _run(script: Path, extra_args: list[str]) -> None:
    """Run a Python script as a subprocess, raising on failure."""
    cmd = [sys.executable, str(script)] + extra_args
    log_info(f"$ {' '.join(cmd)}")
    print("-" * 64)
    result = subprocess.run(cmd, check=False)
    if result.returncode != 0:
        log_error(
            f"Command failed with exit code {result.returncode}: {script.name}")
        sys.exit(result.returncode)


def list_sources() -> None:
    """Print information about the W3C data sources used."""
    print(
        """
Tairitsu WIT Generator — Data Sources
======================================

1. W3C/WHATWG webref  (primary WebIDL source)
   Repository : https://github.com/w3c/webref
   License    : MIT / W3C Software License
   Raw IDL    : https://raw.githubusercontent.com/w3c/webref/main/ed/idl/<spec>.idl
   Maintained : W3C editors via the Reffy crawler

   Key specifications fetched:
     dom            — DOM Living Standard (https://dom.spec.whatwg.org/)
     html           — HTML Living Standard (https://html.spec.whatwg.org/)
     uievents       — UI Events          (https://www.w3.org/TR/uievents/)
     pointerevents  — Pointer Events     (https://www.w3.org/TR/pointerevents3/)
     fetch          — Fetch Standard     (https://fetch.spec.whatwg.org/)
     streams        — Streams Standard   (https://streams.spec.whatwg.org/)
     url            — URL Standard       (https://url.spec.whatwg.org/)
     cssom          — CSS Object Model   (https://www.w3.org/TR/cssom-1/)
     websockets     — WebSockets         (https://websockets.spec.whatwg.org/)
     service-workers— Service Workers    (https://w3c.github.io/ServiceWorker/)
     webcrypto      — Web Cryptography   (https://www.w3.org/TR/WebCryptoAPI/)
     webgl1/webgl2  — WebGL 1 & 2        (https://www.khronos.org/webgl/wiki/)
     webrtc         — WebRTC 1.0         (https://www.w3.org/TR/webrtc/)
     intersection-observer — IntersectionObserver API
     resize-observer       — ResizeObserver API
     performance-timeline  — Performance Timeline L2
     … and 20+ more specs (see scripts/fetch_webidl.py TARGET_SPECS)

2. Generated output
   WIT files  : packages/browser-worlds/wit/generated/<domain>.wit
    Package    : tairitsu-browser:<domain>@0.2.0
   Cache      : target/tairitsu-wit/webidl-cache/  (git-ignored)

3. Hand-written baseline (Phase 0)
   WIT files  : packages/browser-worlds/wit/*.wit
   Package    : tairitsu-browser:<domain>@0.1.0

Coverage target: ≥ 90% of wasm-bindgen-cli browser interface surface
"""
    )


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        "--fetch-only",
        action="store_true",
        help="Only run the fetch step (skip WIT generation)",
    )
    parser.add_argument(
        "--gen-only",
        action="store_true",
        help="Only run the generation step (WebIDL cache must already exist)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Pass --dry-run to both sub-scripts",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Pass --force to fetch (re-download even if cached)",
    )
    parser.add_argument(
        "--list-sources",
        action="store_true",
        help="Print information about data sources and exit",
    )
    parser.add_argument(
        "--stats",
        action="store_true",
        help="Show interface coverage statistics and exit",
    )
    parser.add_argument(
        "--domains",
        nargs="+",
        metavar="DOMAIN",
        help="Generate only specific domain(s)",
    )
    args = parser.parse_args()

    if args.list_sources:
        list_sources()
        return

    print("=" * 64)
    log_info("Tairitsu WebIDL -> WIT Pipeline")
    print("=" * 64)

    fetch_args: list[str] = []
    gen_args: list[str] = []

    if args.dry_run:
        fetch_args.append("--dry-run")
        gen_args.append("--dry-run")
    if args.force:
        fetch_args.append("--force")
    if args.stats:
        gen_args.append("--stats")
    if args.domains:
        gen_args.extend(["--domains"] + args.domains)

    if not args.gen_only:
        log_info("[Step 1/2] Fetching WebIDL spec files from w3c/webref...")
        _run(FETCH_SCRIPT, fetch_args)

    if not args.fetch_only:
        if args.stats:
            log_info("[Step 2/2] Coverage statistics")
        else:
            log_info("[Step 2/2] Generating WIT from cached WebIDL...")
        _run(GENERATE_SCRIPT, gen_args)

    if not args.dry_run and not args.stats and not args.fetch_only:
        print()
        print("=" * 64)
        log_ok("Pipeline complete")
        log_info("Generated WIT: packages/browser-worlds/wit/generated/")
        log_info("Commit the generated files or add to .gitignore as preferred")
        print("=" * 64)


if __name__ == "__main__":
    main()
