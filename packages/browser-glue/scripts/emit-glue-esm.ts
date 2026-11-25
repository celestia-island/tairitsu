#!/usr/bin/env -S node --experimental-strip-types
// W5: emit REAL ESM modules for the glue interfaces.
//
// Before W5 the runtime stringified every interface's functions into blob
// modules and injected an inline `<script type="importmap">` pointing at the
// blob URLs. That is unanalysable (no bundler can see it), dies under any
// normal CSP (`script-src 'self'` blocks the blob URLs and the inline import
// map), and an import map cannot be declared twice.
//
// This script runs at build time and writes one real `.js` file per interface
// (`dist/glue-esm/<name>.js`) whose named exports are the interface's functions.
// The packager copies that directory into the output and rewrites the jco
// wrapper's `@tairitsu-glue/<name>` specifiers to `../glue/<name>.js`, so the
// wrapper's imports are statically resolvable at load time — no blob, no import
// map, nothing inline.
//
// The modules delegate through `globalThis.__TAIRITSU_GLUE__.INTERFACES`, which
// the runtime bundle installs: one copy of the implementation, and the
// per-instance tables keep working exactly as before.
//
// The SOURCE of this script is TypeScript; it is run straight off the tree by
// node's type stripping (`--experimental-strip-types`), so it must stay
// erasable — no enums, no namespaces, no parameter properties. The ARTIFACT it
// emits is JavaScript.

import { mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

/** One module's interface object, as scanned out of its source. */
interface ModuleExports {
  name: string;
  keys: string[];
}

/** What the manifest records per emitted module. */
interface WrittenModule {
  iface: string;
  keys: number;
}

const here = dirname(fileURLToPath(import.meta.url));
const PACKAGE_ROOT = resolve(here, "..");
const RUNTIME_DIR = join(PACKAGE_ROOT, "src", "runtime");
const OUT_DIR = process.env.GLUE_ESM_OUT || join(PACKAGE_ROOT, "dist", "glue-esm");

/** Interface name (as the wrapper imports it) → the module that defines it. */
const EXPORT_SOURCES: Record<string, string> = {
  document: "document",
  element: "element",
  node: "node",
  "non-element-parent-node": "nonElementParentNode",
  "parent-node": "parentNode",
  window: "window",
  "platform-helpers": "platformHelpers",
  "mutation-record": "mutationRecord",
  "resize-observer-entry": "resizeObserverEntry",
  "resize-observer-size": "resizeObserverSize",
  "resize-observer": "resizeObserver",
  "css-style-declaration": "cssStyleDeclaration",
  "media-query-list": "mediaQueryList",
  "element-css-inline-style": "elementCssInlineStyle",
  "event-target": "eventTarget",
  "dom-token-list": "domTokenList",
  "node-list": "nodeList",
  location: "location",
  history: "history",
  observers: "observers",
  "mutation-observer": "mutationObserver",
  event: "event",
  "html-input-element": "htmlInputElement",
  "mouse-event": "mouseEvent",
  "pointer-event": "pointerEvent",
  "wheel-event": "wheelEvent",
};

/** The exported object name inside a runtime module (`<x>_exports`). */
function exportsName(moduleName: string): string {
  return `${moduleName}_exports`;
}

/**
 * The top-level keys of `export const <name>_exports = { … }`, read from the
 * SOURCE rather than by importing: the runtime modules import each other
 * extensionlessly (the bundler's convention), which node's loader cannot
 * resolve, and importing them would pull the whole runtime graph in anyway.
 *
 * The scanner skips strings, template literals and comments and only records
 * identifiers at brace depth 1 — the shape every interface object uses
 * (`name(...) { … },` or `name: …`).
 */
async function moduleExports(moduleName: string): Promise<ModuleExports> {
  const path = join(RUNTIME_DIR, `${moduleName}.ts`);
  const source = await readFile(path, "utf8");
  const name = exportsName(moduleName);
  const marker = `export const ${name} = {`;
  const start = source.indexOf(marker);
  if (start < 0) {
    throw new Error(`${moduleName}.ts does not export ${name}`);
  }

  const keys: string[] = [];
  let depth = 0;
  let parens = 0;
  let i = start + marker.length - 1;
  let expectKey = true;
  let pending = "";
  while (i < source.length) {
    const char = source[i];
    const next = source[i + 1];

    if (char === "/" && next === "/") {
      while (i < source.length && source[i] !== "\n") i++;
      continue;
    }
    if (char === "/" && next === "*") {
      i += 2;
      while (i < source.length && !(source[i] === "*" && source[i + 1] === "/")) i++;
      i += 2;
      continue;
    }
    if (char === '"' || char === "'" || char === "`") {
      const quote = char;
      i++;
      while (i < source.length) {
        if (source[i] === "\\") {
          i += 2;
          continue;
        }
        if (source[i] === quote) break;
        i++;
      }
      i++;
      continue;
    }
    if (char === "(") parens++;
    if (char === ")") parens--;
    if (char === "{") {
      const openedObject = depth === 0;
      depth++;
      // Only the interface object's own `{` starts a key; a method body's `{`
      // must not discard the key that was just read.
      if (openedObject) {
        pending = "";
        expectKey = true;
      }
      i++;
      continue;
    }
    if (char === "}") {
      depth--;
      if (depth === 0) break;
    }
    if (depth === 1) {
      if ((char === "," || char === "\n") && parens <= 0) {
        if (pending.trim()) {
          const match = pending.trim().match(/^["']?([A-Za-z_$][A-Za-z0-9_$]*)/);
          if (match) keys.push(match[1]);
        }
        pending = "";
        expectKey = true;
        i++;
        continue;
      }
      if (expectKey) pending += char;
      if (char === "(" || char === ":") expectKey = false;
    }
    i++;
  }
  if (pending.trim()) {
    const match = pending.trim().match(/^["']?([A-Za-z_$][A-Za-z0-9_$]*)/);
    if (match) keys.push(match[1]);
  }
  const unique = [...new Set(keys)].sort();
  if (unique.length === 0) {
    throw new Error(`${moduleName}: ${name} has no exports`);
  }
  return { name, keys: unique };
}

/**
 * The registry is the source of truth for interface names. Reading its keys from
 * source keeps this script from silently emitting a differently-named file (the
 * bug that shipped once: `css-inline-style` vs `element-css-inline-style`).
 */
async function registryInterfaceNames(): Promise<Set<string>> {
  const source = await readFile(join(RUNTIME_DIR, "registry.ts"), "utf8");
  const names = new Set<string>();
  const re = /"@tairitsu-glue\/([a-z0-9-]+)"\s*:/g;
  let match: RegExpExecArray | null;
  while ((match = re.exec(source)) !== null) names.add(match[1]);
  if (names.size === 0) throw new Error("registry.ts declares no @tairitsu-glue interfaces");
  return names;
}

/**
 * Prove every emitted module actually LOADS as JavaScript.
 *
 * This is the gate that would have caught the defect this script once shipped:
 * a `declare global { … }` block written into the emitted `.js`. Node never
 * strips types from a `.js` file, so the module was a hard `SyntaxError` — and
 * because the packager copies these files verbatim into every built page, the
 * breakage was invisible to any test that imported the emitter's own `.ts`
 * source. Twice learned, so: the emitted bytes are imported here, as `.js`,
 * through the same resolver a browser uses.
 *
 * The capability guard and the "runtime not loaded" guard are both EXPECTED to
 * throw at import time; anything else — a syntax error above all — fails.
 */
async function assertEveryModuleLoads(interfaces: string[]): Promise<void> {
  const expected: RegExp[] = [
    /capability denied: /,
    /tairitsu glue runtime is not loaded/,
  ];
  for (const iface of interfaces) {
    const url = pathToFileURL(join(OUT_DIR, `${iface}.js`)).href;
    try {
      await import(url);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      if (expected.some((pattern) => pattern.test(message))) continue;
      throw new Error(
        `emitted module ${iface}.js does not load as JavaScript (node never strips ` +
          `types from .js): ${message}`,
      );
    }
  }
}

async function main(): Promise<void> {
  await rm(OUT_DIR, { recursive: true, force: true });
  await mkdir(OUT_DIR, { recursive: true });

  // The emitted names must be the registry's names, one for one.
  const declared = await registryInterfaceNames();
  const mapped = new Set(Object.keys(EXPORT_SOURCES));
  const missing = [...declared].filter((name) => !mapped.has(name)).sort();
  const extra = [...mapped].filter((name) => !declared.has(name)).sort();
  if (missing.length || extra.length) {
    throw new Error(
      `interface map does not match registry.ts — missing: ${JSON.stringify(missing)}, ` +
        `not-in-registry: ${JSON.stringify(extra)}`,
    );
  }

  const written: WrittenModule[] = [];
  for (const [iface, moduleName] of Object.entries(EXPORT_SOURCES)) {
    const { name, keys } = await moduleExports(moduleName);
    if (keys.length === 0) {
      throw new Error(`${moduleName}: ${name} has no exports`);
    }
    const spec = `@tairitsu-glue/${iface}`;
    const lines = [
      `// Generated by scripts/emit-glue-esm.ts — do not edit.`,
      `// Static ESM stand-in for the ${spec} interface (W5).`,
      `//`,
      `// The implementation lives in the runtime bundle, which installs`,
      `// globalThis.__TAIRITSU_GLUE__; these named exports delegate to it, so a`,
      `// jco wrapper's static \`import { … } from "${spec}"\` resolves to this file`,
      `// at load time — no blob URL and no import map.`,
      `// W6: the module itself enforces the declaration, because W5 made these`,
      `// imports static — the boot's import filtering cannot see them.`,
      `const isDeclared = () => {`,
      `  const capabilities = globalThis.__TAIRITSU_CAPABILITIES__;`,
      `  if (!capabilities) return true; // no declaration loaded yet: not our call`,
      `  const declared = new Set((capabilities.interfaces || []).map((name) =>`,
      `    String(name).replace(/^@tairitsu-glue\\//, "").replace(/^tairitsu-browser:full\\//, "").replace(/@\\d+\\.\\d+\\.\\d+$/, "")));`,
      `  return declared.has(${JSON.stringify(iface)});`,
      `};`,
      `if (!isDeclared()) {`,
      `  throw new Error("capability denied: " + ${JSON.stringify(spec)} + " is not declared");`,
      `}`,
      `const iface = () => {`,
      `  const glue = globalThis.__TAIRITSU_GLUE__;`,
      `  if (!glue || !glue.INTERFACES) {`,
      `    throw new Error("tairitsu glue runtime is not loaded (expected globalThis.__TAIRITSU_GLUE__)");`,
      `  }`,
      `  return glue.INTERFACES[${JSON.stringify(spec)}];`,
      `};`,
      "",
    ];
    for (const key of keys) {
      if (!/^[A-Za-z_$][A-Za-z0-9_$]*$/.test(key)) {
        throw new Error(`${spec}: export ${JSON.stringify(key)} is not a valid ESM name`);
      }
      lines.push(`export const ${key} = (...args) => iface().${key}(...args);`);
    }
    const file = join(OUT_DIR, `${iface}.js`);
    await writeFile(file, lines.join("\n") + "\n", "utf8");

    // The runtime module above is plain JavaScript — it is loaded raw by the
    // packager and by the browser, neither of which strips types, so a single
    // TS construct in it would be a hard SyntaxError for every packed page.
    // The ambient types therefore live in a companion declaration file that no
    // runtime path reads.
    const declarations = [
      `// Generated by scripts/emit-glue-esm.ts — do not edit.`,
      `// Type companion for ./${iface}.js. Declarations only: nothing here is`,
      `// executed, and the runtime module this describes stays plain JavaScript.`,
      `declare global {`,
      `  var __TAIRITSU_GLUE__: { INTERFACES: Record<string, Record<string, (...args: any[]) => any>> } | undefined;`,
      `  var __TAIRITSU_CAPABILITIES__: { interfaces?: unknown[] } | undefined;`,
      `}`,
      ``,
    ];
    for (const key of keys) {
      declarations.push(`export declare const ${key}: (...args: any[]) => any;`);
    }
    await writeFile(join(OUT_DIR, `${iface}.d.ts`), declarations.join("\n") + "\n", "utf8");

    written.push({ iface, keys: keys.length });
  }

  // W6: the conservative DEFAULT capability set, derived from the registry so a
  // new interface cannot silently become authorized for every component.
  const defaultCapabilities = {
    $comment:
      "W6 default capability set for components without a declaration. " +
      "Interfaces are the glue registry's own keys; wasiModules are the bundled " +
      "preview2 shim modules the boot injects. Emitted by scripts/emit-glue-esm.ts.",
    interfaces: [...declared].sort(),
    wasiModules: ["cli", "clocks", "filesystem", "io", "random", "sockets"],
  };
  await writeFile(
    join(OUT_DIR, "default-capabilities.json"),
    JSON.stringify(defaultCapabilities, null, 2) + "\n",
    "utf8",
  );

  // The packager copies this directory; a manifest makes the copy verifiable
  // (and its absence detectable) without parsing the JS.
  await writeFile(
    join(OUT_DIR, "manifest.json"),
    JSON.stringify(
      {
        generatedBy: "scripts/emit-glue-esm.ts",
        interfaces: written.sort((a, b) => a.iface.localeCompare(b.iface)),
      },
      null,
      2,
    ) + "\n",
    "utf8",
  );

  // Gate the emitted bytes, not the source: every module must import as .js.
  await assertEveryModuleLoads(written.map((entry) => entry.iface));

  const total = written.reduce((sum, entry) => sum + entry.keys, 0);
  console.log(
    `emit-glue-esm: ${written.length} interface modules, ${total} named exports -> ${OUT_DIR}`,
  );
  console.log(`emit-glue-esm: ${written.length}/${written.length} emitted modules import as .js`);
}

await main();
