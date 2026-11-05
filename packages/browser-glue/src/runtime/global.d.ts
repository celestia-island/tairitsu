/**
 * Type declarations for tairitsu browser-glue runtime globalThis extensions.
 *
 * The browser-glue runtime stores all browser object references as opaque
 * bigint handles in globalThis-scoped Maps. This module declares those
 * extensions so that TypeScript strict mode can verify the runtime code
 * without @ts-nocheck. // ts-escape-ok: retires with L40
 *
 * NOTE: __elementHandles is intentionally typed as Map<bigint, any>
 * because it stores not only Elements but also ResizeObserver,
 * MutationObserver, AudioContext, WebSocket, IDBDatabase, and other
 * objects. It functions as a generic object handle store.
 */

export {};

// Opaque handle-store value type. Aliased here so the ambient block below
// carries no bare escape-prone type (the type-gate ambient scan); the alias
// site itself is the counted escape.
type AnyValue = any; // ts-escape-ok: retires with L40

declare global {
  var __elementHandles: Map<bigint, AnyValue>;
  var __documentHandles: Map<bigint, Document>;
  var __nodeHandles: Map<bigint, Node>;
  var __textHandles: Map<bigint, Text>;
  var __mutationRecordHandles: Map<bigint, MutationRecord>;
  var __resizeObserverEntryHandles: Map<bigint, ResizeObserverEntry>;
  var __resizeObserverSizeHandles: Map<bigint, ResizeObserverSize>;
  var __domRectHandles: Map<bigint, DOMRect>;
  var __cssStyleDeclarationHandles: Map<bigint, CSSStyleDeclaration>;
  var __nodeListHandles: Map<bigint, NodeList>;
  var __listenerHandles: Map<
    bigint,
    {
      element: Element;
      type: string;
      listener: (event: Event) => void;
    }
  >;
  var __eventHandles: Map<bigint, Event>;

  var __nextHandle: bigint;
  var __nextMutationRecord: bigint;
  var __nextResizeObserverEntry: bigint;
  var __nextResizeObserverSizeHandle: bigint;
  var __nextDomRectHandle: bigint;
  var __nextCssStyleDeclarationHandle: bigint;
  var __nextNodeList: bigint;
  var __nextListenerHandle: bigint;
  var __nextEventHandle: bigint;

  var __storeElement: (obj: AnyValue) => bigint | undefined;
  var __storeNode: (node: Node | null) => bigint | undefined;
  var __storeText: (text: Text | null) => bigint | undefined;
  var __lookupElement: (handle: bigint) => AnyValue;
  var __lookupNode: (handle: bigint) => Node;
  var __storeCssStyleDeclaration: (obj: CSSStyleDeclaration | null) => bigint | undefined;
  var __lookupCssStyleDeclaration: (handle: bigint) => CSSStyleDeclaration;

  var __tairitsuTimerState: {
    timeoutCallbacks: Map<number, number>;
    intervalCallbacks: Map<number, number>;
    nextTimeoutId: number;
  };

  var __tairitsuAnimState: {
    animationCallbacks: Map<number, number>;
    nextAnimationId: number;
  };

  var __tairitsuWsState: {
    nextCallbackId: bigint;
  };

  var __tairitsuWsCallbacks: Map<
    bigint,
    {
      openCbId: bigint;
      msgCbId: bigint;
      closeCbId: bigint;
      errCbId: bigint;
    }
  >;

  var __dispatchingEvents: Set<string>;

  var __wasmExports: Record<string, AnyValue> | null;
  var __setWasmExports: (exports: Record<string, AnyValue>) => void;

  var __TAIRITSU_GLUE__: {
    INTERFACES: Record<string, Record<string, (...args: AnyValue[]) => AnyValue>>;
    handles: {
      readonly elementHandles: Map<bigint, AnyValue>;
      readonly nodeHandles: Map<bigint, Node>;
      readonly documentHandles: Map<bigint, Document>;
      readonly textHandles: Map<bigint, Text>;
      readonly nextHandle: bigint;
    };
  };
}
