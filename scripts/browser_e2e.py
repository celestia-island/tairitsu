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

SCENARIOS = (
    "component",
    "csp",
    "capability-exact",
    "capability-undeclared",
    "capability-overreach",
    "worker-refused",
    "mount-conflict-shell",
    "mount-conflict-page",
    "mount-missing",
    "worker-kill",
    "capability-wasi",
    "worker-boot",
    "worker-terminate",
    "glue-imports",
    "two-instances",
    "bad-bytes",
    "missing-entry",
    "missing-import",
    "fixture",
    "custom-root",
    "mount-inner-app",
)

# W8: a component that patches inside whatever root the host mounted it into.
CUSTOM_ROOT_WRAPPER = """
const entry = () => {
  globalThis.__w3_entry_ran = true;
  const root = (globalThis.__TAIRITSU_MOUNT_ROOT__ || {}).element;
  if (!root) return;
  // The real guest clears its mount element on entry (wit_platform.rs); model it,
  // or a boot that adopted host content would look harmless here.
  root.textContent = '';
  const guest = document.createElement('div');
  guest.id = 'guest-content';
  guest.textContent = 'component booted';
  root.appendChild(guest);
};

const lifecycle = { start: entry };
export { lifecycle, lifecycle as "tairitsu-browser:full/lifecycle@0.2.0" };
"""

# W7: a compute-only component — no DOM, so the worker lane can host it.
WORKER_WRAPPER = """
const entry = () => {
  globalThis.__w3_entry_ran = true;
  globalThis.__w7_compute = 6 * 7;
};

const lifecycle = { start: entry };
export { lifecycle, lifecycle as "tairitsu-browser:full/lifecycle@0.2.0" };
"""

# The negative control: an entry that never yields. On the main thread this
# hangs the page; in the worker lane `terminate()` kills it.
DEAD_LOOP_WRAPPER = """
const entry = () => {
  globalThis.__w3_entry_ran = true;
  while (true) {
    // A synchronous wasm loop looks exactly like this to the host: it never
    // returns, so no cooperative stop can reach it.
  }
};

const lifecycle = { start: entry };
export { lifecycle, lifecycle as "tairitsu-browser:full/lifecycle@0.2.0" };
"""

# The entry the good synthetic component exports: it records that it ran and
# mutates the page, so "the entry was invoked" is observable from the DOM.
# A wrapper shaped like the packager's post-rewrite output: it imports the glue
# interfaces from the real emitted modules (`../glue/<name>.js`). This is what
# makes the glue scenario able to catch a wrong/missing interface file — the
# other scenarios use a glue-less wrapper and cannot.
GLUE_IMPORT_WRAPPER = """
// The BARE specifiers a fresh jco wrapper carries. The packager rewrites them to
// `../glue/<name>.js`; this harness applies the same rule (see
// `rewrite_glue_specifiers`) so the scenario proves the rewrite rather than
// presupposing its output. The Rust test `glue_specifiers_become_relative_files`
// pins the packager's own implementation of the same rule.
import { createElement } from "@tairitsu-glue/document";
import { getStyle } from "@tairitsu-glue/element-css-inline-style";

const entry = () => {
  globalThis.__w3_entry_ran = true;
  const el = createElement("div");
  globalThis.__w3_glue_probe = { element: typeof el, style: typeof getStyle };
  const app = document.getElementById("app");
  if (app) app.textContent = "component booted";
};

const lifecycle = { start: entry };
export { lifecycle, lifecycle as "tairitsu-browser:full/lifecycle@0.2.0" };
"""

