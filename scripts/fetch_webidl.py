#!/usr/bin/env python3
"""
Fetch WebIDL specification files from the W3C/WHATWG webref repository.

Data source:
    https://github.com/w3c/webref  (MIT / W3C Software License)
    Raw IDL path: ed/idl/<spec>.idl

Files are cached under  target/tairitsu-wit/webidl-cache/<spec>.idl
so subsequent runs are fully offline.

Usage:
    python3 scripts/fetch_webidl.py                  # fetch all target specs
    python3 scripts/fetch_webidl.py --force          # re-download even if cached
    python3 scripts/fetch_webidl.py --dry-run        # show what would be downloaded
    python3 scripts/fetch_webidl.py --specs dom html # fetch specific specs only
    python3 scripts/fetch_webidl.py --discover       # query GitHub API for all specs
    python3 scripts/fetch_webidl.py --list-specs     # print the built-in target list
"""

from __future__ import annotations

import argparse
import json
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Dict, List, Optional

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

WEBREF_RAW_BASE = (
    "https://raw.githubusercontent.com/w3c/webref/main/ed/idl"
)
WEBREF_GITHUB_API = (
    "https://api.github.com/repos/w3c/webref/contents/ed/idl"
)

# Be polite to GitHub's servers.
REQUEST_DELAY_SECONDS = 0.3

# ---------------------------------------------------------------------------
# Target specifications: spec-short-name → primary domain label
# Confirmed accessible via the raw URL above.
# ---------------------------------------------------------------------------

