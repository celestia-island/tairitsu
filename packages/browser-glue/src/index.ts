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
 * ### Hand-written Phase 0 modules
 * - `dom-glue`    — satisfies `tairitsu-browser:dom` import interfaces
 * - `events-glue` — satisfies `tairitsu-browser:events` import interfaces
 * - `fetch-glue`  — satisfies `tairitsu-browser:fetch` import interfaces
 * - `canvas-glue` — satisfies `tairitsu-browser:canvas` import interfaces
 * - `handle-table` — shared object handle management
 *
 * ### Auto-generated Phase A modules (from WIT)
 * - `generated/*-glue` — 22 domain-specific glue modules
 *
 * ## Diagnostic APIs
 *
 * The following diagnostic functions are exported for observability and debugging:
 *
 * - `registerDiagnosticCallbacks()` — Register event system diagnostics
 * - `registerDomDiagnosticCallbacks()` — Register DOM operation diagnostics
 * - `registerHandleDiagnosticCallbacks()` — Register handle table diagnostics
 * - `checkEnvironment()` — Validate browser environment is ready
 * - `getListenerCount()` — Get active event listener count
 * - `getActiveEventCount()` — Get active (in-flight) event count
 * - `getHandleStats()` — Get handle table statistics
 */

// Hand-written Phase 0 modules
export * from "./dom-glue.js";
export * from "./events-glue.js";
export * from "./fetch-glue.js";
export * from "./canvas-glue.js";
export * from "./handle-table.js";

// Auto-generated Phase A modules
export * from "./generated-index.js";

/**
 * Diagnostic types for external consumers.
 */
export type {
  DiagnosticError,
  EventDispatchInfo,
} from "./events-glue.js";

export type {
  DomDiagnosticError,
} from "./dom-glue.js";

export type {
  HandleDiagnosticError,
} from "./handle-table.js";


