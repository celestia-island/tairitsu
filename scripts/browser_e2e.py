#!/usr/bin/env python3
"""Browser-glue e2e through shirabe's HTTP debug API.

2026-10-17 browser-backbone consolidation: every browser facility lives in
shirabe (CDP engine + HTTP debug API + Chromium resolution). This repo's
browser test *orchestration* is this Python script — it starts a shirabe
debug server, loads a page, and asserts DOM/text conditions through the
API. Add scenarios here rather than growing a second CDP implementation.

Requires `shirabe` on PATH (or --shirabe-bin / SHIRABE_BIN), e.g.
`cargo install --git https://github.com/celestia-island/shirabe shirabe`
or `npm i -g @celestia-island/shirabe`, plus a Chromium binary shirabe can
resolve. That is why this lane is not part of `just ci`; the browser-free
half of the same contract lives in `just packager-js-test`
(`packages/packager/test/`).

W3 (fail-closed boot): the DEFAULT scenario now loads a real component
page — the packaged `index.html` (built from `index.html.template`), the
real wrapper loader and the real boot runtime — and asserts the whole
lifecycle: the component instantiates, its entry runs, and only then does
`data-tairitsu-ready` appear. Three negative controls load components that
must fail closed (corrupted bytes, no boot entry, a missing import): each
must show the error surface and must NOT report ready.

Scenarios:
  component            good synthetic component -> ready + entry ran
  bad-bytes            instantiate throws       -> error, not ready
  missing-entry        no boot export           -> error, not ready
  missing-import       instantiate throws on a missing import -> error, not ready
  fixture              legacy selector smoke (fast, no component)

Examples:
  python3 scripts/browser_e2e.py
  python3 scripts/browser_e2e.py --scenario bad-bytes
  python3 scripts/browser_e2e.py --dist target/tairitsu-dist --scenario component
  python3 scripts/browser_e2e.py --scenario fixture --expect "#app"
"""

import argparse
import base64
import functools
import http.server
import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.request

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TEMPLATE_DIR = os.path.join(REPO_ROOT, "packages", "packager", "src", "wasm")

SCENARIOS = ("component", "bad-bytes", "missing-entry", "missing-import", "fixture")

# The entry the good synthetic component exports: it records that it ran and
# mutates the page, so "the entry was invoked" is observable from the DOM.
GOOD_WRAPPER = """
export async function instantiate(imports) {
  const result = {
    tairitsu_component_bootstrap() {
      globalThis.__w3_entry_ran = true;
      const app = document.getElementById('app');
      if (app) app.textContent = 'component booted';
    },
  };
  return result;
}
"""

# A component that cannot be instantiated: the loader must surface this, not
# swallow it and present a ready page.
BAD_BYTES_WRAPPER = """
export async function instantiate(imports) {
  throw new Error('core module is not a valid WebAssembly binary');
}
"""

# A component that instantiates but exposes no boot entry at all.
MISSING_ENTRY_WRAPPER = """
export async function instantiate(imports) {
  return { notAnEntry() {} };
}
"""

# A component that reaches for an import the page does not provide.
MISSING_IMPORT_WRAPPER = """
export async function instantiate(imports) {
  const name = 'wasi:cli/does-not-exist@0.2.0';
  const need = imports[name];
  if (!need) throw new Error('missing import: ' + name);
  return need.run();
}
"""

WRAPPERS = {
    "component": GOOD_WRAPPER,
    "bad-bytes": BAD_BYTES_WRAPPER,
    "missing-entry": MISSING_ENTRY_WRAPPER,
    "missing-import": MISSING_IMPORT_WRAPPER,
}

FAILING = ("bad-bytes", "missing-entry", "missing-import")

# What the visible card must say for each control: the same "an error card
# exists" assertion would pass on a 404'd wrapper or a broken loader, so each
# control names the failure it is supposed to exercise.
EXPECTED_CARD = {
    "bad-bytes": "not a valid WebAssembly binary",
    "missing-entry": "none of the boot entries",
    "missing-import": "missing import: wasi:cli/does-not-exist@0.2.0",
}


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


def evaluate(base, expression):
    """Evaluate in the page and return the JSON result."""
    data = call(base, "/evaluate", {"expression": expression})
    return data.get("result") if isinstance(data, dict) else data


def _free_port():
    """An unused loopback port (the process that binds it is a child anyway)."""
    import socket

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


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