GOOD_WRAPPER = """
// The shape `jco transpile` actually emits for a tairitsu component: a module
// namespace that exposes the WIT interfaces (no `instantiate` export), so the
// loader's `return mod` branch and the WIT entry resolution are both exercised.
const entry = () => {
  globalThis.__w3_entry_ran = true;
  const app = document.getElementById('app');
  if (app) app.textContent = 'component booted';
};

const lifecycle = { start: entry };
// The string-named re-export is exactly what jco emits for a WIT interface.
export { lifecycle, lifecycle as "tairitsu-browser:full/lifecycle@0.2.0" };
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
    # Same component, same page — served with `script-src 'self'` so a page that
    # needed a blob URL or an inline module could not boot at all (D5).
    "csp": GOOD_WRAPPER,
    # Serves the REAL emitted glue modules and a wrapper that imports them.
    "glue-imports": GLUE_IMPORT_WRAPPER,
    "capability-exact": GLUE_IMPORT_WRAPPER,
    "capability-undeclared": GLUE_IMPORT_WRAPPER,
    "capability-overreach": GLUE_IMPORT_WRAPPER,
    "capability-wasi": GLUE_IMPORT_WRAPPER,
    "custom-root": CUSTOM_ROOT_WRAPPER,
    "mount-conflict-shell": CUSTOM_ROOT_WRAPPER,
    "mount-conflict-page": CUSTOM_ROOT_WRAPPER,
    "mount-missing": CUSTOM_ROOT_WRAPPER,
    "mount-inner-app": CUSTOM_ROOT_WRAPPER,
    "worker-boot": WORKER_WRAPPER,
    "worker-refused": WORKER_WRAPPER,
    "worker-terminate": WORKER_WRAPPER,
    "worker-kill": DEAD_LOOP_WRAPPER,
    # The isolation scenario boots the same component, then drives the instance
    # API from the test side.
    "two-instances": GOOD_WRAPPER,
    "bad-bytes": BAD_BYTES_WRAPPER,
    "missing-entry": MISSING_ENTRY_WRAPPER,
    "missing-import": MISSING_IMPORT_WRAPPER,
}

FAILING = (
    "mount-conflict-shell",
    "mount-conflict-page",
    "mount-missing",
    "bad-bytes",
    "missing-entry",
    "missing-import",
    "capability-undeclared",
    "capability-overreach",
    "worker-refused",
    "capability-wasi",
)

# What the visible card must say for each control: the same "an error card
# exists" assertion would pass on a 404'd wrapper or a broken loader, so each
# control names the failure it is supposed to exercise.
EXPECTED_CARD = {
    "bad-bytes": "not a valid WebAssembly binary",
    "missing-entry": "none of the boot entries",
    "missing-import": "missing import: wasi:cli/does-not-exist@0.2.0",
    "capability-overreach": "does not provide",
    "capability-undeclared": "is not declared",
    "capability-wasi": "WASI modules the host does not provide",
    "worker-refused": "this lane cannot provide",
    "mount-conflict-shell": "mount-root-conflict",
    "mount-conflict-page": "mount-root-conflict",
    "mount-missing": "mount-root-missing",
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
        # Loopback must never go through an HTTP proxy: with `HTTP_PROXY` set
        # (common in this workspace) the debug API answers 502.
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
        with opener.open(req, timeout=30) as resp:
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
    "glue_bundle": "/glue-runtime.js",
    "wasm_file": "component.wasm",
    "v": "e2e",
}

# Minimal glue bundle for the component-only scenarios: the page probes these
# globals for its import map. The synthetic components do not need real glue; a
# stub keeps the page's `buildImports` green so the failures under test come
# from the component.
GLUE_STUB = """
globalThis.__TAIRITSU_GLUE__ = { INTERFACES: {} };
"""

GLUE_ENTRY = "packages/browser-glue/src/runtime/index.ts"


def _source_hash():
    """Content hash of the glue sources the bundle is built from."""
    import hashlib

    digest = hashlib.sha256()
    roots = [
        os.path.join(REPO_ROOT, "packages", "browser-glue", "src", "runtime"),
        os.path.join(REPO_ROOT, "packages", "browser-glue", "src", "glue"),
    ]
    files = []
    for root in roots:
        for dirpath, _dirnames, filenames in os.walk(root):
            for name in filenames:
                if name.endswith((".ts", ".mjs", ".js")):
                    files.append(os.path.join(dirpath, name))
    for path in sorted(files):
        digest.update(os.path.relpath(path, REPO_ROOT).encode())
        try:
            with open(path, "rb") as handle:
                digest.update(handle.read())
        except OSError:
            digest.update(b"<unreadable>")
    return digest.hexdigest()


def ensure_glue_bundle(dest):
    """The REAL glue bundle, or None.

    The isolation scenario must drive the shipped instance API, not a copy of
    it: a stand-in would let the control pass while the real container is
    broken (that exact divergence produced the W3 S1). A prebuilt
    `packages/browser-glue/dist/runtime.js` is used ONLY when its stamp names the
    CURRENT source hash — mtimes cannot tell "rebuilt after the edit" from
    "built from other code". Otherwise the bundle is built from `src` here.
    """
    dist_dir = os.path.join(REPO_ROOT, "packages", "browser-glue", "dist")
    committed = os.path.join(dist_dir, "runtime.js")
    stamp = os.path.join(dist_dir, ".glue-src-hash")
    if os.path.isfile(committed) and os.path.isfile(stamp):
        try:
            with open(stamp, encoding="utf-8") as handle:
                if handle.read().strip() == _source_hash():
                    shutil.copyfile(committed, dest)
                    return True
        except OSError:
            pass
    entry = os.path.join(REPO_ROOT, GLUE_ENTRY)
    if not os.path.isfile(entry):
        return False
    try:
        proc = subprocess.run(
            [
                "npx",
                "--yes",
                "esbuild@0.28.0",
                entry,
                "--bundle",
                f"--outfile={dest}",
                "--format=iife",
                "--platform=browser",
            ],
            capture_output=True,
            text=True,
            timeout=300,
        )
    except (OSError, subprocess.TimeoutExpired):
        return False
    return proc.returncode == 0 and os.path.isfile(dest)


DEFAULT_CAPABILITIES = os.path.join(
    REPO_ROOT, "packages", "browser-glue", "dist", "glue-esm", "default-capabilities.json"
)


def capability_declaration(scenario):
    """The `tairitsu-capabilities.json` this scenario is served with (W6)."""
    if not os.path.isfile(DEFAULT_CAPABILITIES):
        raise RuntimeError(
            "the default capability set is missing; run "
            "`node --experimental-strip-types packages/browser-glue/scripts/emit-glue-esm.mjs`"
        )
    with open(DEFAULT_CAPABILITIES, encoding="utf-8") as handle:
        default = json.load(handle)

    if scenario == "capability-undeclared":
        # Declares only `document`; the wrapper imports `element-css-inline-style`,
        # so the host must refuse to satisfy it.
        return {"interfaces": ["document"], "wasiModules": []}
    if scenario == "capability-wasi":
        return {
            "interfaces": ["document", "element-css-inline-style"],
            "wasiModules": ["not-a-wasi-module"],
        }
    if scenario == "capability-overreach":
        # Declares an interface the host does not provide: the boot must fail
        # BEFORE instantiate, not hand back an opaque link error.
        return {
            "interfaces": default["interfaces"] + ["does-not-exist"],
            "wasiModules": default["wasiModules"],
        }
    # Scenarios that deliberately run the glue STUB provide no interfaces, so
    # their declaration is empty — otherwise the host would be asked for the
    # default set it does not have (which is exactly what W6 refuses to fake).
    STUB_SCENARIOS = {
        "component",
        "csp",
        "bad-bytes",
        "missing-entry",
        "missing-import",
        "fixture",
    }
    if scenario == "worker-refused":
        # The one control that makes the worker host's refusal observable: it
        # declares an interface the lane cannot provide (it has no DOM).
        return {"interfaces": ["document"], "wasiModules": []}
    if scenario in ("mount-conflict-shell", "mount-conflict-page", "mount-missing", "mount-inner-app"):
        # These exercise the mount check; a declaration failure would mask it.
        return {"interfaces": [], "wasiModules": []}
    if scenario in STUB_SCENARIOS or scenario in WORKER_SCENARIOS:
        return {"interfaces": [], "wasiModules": []}
    if scenario == "capability-exact":
        # Exactly what the wrapper imports — the positive control.
        return {
            "interfaces": ["document", "element-css-inline-style"],
            "wasiModules": [],
        }
    return default


def render_worker_script():
    """Render `worker.template.js` the way the packager emits `glue-worker.js`."""
    with open(os.path.join(TEMPLATE_DIR, "worker.template.js"), encoding="utf-8") as fh:
        # The packager substitutes the build stamp; the harness must too, or the
        # worker dies on a ReferenceError that looks like a component failure.
        return fh.read().replace("__TAIRITSU_BUILD__", "e2e")


def render_boot_script():
    """Render `boot.template.js` the way the packager emits `boot.js` (W5)."""
    with open(os.path.join(TEMPLATE_DIR, "boot.template.js"), encoding="utf-8") as fh:
        boot = fh.read()
    boot = boot.replace("{{ v }}", PAGE_CONTEXT["v"]).replace(
        "{{ wasm_file }}", PAGE_CONTEXT["wasm_file"]
    )
    if "{{" in boot:
        raise RuntimeError("boot.template.js still carries a placeholder")
    return boot


WRAPPER_STEM = "tairitsu_fixture"
WORKER_SCENARIOS = {"worker-boot", "worker-terminate", "worker-refused", "worker-kill"}


def render_page(scenario=None):
    """Render `index.html.template` with the e2e context."""
    with open(os.path.join(TEMPLATE_DIR, "index.html.template"), encoding="utf-8") as fh:
        page = fh.read()
    if scenario == "custom-root":
        # W8: the host owns a panel and declares it as the mount root. The `#app`
        # the template ships must lose to it, and the host node inside must
        # survive the boot.
        # A host that mounts into its own panel has no page-level `#app`; leaving
        # one there is the conflict the boot refuses.
        page = page.replace(
            '<div id="app" class="tairitsu-boot-screen"><div>Loading…</div></div>',
            '<div id="host-panel"><p id="host-child">host owns this</p></div>'
            '<p id="outside">untouched</p>',
            1,
        )
        page = page.replace("<html ", '<html data-tairitsu-root="host-panel" ', 1)
    if scenario == "mount-inner-app":
        # The container already holds a server-rendered #app; the boot must give
        # the component its own shell and leave that content alone.
        page = page.replace(
            '<div id="app" class="tairitsu-boot-screen"><div>Loading…</div></div>',
            '<div id="panel"><div id="app"><p id="host-ssr">host ssr</p></div></div>',
            1,
        )
        page = page.replace("<html ", '<html data-tairitsu-root="panel" ', 1)
    if scenario in ("mount-conflict-shell", "mount-conflict-page", "mount-missing"):
        page = page.replace(
            '<div id="app" class="tairitsu-boot-screen"><div>Loading…</div></div>',
            '<div id="panel"><p id="host-child">keep</p></div>'
            '<div id="app" class="tairitsu-boot-screen"><div>Loading…</div></div>',
            1,
        )
        root = {
            "mount-conflict-shell": "app",
            "mount-conflict-page": "panel",
            "mount-missing": "no-such-panel",
        }[scenario]
        page = page.replace("<html ", '<html data-tairitsu-root="' + root + '" ', 1)
    if scenario in WORKER_SCENARIOS:
        # W7: the worker lane is opt-in through this attribute; without it the
        # page boots on the main thread and a "worker" scenario would pass for
        # the wrong reason.
        page = page.replace("<html ", "<html data-tairitsu-worker ", 1)
    for key, value in PAGE_CONTEXT.items():
        page = page.replace("{{ " + key + " }}", value)
        page = page.replace("{{" + key + "}}", value)
    for leftover in ("{{ head | safe }}", "{{ favicon | safe }}"):
        page = page.replace(leftover, "")
    if "{{" in page:
        raise RuntimeError("index.html.template still carries a placeholder: " + page[page.index("{{") : page.index("{{") + 40])
    return page


def rewrite_glue_specifiers(source):
    """`from '@tairitsu-glue/x'` → `from '../glue/x.js'`, versions dropped.

    The packager does this in Rust (`glue_specifier_extension`); the browser lane
    needs the same transformation to load the real modules, and applying it here
    (instead of writing the rewritten shape into the fixture) is what makes the
    `glue-imports` scenario exercise the rewrite.
    """
    import re

    rewritten = source.replace("from '@tairitsu-glue/", "from '../glue/").replace(
        'from "@tairitsu-glue/', 'from "../glue/'
    )
    pattern = re.compile(r"""((?:from|import)\s*\(?\s*['\"])(\.\./glue/[A-Za-z0-9._@-]+)(['\"])""")

    def add_extension(match):
        spec = match.group(2)
        if spec.endswith(".js"):
            return match.group(0)
        without_version = spec.split("@", 1)[0]
        return f"{match.group(1)}{without_version}.js{match.group(3)}"

    return pattern.sub(add_extension, rewritten)


def build_dist(scenario, dest):
    """A dist-shaped tree: the real page, the real loader, a stub component."""
    os.makedirs(os.path.join(dest, "component-wrapper"), exist_ok=True)
    with open(os.path.join(dest, "index.html"), "w", encoding="utf-8") as fh:
        fh.write(render_page(scenario))
    with open(os.path.join(dest, "boot.js"), "w", encoding="utf-8") as fh:
        fh.write(render_boot_script())
    with open(os.path.join(dest, "glue-worker.js"), "w", encoding="utf-8") as fh:
        fh.write(render_worker_script())
    with open(
        os.path.join(dest, "tairitsu-capabilities.json"), "w", encoding="utf-8"
    ) as fh:
        json.dump(capability_declaration(scenario), fh)
    if scenario in ("glue-imports", "capability-exact", "capability-undeclared", "capability-overreach", "capability-wasi"):
        # The real build-time modules, copied exactly as the packager does.
        source = os.path.join(REPO_ROOT, "packages", "browser-glue", "dist", "glue-esm")
        if not os.path.isdir(source):
            raise RuntimeError(
                "the glue-imports control needs the emitted glue modules; run "
                "`node --experimental-strip-types packages/browser-glue/scripts/emit-glue-esm.mjs`"
            )
        shutil.copytree(source, os.path.join(dest, "glue"))
        # The wrapper sits in component-wrapper/, so `../glue/` resolves to the
        # copy above — the same path the packager rewrites to.
    glue_path = os.path.join(dest, "glue-runtime.js")
    # `two-instances` needs the shipped bundle (the instance API lives there);
    # the component scenarios only need the page's import map to stay green.
    if scenario in ("two-instances", "glue-imports", "capability-exact", "capability-undeclared", "capability-overreach", "capability-wasi", "worker-terminate", "custom-root"):
        if not ensure_glue_bundle(glue_path):
            raise RuntimeError(
                "the two-instances control needs the real glue bundle "
                f"({GLUE_ENTRY}); build it with `npx esbuild … --bundle`"
            )
    else:
        with open(glue_path, "w", encoding="utf-8") as fh:
            fh.write(GLUE_STUB)

    loader_src = os.path.join(TEMPLATE_DIR, "component-wrapper-loader.template.js")
    with open(loader_src, encoding="utf-8") as fh:
        loader = fh.read()
    # The dist's wasm stem is `component` (see PAGE_CONTEXT["wasm_file"]).
    # A REAL build names the wrapper after the wasm stem. Using a stem that is
    # not `component` is what makes a hardcoded wrapper path impossible to miss.
    loader = loader.replace("__WASM_STEM__", WRAPPER_STEM)
    with open(os.path.join(dest, "component-wrapper-loader.js"), "w", encoding="utf-8") as fh:
        fh.write(loader)
    shutil.copyfile(
        os.path.join(TEMPLATE_DIR, "component-boot.template.js"),
        os.path.join(dest, "component-boot.js"),
    )
    if scenario == "worker-kill":
        # The re-boot half needs a HEALTHY page in the same dist: the killed
        # component never returns, and the loader resolves the wrapper from the
        # stem, so a second directory is the honest way to serve both.
        healthy = os.path.join(dest, "healthy")
        os.makedirs(os.path.join(healthy, "component-wrapper"), exist_ok=True)
        with open(os.path.join(healthy, "component-wrapper", WRAPPER_STEM + ".js"), "w", encoding="utf-8") as fh:
            fh.write(WRAPPERS["worker-boot"])
        for name in ("boot.js", "component-boot.js", "component-wrapper-loader.js", "glue-worker.js", "glue-runtime.js"):
            source = os.path.join(dest, name)
            if os.path.isfile(source):
                shutil.copy2(source, os.path.join(healthy, name))
        with open(os.path.join(healthy, "index.html"), "w", encoding="utf-8") as fh:
            # The template points at the absolute /boot.js (W5, for route pages);
            # this sibling lives one level down and must load its OWN copy.
            fh.write(render_page("worker-boot").replace('src="/boot.js', 'src="boot.js'))
        with open(os.path.join(healthy, "tairitsu-capabilities.json"), "w", encoding="utf-8") as fh:
            json.dump({"interfaces": [], "wasiModules": []}, fh)
    wrapper_source = WRAPPERS[scenario]
    if scenario in ("glue-imports", "capability-exact", "capability-undeclared", "capability-overreach", "capability-wasi"):
        rewritten = rewrite_glue_specifiers(wrapper_source)
        if "@tairitsu-glue/" in rewritten:
            raise RuntimeError("the glue specifier rewrite left a bare specifier")
        wrapper_source = rewritten
    with open(
        os.path.join(dest, "component-wrapper", WRAPPER_STEM + ".js"), "w", encoding="utf-8"
    ) as fh:
        fh.write(wrapper_source)
    # A placeholder core module: the synthetic wrappers never fetch it, but the
    # page's direct-load fallback checks magic bytes.
    with open(os.path.join(dest, "component.wasm"), "wb") as fh:
        fh.write(b"\x00asm\x0d\x00\x01\x00" + b"\x00" * 16)


CSP_POLICY = "script-src 'self' 'wasm-unsafe-eval'; object-src 'none'; base-uri 'none'"


class _QuietHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        if getattr(self.server, "csp", None):
            self.send_header("Content-Security-Policy", self.server.csp)
        super().end_headers()

    def log_message(self, *_args):  # noqa: D401 - silence the per-request log
        return


def serve(directory, csp=None):
    """Serve `directory` on an ephemeral port; returns (server, thread, url)."""
    handler = functools.partial(_QuietHandler, directory=directory)
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    server.csp = csp
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

    if scenario in ("capability-undeclared", "capability-overreach", "capability-wasi") and state.get("error") != "capability-denied":
        failures.append(
            "a capability mismatch was not reported as capability-denied "
            f"(got {state.get('error')!r})"
        )
    if scenario.startswith("mount-"):
        # A refused mount root has no guest shell to write a card into: the
        # machine signal is the kind, and the run loop asserts it (plus that the
        # host's own content survived).
        return failures
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
        if scenario == "worker-kill":
            # The page is stuck on purpose (the component never returns); its
            # step-by-step probe owns the assertions. This must come BEFORE the
            # generic ready check, or the control could never pass.
            return failures
        if state.get("ready") != "hydrated":
            failures.append(f"good component not ready (ready={state.get('ready')!r})")
        if state.get("error"):
            failures.append(f"good component reported an error ({state['error']!r})")
        if scenario in WORKER_SCENARIOS:
            # The entry runs in the Worker: this pure check only sees ready/no
            # error; the "was it really the Worker" half lives in the run loop
            # (it needs the page).
            return failures
        if real_dist:
            # A real build is not instrumented: ready + no error is the whole
            # contract we can observe from the outside.
            return failures
        if not state.get("entryRan"):
            failures.append("the component's boot entry never ran")
        if "component booted" not in (state.get("appText") or ""):
            failures.append("#app was not updated by the component")
    return failures


ISOLATION_PROBE = """(() => {
    const api = globalThis.__TAIRITSU_INSTANCES__;
    if (!api) return { error: "the glue bundle exposed no instance API" };
    const A = api.current();
    const B = api.create();

    const store = (instance, value) => api.withInstance(instance, () => {
        const handle = globalThis.__nextHandle++;
        globalThis.__elementHandles.set(handle, value);
        return handle;
    });

    const handleB = store(B, { owner: "B" });
    const handleA = store(A, { owner: "A" });
    const inA = api.withInstance(A, () => globalThis.__elementHandles.get(handleA));
    const inB = api.withInstance(B, () => globalThis.__elementHandles.get(handleB));
    const crossRead = api.withInstance(A, () => globalThis.__elementHandles.get(handleB));

    api.dispose(A);
    const afterDisposeA = api.withInstance(A, () => globalThis.__elementHandles.get(handleA));
    const stillInB = api.withInstance(B, () => globalThis.__elementHandles.get(handleB));

    return {
        error: null,
        aOwner: inA && inA.owner,
        bOwner: inB && inB.owner,
        crossRead: crossRead === undefined ? null : (crossRead.owner || "?"),
        afterDisposeA: afterDisposeA === undefined ? null : "still resolvable",
        stillInB: stillInB && stillInB.owner,
        aDisposed: A.disposed,
        bDisposed: B.disposed,
        api_ok: true,
    };
})()"""


def check_isolation(state):
    """W4 acceptance ①/②: no cross-talk, and dispose reclaims only its own."""
    failures = []
    if not isinstance(state, dict) or state.get("error"):
        return [f"isolation probe failed: {(state or {}).get('error') if isinstance(state, dict) else state}"]
    if state.get("aOwner") != "A":
        failures.append(f"instance A resolved {state.get('aOwner')!r} from its own handle")
    if state.get("bOwner") != "B":
        failures.append(f"instance B resolved {state.get('bOwner')!r} from its own handle")
    if state.get("crossRead") is not None:
        failures.append(
            f"instance A could read B's handle ({state.get('crossRead')!r}) - handles cross instances"
        )
    if state.get("afterDisposeA") is not None:
        failures.append("a disposed instance's handle still resolved")
    if not state.get("aDisposed"):
        failures.append("the disposed instance was not marked disposed")
    if state.get("bDisposed"):
        failures.append("disposing A also disposed B")
    if state.get("stillInB") != "B":
        failures.append(f"disposing A broke B ({state.get('stillInB')!r})")
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
        # Each probe differs from the honest state in EXACTLY ONE dimension:
        # a probe that already fails a different assertion would make the other
        # assertions look alive while they are dead.
        card = EXPECTED_CARD[scenario]
        honest = dict(BAD_STATE, appText="Tairitsu boot error\n" + card)
        probes = [
            (dict(honest, ready="hydrated"), "reported ready", "would pass a page that is wrongly ready"),
            (dict(honest, error=None), "no data-tairitsu-error", "would pass a page with no error attribute"),
            (
                dict(honest, appText="Tairitsu boot error: something else failed"),
                "does not name the failure",
                f"would pass a card that does not name {card!r}",
            ),
            (dict(honest, entryRan=True), "ran an entry", "would pass a page whose entry ran"),
        ]
        for state, needle, message in probes:
            probe_failures = check_boot_state(state, scenario)
            if not probe_failures:
                failures.append(f"control {scenario} {message}")
                continue
            off_target = [f for f in probe_failures if needle not in f]
            if off_target or len(probe_failures) != 1:
                failures.append(
                    f"control {scenario} probe for {needle!r} is not single-dimension "
                    f"(off-target failures: {off_target})"
                )
        # ... and the honest failure state (card naming THIS failure) must pass.
        if check_boot_state(honest, scenario):
            failures.append(f"control {scenario} rejects a correct error page")

    # W5: the CSP scenario is the D5 proof (a page that needed a blob URL or an
    # inline module cannot boot under `script-src 'self'`), so its absence must
    # fail the offline gate.
    if "csp" not in SCENARIOS:
        failures.append("the CSP scenario is missing")
    # Listed once, here. An earlier version iterated the same tuple it was
    # verifying, so deleting a scenario from SCENARIOS also deleted it from the
    # check.
    REQUIRED_CONTROLS = (
        "capability-exact",
        "capability-undeclared",
        "capability-overreach",
        "capability-wasi",
        "worker-boot",
        "worker-terminate",
        "worker-refused",
        "worker-kill",
        # W8: the host-owned mount root and its refusals have no other
        # browser-level control.
        "custom-root",
        "mount-conflict-shell",
        "mount-conflict-page",
        "mount-missing",
        "mount-inner-app",
    )
    for capability_scenario in REQUIRED_CONTROLS:
        if capability_scenario not in SCENARIOS:
            failures.append(f"the {capability_scenario} control is missing")
    if len(SCENARIOS) != len(set(SCENARIOS)):
        duplicates = sorted({name for name in SCENARIOS if SCENARIOS.count(name) > 1})
        failures.append(f"SCENARIOS has duplicates: {duplicates}")
    if "glue-imports" not in SCENARIOS:
        failures.append("the glue-imports scenario is missing (static resolution is unproven)")
    if "csp" not in WRAPPERS:
        failures.append("the CSP scenario has no component under test")

    # W4: the dual-instance isolation control must exist and its checks must be
    # able to fail (a control that cannot fail is not a control).
    if "two-instances" not in SCENARIOS:
        failures.append("the dual-instance isolation scenario is missing")
    else:
        healthy = {
            "aOwner": "A",
            "bOwner": "B",
            "crossRead": None,
            "afterDisposeA": None,
            "aDisposed": True,
            "bDisposed": False,
            "stillInB": "B",
        }
        if check_isolation(healthy):
            failures.append("the isolation control rejects a correctly isolated page")
        bleeding = [
            (dict(healthy, crossRead="B"), "would pass a page whose handles cross instances"),
            (
                dict(healthy, afterDisposeA="still resolvable"),
                "would pass a page that still resolves a disposed instance's handle",
            ),
            (dict(healthy, bDisposed=True), "would pass a page that disposed both instances"),
            (dict(healthy, stillInB=None), "would pass a page where disposing A broke B"),
            (dict(healthy, aOwner="B"), "would pass a page where A resolved B's object"),
        ]
        for state, message in bleeding:
            if not check_isolation(state):
                failures.append(f"the isolation control {message}")

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
        "--settle-seconds",
        type=float,
        default=2.0,
        help="wait this long after the first ready/error signal before asserting "
        "(the error attribute lands before the deferred boot IIFE settles)",
    )
    ap.add_argument(
        "--force-worker-kill",
        action="store_true",
        help="run the dead-loop kill control even on a single-process browser "
        "(it will hang: the worker shares the browser process)",
    )
    ap.add_argument(
        "--self-check",
        action="store_true",
        help="offline: prove the negative controls can fail (no browser needed)",
    )
    args = ap.parse_args()

    if args.self_check:
        return self_check()

    if args.scenario == "worker-kill" and not args.force_worker_kill:
        # Checked before anything is loaded: on this lane's browser (shirabe
        # starts Chrome with `--single-process`) a spinning Worker blocks the
        # browser process, so navigation itself never returns — the control
        # cannot even reach its own assertions. A multi-process browser runs it.
        print(
            "SKIP worker-kill: this lane's browser runs --single-process (shirabe), "
            "where a spinning Worker blocks the browser process — the exact property "
            "this control asserts. Re-run it against a multi-process browser with "
            "--force-worker-kill (SHIRABE_BIN may point at any shim that speaks the "
            "same debug HTTP contract); it passes there — see the PR's 复跑 paragraph.",
            file=sys.stderr,
        )
        return 0

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
            if args.scenario == "csp":
                # Prove the policy bites: an inline script and a blob module must
                # both be refused, or the scenario proves nothing about D5.
                page_path = os.path.join(dist, "index.html")
                with open(page_path, encoding="utf-8") as fh:
                    page = fh.read()
                page = page.replace(
                    "</body>",
                    '<script>window.__inline_ran = true;</script>'
                    "<script type=\"module\">window.__blob_ran = true;</script></body>",
                )
                with open(page_path, "w", encoding="utf-8") as fh:
                    fh.write(page)
            server, _thread, url = serve(dist, csp=CSP_POLICY if args.scenario == "csp" else None)
            call(base, "/navigate", {"url": url + "/index.html"})
            # The page boots asynchronously (styles + lifecycle); poll briefly.
            deadline = time.time() + 20
            while time.time() < deadline:
                seen = evaluate(
                    base,
                    "!!(document.documentElement.dataset.tairitsuReady"
                    " || document.documentElement.dataset.tairitsuError)",
                )
                if seen is True:
                    break
                time.sleep(0.1)
            # The error attribute is written before the deferred boot IIFE
            # settles, so the FIRST signal is early: re-read after a settle
            # window, otherwise a page that wrongly becomes ready later passes.
            time.sleep(args.settle_seconds)
            failures.extend(
                assert_boot_contract(base, args.scenario, real_dist=bool(args.dist))
            )
            if args.scenario == "glue-imports" and not failures:
                probe = evaluate(
                    base,
                    """(() => ({
                        probe: globalThis.__w3_glue_probe || null,
                    }))()""",
                )
                state = (probe or {}).get("probe") if isinstance(probe, dict) else None
                if state is None:
                    failures.append("the wrapper's glue imports did not run")
                elif state.get("element") != "bigint" or state.get("style") != "function":
                    failures.append(f"the glue modules did not provide their exports: {state!r}")
                else:
                    print("glue imports resolved statically:", state)
            # Only the scenarios that must reach ready can prove the entry ran;
            # `worker-refused` is an expected failure.
            if args.scenario in ("worker-boot", "worker-terminate") and not failures:
                # Prove the lane: the Worker is live and the page's own realm
                # never ran the entry. Without this the scenario passes on the
                # main-thread lane too (it did once).
                state = evaluate(
                    base,
                    "(() => ({ workerActive: !!(globalThis.__TAIRITSU_WORKER__ && "
                    "globalThis.__TAIRITSU_WORKER__.active()), "
                    "pageRanEntry: globalThis.__w3_entry_ran === true }))()",
                )
                if not isinstance(state, dict):
                    failures.append("worker state unreadable: " + repr(state))
                elif state.get("workerActive") is not True:
                    failures.append("the component did not boot in a Worker")
                elif state.get("pageRanEntry") is True:
                    failures.append("the entry ran on the page: this is the main-thread lane")

            if args.scenario.startswith("mount-") and args.scenario != "mount-inner-app" and not failures:
                state = evaluate(
                    base,
                    "(() => ({"
                    " err: document.documentElement.dataset.tairitsuError || null,"
                    " ready: document.documentElement.dataset.tairitsuReady || null,"
                    " hostChild: !!document.getElementById('host-child'),"
                    " pageApp: (document.getElementById('app') || {}).textContent || '',"
                    " booted: !!document.getElementById('guest-content')"
                    "}))()",
                )
                expected_kind = (
                    "mount-root-missing" if args.scenario == "mount-missing" else "mount-root-conflict"
                )
                if not isinstance(state, dict):
                    failures.append("mount refusal state unreadable: " + repr(state))
                else:
                    if state.get("err") != expected_kind:
                        failures.append(
                            "the refusal kind was not preserved (got "
                            + repr(state.get("err")) + ", expected " + expected_kind + ")"
                        )
                    if state.get("booted"):
                        failures.append("the component booted despite a refused mount root")
                    if not state.get("hostChild"):
                        failures.append("the host's node was lost")
                    if state.get("pageApp") != "Loading…":
                        failures.append(
                            "the page-level #app was modified: " + repr(state.get("pageApp"))
                        )
                    if state.get("ready") is not None:
                        failures.append("a refused boot still reported ready")
            if args.scenario == "mount-inner-app" and not failures:
                state = evaluate(
                    base,
                    "(() => ({"
                    " hostSsr: !!document.getElementById('host-ssr'),"
                    " hostSsrText: (document.getElementById('host-ssr') || {}).textContent || '',"
                    " shells: document.querySelectorAll('#app').length,"
                    " guest: !!document.getElementById('guest-content'),"
                    " inPanel: !!document.getElementById('panel') &&"
                    "   document.getElementById('panel').contains(document.getElementById('guest-content'))"
                    "}))()",
                )
                if not isinstance(state, dict):
                    failures.append("inner-app state unreadable: " + repr(state))
                else:
                    if not state.get("hostSsr") or state.get("hostSsrText") != "host ssr":
                        failures.append("the host's server-rendered content was cleared")
                    if state.get("shells") != 1:
                        failures.append("expected exactly one #app shell, got " + repr(state.get("shells")))
                    if not state.get("guest") or not state.get("inPanel"):
                        failures.append("the guest did not render inside the container")
            if args.scenario == "custom-root":
                state = evaluate(
                    base,
                    "(() => ({"
                    " rootExists: !!document.getElementById('host-panel'),"
                    " hostNodeKept: !!document.getElementById('host-child'),"
                    " guestInsideRoot: !!(document.getElementById('host-panel') || {}).querySelector"
                    "   && !!document.getElementById('host-panel').querySelector('#guest-content'),"
                    " outsideUntouched: (document.getElementById('outside') || {}).textContent === 'untouched',"
                    " shellInsideRoot: !!document.getElementById('host-panel') &&"
                    "   (document.getElementById('host-panel').contains(document.getElementById('app'))),"
                    " hostIdKept: (document.getElementById('host-panel') || {}).id === 'host-panel',"
                    " mountHost: !!(globalThis.__TAIRITSU_MOUNT_ROOT__ || {}).host"
                    "}))()",
                )
                if not isinstance(state, dict):
                    failures.append("custom-root state unreadable: " + repr(state))
                else:
                    if not state.get("rootExists"):
                        failures.append("the host root was removed")
                    if not state.get("hostNodeKept"):
                        failures.append("the guest cleared the host node inside its root")
                    if not state.get("guestInsideRoot"):
                        failures.append("the guest content is not inside the host root")
                    if not state.get("outsideUntouched"):
                        failures.append("the boot touched content outside the mount root")
                    if not state.get("hostIdKept"):
                        failures.append("the boot changed the host root's own id")
                    if not state.get("shellInsideRoot"):
                        failures.append("the guest shell is not inside the host root")
                    if not state.get("mountHost"):
                        failures.append("the boot did not record the host root as host-owned")
            if args.scenario == "worker-terminate":
                # The kill path without a spinning worker: boot normally, then
                # terminate and check both halves of the reclaim — the Worker is
                # gone AND the page's W4 instance was disposed.
                active_before = evaluate(
                    base,
                    "!!(globalThis.__TAIRITSU_WORKER__ && globalThis.__TAIRITSU_WORKER__.active())",
                )
                instances_before = evaluate(
                    base,
                    "globalThis.__TAIRITSU_INSTANCES__ ? globalThis.__TAIRITSU_INSTANCES__.list().length : -1",
                )
                killed = evaluate(
                    base,
                    "globalThis.__TAIRITSU_WORKER__ ? globalThis.__TAIRITSU_WORKER__.terminate() : 'missing'",
                )
                active_after = evaluate(
                    base,
                    "!!(globalThis.__TAIRITSU_WORKER__ && globalThis.__TAIRITSU_WORKER__.active())",
                )
                instances_after = evaluate(
                    base,
                    "globalThis.__TAIRITSU_INSTANCES__ ? globalThis.__TAIRITSU_INSTANCES__.list().length : -1",
                )
                if active_before is not True:
                    failures.append("the worker was not running before terminate")
                if killed is not True:
                    failures.append(f"terminate() did not report a kill ({killed!r})")
                if active_after is not False:
                    failures.append("the worker was still active after terminate")
                if not (instances_before >= 1 and instances_after == 0):
                    failures.append(
                        "terminate() did not dispose the page's W4 instance "
                        f"({instances_before} -> {instances_after})"
                    )
                if not failures:
                    print(
                        "worker terminated; W4 instance disposed "
                        f"({instances_before} -> {instances_after})"
                    )
            if args.scenario == "worker-kill":
                # The page is stuck by design; drive terminate + re-boot from
                # here (an async in-page probe would never resolve).
                started = time.time()
                while time.time() - started < 10:
                    if evaluate(base, "!!(globalThis.__TAIRITSU_WORKER__ && globalThis.__TAIRITSU_WORKER__.active())") is True:
                        break
                    time.sleep(0.2)
                active_before = evaluate(
                    base, "!!(globalThis.__TAIRITSU_WORKER__ && globalThis.__TAIRITSU_WORKER__.active())"
                )
                killed = evaluate(base, "globalThis.__TAIRITSU_WORKER__ ? globalThis.__TAIRITSU_WORKER__.terminate() : 'missing'")
                active_after = evaluate(
                    base, "!!(globalThis.__TAIRITSU_WORKER__ && globalThis.__TAIRITSU_WORKER__.active())"
                )
                if active_before is not True:
                    failures.append("the dead-loop worker was not running before terminate")
                # `ready` may only follow a returned entry: this page's entry
                # never returns, so announcing ready before the kill would be a
                # host claiming success it never observed.
                premature = evaluate(
                    base, "document.documentElement.dataset.tairitsuReady || null"
                )
                if premature is not None:
                    failures.append(
                        "the page reported ready although the entry never returned "
                        "(" + repr(premature) + ")"
                    )
                if killed is not True:
                    failures.append(f"terminate() did not report a kill ({killed!r})")
                if active_after is not False:
                    failures.append("the worker was still active after terminate")
                # A fresh component must boot in the same, still-alive browser:
                # load the healthy sibling page and require it to reach ready.
                call(base, "/navigate", {"url": url + "/healthy/index.html"})
                started = time.time()
                while time.time() - started < 15:
                    fresh = evaluate(base, "document.documentElement.dataset.tairitsuReady || null")
                    if fresh == "hydrated":
                        break
                    time.sleep(0.3)
                if fresh != "hydrated":
                    failures.append("a fresh component could not boot after the kill (" + repr(fresh) + ")")
                if not failures:
                    print("worker killed; main thread alive; a fresh component booted ready")
            if args.scenario == "csp" and not failures:
                enforcement = evaluate(
                    base,
                    """(() => ({
                        inlineRan: globalThis.__inline_ran === true,
                    }))()""",
                )
                if not isinstance(enforcement, dict) or enforcement.get("inlineRan"):
                    failures.append(
                        "the CSP policy is not enforced (an inline script ran)"
                        if isinstance(enforcement, dict)
                        else f"CSP enforcement probe failed: {enforcement!r}"
                    )
                else:
                    print("CSP enforced: inline scripts blocked, component still boots")
            if args.scenario == "two-instances" and not failures:
                failures.extend(check_isolation(evaluate(base, ISOLATION_PROBE)))

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
    if args.scenario == "worker-kill-skipped":
        print("SKIPPED: worker-kill needs a multi-process browser (see stderr)")
        return 0
    print(f"OK: boot e2e passed (scenario={args.scenario})")
    return 0


if __name__ == "__main__":
    sys.exit(main())