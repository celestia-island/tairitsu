// The JS values jco materialises for the guest-callback records.
//
// `__wasmExports["tairitsu-browser:full/<iface>@0.2.0"]` is the TRANSPILED
// wrapper's export namespace, so every value crossing into it must already have
// the shape jco generates — not the DOM object, and not a table handle. `jco
// types` on `resize-observer-callbacks` / `mutation-observer-callbacks` emits:
//
//   export function onResize(callbackId: bigint, entries: Array<ResizeEntry>): void;
//   export interface ResizeEntry { target: bigint, contentRect: DomRect }   // (see below)
//   export function onMutate(callbackId: bigint, entries: Array<MutationEntry>): void;
//   export interface MutationEntry {
//     mutationType: string, target: bigint, addedNodes: Array<bigint>,
//     removedNodes: Array<bigint>, previousSibling: bigint | undefined,
//     nextSibling: bigint | undefined, attributeName: string | undefined,
//     attributeNamespace: string | undefined, oldValue: string | undefined,
//   }
//
// (kebab `content-rect` → `contentRect`; every option<T> becomes `T | undefined`;
// `list<T>` becomes a plain `Array<T>`; a `u64` handle stays a `bigint`.)
//
// Why one module: the two call sites are the resize and mutation observer
// bridges in `platformHelpers.ts`, which previously forwarded raw DOM entries
// (and, for mutations, integer handle lists) — values jco cannot convert, so
// the guest's `list<resize-entry>` / `list<mutation-entry>` parameters were
// garbage or the call threw. Keeping the conversion in one place means the
// wire shape is stated once and both bridges read the same table.
//
// The handle tables stay the runtime's own: `resize-observer-entry` and
// `mutation-record` still resolve the handles this file mints, so a guest that
// kept the old `u64` handle path (or read a record back through the WIT getters)
// sees the same object it always did.

type EntrySink = { store(handle: bigint, entry: unknown): void };
type RecordSink = { store(handle: bigint, record: unknown): void };

/** Options applied to `option<T>` fields so jco sees `undefined`, not a hole. */
const NONE = undefined;

function rectOf(entry: unknown): Record<string, number> {
  // `content-rect` is `dom-rect` = `{ x, y, width, height }` (all f64).
  const source = (entry as { contentRect?: Partial<DOMRect> }).contentRect ?? {};
  return {
    x: Number(source.x ?? 0),
    y: Number(source.y ?? 0),
    width: Number(source.width ?? 0),
    height: Number(source.height ?? 0),
  };
}

function nodeList(nodes: unknown): bigint[] {
  if (!nodes) return [];
  const list = Array.from(nodes as ArrayLike<unknown>);
  return list.map((node) => globalThis.__storeElement(node) ?? 0n);
}

function nodeHandle(node: unknown): bigint | undefined {
  if (!node) return NONE;
  return globalThis.__storeElement(node) ?? NONE;
}

/**
 * One DOM `ResizeObserverEntry` → the `resize-entry` record jco expects.
 *
 * `target` is a fresh element handle rather than the entry's own handle: the
 * record is consumed by the guest synchronously, and handing out the entry
 * handle too would leak one table slot per callback for a value nobody can
 * look up (the WIT declares no op that takes a resize-entry handle).
 */
export function resizeEntryValue(entry: unknown, sink: EntrySink): Record<string, unknown> {
  const handle = globalThis.__nextResizeObserverEntry++;
  sink.store(handle, entry);
  return {
    target: globalThis.__storeElement((entry as { target?: unknown }).target) ?? 0n,
    contentRect: rectOf(entry),
  };
}

/**
 * One DOM `MutationRecord` → the `mutation-entry` record jco expects.
 *
 * `added-nodes` / `removed-nodes` are `option<u64>` in the WIT — a single
 * handle, not a list — so they carry the FIRST node of each NodeList (the
 * record's own handle list still exposes every node through
 * `mutation-record`, which is why those two getters stay wired).
 */
export function mutationEntryValue(record: unknown, sink: RecordSink): Record<string, unknown> {
  const handle = globalThis.__nextMutationRecord++;
  sink.store(handle, record);
  const typed = record as Partial<MutationRecord>;
  const added = typed.addedNodes ? Array.from(typed.addedNodes) : [];
  const removed = typed.removedNodes ? Array.from(typed.removedNodes) : [];
  return {
    mutationType: String(typed.type ?? ""),
    target: globalThis.__storeElement(typed.target) ?? 0n,
    addedNodes: nodeList(added),
    removedNodes: nodeList(removed),
    previousSibling: nodeHandle(typed.previousSibling),
    nextSibling: nodeHandle(typed.nextSibling),
    attributeName: typed.attributeName ?? NONE,
    attributeNamespace: typed.attributeNamespace ?? NONE,
    oldValue: typed.oldValue ?? NONE,
  };
}