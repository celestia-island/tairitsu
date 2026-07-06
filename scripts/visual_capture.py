#!/usr/bin/env python3
"""Capture screenshots via tairitsu debug API for visual regression."""
from __future__ import annotations

import argparse
import base64
import json
import sys
import time
from pathlib import Path

try:
    import requests
except ImportError:
    print("ERROR: 'requests' not installed. Run: pip install requests", file=sys.stderr)
    sys.exit(1)

PAGES = [
    "home", "button", "form", "search", "switch", "feedback",
    "display", "avatar", "image", "tag", "empty", "comment",
]


def main():
    ap = argparse.ArgumentParser(description="Capture screenshots via debug API")
    ap.add_argument("--debug-port", type=int, default=3001)
    ap.add_argument("--output-dir", default="target/visual-diff/actual")
    args = ap.parse_args()

    base_url = f"http://localhost:{args.debug_port}"
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    try:
        r = requests.get(f"{base_url}/health", timeout=5)
        r.raise_for_status()
    except Exception:
        print(f"Error: Debug API not running at {base_url}", file=sys.stderr)
        print("Start with: just dev-debug", file=sys.stderr)
        sys.exit(1)

    captured = 0
    skipped = 0

    for page in PAGES:
        print(f"  Capturing /{page}... ", end="", flush=True)

        try:
            r = requests.post(f"{base_url}/navigate", json={"url": f"/{page}"}, timeout=30)
            r.raise_for_status()
        except Exception:
            print("SKIP (navigate failed)")
            skipped += 1
            continue

        time.sleep(1)

        try:
            r = requests.post(f"{base_url}/screenshot", json={"full_page": False}, timeout=60)
            r.raise_for_status()
            resp = r.json()
        except Exception:
            print("SKIP (screenshot failed)")
            skipped += 1
            continue

        if not resp.get("ok"):
            print("SKIP (server returned error)")
            skipped += 1
            continue

        b64 = resp.get("data", {}).get("data", "")
        if not b64:
            print("SKIP (no data)")
            skipped += 1
            continue

        out = output_dir / f"{page}.png"
        out.write_bytes(base64.b64decode(b64))
        print("OK")
        captured += 1

    print(f"\nScreenshots saved to {output_dir}/")
    print(f"Captured: {captured}, Skipped: {skipped}")


if __name__ == "__main__":
    main()