TARGET_SPECS: Dict[str, str] = {
    # DOM Living Standard
    "dom": "dom",
    # HTML Living Standard (Window, Navigator, HTMLElement tree, Workers…)
    "html": "html",
    # UI Events (MouseEvent, KeyboardEvent, FocusEvent, WheelEvent…)
    "uievents": "events",
    # Pointer Events
    "pointerevents": "events",
    # Touch Events
    "touch-events": "events",
    # Clipboard API
    "clipboard-apis": "events",
    # Fetch Living Standard
    "fetch": "fetch",
    # XMLHttpRequest
    "xhr": "fetch",
    # Streams Living Standard
    "streams": "fetch",
    # File API
    "fileapi": "fetch",
    # URL Living Standard
    "url": "url",
    # CSS Object Model
    "cssom": "css",
    # CSS Object Model View Module (scroll, resize, …)
    "cssom-view": "css",
    # CSS Animations
    "css-animations": "css",
    # CSS Transitions
    "css-transitions": "css",
    # CSS Fonts
    "css-fonts": "css",
    # CSS Conditional Rules (@supports/@media)
    "css-conditional": "css",
    # Web Storage (localStorage / sessionStorage)
    "storage": "storage",
    # WebSockets
    "websockets": "websocket",
    # Service Workers
    "service-workers": "workers",
    # Web Cryptography
    "webcrypto": "crypto",
    # Media Capture and Streams (getUserMedia)
    "mediacapture-streams": "media",
    # Media Capabilities
    "media-capabilities": "media",
    # MediaSession
    "mediasession": "media",
    # MediaStream Recording
    "mediastream-recording": "media",
    # Speech API
    "speech-api": "media",
    # Screen Capture
    "screen-capture": "media",
    # WebRTC
    "webrtc": "webrtc",
    # WebGL 1
    "webgl1": "canvas",
    # WebGL 2
    "webgl2": "canvas",
    # WebCodecs
    "webcodecs": "canvas",
    # Intersection Observer
    "intersection-observer": "observers",
    # Resize Observer
    "resize-observer": "observers",
    # Performance Timeline
    "performance-timeline": "performance",
    # High Resolution Time
    "hr-time": "performance",
    # Resource Timing
    "resource-timing": "performance",
    # Navigation Timing
    "navigation-timing": "performance",
    # User Timing
    "user-timing": "performance",
    # Notifications
    "notifications": "notifications",
    # Permissions
    "permissions": "permissions",
    # Geolocation
    "geolocation": "device",
    # Screen Orientation
    "screen-orientation": "device",
    # Battery Status
    "battery-status": "device",
    # Vibration
    "vibration": "device",
    # Fullscreen
    "fullscreen": "dom",
    # Gamepad
    "gamepad": "device",
    # Credential Management
    "credential-management": "auth",
    # Payment Request
    "payment-request": "payments",
    # Web MIDI
    "webmidi": "device",
    # WASM JS API
    "wasm-js-api": "wasm",
}


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _request(url: str, timeout: int = 30) -> bytes:
    """Perform an HTTP GET, returning the response body as bytes."""
    req = urllib.request.Request(
        url,
        headers={
            "Accept": "*/*",
            "User-Agent": (
                "tairitsu-wit-generator/0.1 "
                "(https://github.com/celestia-island/tairitsu)"
            ),
        },
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return resp.read()


def fetch_spec_idl(spec: str, timeout: int = 30) -> Optional[bytes]:
    """Download the raw WebIDL for *spec* from webref.  Returns None on 404."""
    url = f"{WEBREF_RAW_BASE}/{spec}.idl"
    try:
        return _request(url, timeout=timeout)
    except urllib.error.HTTPError as exc:
        if exc.code == 404:
            return None
        raise
    except urllib.error.URLError as exc:
        print(f"  [NETWORK ERROR] {exc}", file=sys.stderr)
        raise


def discover_available_specs(cache_dir: Path) -> List[str]:
    """Query GitHub API for all spec names available in webref/ed/idl."""
    cache_file = cache_dir / "_available_specs.json"
    if cache_file.exists():
        with cache_file.open("r", encoding="utf-8") as fh:
            return json.load(fh)

    print(f"Querying GitHub API: {WEBREF_GITHUB_API}")
    try:
        data = json.loads(_request(WEBREF_GITHUB_API))
    except Exception as exc:  # noqa: BLE001
        print(f"  [WARN] Could not query GitHub API: {exc}", file=sys.stderr)
        return []

    if not isinstance(data, list):
        print("  [WARN] Unexpected GitHub API response format", file=sys.stderr)
        return []

    names = [
        entry["name"][: -len(".idl")]
        for entry in data
        if isinstance(entry, dict) and entry.get("name", "").endswith(".idl")
    ]
    names.sort()

    cache_dir.mkdir(parents=True, exist_ok=True)
    with cache_file.open("w", encoding="utf-8") as fh:
        json.dump(names, fh, indent=2)

    print(f"  Found {len(names)} specs in webref")
    return names


# ---------------------------------------------------------------------------
# Main logic
# ---------------------------------------------------------------------------


def run_fetch(
    specs: List[str],
    cache_dir: Path,
    *,
    force: bool = False,
    dry_run: bool = False,
) -> None:
    """Download and cache WebIDL for every spec in *specs*."""
    cache_dir.mkdir(parents=True, exist_ok=True)

    fetched = cached = skipped = 0

    for spec in specs:
        dest = cache_dir / f"{spec}.idl"

        if dest.exists() and not force:
            cached += 1
            if dry_run:
                print(f"  [cached]       {spec}.idl")
            continue

        if dry_run:
            print(f"  [would fetch]  {spec}.idl")
            continue

        print(f"  Fetching {spec}.idl …", end="", flush=True)
        try:
            content = fetch_spec_idl(spec)
        except Exception as exc:  # noqa: BLE001
            print(f" ERROR: {exc}")
            skipped += 1
            continue

        if content is None:
            print(" NOT FOUND")
            skipped += 1
            continue

        dest.write_bytes(content)
        print(f" OK ({len(content):,} bytes)")
        fetched += 1
        time.sleep(REQUEST_DELAY_SECONDS)

    if not dry_run:
        print()
        print(
            f"Result: {fetched} fetched, {cached} already cached,"
            f" {skipped} not found/skipped"
        )
        print(f"Cache : {cache_dir}")


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Re-download even if the file is already cached",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print what would be downloaded without actually downloading",
    )
    parser.add_argument(
        "--specs",
        nargs="+",
        metavar="SPEC",
        help="Fetch specific spec(s) by short name (e.g. dom html fetch)",
    )
    parser.add_argument(
        "--discover",
        action="store_true",
        help="Query GitHub API to list all available specs in webref",
    )
    parser.add_argument(
        "--list-specs",
        action="store_true",
        help="Print the built-in target spec list and exit",
    )
    parser.add_argument(
        "--cache-dir",
        metavar="DIR",
        help="Override the cache directory (default: target/tairitsu-wit/webidl-cache)",
    )
    args = parser.parse_args()

    project_root = Path(__file__).parent.parent
    if args.cache_dir:
        cache_dir = Path(args.cache_dir)
    else:
        cache_dir = project_root / "target" / "tairitsu-wit" / "webidl-cache"

    if args.list_specs:
        print(f"Built-in target specs ({len(TARGET_SPECS)}):")
        for spec, domain in sorted(TARGET_SPECS.items()):
            dest = cache_dir / f"{spec}.idl"
            status = "✓ cached" if dest.exists() else "  missing"
            print(f"  {status}  {spec:<35} → {domain}")
        return

    if args.discover:
        specs = discover_available_specs(cache_dir)
        print(f"\nAll available specs in webref/ed/idl ({len(specs)}):")
        for s in specs:
            print(f"  {s}")
        return

    specs = args.specs if args.specs else list(TARGET_SPECS.keys())

    print("=" * 64)
    print("Tairitsu WebIDL Fetcher")
    print("=" * 64)
    print(f"Source : {WEBREF_RAW_BASE}")
    print(f"Cache  : {cache_dir}")
    print(f"Specs  : {len(specs)} targeted")
    if args.force:
        print("Mode   : force re-download")
    if args.dry_run:
        print("Mode   : dry run (no downloads)")
    print()

    run_fetch(specs, cache_dir, force=args.force, dry_run=args.dry_run)


if __name__ == "__main__":
    main()
