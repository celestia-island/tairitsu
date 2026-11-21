import assert from "node:assert/strict";
import test from "node:test";

import { buildImports, HostClientError, loadComponent } from "../src/index.ts";

const fakeGlue = { document: { createElement: () => "el" }, element: { setAttribute: () => {} } };

test("buildImports injects only what the declaration names", () => {
  const imports = buildImports({ interfaces: ["document"] }, fakeGlue);
  assert.deepEqual(Object.keys(imports), ["tairitsu-browser:full/document@0.2.0"]);
});

test("a declared interface with no provider is refused before instantiate", () => {
  assert.throws(
    () => buildImports({ interfaces: ["document", "element"] }, { document: fakeGlue.document }),
    (error) => error instanceof HostClientError && /does not provide: element/.test(error.message),
  );
});

test("an unknown WASI module is refused, like an unknown interface", () => {
  assert.throws(
    () => buildImports({ wasiModules: ["nope"] }),
    (error) => error instanceof HostClientError && /WASI modules the host does not provide: nope/.test(error.message),
  );
});

test("a declared WASI module with no provider is refused", () => {
  assert.throws(
    () => buildImports({ wasiModules: ["clocks"] }, {}, {}),
    (error) => error instanceof HostClientError && /does not provide: clocks/.test(error.message),
  );
});

test("call is synchronous and resolves through the namespace or lifecycle", async () => {
  const component = await loadComponent({
    wrapper: "memory:component",
    loaders: {
      importModule: async () => ({
        encode: (text) => ({ size: 1, text }),
        lifecycle: { start: () => "started" },
      }),
    },
  });
  assert.deepEqual(component.call("encode", "hi"), { size: 1, text: "hi" });
  assert.equal(component.call("start"), "started");
});

test("a wrapper with an instantiate() ABI shape is supported", async () => {
  const component = await loadComponent({
    wrapper: "memory:abi",
    loaders: {
      importModule: async () => ({
        instantiate: async (imports) => ({ encode: () => Object.keys(imports).length }),
      }),
    },
  });
  assert.equal(component.call("encode"), 0);
});

test("a wrapper that cannot be imported fails closed", async () => {
  await assert.rejects(
    loadComponent({ wrapper: "memory:missing", loaders: { importModule: async () => { throw new Error("404"); } } }),
    (error) => error instanceof HostClientError && error.kind === "wrapper-not-found",
  );
});

test("a broken instantiate reports both ABI attempts", async () => {
  await assert.rejects(
    loadComponent({
      wrapper: "memory:broken",
      loaders: { importModule: async () => ({ instantiate: async () => { throw new Error("boom"); } }) },
    }),
    (error) =>
      error instanceof HostClientError &&
      error.kind === "instantiate-failed" &&
      /instantiate\(imports\): boom/.test(error.message),
  );
});

test("an unknown export name is an error, not undefined", async () => {
  const component = await loadComponent({
    wrapper: "memory:none",
    loaders: { importModule: async () => ({}) },
  });
  assert.throws(
    () => component.call("encode", "x"),
    (error) => error instanceof HostClientError && error.kind === "no-export",
  );
});

test("stop is cooperative: no new calls, in-flight drains", async () => {
  let release;
  const gate = new Promise((resolve) => (release = resolve));
  const component = await loadComponent({
    wrapper: "memory:slow",
    loaders: {
      importModule: async () => ({
        encode: () => {
          release();
          return "ok";
        },
      }),
    },
  });
  assert.equal(component.call("encode"), "ok");
  await component.whenIdle();
  assert.equal(component.stopped, false);
  await component.stop();
  assert.equal(component.stopped, true);
  assert.throws(() => component.call("encode"), (error) => error.kind === "stopped");
  await gate;
});

test("the call surface is SYNCHRONOUS (the sync-facade contract)", async () => {
  const component = await loadComponent({
    wrapper: "memory:sync",
    loaders: { importModule: async () => ({ encode: () => [1, 2, 3] }) },
  });
  const result = component.call("encode");
  assert.ok(Array.isArray(result), "call must return the value, not a promise");
  assert.equal(typeof result.then, "undefined", "call must not return a thenable");
  // and the module itself only exposes the async load
  assert.equal(typeof loadComponent, "function");
  assert.equal(Object.getPrototypeOf(loadComponent).constructor.name, "AsyncFunction");
});

test("a non-object instantiate result is a fail-closed error, not a hollow surface", async () => {
  for (const value of [undefined, null, 42, "text"]) {
    await assert.rejects(
      loadComponent({
        wrapper: "memory:hollow",
        loaders: { importModule: async () => ({ instantiate: async () => value }) },
      }),
      (error) => error instanceof HostClientError && error.kind === "instantiate-failed",
    );
  }
});

test("the declaration is published for the glue modules' W6 guard", async () => {
  const previous = globalThis.__TAIRITSU_CAPABILITIES__;
  try {
    await loadComponent({
      wrapper: "memory:caps",
      capabilities: { interfaces: [], wasiModules: [] },
      loaders: { importModule: async () => ({ encode: () => 1 }) },
    });
    assert.deepEqual(globalThis.__TAIRITSU_CAPABILITIES__, { interfaces: [], wasiModules: [] });
  } finally {
    if (previous === undefined) delete globalThis.__TAIRITSU_CAPABILITIES__;
    else globalThis.__TAIRITSU_CAPABILITIES__ = previous;
  }
});

test("an async export is refused instead of leaking a promise", async () => {
  const component = await loadComponent({
    wrapper: "memory:async",
    loaders: { importModule: async () => ({ encode: async () => "late" }) },
  });
  assert.throws(
    () => component.call("encode"),
    (error) => error instanceof HostClientError && error.kind === "async-export",
  );
});

test("reading exports cannot bypass stop()", async () => {
  const component = await loadComponent({
    wrapper: "memory:exports",
    loaders: { importModule: async () => ({ encode: () => "ok" }) },
  });
  assert.equal(component.exports.encode(), "ok");
  await component.stop();
  assert.throws(
    () => component.exports.encode(),
    (error) => error instanceof HostClientError && error.kind === "stopped",
  );
});

test("the page lane's WASI import keys are used, not fabricated ones", () => {
  const imports = buildImports({ wasiModules: ["io", "clocks"] }, {}, { io: {}, clocks: {} });
  assert.ok("wasi:io/streams@0.2.0" in imports, "io must use the real key");
  assert.ok("wasi:clocks/monotonic-clock@0.2.0" in imports, "clocks must use the real key");
  assert.ok(!("wasi-io" in imports), "no fabricated key");
});

test("a namespaced lifecycle entry and an { exports } wrapper are reachable", async () => {
  const component = await loadComponent({
    wrapper: "memory:shapes",
    loaders: {
      importModule: async () => ({
        instance: { exports: { encode: () => "wrapped" } },
        "tairitsu-browser:full/lifecycle@0.2.0": { start: () => "started" },
      }),
    },
  });
  assert.equal(component.call("encode"), "wrapped");
  assert.equal(component.call("start"), "started");
});

test("prototype-chain names are not treated as provided capabilities", () => {
  assert.throws(
    () => buildImports({ interfaces: ["constructor"] }, {}),
    (error) => error instanceof HostClientError && error.kind === "capability-denied",
  );
});
