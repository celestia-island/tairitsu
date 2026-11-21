// W7: the component host that runs in a dedicated Worker.
//
// The lane exists for one property the main thread cannot offer: a component
// that never yields (an infinite loop in wasm, a runaway computation) can be
// killed. `terminate()` on the Worker drops the whole realm; the page survives,
// and a new Worker can boot a fresh instance.
//
// Scope: this lane runs components that do NOT touch the DOM. The glue's handle
// tables wrap live DOM objects, and a Worker has no DOM, so a component that
// needs `document`/`window` stays on the main thread. The capability check below
// enforces that: a declaration naming any interface is refused here, because
// this host can provide none of them. That is the boundary between the worker
// lane and the main-thread lane, not a missing feature.
//
// Protocol (page → worker):
//   { type: "boot", wasm: "<file>", wrapper: "<path>", capabilities: {...} }
//   { type: "call", name: "<export>", args: [...] }
//   { type: "stop" }                       // cooperative; see below
// (worker → page):
//   { type: "ready", entry: "<name>" }
//   { type: "error", kind: "<kind>", message: "<text>" }
//   { type: "result", id: <n>, value: <any> }
//
// `stop` is the cooperative half: a component that yields can be asked to wind
// down. It cannot interrupt a synchronous wasm loop — that is what `terminate()`
// on the Worker is for, and why the hard kill does not live here.

let entryName = null;
let stopped = false;

const post = (message) => self.postMessage(message);

const fail = (kind, error) => {
  const message = error && error.message ? error.message : String(error || kind);
  post({ type: "error", kind, message });
};

const normalize = (name) =>
  String(name || "")
    .replace(/^@tairitsu-glue\//, "")
    .replace(/^tairitsu-browser:full\//, "")
    .replace(/@\d+\.\d+\.\d+$/, "");

async function boot({ wrapper, capabilities }) {
  // W6 applies here too: the declaration decides, and this host can provide no
  // glue interfaces (no DOM). A declaration that asks for one is refused rather
  // than silently ignored — the component belongs on the main thread.
  const declaredInterfaces = (capabilities.interfaces || []).map(normalize);
  if (declaredInterfaces.length > 0) {
    throw Object.assign(
      new Error(
        "the component declares glue interfaces this lane cannot provide (" +
          declaredInterfaces.sort().join(", ") +
          ") — components that touch the DOM stay on the main thread",
      ),
      { kind: "capability-denied" },
    );
  }
  const declaredWasi = capabilities.wasiModules || [];
  if (declaredWasi.length > 0) {
    throw Object.assign(
      new Error(
        "the component declares WASI modules this lane does not provide (" +
          declaredWasi.sort().join(", ") +
          ")",
      ),
      { kind: "capability-denied" },
    );
  }

  // No glue imports and no WASI shim: a compute-only component needs neither.
  const imports = {};
  let loaded;
  try {
    loaded = await import(wrapper);
  } catch (error) {
    fail("instantiate-failed", error);
    return;
  }
  const { bootComponent } = await import("./component-boot.js");
  const surface = {
    ready: (entry) => {
      entryName = entry;
      post({ type: "ready", entry });
    },
    fail: (kind, error) => fail(kind, error),
  };
  try {
    await bootComponent({ result: loaded, surface, log: console });
  } catch (error) {
    // `bootComponent` already reported through the surface; nothing to add.
  }
}

self.onmessage = async (event) => {
  const message = event.data || {};
  if (stopped && message.type !== "stop") {
    fail("stopped", new Error("this worker host was stopped"));
    return;
  }
  if (message.type === "boot") {
    try {
      await boot(message);
    } catch (error) {
      fail(error.kind || "boot-failed", error);
    }
    return;
  }
  if (message.type === "call") {
    fail("call-unsupported", new Error("this host exposes no callable surface yet"));
    return;
  }
  if (message.type === "stop") {
    stopped = true;
    post({ type: "stopped" });
    return;
  }
  fail("unknown-message", new Error("unknown message type: " + String(message.type)));
};