/**
 * @tairitsu/browser-glue
 *
 * TypeScript/JavaScript adaptor glue for Tairitsu browser WIT worlds.
 *
 * This package bridges the WIT world interfaces (defined in
 * `packages/browser-worlds`) and the actual browser DOM APIs. It is intended
 * to be loaded by a WASM host (e.g. the Tairitsu runtime) to satisfy the
 * imports declared in the WIT worlds.
 *
 * ## Package layout
 *
 * - `dom-glue`    — satisfies `tairitsu-browser:dom` import interfaces
 * - `events-glue` — satisfies `tairitsu-browser:events` import interfaces
 * - `fetch-glue`  — satisfies `tairitsu-browser:fetch` import interfaces
 * - `canvas-glue` — satisfies `tairitsu-browser:canvas` import interfaces
 *
 * ## Status
 * Core DOM/events/fetch/canvas glue is implemented against browser APIs.
 * Additional API-surface expansion is tracked in PLAN.md.
 */

export * from "./dom-glue.js";
export * from "./events-glue.js";
export * from "./fetch-glue.js";
export * from "./canvas-glue.js";

/**
 * Minimal component instantiation helper used by packager-generated HTML.
 */
export async function instantiate(
	compile: () => Promise<WebAssembly.Module>,
	imports: WebAssembly.Imports = {}
) {
	const module = await compile();
	const instance = await WebAssembly.instantiate(module, imports);
	return { module, instance };
}
