/**
 * Module-resolution hook for the browser-glue test suite.
 *
 * The runtime sources are written for a bundler (`moduleResolution: "bundler"`),
 * i.e. their relative imports carry no extension — `import { x } from "./node"`.
 * Node's ESM resolver requires one, so the suite could either duplicate the
 * modules or teach the resolver what the bundler already knows. This hook does the
 * latter: an extensionless relative specifier is retried as `<specifier>.ts`.
 *
 * That keeps the tests pointed at the REAL sources (they import
 * `src/runtime/registry.ts` and assert against its `INTERFACES` object) instead of
 * regex-matching the file.
 */

export async function resolve(specifier, context, nextResolve) {
  const bare =
    specifier.startsWith("./") || specifier.startsWith("../");
  const hasExtension = /\.[a-z0-9]+$/i.test(specifier);
  if (bare && !hasExtension) {
    try {
      return await nextResolve(`${specifier}.ts`, context);
    } catch {
      // fall through to the default resolution / error
    }
  }
  return nextResolve(specifier, context);
}