# The page's own placeholders (a subset of the packager's Tera context: only
# the ones that matter for a browser-loadable page).
PAGE_CONTEXT = {
    "charset": "utf-8",
    "lang": "en",
    "viewport": "width=device-width, initial-scale=1",
    "title": "tairitsu boot e2e",
    "body_class": "",
    "head": "",
    "favicon": "",
    "glue_bundle": "/glue-stub.js",
    "wasm_file": "component.wasm",
    "v": "e2e",
}

# Minimal glue bundle: the page probes these globals for its import map. The
# synthetic components do not need real glue; a stub keeps the page's
# `buildImports` green so the failures under test come from the component.
GLUE_STUB = """
globalThis.__TAIRITSU_GLUE__ = { INTERFACES: {} };
"""


def render_page():
    """Render `index.html.template` with the e2e context."""
    with open(os.path.join(TEMPLATE_DIR, "index.html.template"), encoding="utf-8") as fh:
        page = fh.read()
    for key, value in PAGE_CONTEXT.items():
        page = page.replace("{{ " + key + " }}", value)
        page = page.replace("{{" + key + "}}", value)
    for leftover in ("{{ head | safe }}", "{{ favicon | safe }}"):
        page = page.replace(leftover, "")
    if "{{" in page:
        raise RuntimeError("index.html.template still carries a placeholder: " + page[page.index("{{") : page.index("{{") + 40])
    return page


def build_dist(scenario, dest):
    """A dist-shaped tree: the real page, the real loader, a stub component."""
    os.makedirs(os.path.join(dest, "component-wrapper"), exist_ok=True)
    with open(os.path.join(dest, "index.html"), "w", encoding="utf-8") as fh:
        fh.write(render_page())
    with open(os.path.join(dest, "glue-stub.js"), "w", encoding="utf-8") as fh:
        fh.write(GLUE_STUB)

    loader_src = os.path.join(TEMPLATE_DIR, "component-wrapper-loader.template.js")
    with open(loader_src, encoding="utf-8") as fh:
        loader = fh.read()
    # The dist's wasm stem is `component` (see PAGE_CONTEXT["wasm_file"]).
    loader = loader.replace("__WASM_STEM__", "component")
    with open(os.path.join(dest, "component-wrapper-loader.js"), "w", encoding="utf-8") as fh:
        fh.write(loader)
    shutil.copyfile(
        os.path.join(TEMPLATE_DIR, "component-boot.template.js"),
        os.path.join(dest, "component-boot.js"),
    )
    with open(
        os.path.join(dest, "component-wrapper", "component.js"), "w", encoding="utf-8"
    ) as fh:
        fh.write(WRAPPERS[scenario])
    # A placeholder core module: the synthetic wrappers never fetch it, but the
    # page's direct-load fallback checks magic bytes.
    with open(os.path.join(dest, "component.wasm"), "wb") as fh:
        fh.write(b"\x00asm\x0d\x00\x01\x00" + b"\x00" * 16)


class _QuietHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *_args):  # noqa: D401 - silence the per-request log
        return


def serve(directory):
    """Serve `directory` on an ephemeral port; returns (server, thread, url)."""
    handler = functools.partial(_QuietHandler, directory=directory)
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    port = server.server_address[1]
    return server, thread, f"http://127.0.0.1:{port}"


def check_boot_state(state, scenario, *, real_dist=False):
    """The W3 assertions over ONE observed page state (pure: no browser).

    `state` = {ready, error, entryRan, appText}. Split out so `--self-check`
    can prove the assertions still bite (a control that cannot fail is not a
    control).
    """
    failures = []
    if not isinstance(state, dict):
        return [f"page state unreadable: {state!r}"]

    if scenario in FAILING:
        if state.get("ready") is not None:
            failures.append(f"failed component reported ready ({state['ready']!r})")
        if not state.get("error"):
            failures.append("failed component set no data-tairitsu-error")
        if state.get("entryRan"):
            failures.append("failed component ran an entry")
        if "Tairitsu boot error" not in (state.get("appText") or ""):
            failures.append("no visible error card in #app")
        expected = EXPECTED_CARD.get(scenario)
        if expected and expected not in (state.get("appText") or ""):
            failures.append(f"{scenario}: the card does not name the failure ({expected!r} missing)")
    else:
        if state.get("ready") != "hydrated":
            failures.append(f"good component not ready (ready={state.get('ready')!r})")
        if state.get("error"):
            failures.append(f"good component reported an error ({state['error']!r})")
        if real_dist:
            # A real build is not instrumented: ready + no error is the whole
            # contract we can observe from the outside.
            return failures
        if not state.get("entryRan"):
            failures.append("the component's boot entry never ran")
        if "component booted" not in (state.get("appText") or ""):
            failures.append("#app was not updated by the component")
    return failures


