/**
 * Type declarations for tairitsu browser-glue runtime globalThis extensions.
 *
 * The browser-glue runtime stores all browser object references as opaque
 * bigint handles in globalThis-scoped Maps. This module declares those
 * extensions so that TypeScript strict mode can verify the runtime code
 * without @ts-nocheck.
 *
 * NOTE: __elementHandles is intentionally typed as Map<bigint, any>
 * because it stores not only Elements but also ResizeObserver,
 * MutationObserver, AudioContext, WebSocket, IDBDatabase, and other
 * objects. It functions as a generic object handle store.
 */

export {};

declare global {
  var __elementHandles: Map<bigint, any>;
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

  var __storeElement: (obj: any) => bigint | undefined;
  var __storeNode: (node: Node | null) => bigint | undefined;
  var __storeText: (text: Text | null) => bigint | undefined;
  var __lookupElement: (handle: bigint) => any;
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

  var __wasmExports: Record<string, any> | null;
  var __setWasmExports: (exports: Record<string, any>) => void;

  var __TAIRITSU_GLUE__: {
    INTERFACES: Record<string, Record<string, (...args: any[]) => any>>;
    handles: {
      readonly elementHandles: Map<bigint, any>;
      readonly nodeHandles: Map<bigint, Node>;
      readonly documentHandles: Map<bigint, Document>;
      readonly textHandles: Map<bigint, Text>;
      readonly nextHandle: bigint;
    };
  };
}
