// W3 fail-closed boot — the boot lifecycle decides the entry, the ready
// attribute and the error surface, and it is testable without a browser
// (the page only adapts `surface` to `data-tairitsu-ready` /
// `data-tairitsu-error`).
//
// The three G2-W3 negative controls live here: no entry, an entry that
// throws, and a malformed export surface. Each must fail the boot (the page
// shows an error and never reports ready) rather than producing a ready page.

import assert from "node:assert/strict";
import test from "node:test";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = fileURLToPath(new URL(".", import.meta.url));
const BOOT_MODULE = resolve(here, "../src/wasm/component-boot.template.js");
const { bootComponent, resolveEntry, ENTRY_NAMES, BootError } = await import(BOOT_MODULE);

/** The page adapter, recorded so a test can assert what the lifecycle said. */
function recorder() {
  const events = [];
  return {
    events,
    surface: {
      ready: (entryName) => events.push({ kind: "ready", entryName }),
      fail: (kind, error) => events.push({ kind: "fail", failure: kind, message: error.message }),
    },
    log: { info: () => {}, error: () => {} },
  };
}

const noop = () => {};

test("the entry contract is explicit and ordered", () => {
  assert.deepEqual(ENTRY_NAMES, ["tairitsu_component_bootstrap", "run", "main", "init"]);
});

test("the documented bootstrap export wins over run/main/init", () => {
  const result = {
    tairitsu_component_bootstrap: noop,
    run: noop,
    main: noop,
    init: noop,
  };
  assert.equal(resolveEntry(result).name, "tairitsu_component_bootstrap");
});

test("run wins when there is no bootstrap export, and the choice is logged", async () => {
  const { events, surface } = recorder();
  const logged = [];
  const { entry } = await bootComponent({
    result: { run: noop, main: noop },
    surface,
    log: { info: (line) => logged.push(line), error: () => {} },
  });
  assert.equal(entry, "run");
  assert.deepEqual(events, [{ kind: "ready", entryName: "run" }]);
  assert.ok(logged.some((line) => line.includes("boot entry: run")), "entry choice is logged");
});

test("an entry on the wrapper's exports/instance surface is found", () => {
  const fromExports = { exports: { init: noop } };
  assert.equal(resolveEntry(fromExports).name, "init");
  const fromInstance = { instance: { exports: { main: noop } } };
  assert.equal(resolveEntry(fromInstance).name, "main");
});

test("NEGATIVE CONTROL 1 — no entry is a hard error, never a ready page", async () => {
  const { events, surface } = recorder();
  await assert.rejects(
    () => bootComponent({ result: { exports: { unrelated: noop } }, surface, log: { info() {}, error() {} } }),
    (error) => {
      assert.ok(error instanceof BootError);
      assert.equal(error.kind, "no-entry");
      assert.match(error.message, /tairitsu_component_bootstrap/);
      return true;
    },
  );
  assert.equal(events.length, 1, "exactly one surface call");
  assert.equal(events[0].kind, "fail");
  assert.equal(events[0].failure, "no-entry");
  assert.ok(!events.some((event) => event.kind === "ready"), "ready was never reported");
});

test("NEGATIVE CONTROL 2 — an entry that throws fails the boot with its cause", async () => {
  const { events, surface } = recorder();
  const boom = new Error("guest exploded");
  await assert.rejects(
    () =>
      bootComponent({
        result: {
          tairitsu_component_bootstrap: () => {
            throw boom;
          },
        },
        surface,
        log: { info() {}, error() {} },
      }),
    (error) => {
      assert.equal(error.kind, "entry-failed");
      assert.equal(error.cause, boom, "the guest error is kept as the cause");
      assert.match(error.message, /guest exploded/);
      return true;
    },
  );
  assert.equal(events.length, 1);
  assert.equal(events[0].kind, "fail");
  assert.equal(events[0].failure, "entry-failed");
  assert.ok(!events.some((event) => event.kind === "ready"));
});

test("NEGATIVE CONTROL 3 — ready is reported only AFTER the entry returned", async () => {
  const order = [];
  const { surface } = recorder();
  const slowSurface = {
    ready: (entryName) => {
      order.push("ready");
      surface.ready(entryName);
    },
    fail: (kind, error) => {
      order.push("fail");
      surface.fail(kind, error);
    },
  };
  await bootComponent({
    result: {
      run: async () => {
        order.push("entry-start");
        await new Promise((resolvePromise) => setTimeout(resolvePromise, 5));
        order.push("entry-done");
      },
    },
    surface: slowSurface,
    log: { info() {}, error() {} },
  });
  assert.deepEqual(order, ["entry-start", "entry-done", "ready"]);
});

test("a rejected entry never reports ready even when it resolves later", async () => {
  let released;
  const pending = new Promise((resolvePromise) => {
    released = resolvePromise;
  });
  const { events, surface } = recorder();
  const booting = bootComponent({
    result: {
      run: async () => {
        await pending;
        throw new Error("late failure");
      },
    },
    surface,
    log: { info() {}, error() {} },
  });
  released();
  await assert.rejects(() => booting);
  assert.deepEqual(events.map((event) => event.kind), ["fail"]);
});

test("the page template no longer fishes for names by depth (source check)", async () => {
  const page = await readFile(resolve(here, "../src/wasm/index.html.template"), "utf8");
  assert.equal(/tryInvokeBootExports/.test(page), false, "the depth-3 entry walk is gone");
  assert.equal(
    /no exported run\/start entry found/.test(page),
    false,
    "the soft 'nothing found' path is gone",
  );
  assert.match(page, /component-boot\.js/, "the page imports the boot runtime");
  assert.match(page, /bootSettled/, "ready is gated on the lifecycle");
});