def assert_boot_contract(base, scenario, *, real_dist=False):
    """Observe the live page, then apply the W3 assertions."""
    state = evaluate(
        base,
        """(() => {
            const root = document.documentElement;
            return {
                ready: root.dataset.tairitsuReady || null,
                error: root.dataset.tairitsuError || null,
                entryRan: globalThis.__w3_entry_ran === true,
                appText: (document.getElementById('app') || {}).textContent || '',
                bodyText: document.body ? document.body.innerText : '',
            };
        })()""",
    )
    return check_boot_state(state, scenario, real_dist=real_dist)


GOOD_STATE = {
    "ready": "hydrated",
    "error": None,
    "entryRan": True,
    "appText": "component booted",
}
BAD_STATE = {
    "ready": None,
    "error": "no-entry",
    "entryRan": False,
    "appText": "Tairitsu boot error",
}


def self_check():
    """Offline: the negative controls must be able to fail, and must be present.

    This is the browser-free half of G2-W3 ①/③: if a control is deleted, or if
    the assertions stop catching a wrongly-ready page, this goes red — without
    needing shirabe or a Chromium binary (so it belongs in `just ci`).
    """
    failures = []

    expected_controls = {"bad-bytes", "missing-entry", "missing-import"}
    present = expected_controls & set(FAILING)
    missing = expected_controls - present
    if missing:
        failures.append(f"negative controls missing: {sorted(missing)}")
    for scenario in sorted(present):
        if scenario not in WRAPPERS:
            failures.append(f"control {scenario} has no component under test")
        # A control that cannot fail is not a control: the same scenario with a
        # page that wrongly reports ready MUST produce failures.
        if not check_boot_state(GOOD_STATE, scenario):
            failures.append(f"control {scenario} would pass a page that is wrongly ready")
        if not check_boot_state(dict(BAD_STATE, error=None), scenario):
            failures.append(f"control {scenario} would pass a page with no error attribute")
        if not check_boot_state(dict(BAD_STATE, appText=""), scenario):
            failures.append(f"control {scenario} would pass a page with no visible error")
        if not check_boot_state(dict(BAD_STATE, entryRan=True), scenario):
            failures.append(f"control {scenario} would pass a page whose entry ran")
        card = EXPECTED_CARD[scenario]
        generic = {"ready": None, "error": "instantiate-failed", "entryRan": False,
                   "appText": "Tairitsu boot error: something else failed"}
        if not check_boot_state(generic, scenario):
            failures.append(
                f"control {scenario} would pass a card that does not name {card!r}"
            )
        # ... and the honest failure state (card naming THIS failure) must pass.
        honest = dict(BAD_STATE, appText="Tairitsu boot error\n" + EXPECTED_CARD[scenario])
        if check_boot_state(honest, scenario):
            failures.append(f"control {scenario} rejects a correct error page")

    # The good-component assertions must bite in both directions too.
    if check_boot_state(GOOD_STATE, "component"):
        failures.append("the good scenario rejects a healthy page")
    if not check_boot_state(dict(GOOD_STATE, ready=None), "component"):
        failures.append("the good scenario accepts a page that never became ready")
    if not check_boot_state(dict(GOOD_STATE, entryRan=False), "component"):
        failures.append("the good scenario accepts a page whose entry never ran")
    if not check_boot_state(dict(GOOD_STATE, error="entry-failed"), "component"):
        failures.append("the good scenario accepts a page carrying an error")

    if failures:
        for line in failures:
            print(f"FAIL: {line}", file=sys.stderr)
        return 1
    print(
        "OK: browser-e2e self-check passed "
        f"({len(expected_controls & set(FAILING))} negative controls bite, "
        "good-component assertions bite both ways)"
    )
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--shirabe-bin",
        default=os.environ.get("SHIRABE_BIN", "shirabe"),
        help="shirabe binary (default: $SHIRABE_BIN or PATH lookup)",
    )
    ap.add_argument(
        "--port",
        type=int,
        default=0,
        help="shirabe debug port (default 0 = pick a free one; a fixed port can "
        "collide with a stale or another agent's instance, and then this script "
        "talks to that browser instead of its own)",
    )
    ap.add_argument(
        "--scenario",
        choices=SCENARIOS,
        default="component",
        help="component (real lifecycle, default) or one of the negative controls",
    )
    ap.add_argument(
        "--dist",
        default=None,
        help="serve an already-built dist (e.g. from `tairitsu build`) instead of "
        "the synthetic one; the boot contract is asserted the same way",
    )
    ap.add_argument(
        "--url",
        default=None,
        help="page to load; only meaningful with --scenario fixture",
    )
    ap.add_argument(
        "--expect",
        action="append",
        default=[],
        metavar="SELECTOR",
        help="fixture mode: CSS selector that must match (repeatable)",
    )
    ap.add_argument("--expect-text", default=None, help="fixture mode: text that must appear")
    ap.add_argument("--screenshot", default=None, help="save a PNG screenshot here")
    ap.add_argument(
        "--self-check",
        action="store_true",
        help="offline: prove the negative controls can fail (no browser needed)",
    )
    args = ap.parse_args()

    if args.self_check:
        return self_check()

    port = args.port or _free_port()
    base = f"http://127.0.0.1:{port}"
    # shirabe enforces a MACHINE-WIDE tab cap (SHIRABE_MAX_TABS, default 3) and
    # reuses one browser, so a run collides with any other agent/reviewer using
    # it ("tab cap reached (3/3)"). Raise the cap for this harness instead of
    # closing somebody else's tabs; the instance is torn down below.
    env = dict(os.environ)
    env.setdefault("SHIRABE_MAX_TABS", "8")
    # `shirabe` is a node shim over a native binary: terminating the shim
    # leaves the native process (and its tab slot) behind, so the harness owns
    # a process GROUP and kills the group.
    proc = subprocess.Popen(
        [args.shirabe_bin, "debug", "--port", str(port)],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        env=env,
        start_new_session=True,
    )
    failures = []
    tmp_root = None
    server = None
    try:
        wait_healthy(base, 60)

        if args.scenario == "fixture":
            url = args.url or "about:blank"
            call(base, "/navigate", {"url": url})
            if url == "about:blank":
                call(
                    base,
                    "/evaluate",
                    {
                        "expression": "document.body.innerHTML = %s; 'fixture'"
                        % json.dumps(DEFAULT_FIXTURE)
                    },
                )
            selectors = args.expect or (["#app"] if url == "about:blank" else [])
            for selector in selectors:
                ok = evaluate(base, f"document.querySelector({json.dumps(selector)}) !== null")
                if ok is not True:
                    failures.append(f"selector not found: {selector}")
            if args.expect_text:
                ok = evaluate(
                    base, f"document.body.innerText.includes({json.dumps(args.expect_text)})"
                )
                if ok is not True:
                    failures.append(f"text not found: {args.expect_text!r}")
        else:
            if args.dist:
                dist = args.dist
                if not os.path.isfile(os.path.join(dist, "index.html")):
                    raise RuntimeError(f"--dist {dist} has no index.html")
            else:
                tmp_root = tempfile.mkdtemp(prefix="tairitsu-w3-e2e-")
                dist = os.path.join(tmp_root, "dist")
                build_dist(args.scenario, dist)
            server, _thread, url = serve(dist)
            call(base, "/navigate", {"url": url + "/index.html"})
            # The page boots asynchronously (styles + lifecycle); poll briefly.
            deadline = time.time() + 20
            while time.time() < deadline:
                ready = evaluate(base, "!!(document.documentElement.dataset.tairitsuReady || document.documentElement.dataset.tairitsuError)")
                if ready is True:
                    break
                time.sleep(0.1)
            failures.extend(
                assert_boot_contract(base, args.scenario, real_dist=bool(args.dist))
            )

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
        try:
            # Leave no tab behind on an instance that outlives this run.
            call(base, "/browser/close", {})
        except Exception:  # noqa: BLE001 - best effort teardown
            pass
        if server is not None:
            server.shutdown()
            server.server_close()
        if tmp_root is not None:
            shutil.rmtree(tmp_root, ignore_errors=True)
        try:
            os.killpg(os.getpgid(proc.pid), signal.SIGTERM)
        except (ProcessLookupError, PermissionError):
            pass
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
            except (ProcessLookupError, PermissionError):
                proc.kill()

    if failures:
        print("FAIL:", "; ".join(failures), file=sys.stderr)
        return 1
    print(f"OK: boot e2e passed (scenario={args.scenario})")
    return 0


if __name__ == "__main__":
    sys.exit(main())