/**
 * Tairitsu Browser Glue - Runtime Entry Point
 *
 * Bootstraps the browser glue: installs the per-instance runtime and exposes the
 * interface table the build-time ESM modules delegate to (W5 removed the blob
 * import map).
 *
 * Usage: <script type="module" src="/__tairitsu_glue__.js"></script>
 *
 * After loading, this module will:
 * 1. Initialize shared handle tables on globalThis
 * 2. Expose the interface implementations for the emitted ESM modules
 */

// Side-effect imports that initialize globalThis handle tables and helpers
import "./handles";
import "./helpers";
import "./wasmExports";

import { INTERFACES } from "./registry";

// W5: no import map. The interfaces are real ESM modules emitted at build time
// and copied beside the page; this entry only installs the runtime they delegate
// through.
console.log("[tairitsu] Glue runtime loaded with", Object.keys(INTERFACES).length, "interfaces");

// Export debug handle for diagnostics
globalThis.__TAIRITSU_GLUE__ = {
  INTERFACES,
  handles: {
    get elementHandles() {
      return globalThis.__elementHandles;
    },
    get nodeHandles() {
      return globalThis.__nodeHandles;
    },
    get documentHandles() {
      return globalThis.__documentHandles;
    },
    get textHandles() {
      return globalThis.__textHandles;
    },
    get nextHandle() {
      return globalThis.__nextHandle;
    },
  },
};
