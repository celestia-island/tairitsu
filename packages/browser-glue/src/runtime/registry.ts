import { document_exports } from "./document";
import { element_exports } from "./element";
import { node_exports } from "./node";
import { nonElementParentNode_exports } from "./nonElementParentNode";
import { parentNode_exports } from "./parentNode";
import { window_exports } from "./window";
import { platformHelpers_exports } from "./platformHelpers";
import { mutationRecord_exports } from "./mutationRecord";
import { resizeObserverEntry_exports } from "./resizeObserverEntry";
import { resizeObserverSize_exports } from "./resizeObserverSize";
import { resizeObserver_exports } from "./resizeObserver";
import { cssStyleDeclaration_exports } from "./cssStyleDeclaration";
import { mediaQueryList_exports } from "./mediaQueryList";
import { elementCssInlineStyle_exports } from "./elementCssInlineStyle";
import { eventTarget_exports } from "./eventTarget";
import { domTokenList_exports } from "./domTokenList";
import { nodeList_exports } from "./nodeList";
import { location_exports } from "./location";
import { history_exports } from "./history";
import { observers_exports } from "./observers";
import { mutationObserver_exports } from "./mutationObserver";
import { event_exports } from "./event";
// Interfaces a component built against `tairitsu-web` imports unconditionally but
// that were never wired here — a page could not instantiate at all without them
// (`html-input-element` is read by tairitsu-web itself; the three event types are
// part of the vdom event bridge's import set).
import { htmlInputElement_exports } from "./htmlInputElement";
import { mouseEvent_exports } from "./mouseEvent";
import { pointerEvent_exports } from "./pointerEvent";
import { wheelEvent_exports } from "./wheelEvent";

export const INTERFACES = {
  "@tairitsu-glue/document": document_exports,
  "@tairitsu-glue/element": element_exports,
  "@tairitsu-glue/node": node_exports,
  "@tairitsu-glue/non-element-parent-node": nonElementParentNode_exports,
  "@tairitsu-glue/parent-node": parentNode_exports,
  "@tairitsu-glue/window": window_exports,
  "@tairitsu-glue/platform-helpers": platformHelpers_exports,
  "@tairitsu-glue/mutation-record": mutationRecord_exports,
  "@tairitsu-glue/resize-observer-entry": resizeObserverEntry_exports,
  "@tairitsu-glue/resize-observer-size": resizeObserverSize_exports,
  "@tairitsu-glue/resize-observer": resizeObserver_exports,
  "@tairitsu-glue/css-style-declaration": cssStyleDeclaration_exports,
  "@tairitsu-glue/media-query-list": mediaQueryList_exports,
  "@tairitsu-glue/element-css-inline-style": elementCssInlineStyle_exports,
  "@tairitsu-glue/event-target": eventTarget_exports,
  "@tairitsu-glue/dom-token-list": domTokenList_exports,
  "@tairitsu-glue/node-list": nodeList_exports,
  "@tairitsu-glue/location": location_exports,
  "@tairitsu-glue/history": history_exports,
  "@tairitsu-glue/observers": observers_exports,
  "@tairitsu-glue/mutation-observer": mutationObserver_exports,
  "@tairitsu-glue/event": event_exports,
  "@tairitsu-glue/html-input-element": htmlInputElement_exports,
  "@tairitsu-glue/mouse-event": mouseEvent_exports,
  "@tairitsu-glue/pointer-event": pointerEvent_exports,
  "@tairitsu-glue/wheel-event": wheelEvent_exports,
};

// W5: the import map and the stringified module code are gone. The interfaces
// are emitted as REAL ESM files at build time (`scripts/emit-glue-esm.ts`), the
// packager copies them beside the page and rewrites the jco wrapper's
// `@tairitsu-glue/<name>` specifiers to those files. Nothing is stringified at
// runtime, no blob URL is created and no inline import map is injected — which
// is what makes the page survive `script-src 'self'`.
