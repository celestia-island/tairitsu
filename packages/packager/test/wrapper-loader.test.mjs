// W3 fail-closed boot — the loader must never swallow an instantiation failure.
//
// The regressions these pin, in order of severity:
//   * a wrapper that exists and cannot be instantiated must REJECT, with both
//     ABI attempts named in the message (before W3 each `catch` discarded the
//     error and the page went blank);
//   * a wrapper that is absent must reject with the "not found" message and
//     the recorded attempts;
//   * the source must not carry an empty `catch` block again (G2-W3 ③).

import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

const here = fileURLToPath(new URL(".", import.meta.url));
const TEMPLATE = resolve(here, "../src/wasm/component-wrapper-loader.template.js");
const BOOT_TEMPLATE = resolve(here, "../src/wasm/component-boot.template.js");

/** A throwaway dist-like tree: the loader copy plus an optional wrapper. */
async function makeTree(t, { wrapperSource } = {}) {
  const root = await mkdtemp(join(tmpdir(), "w3-loader-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const loaderSource = (await readFile(TEMPLATE, "utf8")).replace("__WASM_STEM__", "demo");
  const loaderPath = join(root, "component-wrapper-loader.js");
  await writeFile(loaderPath, loaderSource);
  if (wrapperSource !== undefined) {
    await mkdir(join(root, "component-wrapper"), { recursive: true });
    await writeFile(join(root, "component-wrapper", "demo.js"), wrapperSource);
  }
  return { root, loaderPath };
}

/** The loader probes candidates with HEAD; node's fetch has no file: support. */
function stubFetch(t, { ok = true } = {}) {
  const original = globalThis.fetch;
  globalThis.fetch = async () => ({ ok, status: ok ? 200 : 404 });
  t.after(() => {
    globalThis.fetch = original;
  });
}

test("a wrapper that instantiates cleanly is returned untouched", async (t) => {
  stubFetch(t);
  const { loaderPath } = await makeTree(t, {
    wrapperSource: `export async function instantiate(imports) {
      globalThis.__w3_calls = [...(globalThis.__w3_calls || []), Object.keys(imports)];
      return { ok: true, marked: Object.keys(imports) };
    }`,
  });
  const { instantiateWithWrapper } = await import(pathToFileURL(loaderPath).href);
  const result = await instantiateWithWrapper({ "wasi:cli/stdout@0.2.0": { write() {} } });
  assert.equal(result.ok, true);
  assert.deepEqual(result.marked, ["wasi:cli/stdout@0.2.0"]);
});

test("a wrapper whose BOTH ABI shapes fail rejects with the collected attempts", async (t) => {
  stubFetch(t);
  const { loaderPath } = await makeTree(t, {
    wrapperSource: `export async function instantiate(first) {
      throw new Error("first-shape-failed");
    }`,
  });
  const { instantiateWithWrapper } = await import(pathToFileURL(loaderPath).href);
  await assert.rejects(
    () => instantiateWithWrapper({}),
    (error) => {
      assert.match(error.message, /failed to instantiate/);
      assert.match(error.message, /first-shape-failed/, "the first ABI failure is reported");
      assert.ok(
        /instantiate\(resolveCoreModule, imports\)/.test(error.message),
        "the second ABI shape is named too",
      );
      return true;
    },
  );
});

test("an absent wrapper rejects with 'not found' instead of returning undefined", async (t) => {
  stubFetch(t, { ok: false });
  const { loaderPath } = await makeTree(t, { wrapperSource: undefined });
  const { instantiateWithWrapper } = await import(pathToFileURL(loaderPath).href);
  await assert.rejects(
    () => instantiateWithWrapper({}),
    (error) => {
      assert.match(error.message, /Component wrapper not found/);
      assert.match(error.message, /probe/, "the probe failure is recorded");
      return true;
    },
  );
});

test("the loader source carries no empty catch block (G2-W3 ③)", async () => {
  const source = await readFile(TEMPLATE, "utf8");
  // `catch (e) {}` / `catch {}` with a body that is only whitespace is the
  // shape that swallowed instantiation errors; its absence is the source-level
  // half of the W3 contract (the boot-runtime half is in boot-runtime.test.mjs).
  assert.equal(
    /catch\s*(?:\([^)]*\))?\s*\{\s*\}/.test(source),
    false,
    "an empty catch block reappeared in the wrapper loader",
  );
  // and every catch that does continue must record why (attempts.push)
  const catches = [...source.matchAll(/catch\s*(?:\([^)]*\))?\s*\{([\s\S]*?)\n\s*\}/g)];
  assert.ok(catches.length > 0, "the loader still handles failures");
  for (const [, body] of catches) {
    assert.match(
      body,
      /attempts\.push|throw /,
      "a catch block neither records the attempt nor rethrows",
    );
  }
});

test("the boot runtime ships beside the loader and exports the lifecycle", async () => {
  const source = await readFile(BOOT_TEMPLATE, "utf8");
  assert.match(source, /export const ENTRY_NAMES/);
  assert.match(source, /export async function bootComponent/);
  assert.match(source, /export class BootError/);
});