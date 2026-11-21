import assert from "node:assert/strict";
import test from "node:test";

import { buildImports, createSurface, HostClientError, loadComponent, publishCapabilities } from "../src/index.ts";

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

test("the declaration is visible while the wrapper is imported, then restored", async () => {
  // The glue modules a wrapper statically imports are evaluated DURING the
  // import; the declaration must be there then (not only afterwards).
  let seenDuringImport;
  await loadComponent({
    wrapper: "memory:caps",
    capabilities: { interfaces: ["document"], wasiModules: [] },
    glue: { document: {} },
    loaders: {
      importModule: async () => {
        seenDuringImport = globalThis.__TAIRITSU_CAPABILITIES__;
        return { encode: () => 1 };
      },
    },
  });
  assert.deepEqual(seenDuringImport, { interfaces: ["document"], wasiModules: [] });
  assert.equal(globalThis.__TAIRITSU_CAPABILITIES__, undefined, "the previous value is restored");
});

test("a consumer can publish the declaration before a STATIC import", async () => {
  const previous = publishCapabilities({ interfaces: [], wasiModules: [] });
  try {
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

test("nested holders are behind the same stop() gate", async () => {
  const component = await loadComponent({
    wrapper: "memory:nested",
    loaders: {
      importModule: async () => ({
        exports: { encode: () => "nested" },
        lifecycle: { start: () => "lifecycle" },
        instance: { encode: () => "instance-shape" },
      }),
    },
  });
  assert.equal(component.exports.exports.encode(), "nested");
  assert.equal(component.exports.lifecycle.start(), "lifecycle");
  assert.equal(component.exports.instance.encode(), "instance-shape");
  await component.stop();
  for (const invoke of [
    () => component.exports.exports.encode(),
    () => component.exports.lifecycle.start(),
    () => component.exports.instance.encode(),
  ]) {
    assert.throws(invoke, (error) => error.kind === "stopped");
  }
});

test("a refusing async export does not leak an unhandled rejection", async () => {
  const component = await loadComponent({
    wrapper: "memory:reject",
    loaders: { importModule: async () => ({ encode: async () => { throw new Error("guest blew up"); } }) },
  });
  assert.throws(() => component.call("encode"), (error) => error.kind === "async-export");
  await new Promise((resolve) => setTimeout(resolve, 20));
});

test("a capability denial during import keeps its kind", async () => {
  await assert.rejects(
    loadComponent({
      wrapper: "memory:denied",
      loaders: {
        importModule: async () => {
          throw new Error("capability denied: @tairitsu-glue/document is not declared");
        },
      },
    }),
    (error) => error.kind === "capability-denied",
  );
});

test("the bundler-safe route hands in an already-imported module", async () => {
  let seenDuringImport;
  const component = await loadComponent({
    module: {
      get encode() {
        return () => "from-module";
      },
    },
    capabilities: { interfaces: [], wasiModules: [] },
  });
  assert.equal(component.call("encode"), "from-module");
  assert.equal(globalThis.__TAIRITSU_CAPABILITIES__, undefined, "declaration restored after the module route");
  assert.equal(seenDuringImport, undefined);
});

test("createSurface is the synchronous door for a static import", () => {
  const component = createSurface({ encode: (text) => `encoded:${text}` }, {
    interfaces: [],
    wasiModules: [],
  });
  const result = component.call("encode", "hi");
  assert.equal(result, "encoded:hi");
  assert.equal(typeof result.then, "undefined");
});

test("createSurface refuses a declaration it cannot provide, and says so", () => {
  // A statically imported wrapper binds its imports before this call: the sync
  // door can only refuse a capability, never provide one.
  for (const declaration of [
    { interfaces: ["document"] },
    { wasiModules: ["clocks"] },
  ]) {
    assert.throws(
      () => createSurface({ encode: () => 1 }, declaration),
      (error) => error instanceof HostClientError && error.kind === "sync-door-cannot-provide",
    );
  }
  assert.throws(
    () => createSurface({ encode: () => 1 }, {}, { document: {} }),
    (error) => error instanceof HostClientError && error.kind === "sync-door-cannot-provide",
  );
});

test("every instantiate ABI name is refused by the sync door", () => {
  for (const name of ["instantiate", "default", "init"]) {
    assert.throws(
      () => createSurface({ [name]: () => ({}) }),
      (error) => error instanceof HostClientError && error.kind === "needs-loading",
    );
  }
});

test("the sync door rejects a non-object module like the async one", () => {
  for (const value of [null, 42, "text"]) {
    assert.throws(
      () => createSurface(value),
      (error) => error instanceof HostClientError && error.kind === "instantiate-failed",
    );
  }
});

test("the sync door's exports view is behind the same stop gate", async () => {
  const component = createSurface({ encode: () => "ok" });
  assert.equal(component.exports.encode(), "ok");
  await component.stop();
  assert.throws(
    () => component.exports.encode(),
    (error) => error instanceof HostClientError && error.kind === "stopped",
  );
});

test("the sync door refuses a thenable export", () => {
  const component = createSurface({ encode: async () => "late" });
  assert.throws(
    () => component.call("encode"),
    (error) => error instanceof HostClientError && error.kind === "async-export",
  );
});

test("createSurface points an instantiable module at loadComponent", () => {
  assert.throws(
    () => createSurface({ instantiate: () => ({}) }),
    (error) => error instanceof HostClientError && error.kind === "needs-loading",
  );
});

test("createSurface's surface is gated by stop() like the async door", async () => {
  const component = createSurface({ encode: () => "ok" });
  assert.equal(component.call("encode"), "ok");
  await component.stop();
  assert.throws(() => component.call("encode"), (error) => error.kind === "stopped");
});
