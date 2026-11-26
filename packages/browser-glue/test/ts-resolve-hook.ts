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
 *
 * This file is TypeScript source run straight off the tree by node's type
 * stripping, so it stays erasable (no enums, namespaces or parameter properties).
 */

import type { ResolveFnOutput, ResolveHookContext } from "node:module";

/** The subset of node's resolve hook this file needs, spelled out locally. */
type NextResolve = (
  specifier: string,
  context?: Partial<ResolveHookContext>,
) => ResolveFnOutput | Promise<ResolveFnOutput>;

export async function resolve(
  specifier: string,
  context: ResolveHookContext,
  nextResolve: NextResolve,
): Promise<ResolveFnOutput> {
  const bare = specifier.startsWith("./") || specifier.startsWith("../");
  const hasExtension = /\.[a-z0-9]+$/i.test(specifier);
  if (bare && !hasExtension) {
    try {
      return await nextResolve(`${specifier}.ts`, context);
    } catch {
      // fall through to the default resolution / error
    }
  }
  return await nextResolve(specifier, context);
}
