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
//     mutationType: string, target: bigint, addedNodes?: bigint,
//     removedNodes?: bigint, previousSibling?: bigint, nextSibling?: bigint,
//     attributeName?: string, attributeNamespace?: string, oldValue?: string,
//   }
//
// (kebab fields camelCase; `list<T>` becomes a plain `Array<T>`; a `u64` handle
// stays a `bigint`. An `option<T>` becomes an OPTIONAL PROPERTY — a single `T`,
// present-or-`undefined` — which is why `added-nodes: option<u64>` is ONE handle
// and not a list: measured with `jco types` 1.42.0 on the verbatim records, and
// the reason this file's first draft (which marshalled `bigint[]`) was wrong.)
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

/**
 * A DOM node → a handle in the NODE domain.
 *
 * The domain matters: `mutation-record`'s WIT getters resolve
 * `get-previous-sibling` / `get-next-sibling` through `__storeNode`
 * (`__nodeHandles` + `__textHandles`), while `target` and `get-target` go
 * through `__storeElement`. A sibling minted into the element table is a handle
 * the guest can hold and never resolve — the same class of mismatch this whole
 * change is about, one layer down. Nothing in the test suite notices (both
 * tables answer `bigint`), which is why `callback-bridge.test.mjs` asserts the
 * TABLE the handle lands in, not just its type.
 */
function nodeHandle(node: unknown): bigint | undefined {
  if (!node) return NONE;
  const handle = globalThis.__storeNode(node as Node) ?? NONE;
  return handle === 0n ? NONE : handle;
}

/**
 * One DOM `ResizeObserverEntry` → the `resize-entry` record jco expects.
 *
 * The entry is also stored under a fresh handle in the instance's
 * `resizeObserverEntryHandles` table, because `resize-observer-entry`'s WIT
 * getters read from that table. The slot lives as long as the instance (the
 * table is cleared on dispose, like every other per-instance handle table); for
 * the callback itself nothing hands that handle to the guest, so it is table
 * growth proportional to callbacks, not a per-call leak a caller can observe.
 *
 * Handle DOMAIN: `target` is minted with `__storeElement`, because that is the
 * table `resize-observer-entry`'s `get-target` reads (the same choice the
 * mutation entry makes for its `target`, and the same one `mutationRecord`
 * makes). Minting it as a node handle yields a bigint the guest can hold and
 * never resolve, and both tables answer `bigint`, so only an assertion on the
 * TABLE — not on the type — can catch it (`callback-bridge.test.mjs`).
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
 * `added-nodes` / `removed-nodes` are `option<u64>`: ONE handle or `undefined`,
 * never a list (`jco types` emits `addedNodes?: bigint`). They carry the first
 * node of each NodeList. The record handle minted below is stored so that a
 * `mutation-record` handle minted elsewhere (the `MutationObserver` op path)
 * resolves the same object; the callback itself carries no record handle, so a
 * guest that wants every added node asks through the handle it was given.
 *
 * Handle DOMAINS: `target` is minted with `__storeElement` (matches
 * `mutation-record`'s `get-target`), while the sibling and node-list fields are
 * minted with `__storeNode` (matches its `get-previous-sibling` /
 * `get-next-sibling`). Mixing the two hands the guest a handle that looks fine
 * and resolves nowhere.
 */
export function mutationEntryValue(record: unknown, sink: RecordSink): Record<string, unknown> {
  const handle = globalThis.__nextMutationRecord++;
  sink.store(handle, record);
  const typed = record as Partial<MutationRecord>;
  const first = (nodes: NodeList | undefined): bigint | undefined => {
    const node = nodes && nodes.length ? nodes[0] : undefined;
    return nodeHandle(node);
  };
  return {
    mutationType: String(typed.type ?? ""),
    target: globalThis.__storeElement(typed.target) ?? 0n,
    addedNodes: first(typed.addedNodes),
    removedNodes: first(typed.removedNodes),
    previousSibling: nodeHandle(typed.previousSibling),
    nextSibling: nodeHandle(typed.nextSibling),
    attributeName: typed.attributeName ?? NONE,
    attributeNamespace: typed.attributeNamespace ?? NONE,
    oldValue: typed.oldValue ?? NONE,
  };
}

/**
 * One DOM `GeolocationPosition` → the `geo-position` record jco expects.
 *
 * `on-position-success(callback-id, position: geo-position)` takes the WHOLE
 * record (`jco types`: `onPositionSuccess(callbackId: bigint, position:
 * GeoPosition)`), not two loose numbers. `option<f64>` fields become optional
 * properties — `accuracy` is the only one the WIT declares as required.
 */
export function geoPositionValue(position: unknown): Record<string, unknown> {
  const coords = (position as { coords?: Partial<GeolocationCoordinates> }).coords ?? {};
  const optional = (value: unknown): number | undefined =>
    typeof value === "number" && Number.isFinite(value) ? value : NONE;
  return {
    latitude: Number(coords.latitude ?? 0),
    longitude: Number(coords.longitude ?? 0),
    altitude: optional(coords.altitude),
    accuracy: Number(coords.accuracy ?? 0),
    altitudeAccuracy: optional(coords.altitudeAccuracy),
    heading: optional(coords.heading),
    speed: optional(coords.speed),
  };
}

/**
 * One `GeolocationPositionError` → the `geo-position-error` record.
 * (`onPositionError(callbackId: bigint, error: GeoPositionError)`, `code: u8`.)
 */
export function geoPositionErrorValue(error: unknown): Record<string, unknown> {
  const failure = error as Partial<GeolocationPositionError>;
  return {
    code: Number(failure.code ?? 0),
    message: String(failure.message ?? ""),
  };
}
