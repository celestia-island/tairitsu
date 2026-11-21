#!/usr/bin/env node
// W5: emit REAL ESM modules for the glue interfaces.
//
// Before W5 the runtime stringified every interface's functions into blob
// modules and injected an inline `<script type="importmap">` pointing at the
// blob URLs. That is unanalysable (no bundler can see it), dies under any
// normal CSP (`script-src 'self'` blocks the blob URLs and the inline import
// map), and an import map cannot be declared twice.
//
// This script runs at build time and writes one real `.js` file per interface
// (`dist/glue/<name>.js`) whose named exports are the interface's functions.
// The packager copies that directory into the output and rewrites the jco
// wrapper's `@tairitsu-glue/<name>` specifiers to `../glue/<name>.js`, so the
// wrapper's imports are statically resolvable at load time — no blob, no import
// map, nothing inline.
//
// The modules delegate through `globalThis.__TAIRITSU_GLUE__.INTERFACES`, which
// the runtime bundle installs: one copy of the implementation, and the
// per-instance tables keep working exactly as before.

import { mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const PACKAGE_ROOT = resolve(here, "..");
const RUNTIME_DIR = join(PACKAGE_ROOT, "src", "runtime");
const OUT_DIR = process.env.GLUE_ESM_OUT || join(PACKAGE_ROOT, "dist", "glue");

/** Interface name (as the wrapper imports it) → the module that defines it. */
const EXPORT_SOURCES = {
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
  "css-inline-style": "elementCssInlineStyle",
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
function exportsName(moduleName) {
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
async function moduleExports(moduleName) {
  const path = join(RUNTIME_DIR, `${moduleName}.ts`);
  const source = await readFile(path, "utf8");
  const name = exportsName(moduleName);
  const marker = `export const ${name} = {`;
  const start = source.indexOf(marker);
  if (start < 0) {
    throw new Error(`${moduleName}.ts does not export ${name}`);
  }

  const keys = [];
  let depth = 0;
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
      if (char === "," || char === "\n") {
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

async function main() {
  await rm(OUT_DIR, { recursive: true, force: true });
  await mkdir(OUT_DIR, { recursive: true });

  const written = [];
  for (const [iface, moduleName] of Object.entries(EXPORT_SOURCES)) {
    const { name, keys } = await moduleExports(moduleName);
    if (keys.length === 0) {
      throw new Error(`${moduleName}: ${name} has no exports`);
    }
    const spec = `@tairitsu-glue/${iface}`;
    const lines = [
      `// Generated by scripts/emit-glue-esm.mjs — do not edit.`,
      `// Static ESM stand-in for the ${spec} interface (W5).`,
      `//`,
      `// The implementation lives in the runtime bundle, which installs`,
      `// globalThis.__TAIRITSU_GLUE__; these named exports delegate to it, so a`,
      `// jco wrapper's static \`import { … } from "${spec}"\` resolves to this file`,
      `// at load time — no blob URL and no import map.`,
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
    written.push({ iface, keys: keys.length });
  }

  // The packager copies this directory; a manifest makes the copy verifiable
  // (and its absence detectable) without parsing the JS.
  await writeFile(
    join(OUT_DIR, "manifest.json"),
    JSON.stringify(
      {
        generatedBy: "scripts/emit-glue-esm.mjs",
        interfaces: written.sort((a, b) => a.iface.localeCompare(b.iface)),
      },
      null,
      2,
    ) + "\n",
    "utf8",
  );

  const total = written.reduce((sum, entry) => sum + entry.keys, 0);
  console.log(
    `emit-glue-esm: ${written.length} interface modules, ${total} named exports -> ${OUT_DIR}`,
  );
}

await main();