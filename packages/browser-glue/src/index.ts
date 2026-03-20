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
 * - `dom`    — satisfies `tairitsu-browser:dom` import interfaces
 * - `events` — satisfies `tairitsu-browser:events` import interfaces
 * - `http`   — satisfies `tairitsu-browser:fetch` import interfaces
 * - `canvas` — satisfies `tairitsu-browser:canvas` import interfaces
 * - `handles` — shared object handle management
 *
 * ### Auto-generated Phase A modules (from WIT)
 * - `generated/*Glue` — 25 domain-specific glue modules
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
export * from "./dom";
export * from "./events";
export * from "./http";
export * from "./canvas";
export * from "./handles";

// Async utilities
export * from "./async";

// Auto-generated Phase A modules
export * from "./generated/index";

/**
 * Diagnostic types for external consumers.
 */
export type {
  DiagnosticError,
  EventDispatchInfo,
} from "./events";

export type {
  DomDiagnosticError,
} from "./dom";

export type {
  HandleDiagnosticError,
} from "./handles";


