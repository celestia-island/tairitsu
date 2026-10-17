#!/usr/bin/env python3
"""Browser-glue e2e through shirabe's HTTP debug API.

2026-10-17 browser-backbone consolidation: every browser facility lives in
shirabe (CDP engine + HTTP debug API + Chromium resolution). This repo's
browser test *orchestration* is this Python script — it starts a shirabe
debug server, loads a page, and asserts DOM/text conditions through the
API. Add scenarios here rather than growing a second CDP implementation.

Requires `shirabe` on PATH (or --shirabe-bin / SHIRABE_BIN), e.g.
`cargo install --git https://github.com/celestia-island/shirabe shirabe`
or `npm i -g @celestia-island/shirabe`.

Examples:
  python3 scripts/browser_e2e.py
  python3 scripts/browser_e2e.py --url http://127.0.0.1:8080 --expect "#app"
  python3 scripts/browser_e2e.py --screenshot /mnt/work/e2e.png
"""

import argparse
import base64
import json
import os
import subprocess
import sys
import time
import urllib.error
import urllib.request


def call(base, path, body=None):
    """POST/GET the shirabe debug API; unwrap the {ok, data, error} envelope."""
    req = urllib.request.Request(
        base + path,
        method="POST" if body is not None else "GET",
        data=None if body is None else json.dumps(body).encode(),
        headers={"content-type": "application/json"},
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            envelope = json.load(resp)
    except urllib.error.HTTPError as exc:
        # Non-2xx carries the same {ok, error} envelope; surface its message.
        detail = exc.read().decode(errors="replace")[:500]
        try:
            envelope = json.loads(detail)
        except json.JSONDecodeError:
            raise RuntimeError(f"{path} -> HTTP {exc.code}: {detail}") from exc
        raise RuntimeError(
            envelope.get("error") or f"{path} -> HTTP {exc.code}: {detail}"
        ) from exc
    if isinstance(envelope, dict) and envelope.get("ok") is False:
        raise RuntimeError(envelope.get("error") or f"{path} failed")
    if isinstance(envelope, dict) and "data" in envelope:
        return envelope["data"]
    return envelope


def wait_healthy(base, timeout):
    deadline = time.time() + timeout
    last = None
    while time.time() < deadline:
        try:
            call(base, "/health")
            return
        except Exception as exc:  # noqa: BLE001 - probe loop
            last = exc
            time.sleep(0.2)
    raise RuntimeError(f"shirabe debug server at {base} never became healthy: {last}")


DEFAULT_FIXTURE = (
    '<div id="app" data-glue="ok">tairitsu browser-glue fixture</div>'
)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--shirabe-bin",
        default=os.environ.get("SHIRABE_BIN", "shirabe"),
        help="shirabe binary (default: $SHIRABE_BIN or PATH lookup)",
    )
    ap.add_argument("--port", type=int, default=3919)
    ap.add_argument(
        "--url",
        default="about:blank",
        help="page to load; about:blank injects the built-in fixture",
    )
    ap.add_argument(
        "--expect",
        action="append",
        default=[],
        metavar="SELECTOR",
        help="CSS selector that must match (repeatable)",
    )
    ap.add_argument("--expect-text", default=None, help="text that must appear in body text")
    ap.add_argument("--screenshot", default=None, help="save a PNG screenshot here")
    args = ap.parse_args()

    base = f"http://127.0.0.1:{args.port}"
    proc = subprocess.Popen(
        [args.shirabe_bin, "debug", "--port", str(args.port)],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    failures = []
    try:
        wait_healthy(base, 60)
        call(base, "/navigate", {"url": args.url})
        if args.url == "about:blank":
            call(
                base,
                "/evaluate",
                {
                    "expression": "document.body.innerHTML = %s; 'fixture'"
                    % json.dumps(DEFAULT_FIXTURE)
                },
            )
        for selector in args.expect or (["#app"] if args.url == "about:blank" else []):
            data = call(
                base,
                "/evaluate",
                {"expression": f"document.querySelector({json.dumps(selector)}) !== null"},
            )
            ok = data.get("result") if isinstance(data, dict) else data
            if ok is not True:
                failures.append(f"selector not found: {selector}")
        if args.expect_text:
            data = call(
                base,
                "/evaluate",
                {"expression": f"document.body.innerText.includes({json.dumps(args.expect_text)})"},
            )
            ok = data.get("result") if isinstance(data, dict) else data
            if ok is not True:
                failures.append(f"text not found: {args.expect_text!r}")
        if args.screenshot:
            data = call(base, "/screenshot", {})
            raw = data.get("data") if isinstance(data, dict) else None
            if not isinstance(raw, str):
                failures.append("screenshot response carried no base64 data")
            else:
                with open(args.screenshot, "wb") as fh:
                    fh.write(base64.b64decode(raw))
                print(f"screenshot -> {args.screenshot}")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()

    if failures:
        print("FAIL:", "; ".join(failures), file=sys.stderr)
        return 1
    print("OK: browser-glue e2e passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
