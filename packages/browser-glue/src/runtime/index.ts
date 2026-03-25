// @ts-nocheck
/**
 * Tairitsu Browser Glue - Runtime Entry Point
 *
 * Bootstraps the browser glue by registering all interface implementations as
 * importable ES modules via blob URLs and an import map.
 *
 * Usage: <script type="module" src="/__tairitsu_glue__.js"></script>
 *
 * After loading, this module will:
 * 1. Initialize shared handle tables on globalThis
 * 2. Generate blob URLs for each interface implementation
 * 3. Register an import map mapping tairitsu-browser:full/* to those URLs
 */

// Side-effect imports that initialize globalThis handle tables and helpers
import "./handles";
import "./helpers";
import "./wasm-exports";

import { INTERFACES, registerImportMap } from "./registry";

// Auto-register when loaded
registerImportMap();

// Export debug handle for diagnostics
globalThis.__TAIRITSU_GLUE__ = {
  INTERFACES,
  handles: {
    get elementHandles() { return globalThis.__elementHandles; },
    get nodeHandles() { return globalThis.__nodeHandles; },
    get documentHandles() { return globalThis.__documentHandles; },
    get textHandles() { return globalThis.__textHandles; },
    get nextHandle() { return globalThis.__nextHandle; },
  },
};
