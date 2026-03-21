#!/usr/bin/env python3
"""Configuration and type mapping constants for browser glue generator."""

# Interface name → browser class mapping
INTERFACE_TO_BROWSER_CLASS = {
    "node": "Node",
    "element": "Element",
    "document": "Document",
    "window": "Window",
    "headers": "Headers",
    "request": "Request",
    "response": "Response",
    "body": "Body",
    "storage-manager": "StorageManager",
    "navigator-storage": "NavigatorStorage",
    "storage": "Storage",
    "url": "URL",
    "url-search-params": "URLSearchParams",
    "crypto": "Crypto",
    "subtle-crypto": "SubtleCrypto",
    "crypto-key": "CryptoKey",
    "event": "Event",
    "event-target": "EventTarget",
    "navigator": "Navigator",
    "battery-manager": "BatteryManager",
    "gamepad": "Gamepad",
    "gamepad-button": "GamepadButton",
    "gamepad-haptic-actuator": "GamepadHapticActuator",
    "geolocation-coordinates": "GeolocationCoordinates",
    "geolocation-position": "GeolocationPosition",
    "geolocation-position-error": "GeolocationPositionError",
    "screen-orientation": "ScreenOrientation",
    "media-capabilities": "MediaCapabilities",
    "media-devices": "MediaDevices",
    "media-session": "MediaSession",
    "media-stream": "MediaStream",
    "canvas-rendering-context": "CanvasRenderingContext2D",
    "webgl-rendering-context": "WebGLRenderingContext",
    "notification": "Notification",
    "permissions": "Permissions",
    "permission-status": "PermissionStatus",
    "readable-stream": "ReadableStream",
    "writable-stream": "WritableStream",
    "transform-stream": "TransformStream",
    "worker": "Worker",
    "service-worker": "ServiceWorker",
    "idb-database": "IDBDatabase",
    "idb-request": "IDBRequest",
    "idb-transaction": "IDBTransaction",
    "geolocation": "Geolocation",
    "performance": "Performance",
    "rtc-peer-connection": "RTCPeerConnection",
    "credential": "Credential",
    "password-credential": "PasswordCredential",
    "federated-credential": "FederatedCredential",
    "credential-user-data": "any",
    "module": "WebAssembly.Module",
    "origin": "Origin",
    "speech-recognition": "SpeechRecognition",
}

# Types that don't exist in TypeScript DOM library (use 'any' for handles)
MISSING_TYPES_IN_DOM = {
    "credential-user-data": "any",
    "password-credential": "any",
    "federated-credential": "any",
}

# Known global singleton objects (not instantiated, use global reference)
GLOBAL_SINGLETONS = {
    "window": "window",
    "document": "document",
    "navigator": "navigator",
    "crypto": "window.crypto",
    "screen": "window.screen",
    "location": "window.location",
    "history": "window.history",
    "performance": "window.performance",
}

# Known async operation patterns (return Promises in browser)
# NOTE: These are global patterns - use ASYNC_METHODS_BY_INTERFACE for interface-specific overrides
ASYNC_PATTERNS = [
    "estimate", "persist", "persisted", "fetch", "respond",
    "array-buffer", "blob", "text", "json", "bytes", "form-data",
    "get-reader", "get-writer", "pipe-to", "pipe-through",
    "from", "create-image-bitmap",
    "get-user-media", "enumerate-devices",
    "query", "request",
    "get", "store", "create", "prevent-silent-access",
]

# Methods that are async only on specific interfaces
# Maps (interface_name, method_name) -> True (is async) or False (NOT async, overrides ASYNC_PATTERNS)
ASYNC_METHOD_OVERRIDES = {
    # WebCodecs - flush is async, but close/reset are NOT async
    ("audio-decoder", "flush"): True,
    ("audio-decoder", "close"): False,
    ("audio-decoder", "reset"): False,
    ("audio-decoder", "is-config-supported"): True,
    ("video-decoder", "flush"): True,
    ("video-decoder", "close"): False,
    ("video-decoder", "reset"): False,
    ("video-decoder", "is-config-supported"): True,
    ("audio-encoder", "flush"): True,
    ("audio-encoder", "close"): False,
    ("audio-encoder", "reset"): False,
    ("audio-encoder", "is-config-supported"): True,
    ("video-encoder", "flush"): True,
    ("video-encoder", "close"): False,
    ("video-encoder", "reset"): False,
    ("video-encoder", "is-config-supported"): True,
    # ImageDecoder - close/reset are NOT async, but decode/parse are async
    ("image-decoder", "close"): False,
    ("image-decoder", "reset"): False,
    ("image-decoder", "decode"): True,
    ("image-decoder", "parse"): True,
    ("image-decoder", "is-type-supported"): True,
    # Streams - readers/writers are async
    ("readable-stream-byob-reader", "read"): True,
    ("readable-stream-default-reader", "read"): True,
    ("writable-stream-default-writer", "write"): True,
    ("writable-stream-default-writer", "close"): True,
    ("writable-stream-default-writer", "abort"): True,
    ("writable-stream-default-writer", "ready"): True,
    # ReadableStream/WritableStream methods
    ("readable-stream", "cancel"): True,
    ("writable-stream", "abort"): True,
    ("writable-stream", "close"): True,
    # SubtleCrypto - all methods are async
    ("subtle-crypto", "decrypt"): True,
    ("subtle-crypto", "encrypt"): True,
    ("subtle-crypto", "sign"): True,
    ("subtle-crypto", "verify"): True,
    ("subtle-crypto", "digest"): True,
    ("subtle-crypto", "generate-key"): True,
    ("subtle-crypto", "derive-key"): True,
    ("subtle-crypto", "derive-bits"): True,
    ("subtle-crypto", "import-key"): True,
    ("subtle-crypto", "export-key"): True,
    ("subtle-crypto", "wrap-key"): True,
    ("subtle-crypto", "unwrap-key"): True,
    # Cache Storage
    ("cache-storage", "match"): True,
    ("cache-storage", "has"): True,
    ("cache-storage", "open"): True,
    ("cache-storage", "delete"): True,
    ("cache-storage", "keys"): True,
    # Cache API
    ("cache", "match"): True,
    ("cache", "match-all"): True,
    ("cache", "add"): True,
    ("cache", "add-all"): True,
    ("cache", "put"): True,
    ("cache", "delete"): True,
    ("cache", "keys"): True,
    # Clipboard - all methods are async
    ("clipboard", "read"): True,
    ("clipboard", "write"): True,
    ("clipboard", "read-text"): True,
    ("clipboard", "write-text"): True,
    # Permissions
    ("permissions", "query"): True,
    # Notification
    ("notification", "request-permission"): True,
    # Geolocation
    ("geolocation", "get-current-position"): True,
    ("geolocation", "watch-position"): False,  # Returns watch ID, not Promise
    # Headers - get is synchronous
    ("headers", "get"): False,
    # FormData - get is synchronous
    ("form-data", "get"): False,
    # URLSearchParams - all is synchronous
    ("url-search-params", "all"): False,
    ("url-search-params", "get"): False,
    # Response.json is synchronous (static factory method)
    ("response", "json"): False,
    # ReadableStream.from is synchronous (returns ReadableStream)
    ("readable-stream", "from"): False,
    # ReadableStreamBYOBRequest.respond is synchronous (returns void)
    ("readable-stream-byob-request", "respond"): False,
    ("readable-stream-byob-request", "respond-with-new-view"): False,
    # ReadableStream.pipeThrough returns ReadableStream, not Promise
    ("readable-stream", "pipe-through"): False,
    # GamepadHapticActuator - async methods
    ("gamepad-haptic-actuator", "play-effect"): True,
    ("gamepad-haptic-actuator", "reset"): True,
    # MediaCapabilities - async methods
    ("media-capabilities", "decoding-info"): True,
    ("media-capabilities", "encoding-info"): True,
    # MediaDevices - async methods
    ("media-devices", "get-user-media"): True,
    ("media-devices", "get-display-media"): True,
    ("media-devices", "enumerate-devices"): True,
    # RTCPeerConnection - async methods
    ("rtc-peer-connection", "create-offer"): True,
    ("rtc-peer-connection", "create-answer"): True,
    ("rtc-peer-connection", "set-local-description"): True,
    ("rtc-peer-connection", "set-remote-description"): True,
    ("rtc-peer-connection", "add-ice-candidate"): True,
    ("rtc-peer-connection", "get-stats"): True,
    ("rtc-peer-connection", "restart-ice"): True,
    ("rtc-peer-connection", "generate-certificate"): True,
    # ServiceWorkerRegistration - async methods
    ("service-worker-registration", "show-notification"): True,
    ("service-worker-registration", "get-notifications"): True,
    ("service-worker-registration", "update"): True,
    ("service-worker-registration", "unregister"): True,
    # ServiceWorkerContainer - async methods
    ("service-worker-container", "get-ready"): True,
    ("service-worker-container", "register"): True,
    ("service-worker-container", "get-registration"): True,
    ("service-worker-container", "get-registrations"): True,
    # PaymentRequest - async methods
    ("payment-request", "abort"): True,
    ("payment-request", "can-make-payment"): True,
    ("payment-request", "show"): True,
    # PaymentResponse - async methods
    ("payment-response", "complete"): True,
    ("payment-response", "retry"): True,
    # CSSStyleSheet - async methods
    ("css-style-sheet", "replace"): True,
    # Document - async methods
    ("document", "exit-fullscreen"): True,
    ("document", "exit-picture-in-picture"): True,
    # OffscreenCanvas - async methods
    ("offscreencanvas", "convert-to-blob"): True,
    # CustomElementRegistry - async methods
    ("custom-element-registry", "when-defined"): True,
    # FontFace - async methods
    ("font-face", "load"): True,
    # VideoFrame - async methods
    ("video-frame", "copy-to"): True,
    # ImageDecoder - async getters
    ("image-decoder", "get-completed"): True,
    # ImageTrackList - async getters
    ("image-track-list", "get-ready"): True,
    # ReadableStreamGenericReader - async methods
    ("readable-stream-generic-reader", "get-closed"): True,
    ("readable-stream-generic-reader", "cancel"): True,
    # WritableStreamDefaultWriter - async getters
    ("writable-stream-default-writer", "get-closed"): True,
    # Element - async methods
    ("element", "request-fullscreen"): True,
    # HTMLImageElement - async methods
    ("html-image-element", "decode"): True,
    # HTMLMediaElement - async methods
    ("html-media-element", "play"): True,
    # OffscreenCanvas - async methods
    ("offscreen-canvas", "convert-to-blob"): True,
    # Worklet - async methods
    ("worklet", "add-module"): True,
    # MediaStreamTrack - async methods
    ("media-stream-track", "apply-constraints"): True,
    # RTCRtpSender - async methods
    ("rtc-rtp-sender", "replace-track"): True,
    # NavigationPreloadManager - async methods
    ("navigation-preload-manager", "enable"): True,
    ("navigation-preload-manager", "disable"): True,
    ("navigation-preload-manager", "set-header-value"): True,
    ("navigation-preload-manager", "get-state"): True,
    # PromiseRejectionEvent - async getter
    ("promise-rejection-event", "get-promise"): True,
}

# Attribute getters that return handles (need wrapping)
HANDLE_GETTER_PATTERNS = [
    "get-headers", "get-body", "get-signal",
    "get-storage", "get-crypto", "get-performance",
]

# Special browser API name mappings (WIT name -> browser API name)
BROWSER_API_NAME_MAPPINGS = {
    "get-random-values": "getRandomValues",
    "random-uuid": "randomUUID",
    "is-conditional-mediation-available": "isConditionalMediationAvailable",
    "to-json": "toJSON",
    "inner-html": "innerHTML",
    "outer-html": "outerHTML",
    "namespace-uri": "namespaceURI",
    "document-uri": "documentURI",
    "base-uri": "baseURI",
    "voice-uri": "voiceURI",
    "script-url": "scriptURL",
    "old-url": "oldURL",
    "new-url": "newURL",
    "current-css-zoom": "currentCSSZoom",
    "has-ua-visual-transition": "hasUAVisualTransition",
    "elements-by-tag-name": "getElementsByTagName",
    "elements-by-tag-name-ns": "getElementsByTagNameNS",
    "elements-by-class-name": "getElementsByClassName",
    "element-by-id": "getElementById",
    "elements-by-name": "getElementsByName",
    "client-rects": "getClientRects",
    "bounding-client-rect": "getBoundingClientRect",
    "client-rect": "getClientRect",
    "attribute": "getAttribute",
    "attribute-ns": "getAttributeNS",
    "get-attribute-ns": "getAttributeNS",
    "attribute-node": "getAttributeNode",
    "attribute-node-ns": "getAttributeNodeNS",
    "get-attribute-node-ns": "getAttributeNodeNS",
    "attribute-names": "getAttributeNames",
    "get-transform": "getTransform",
    "set-attribute": "setAttribute",
    "set-attribute-ns": "setAttributeNS",
    "remove-attribute-ns": "removeAttributeNS",
    "has-attribute-ns": "hasAttributeNS",
    "named-item": "namedItem",
    "named-item-ns": "getNamedItemNS",
    "get-named-item-ns": "getNamedItemNS",
    "get-named-item": "getNamedItem",
    "track-by-id": "getTrackById",
    "create-element-ns": "createElementNS",
    "create-attribute-ns": "createAttributeNS",
    "create-cdata-section": "createCDATASection",
    "create-ns-resolver": "createNSResolver",
    "insert-adjacent-html": "insertAdjacentHTML",
    "pointer-capture": "hasPointerCapture",
    "set-start-before": "setStartBefore",
    "set-start-after": "setStartAfter",
    "set-end-before": "setEndBefore",
    "lookup-namespace-uri": "lookupNamespaceURI",
    "remove-named-item-ns": "removeNamedItemNS",
    "modifier-state": "getModifierState",
    "coalesced-events": "getCoalescedEvents",
    "predicted-events": "getPredictedEvents",
    "set-cookie": "getSetCookie",
    "context-attributes": "getContextAttributes",
    "image-data": "getImageData",
    "set-transform": "setTransform",
    "form-value": "setFormValue",
    "drag-image": "setDragImage",
    "add-search-provider": "AddSearchProvider",
    "is-search-provider-installed": "IsSearchProviderInstalled",
    "audio-tracks": "getAudioTracks",
    "video-tracks": "getVideoTracks",
    "constraints": "getConstraints",
    "supported-constraints": "getSupportedConstraints",
    "user-media": "getUserMedia",
    "display-media": "getDisplayMedia",
    "action-handler": "setActionHandler",
    "position-state": "setPositionState",
    "microphone-active": "setMicrophoneActive",
    "camera-active": "setCameraActive",
    "notifications": "getNotifications",
    "resource-timing-buffer-size": "setResourceTimingBufferSize",
    "entries-by-type": "getEntriesByType",
    "entries-by-name": "getEntriesByName",
    "receivers": "getReceivers",
    "transceivers": "getTransceivers",
    "capabilities": "getCapabilities",
    "configuration": "getConfiguration",
    "css-keyframe-rule": "KEYFRAME_RULE",
    "current-position": "getCurrentPosition",
    "random-values": "getRandomValues",
    "registrations": "getRegistrations",
    "header-value": "setHeaderValue",
    "computed-style": "getComputedStyle",
    "html-unsafe": "setHTMLUnsafe",
    "context-attributes": "getContextAttributes",
    "supported-extensions": "getSupportedExtensions",
    "extension": "getExtension",
    "active-attrib": "getActiveAttrib",
    "active-uniform": "getActiveUniform",
    "attached-shaders": "getAttachedShaders",
    "attrib-location": "getAttribLocation",
    "buffer-parameter": "getBufferParameter",
    "parameter": "getParameter",
    "framebuffer-attachment-parameter": "getFramebufferAttachmentParameter",
    "program-parameter": "getProgramParameter",
    "program-info-log": "getProgramInfoLog",
    "renderbuffer-parameter": "getRenderbufferParameter",
    "shader-parameter": "getShaderParameter",
    "shader-precision-format": "getShaderPrecisionFormat",
    "shader-info-log": "getShaderInfoLog",
    "shader-source": "shaderSource",
    "tex-parameter": "getTexParameter",
    "uniform": "getUniform",
    "uniform-location": "getUniformLocation",
    "vertex-attrib": "getVertexAttrib",
    "vertex-attrib-offset": "getVertexAttribOffset",
    "internalformat-parameter": "getInternalformatParameter",
    "frag-data-location": "getFragDataLocation",
    "is-query": "isQuery",
    "query": "getQuery",
    "query-parameter": "getQueryParameter",
    "sampler-parameter": "getSamplerParameter",
    "sync-parameter": "getSyncParameter",
    "transform-feedback-varying": "getTransformFeedbackVarying",
    "transform-feedback-varyings": "transformFeedbackVaryings",
    "indexed-parameter": "getIndexedParameter",
    "uniform-indices": "getUniformIndices",
    "active-uniforms": "getActiveUniforms",
    "uniform-block-index": "getUniformBlockIndex",
    "active-uniform-block-parameter": "getActiveUniformBlockParameter",
    "active-uniform-block-name": "getActiveUniformBlockName",
    "property-value": "getPropertyValue",
    "property-priority": "getPropertyPriority",
    "svg-document": "getSVGDocument",
    "get-svg-document": "getSVGDocument",
    "custom-validity": "setCustomValidity",
    "range-text": "setRangeText",
    "selection-range": "setSelectionRange",
    "create-html-document": "createHTMLDocument",
    "fingerprints": "getFingerprints",
    "parameters": "getParameters",
    "contributing-sources": "getContributingSources",
    "synchronization-sources": "getSynchronizationSources",
    "codec-preferences": "setCodecPreferences",
    "remote-certificates": "getRemoteCertificates",
    "selected-candidate-pair": "getSelectedCandidatePair",
    "insert-dtmf": "insertDTMF",
    "can-insert-dtmf": "canInsertDTMF",
    "request-header": "setRequestHeader",
    "response-url": "responseURL",
    "response-header": "getResponseHeader",
    "all-response-headers": "getAllResponseHeaders",
    "response-xml": "responseXML",
    "init-ui-event": "initUIEvent",
    "to-data-url": "toDataURL",
    "source-element": "srcElement",
    "url": "url",
    "result-val": "result",
    "start-before": "setStartBefore",
    "start-after": "setStartAfter",
    "end-before": "setEndBefore",
    "types": "types",
    "get-buffer-sub-data": "getBufferSubData",
    "get-completed": "completed",
    "get-closed": "closed",
    "get-promise": "promise",
    "set-html-unsafe": "setHTMLUnsafe",
    "set-attribute-node-ns": "setAttributeNodeNS",
}

# Static method return type overrides (method name -> actual return type)
STATIC_METHOD_RETURN_OVERRIDES = {
    ("credential", "isConditionalMediationAvailable"): "Promise<boolean>",
    # WebCodecs static methods return Promises
    ("audio-decoder", "isConfigSupported"): "Promise<AudioDecoderSupport>",
    ("video-decoder", "isConfigSupported"): "Promise<VideoDecoderSupport>",
    ("audio-encoder", "isConfigSupported"): "Promise<AudioEncoderSupport>",
    ("video-encoder", "isConfigSupported"): "Promise<VideoEncoderSupport>",
}

# Static methods that need type assertions (methods missing from TypeScript DOM lib)
STATIC_METHOD_NEEDS_TYPE_ASSERTION = {
    ("Credential", "isConditionalMediationAvailable"),
    ("Origin", "from"),
    ("SpeechRecognition", "available"),
    ("SpeechRecognition", "install"),
    ("Module", "exports"),
    ("Module", "imports"),
    ("Module", "customSections"),
    ("ImageTrackList", "getReady"),
    ("ReadableStream", "from"),
    ("FileReader", "newFileReader"),
    ("Permissions", "getQuery"),
    ("Table", "get"),
    ("URLSearchParams", "all"),
    ("WebSocket", "connect"),
}

# Functions that return browser objects (need to wrap in handle)
# Maps (interface, method_name) -> target_interface for handle wrapping
# NOTE: method_name should be in camelCase (the format used by the code generator)
HANDLE_RETURNING_FUNCTIONS = {
    ("crypto", "getSubtle"): "subtle-crypto",
    # WebGL - create methods return objects (web-gl-xxx naming)
    ("web-gl-rendering-context-base", "createBuffer"): "web-gl-object",
    ("web-gl-rendering-context-base", "createFramebuffer"): "web-gl-object",
    ("web-gl-rendering-context-base", "createProgram"): "web-gl-object",
    ("web-gl-rendering-context-base", "createRenderbuffer"): "web-gl-object",
    ("web-gl-rendering-context-base", "createShader"): "web-gl-object",
    ("web-gl-rendering-context-base", "createTexture"): "web-gl-object",
    ("web-gl-rendering-context-base", "getExtension"): "any",
    ("web-gl-rendering-context-base", "getActiveAttrib"): "web-gl-active-info",
    ("web-gl-rendering-context-base", "getActiveUniform"): "web-gl-active-info",
    ("web-gl-rendering-context-base", "getShaderPrecisionFormat"): "web-gl-shader-precision-format",
    ("web-gl-rendering-context-base", "getUniformLocation"): "web-gl-uniform-location",
    ("web-gl-rendering-context-base", "getContextAttributes"): "any",
    ("web-gl-rendering-context-base", "getAttachedShaders"): "web-gl-shader-list",
    ("web-gl-rendering-context-base", "getSupportedExtensions"): "string-list",
    ("web-gl-rendering-context-base", "getBufferParameter"): "any",
    ("web-gl-rendering-context-base", "getFramebufferAttachmentParameter"): "any",
    ("web-gl-rendering-context-base", "getParameter"): "any",
    ("web-gl-rendering-context-base", "getProgramParameter"): "any",
    ("web-gl-rendering-context-base", "getRenderbufferParameter"): "any",
    ("web-gl-rendering-context-base", "getShaderParameter"): "any",
    ("web-gl-rendering-context-base", "getTexParameter"): "any",
    ("web-gl-rendering-context-base", "getUniform"): "any",
    ("web-gl-rendering-context-base", "getVertexAttrib"): "any",
    ("web-gl-rendering-context-base", "getVertexAttribOffset"): "any",
    # WebGL2
    ("web-gl2-rendering-context-base", "createQuery"): "web-gl-object",
    ("web-gl2-rendering-context-base", "createSampler"): "web-gl-object",
    ("web-gl2-rendering-context-base", "createTransformFeedback"): "web-gl-object",
    ("web-gl2-rendering-context-base", "createVertexArray"): "web-gl-object",
    ("web-gl2-rendering-context-base", "getTransformFeedbackVarying"): "web-gl-active-info",
    ("web-gl2-rendering-context-base", "getInternalformatParameter"): "any",
    ("web-gl2-rendering-context-base", "getFragDataLocation"): "number",
    ("web-gl2-rendering-context-base", "getQuery"): "any",
    ("web-gl2-rendering-context-base", "getUniformIndices"): "number-list",
    ("web-gl2-rendering-context-base", "getActiveUniforms"): "any",
    ("web-gl2-rendering-context-base", "getIndexedParameter"): "any",
    ("web-gl2-rendering-context-base", "getBufferParameter"): "any",
    ("web-gl2-rendering-context-base", "getFramebufferAttachmentParameter"): "any",
    ("web-gl2-rendering-context-base", "getParameter"): "any",
    ("web-gl2-rendering-context-base", "getProgramParameter"): "any",
    ("web-gl2-rendering-context-base", "getRenderbufferParameter"): "any",
    ("web-gl2-rendering-context-base", "getSamplerParameter"): "any",
    ("web-gl2-rendering-context-base", "getSyncParameter"): "any",
    ("web-gl2-rendering-context-base", "getTexParameter"): "any",
    ("web-gl2-rendering-context-base", "getUniform"): "any",
    ("web-gl2-rendering-context-base", "getUniformBlockIndex"): "number",
    ("web-gl2-rendering-context-base", "getActiveUniformBlockParameter"): "any",
    ("web-gl2-rendering-context-base", "getActiveUniformBlockName"): "string",
    ("web-gl2-rendering-context-base", "getQueryParameter"): "any",
    ("web-gl2-rendering-context-base", "fenceSync"): "web-gl-object",
    # WebCodecs
    ("audio-data", "clone"): "audio-data",
    ("video-frame", "clone"): "video-frame",
    ("video-frame", "getColorSpace"): "video-color-space",
    ("video-frame", "getCodedRect"): "dom-rect-read-only",
    ("video-frame", "getVisibleRect"): "dom-rect-read-only",
    ("video-color-space", "toJson"): "any",
    ("image-track-list", "getSelectedTrack"): "image-track",
    ("web-gl-rendering-context-base", "getCanvas"): "any",
    # Canvas
    ("html-canvas-element", "getContext"): "any",
    ("offscreencanvas", "getContext"): "any",
    ("canvas-rendering-context", "getCanvas"): "any",
    ("canvas-rendering-context", "createLinearGradient"): "canvas-gradient",
    ("canvas-rendering-context", "createRadialGradient"): "canvas-gradient",
    ("canvas-rendering-context", "createConicGradient"): "canvas-gradient",
    ("canvas-rendering-context", "createImageData"): "image-data",
    ("canvas-rendering-context", "getImageData"): "image-data",
    ("canvas-rendering-context", "createPattern"): "canvas-pattern",
    ("canvas-rendering-context", "getContextAttributes"): "any",
    ("canvas-rendering-context", "measureText"): "text-metrics",
    ("canvas-rendering-context", "getLineDash"): "float-32-list",
    ("canvas-rendering-context", "getTransform"): "dom-matrix",
    ("canvas-rendering-context", "getBoundingClientRect"): "dom-rect",
    ("canvas-rendering-context", "getClientRects"): "dom-rect-list",
    # CanvasImageData
    ("canvas-image-data", "getImageData"): "image-data",
    # CanvasTransform
    ("canvas-transform", "getTransform"): "dom-matrix",
    # ImageDecoder
    ("image-decoder", "tracks"): "image-track-list",
    ("image-track-list", "imageTrack"): "image-track",
    ("image-decoder", "decode"): "image-decode-result",
    ("image-decoder", "type"): "string",
    # DOM
    ("node", "getRootNode"): "node",
    ("element", "closest"): "element",
    ("element", "matches"): "boolean",
    ("element", "webkitMatchesSelector"): "boolean",
    ("element", "getElementsByTagName"): "html-collection",
    ("element", "getElementsByTagNameNS"): "html-collection",
    ("element", "getElementsByClassName"): "html-collection",
    ("element", "querySelector"): "element",
    ("element", "querySelectorAll"): "node-list",
    ("element", "getBoundingClientRect"): "dom-rect",
    ("element", "getClientRects"): "dom-rect-list",
    ("element", "getAttribute"): "string",
    ("element", "getAttributeNS"): "string",
    ("element", "getAttributeNode"): "attr",
    ("element", "getAttributeNodeNS"): "attr",
    ("element", "getAttributeNames"): "string-list",
    ("element", "hasAttribute"): "boolean",
    ("element", "hasAttributeNS"): "boolean",
    ("element", "hasAttributes"): "boolean",
    ("element", "insertAdjacentElement"): "element",
    ("element", "insertAdjacentText"): "void",
    ("element", "removeAttribute"): "void",
    ("element", "removeAttributeNS"): "void",
    ("element", "removeAttributeNode"): "attr",
    ("element", "requestFullscreen"): "promise-void",
    ("element", "requestPointerLock"): "void",
    ("element", "scroll"): "void",
    ("element", "scrollBy"): "void",
    ("element", "scrollIntoView"): "void",
    ("element", "scrollTo"): "void",
    ("element", "setAttribute"): "void",
    ("element", "setAttributeNS"): "void",
    ("element", "setAttributeNode"): "attr",
    ("element", "setAttributeNodeNS"): "attr",
    ("element", "toggleAttribute"): "boolean",
    ("element", "attachShadow"): "shadow-root",
    ("element", "checkVisibility"): "boolean",
    ("element", "getAnimations"): "animation-list",
    ("element", "getHTML"): "string",
    ("element", "getPopoverTargetElement"): "element",
    ("element", "setPopoverTargetElement"): "void",
    ("element", "setHTMLUnsafe"): "void",
    ("element", "showPopover"): "void",
    ("element", "hidePopover"): "void",
    ("element", "togglePopover"): "boolean",
    # Document
    ("document", "createElement"): "element",
    ("document", "createElementNS"): "element",
    ("document", "createDocumentFragment"): "document-fragment",
    ("document", "createTextNode"): "text",
    ("document", "createComment"): "comment",
    ("document", "createCDATASection"): "cdata-section",
    ("document", "createProcessingInstruction"): "processing-instruction",
    ("document", "createAttribute"): "attr",
    ("document", "createAttributeNS"): "attr",
    ("document", "createEvent"): "event",
    ("document", "createRange"): "range",
    ("document", "createNodeIterator"): "node-iterator",
    ("document", "createTreeWalker"): "tree-walker",
    ("document", "getElementById"): "element",
    ("document", "getElementsByTagName"): "html-collection",
    ("document", "getElementsByTagNameNS"): "html-collection",
    ("document", "getElementsByClassName"): "html-collection",
    ("document", "getElementsByName"): "node-list",
    ("document", "querySelector"): "element",
    ("document", "querySelectorAll"): "node-list",
    ("document", "adoptNode"): "node",
    ("document", "importNode"): "node",
    ("document", "getSelection"): "selection",
    ("document", "hasFocus"): "boolean",
    ("document", "createExpression"): "xpath-expression",
    ("document", "createNSResolver"): "xpath-ns-resolver",
    ("document", "evaluate"): "xpath-result",
    ("document", "getComputedStyle"): "css-style-declaration",
    ("document", "getAnimations"): "animation-list",
    ("document", "elementFromPoint"): "element",
    ("document", "elementsFromPoint"): "element-list",
    ("document", "caretPositionFromPoint"): "caret-position",
    ("document", "caretRangeFromPoint"): "range",
    ("document", "createHTMLDocument"): "document",
    ("document", "parseHTMLUnsafe"): "document-fragment",
    ("document", "open"): "document",
    ("document", "close"): "void",
    ("document", "write"): "void",
    ("document", "writeln"): "void",
    ("document", "execCommand"): "boolean",
    ("document", "queryCommandEnabled"): "boolean",
    ("document", "queryCommandIndeterm"): "boolean",
    ("document", "queryCommandState"): "boolean",
    ("document", "queryCommandSupported"): "boolean",
    ("document", "queryCommandValue"): "string",
    ("document", "clear"): "void",
    ("document", "exitPictureInPicture"): "promise-void",
    ("document", "exitPointerLock"): "void",
    ("document", "releaseCapture"): "void",
    ("document", "createElementFromPoint"): "element",
    # Window
    ("window", "getComputedStyle"): "css-style-declaration",
    ("window", "getSelection"): "selection",
    ("window", "matchMedia"): "media-query-list",
    ("window", "open"): "window",
    ("window", "alert"): "void",
    ("window", "confirm"): "boolean",
    ("window", "prompt"): "string",
    ("window", "print"): "void",
    ("window", "postMessage"): "void",
    ("window", "captureEvents"): "void",
    ("window", "releaseEvents"): "void",
    ("window", "focus"): "void",
    ("window", "blur"): "void",
    ("window", "close"): "void",
    ("window", "stop"): "void",
    ("window", "moveTo"): "void",
    ("window", "moveBy"): "void",
    ("window", "resizeTo"): "void",
    ("window", "resizeBy"): "void",
    ("window", "scroll"): "void",
    ("window", "scrollTo"): "void",
    ("window", "scrollBy"): "void",
    ("window", "scrollByLines"): "void",
    ("window", "scrollByPages"): "void",
    ("window", "atob"): "string",
    ("window", "btoa"): "string",
    ("window", "setTimeout"): "number",
    ("window", "setInterval"): "number",
    ("window", "clearTimeout"): "void",
    ("window", "clearInterval"): "void",
    ("window", "queueMicrotask"): "void",
    ("window", "createImageBitmap"): "promise-image-bitmap",
    ("window", "fetch"): "promise-response",
    ("window", "reportError"): "void",
    ("window", "requestAnimationFrame"): "number",
    ("window", "cancelAnimationFrame"): "void",
    ("window", "requestIdleCallback"): "number",
    ("window", "cancelIdleCallback"): "void",
    # CSS
    ("css-style-sheet", "insertRule"): "number",
    ("css-style-sheet", "deleteRule"): "void",
    ("css-style-sheet", "addRule"): "number",
    ("css-style-sheet", "removeRule"): "void",
    ("css-style-sheet", "replace"): "promise-css-style-sheet",
    ("css-style-sheet", "replaceSync"): "void",
    ("css-style-declaration", "getPropertyValue"): "string",
    ("css-style-declaration", "getPropertyPriority"): "string",
    ("css-style-declaration", "setProperty"): "void",
    ("css-style-declaration", "removeProperty"): "string",
    ("css-style-declaration", "item"): "string",
    ("css-rule-list", "item"): "css-rule",
    ("css-media-rule", "insertRule"): "number",
    ("css-media-rule", "deleteRule"): "void",
    ("css-keyframes-rule", "appendRule"): "void",
    ("css-keyframes-rule", "deleteRule"): "void",
    ("css-keyframes-rule", "findRule"): "css-keyframe-rule",
    ("css-grouping-rule", "insertRule"): "number",
    ("css-grouping-rule", "deleteRule"): "void",
    ("media-list", "appendMedium"): "void",
    ("media-list", "deleteMedium"): "void",
    ("media-list", "item"): "string",
    # ParentNode
    ("parent-node", "querySelector"): "element",
    ("parent-node", "querySelectorAll"): "node-list",
    ("parent-node", "getElementsByTag"): "html-collection",
    ("parent-node", "getElementsByTagNS"): "html-collection",
    ("parent-node", "getElementsByClassName"): "html-collection",
    ("parent-node", "getElementsByName"): "node-list",
    # NonElementParentNode
    ("non-element-parent-node", "getElementById"): "element",
    # CSSRule - parent properties return CSSRule/CSSStyleSheet
    ("css-rule", "getParentRule"): "css-rule",
    ("css-rule", "getParentStyleSheet"): "css-style-sheet",
    # CSSKeyframeRule - style property
    ("css-keyframe-rule", "getStyle"): "css-style-declaration",
    # CSSStyleRule - style property
    ("css-style-rule", "getStyle"): "css-style-declaration",
    # CSSMediaRule - media property
    ("css-media-rule", "getMedia"): "media-list",
    # CSSImportRule - styleSheet/media properties
    ("css-import-rule", "getStyleSheet"): "css-style-sheet",
    ("css-import-rule", "getMedia"): "media-list",
    # CSSGroupingRule - cssRules property
    ("css-grouping-rule", "getCssRules"): "css-rule-list",
    # CSSStyleSheet - ownerRule/media/cssRules properties
    ("css-style-sheet", "getOwnerRule"): "css-rule",
    ("css-style-sheet", "getMedia"): "media-list",
    ("css-style-sheet", "getCssRules"): "css-rule-list",
    # StyleSheet - ownerNode/parentStyleSheet properties
    ("style-sheet", "getOwnerNode"): "event-target",
    ("style-sheet", "getParentStyleSheet"): "css-style-sheet",
    # Crypto - getRandomValues returns Uint8Array
    ("crypto", "getRandomValues"): "uint8-array",
    # Window - object properties
    ("window", "getWindow"): "window",
    ("window", "getSelf"): "window",
    ("window", "getDocument"): "document",
    ("window", "getName"): "string",
    ("window", "getLocation"): "location",
    ("window", "getHistory"): "history",
    ("window", "getCustomElements"): "custom-element-registry",
    ("window", "getLocationbar"): "bar-prop",
    ("window", "getMenubar"): "bar-prop",
    ("window", "getPersonalbar"): "bar-prop",
    ("window", "getScrollbars"): "bar-prop",
    ("window", "getStatusbar"): "bar-prop",
    ("window", "getToolbar"): "bar-prop",
    ("window", "getNavigator"): "navigator",
    ("window", "getScreen"): "screen",
    ("window", "getVisualViewport"): "visual-viewport",
    ("window", "getOpener"): "window",
    ("window", "getParent"): "window",
    ("window", "getFrames"): "window",
    ("window", "getTop"): "window",
    # Screen - orientation property
    ("screen", "getOrientation"): "screen-orientation",
    # Document - many object properties
    ("document", "getLocation"): "location",
    ("document", "getDomain"): "string",
    ("document", "getReferrer"): "string",
    ("document", "getCookie"): "string",
    ("document", "getLastModified"): "string",
    ("document", "getReadyState"): "string",
    ("document", "getTitle"): "string",
    ("document", "getDir"): "string",
    ("document", "getHead"): "html-element",
    ("document", "getBody"): "html-element",
    ("document", "getDocumentElement"): "element",
    ("document", "getDoctype"): "document-type",
    ("document", "getImplementation"): "dom-implementation",
    ("document", "getActiveElement"): "element",
    ("document", "getStyleSheets"): "style-sheet-list",
    ("document", "getScrollingElement"): "element",
    ("document", "getEmbeds"): "html-collection",
    ("document", "getPlugins"): "html-collection",
    ("document", "getForms"): "html-collection",
    ("document", "getLinks"): "html-collection",
    ("document", "getAnchors"): "html-collection",
    ("document", "getImages"): "html-collection",
    ("document", "getScripts"): "html-collection",
    ("document", "getDefaultView"): "window",
    ("document", "getCurrentScript"): "event-target",
    ("document", "getFirstChild"): "node",
    ("document", "getFirstChildElement"): "element",
    ("document", "getLastChild"): "node",
    ("document", "getLastChildElement"): "element",
    # Navigator - object properties
    ("navigator", "getClipboard"): "clipboard",
    ("navigator", "getCredentials"): "credentials-container",
    ("navigator", "getGeolocation"): "geolocation",
    ("navigator", "getUserActivation"): "user-activation",
    ("navigator", "getMediaCapabilities"): "media-capabilities",
    ("navigator", "getMediaDevices"): "media-devices",
    ("navigator", "getMediaSession"): "media-session",
    ("navigator", "getPermissions"): "permissions",
    ("navigator", "getServiceWorker"): "service-worker-container",
    ("navigator", "getBattery"): "battery-manager",
    ("navigator", "getGamepads"): "gamepad-list",
    ("navigator", "getLanguages"): "string-list",
    ("navigator", "getPlugins"): "plugin-array",
    ("navigator", "getMimeTypes"): "mime-type-array",
    ("navigator", "getExternal"): "external",
    ("navigator", "getSpeechSynthesis"): "speech-synthesis",
    # Element - object properties
    ("element", "getAttributes"): "named-node-map",
    ("element", "getClassList"): "dom-token-list",
    ("element", "getChildren"): "html-collection",
    ("element", "getFirstChild"): "node",
    ("element", "getFirstChildElement"): "element",
    ("element", "getLastChild"): "node",
    ("element", "getLastChildElement"): "element",
    ("element", "getNextElementSibling"): "element",
    ("element", "getPreviousElementSibling"): "element",
    ("element", "getParentElement"): "element",
    ("element", "getShadowRoot"): "shadow-root",
    ("element", "getInternals"): "element-internals",
    # HTMLElement - object properties
    ("html-element", "getDataset"): "dom-string-map",
    ("html-element", "getClassList"): "dom-token-list",
    ("html-element", "getOffsetParent"): "element",
    ("html-element", "getStyle"): "css-style-declaration",
    # HTMLImageElement
    ("html-image-element", "getComplete"): "boolean",
    ("html-image-element", "getNaturalWidth"): "number",
    ("html-image-element", "getNaturalHeight"): "number",
    ("html-image-element", "getCurrentSrc"): "string",
    # HTMLMediaElement
    ("html-media-element", "getError"): "media-error",
    ("html-media-element", "getSrcObject"): "media-provider",
    ("html-media-element", "getBuffered"): "time-ranges",
    ("html-media-element", "getSeekable"): "time-ranges",
    ("html-media-element", "getPlayed"): "time-ranges",
    ("html-media-element", "getTextTracks"): "text-track-list",
    # HTMLVideoElement
    ("html-video-element", "getVideoTracks"): "video-track-list",
    ("html-video-element", "getAudioTracks"): "audio-track-list",
    # HTMLTrackElement
    ("html-track-element", "getTrack"): "text-track",
    ("html-track-element", "getReadyState"): "number",
    # TextTrack
    ("text-track", "getCues"): "text-track-cue-list",
    ("text-track", "getActiveCues"): "text-track-cue-list",
    ("text-track", "getKind"): "string",
    ("text-track", "getLabel"): "string",
    ("text-track", "getLanguage"): "string",
    ("text-track", "getId"): "string",
    ("text-track", "getMode"): "string",
    # TextTrackList
    ("text-track-list", "getLength"): "number",
    ("text-track-list", "getTextTrack"): "text-track",
    # HTMLTableElement
    ("html-table-element", "getCaption"): "html-table-caption-element",
    ("html-table-element", "getTHead"): "html-table-section-element",
    ("html-table-element", "getTFoot"): "html-table-section-element",
    ("html-table-element", "getRows"): "html-collection",
    ("html-table-element", "getTBodies"): "html-collection",
    # HTMLFormElement
    ("html-form-element", "getElements"): "html-form-controls-collection",
    ("html-form-element", "getLength"): "number",
    # HTMLInputElement
    ("html-input-element", "getList"): "html-element",
    ("html-input-element", "getForm"): "html-form-element",
    ("html-input-element", "getLabels"): "node-list",
    ("html-input-element", "getValidity"): "validity-state",
    ("html-input-element", "getValidationMessage"): "string",
    # HTMLSelectElement
    ("html-select-element", "getForm"): "html-form-element",
    ("html-select-element", "getOptions"): "html-options-collection",
    ("html-select-element", "getSelectedOptions"): "html-collection",
    ("html-select-element", "getValidity"): "validity-state",
    # HTMLTextAreaElement
    ("html-text-area-element", "getForm"): "html-form-element",
    ("html-text-area-element", "getValidity"): "validity-state",
    # HTMLButtonElement
    ("html-button-element", "getForm"): "html-form-element",
    ("html-button-element", "getValidity"): "validity-state",
    # HTMLIFrameElement
    ("html-iframe-element", "getContentDocument"): "document",
    ("html-iframe-element", "getContentWindow"): "window",
    # HTMLObjectElement
    ("html-object-element", "getContentDocument"): "document",
    ("html-object-element", "getContentWindow"): "window",
    ("html-object-element", "getForm"): "html-form-element",
    ("html-object-element", "getValidity"): "validity-state",
    # HTMLFieldSetElement
    ("html-field-set-element", "getElements"): "html-collection",
    ("html-field-set-element", "getForm"): "html-form-element",
    # HTMLOutputElement
    ("html-output-element", "getForm"): "html-form-element",
    ("html-output-element", "getValidity"): "validity-state",
    # HTMLLabelElement
    ("html-label-element", "getControl"): "html-element",
    ("html-label-element", "getForm"): "html-form-element",
    # Range - object properties
    ("range", "getStartContainer"): "node",
    ("range", "getEndContainer"): "node",
    ("range", "getCommonAncestorContainer"): "node",
    ("range", "getCollapsed"): "boolean",
    ("range", "getBoundingClientRect"): "dom-rect",
    ("range", "getClientRects"): "dom-rect-list",
    ("range", "cloneContents"): "document-fragment",
    ("range", "createContextualFragment"): "document-fragment",
    # DOMImplementation
    ("dom-implementation", "createDocumentType"): "document-type",
    ("dom-implementation", "createDocument"): "document",
    ("dom-implementation", "createHTMLDocument"): "document",
    # DocumentType
    ("document-type", "getName"): "string",
    ("document-type", "getPublicId"): "string",
    ("document-type", "getSystemId"): "string",
    # Node - object properties
    ("node", "getParentNode"): "node",
    ("node", "getParentElement"): "element",
    ("node", "getFirstChild"): "node",
    ("node", "getLastChild"): "node",
    ("node", "getNextSibling"): "node",
    ("node", "getPreviousSibling"): "node",
    ("node", "getOwnerDocument"): "document",
    ("node", "getchildNodes"): "node-list",
    # Event - object properties
    ("event", "getTarget"): "event-target",
    ("event", "getCurrentTarget"): "event-target",
    ("event", "getSrcElement"): "event-target",
    # DocumentOrShadowRoot - object properties
    ("document-or-shadow-root", "getActiveElement"): "element",
    ("document-or-shadow-root", "getStyleSheets"): "style-sheet-list",
    ("document-or-shadow-root", "getAdoptedStyleSheets"): "css-style-sheet-list",
    # VisualViewport
    ("visual-viewport", "getOffsetLeft"): "number",
    ("visual-viewport", "getOffsetTop"): "number",
    ("visual-viewport", "getPageLeft"): "number",
    ("visual-viewport", "getPageTop"): "number",
    ("visual-viewport", "getWidth"): "number",
    ("visual-viewport", "getHeight"): "number",
    ("visual-viewport", "getScale"): "number",
    # LinkStyle
    ("link-style", "getSheet"): "css-style-sheet",
    # ImageDecoder
    ("image-decoder", "getTracks"): "image-track-list",
    ("image-decoder", "type"): "string",
    # CryptoKey - enum and object properties
    ("crypto-key", "type"): "string",
    ("crypto-key", "getAlgorithm"): "any",
    ("crypto-key", "getUsages"): "string-list",
    # Gamepad
    ("gamepad", "getButtons"): "gamepad-button-list",
    ("gamepad", "getAxes"): "float-32-list",
    ("gamepad", "getHapticActuators"): "gamepad-haptic-actuator-list",
    ("gamepad", "getVibrationActuator"): "gamepad-haptic-actuator",
    # MediaList
    ("media-list", "getLength"): "number",
    ("media-list", "getMediaText"): "string",
    # CSSPageRule
    ("css-page-rule", "getStyle"): "css-style-declaration",
    # CSSMarginRule
    ("css-margin-rule", "getStyle"): "css-style-declaration",
    # CSSFontFaceRule
    ("css-font-face-rule", "getStyle"): "css-style-declaration",
    # CSSKeyframesRule
    ("css-keyframes-rule", "getCssRules"): "css-rule-list",
    # Window
    ("window", "getClientInformation"): "navigator",
    ("window", "getExternal"): "external",
    ("window", "getSpeechSynthesis"): "speech-synthesis",
    ("window", "getEvent"): "event",
    # Document
    ("document", "getFirstChild"): "node",
    ("document", "getFirstChildElement"): "element",
    ("document", "getLastChild"): "node",
    ("document", "getLastChildElement"): "element",
    ("document", "getElementsByTag"): "html-collection",
    ("document", "getElementsByTagNS"): "html-collection",
    ("document", "getElementsByName"): "node-list",
    # Element
    ("element", "getFirstChild"): "node",
    ("element", "getFirstChildElement"): "element",
    ("element", "getLastChild"): "node",
    ("element", "getLastChildElement"): "element",
    ("element", "getNextElementSibling"): "element",
    ("element", "getPreviousElementSibling"): "element",
    ("element", "getParentElement"): "element",
    ("element", "getShadowRoot"): "shadow-root",
    ("element", "getInternals"): "element-internals",
    ("element", "getAttributes"): "named-node-map",
    ("element", "getClassList"): "dom-token-list",
    ("element", "getChildren"): "html-collection",
    ("element", "getElementsByTagName"): "html-collection",
    ("element", "getElementsByTagNameNS"): "html-collection",
    ("element", "getElementsByClassName"): "html-collection",
    # Node
    ("node", "getParentNode"): "node",
    ("node", "getParentElement"): "element",
    ("node", "getFirstChild"): "node",
    ("node", "getLastChild"): "node",
    ("node", "getNextSibling"): "node",
    ("node", "getPreviousSibling"): "node",
    ("node", "getOwnerDocument"): "document",
    ("node", "getChildNodes"): "node-list",
    # Range
    ("range", "getStartContainer"): "node",
    ("range", "getEndContainer"): "node",
    ("range", "getCommonAncestorContainer"): "node",
    ("range", "getBoundingClientRect"): "dom-rect",
    ("range", "getClientRects"): "dom-rect-list",
    ("range", "cloneContents"): "document-fragment",
    ("range", "createContextualFragment"): "document-fragment",
    # Event
    ("event", "getTarget"): "event-target",
    ("event", "getCurrentTarget"): "event-target",
    ("event", "getSrcElement"): "event-target",
    # AbortController
    ("abort-controller", "getSignal"): "abort-signal",
    # Request
    ("request", "getSignal"): "abort-signal",
    # Response
    ("response", "getHeaders"): "headers",
    # FormData
    ("form-data", "get"): "any",
    # ParentNode
    ("parent-node", "getFirstChildElement"): "element",
    ("parent-node", "getLastChildElement"): "element",
    ("parent-node", "getChildElementCount"): "number",
    # NonDocumentTypeChildNode
    ("non-document-type-child-node", "getPreviousElementSibling"): "element",
    ("non-document-type-child-node", "getNextElementSibling"): "element",
    # GeolocationCoordinates
    ("geolocation-coordinates", "getAccuracy"): "number",
    ("geolocation-coordinates", "getAltitude"): "number",
    ("geolocation-coordinates", "getAltitudeAccuracy"): "number",
    ("geolocation-coordinates", "getHeading"): "number",
    ("geolocation-coordinates", "getLatitude"): "number",
    ("geolocation-coordinates", "getLongitude"): "number",
    ("geolocation-coordinates", "getSpeed"): "number",
    # GeolocationPosition
    ("geolocation-position", "getCoords"): "geolocation-coordinates",
    # MutationRecord
    ("mutation-record", "getTarget"): "node",
    ("mutation-record", "getPreviousSibling"): "node",
    ("mutation-record", "getNextSibling"): "node",
    ("mutation-record", "getAddedNodes"): "node-list",
    ("mutation-record", "getRemovedNodes"): "node-list",
    # StyleSheet
    ("style-sheet", "getOwnerNode"): "event-target",
    ("style-sheet", "getParentStyleSheet"): "css-style-sheet",
    # CSSStyleSheet
    ("css-style-sheet", "getOwnerRule"): "css-rule",
    ("css-style-sheet", "getMedia"): "media-list",
    ("css-style-sheet", "getCssRules"): "css-rule-list",
    # CSSRule
    ("css-rule", "getParentRule"): "css-rule",
    ("css-rule", "getParentStyleSheet"): "css-style-sheet",
    # CSSImportRule
    ("css-import-rule", "getStyleSheet"): "css-style-sheet",
    ("css-import-rule", "getMedia"): "media-list",
    # CSSGroupingRule
    ("css-grouping-rule", "getCssRules"): "css-rule-list",
    # CSSMediaRule
    ("css-media-rule", "getMedia"): "media-list",
    # CSSStyleRule
    ("css-style-rule", "getStyle"): "css-style-declaration",
    # CSSKeyframeRule
    ("css-keyframe-rule", "getStyle"): "css-style-declaration",
    # Selection
    ("selection", "getAnchorNode"): "node",
    ("selection", "getFocusNode"): "node",
    ("selection", "getRangeAt"): "range",
    # Attr
    ("attr", "getOwnerElement"): "element",
    # NamedNodeMap
    ("named-node-map", "item"): "attr",
    # DOMTokenList
    ("dom-token-list", "item"): "string",
    # HTMLCollection
    ("html-collection", "item"): "element",
    ("html-collection", "namedItem"): "element",
    # HTMLAllCollection
    ("html-all-collection", "item"): "element",
    ("html-all-collection", "namedItem"): "any",
    # NodeList
    ("node-list", "item"): "node",
    # Performance
    ("performance", "getEntries"): "any",
    ("performance", "getEntriesByType"): "any",
    ("performance", "getEntriesByName"): "any",
    # PerformanceNavigationTiming
    ("performance-navigation-timing", "toJSON"): "any",
    # PerformanceTimingConfidence
    ("performance-timing-confidence", "toJSON"): "string",
    ("performance-timing-confidence", "getRandomizedTriggerRate"): "number",
    # Notification
    ("notification", "getTag"): "string",
    ("notification", "getBadge"): "string",
    ("notification", "getBody"): "string",
    ("notification", "getIcon"): "string",
    ("notification", "getImage"): "string",
    ("notification", "getLang"): "string",
    ("notification", "getRenotify"): "boolean",
    ("notification", "getRequireInteraction"): "boolean",
    ("notification", "getSilent"): "boolean",
    ("notification", "getTimestamp"): "number",
    ("notification", "getVibrate"): "number-list",
    # ScreenOrientation
    ("screen-orientation", "lock"): "promise-void",
    ("screen-orientation", "type"): "string",
    # HTML elements
    ("html-element", "getOffsetParent"): "element",
    ("html-element", "getStyle"): "css-style-declaration",
    ("html-form-element", "getElements"): "html-form-controls-collection",
    ("html-input-element", "getList"): "html-element",
    ("html-input-element", "getForm"): "html-form-element",
    ("html-input-element", "getLabels"): "node-list",
    ("html-input-element", "getValidity"): "validity-state",
    ("html-select-element", "getForm"): "html-form-element",
    ("html-select-element", "getOptions"): "html-options-collection",
    ("html-select-element", "getSelectedOptions"): "html-collection",
    ("html-select-element", "getValidity"): "validity-state",
    ("html-text-area-element", "getForm"): "html-form-element",
    ("html-text-area-element", "getValidity"): "validity-state",
    ("html-button-element", "getForm"): "html-form-element",
    ("html-button-element", "getValidity"): "validity-state",
    ("html-label-element", "getControl"): "html-element",
    ("html-label-element", "getForm"): "html-form-element",
    ("html-field-set-element", "getElements"): "html-collection",
    ("html-field-set-element", "getForm"): "html-form-element",
    ("html-output-element", "getForm"): "html-form-element",
    ("html-output-element", "getValidity"): "validity-state",
    ("html-object-element", "getContentDocument"): "document",
    ("html-object-element", "getContentWindow"): "window",
    ("html-object-element", "getForm"): "html-form-element",
    ("html-object-element", "getValidity"): "validity-state",
    ("html-iframe-element", "getContentDocument"): "document",
    ("html-iframe-element", "getContentWindow"): "window",
    ("html-table-element", "getCaption"): "html-table-caption-element",
    ("html-table-element", "getTHead"): "html-table-section-element",
    ("html-table-element", "getTFoot"): "html-table-section-element",
    ("html-table-element", "getRows"): "html-collection",
    ("html-table-element", "getTBodies"): "html-collection",
    # MediaStream
    ("media-stream", "getTracks"): "any",
    ("media-stream", "getAudioTracks"): "any",
    ("media-stream", "getVideoTracks"): "any",
    # MediaStreamTrack
    ("media-stream-track", "getCapabilities"): "any",
    ("media-stream-track", "getConstraints"): "any",
    ("media-stream-track", "getSettings"): "any",
    # MediaCapabilities
    ("media-capabilities", "decodingInfo"): "promise-any",
    ("media-capabilities", "encodingInfo"): "promise-any",
    # Permissions
    ("permissions", "query"): "promise-permission-status",
    # Animation
    ("animation", "getEffect"): "any",
    ("animation", "getTimeline"): "any",
    ("animation", "getFinished"): "promise-void",
    ("animation", "getReady"): "promise-void",
    # KeyframeEffect
    ("keyframe-effect", "getTarget"): "element",
    ("keyframe-effect", "getComposite"): "string",
    ("keyframe-effect", "getIterationComposite"): "string",
    # RTC
    ("rtc-peer-connection", "getLocalDescription"): "any",
    ("rtc-peer-connection", "getRemoteDescription"): "any",
    ("rtc-peer-connection", "getCurrentLocalDescription"): "any",
    ("rtc-peer-connection", "getPendingLocalDescription"): "any",
    ("rtc-peer-connection", "getCurrentRemoteDescription"): "any",
    ("rtc-peer-connection", "getPendingRemoteDescription"): "any",
    ("rtc-peer-connection", "getSignalingState"): "string",
    ("rtc-peer-connection", "getIceConnectionState"): "string",
    ("rtc-peer-connection", "getIceGatheringState"): "string",
    ("rtc-peer-connection", "getConnectionState"): "string",
    # HTMLHyperlinkElementUtils
    ("html-hyperlink-element-utils", "getHref"): "string",
    # MediaSession
    ("media-session", "getMetadata"): "any",
    ("media-session", "getPlaybackState"): "string",
    # ServiceWorkerContainer
    ("service-worker-container", "getController"): "any",
    ("service-worker-container", "getReady"): "promise-any",
    # WebAssembly
    ("instance", "getExports"): "any",
    ("table", "get"): "any",
    ("global", "getValue"): "any",
    # Observers
    ("intersection-observer", "getRoot"): "event-target",
    ("intersection-observer", "getThresholds"): "number-list",
    ("mutation-observer", "getRecords"): "any",
    # WebSocket
    ("ws", "getUrl"): "string",
    ("ws", "getProtocol"): "string",
    ("ws", "getExtensions"): "string",
    # Clipboard
    ("clipboard", "read"): "promise-any",
    ("clipboard", "readText"): "promise-string",
    # File
    ("file", "getName"): "string",
    ("file", "getType"): "string",
    ("file", "getLastModified"): "number",
    # FileReader
    ("file-reader", "getResult"): "any",
    ("file-reader", "getReadyState"): "number",
    # AbortSignal static methods
    ("abort-signal", "abort"): "abort-signal",
    ("abort-signal", "timeout"): "abort-signal",
    ("abort-signal", "any"): "abort-signal",
    # Window getters
    ("window", "getFrameElement"): "element",
    # Document getters
    ("document", "getElementsByTagNS"): "html-collection",
    # Element getters
    ("element", "getElementsByTagNS"): "html-collection",
    # Node getters
    ("node", "getFirstChild"): "node",
    ("node", "getLastChild"): "node",
    ("node", "getNextSibling"): "node",
    ("node", "getPreviousSibling"): "node",
    ("node", "getParentNode"): "node",
    ("node", "getParentElement"): "element",
    ("node", "getOwnerDocument"): "document",
    ("node", "getChildNodes"): "node-list",
    # Range getters
    ("range", "getStartContainer"): "node",
    ("range", "getEndContainer"): "node",
    ("range", "getCommonAncestorContainer"): "node",
    ("range", "cloneContents"): "document-fragment",
    ("range", "createContextualFragment"): "document-fragment",
    # Event getters
    ("event", "getTarget"): "event-target",
    ("event", "getCurrentTarget"): "event-target",
    ("event", "composedPath"): "event-target-list",
    # Attr getters
    ("attr", "getOwnerElement"): "element",
    # Selection getters
    ("selection", "getAnchorNode"): "node",
    ("selection", "getFocusNode"): "node",
    ("selection", "getRangeAt"): "range",
    # MutationObserver
    ("mutation-observer", "takeRecords"): "mutation-record-list",
    # XPath
    ("document", "createExpression"): "xpath-expression",
    ("document", "createNSResolver"): "xpath-ns-resolver",
    ("document", "evaluate"): "xpath-result",
    ("xpath-expression", "evaluate"): "xpath-result",
    # Gamepad
    ("gamepad", "getHapticActuators"): "gamepad-haptic-actuator-list",
    ("gamepad", "getVibrationActuator"): "gamepad-haptic-actuator",
    # Touch
    ("touch-event", "getTouches"): "touch-list",
    ("touch-event", "getTargetTouches"): "touch-list",
    ("touch-event", "getChangedTouches"): "touch-list",
    ("touch-event", "getTouch"): "touch",
    # PointerEvent
    ("pointer-event", "getCoalescedEvents"): "pointer-event-list",
    ("pointer-event", "getPredictedEvents"): "pointer-event-list",
    # DragEvent
    ("drag-event", "getDataTransfer"): "data-transfer",
    # Fetch API
    ("window", "getPerformance"): "performance",
    ("window", "getCacheStorage"): "cache-storage",
    ("window", "getCrypto"): "crypto",
    ("request", "getSignal"): "abort-signal",
    ("response", "getHeaders"): "headers",
    ("response", "getBody"): "readable-stream",
    ("headers", "get"): "string",
    # Streams
    ("readable-stream", "getReader"): "any",
    ("readable-stream", "getByobRequest"): "readable-stream-byob-request",
    ("readable-stream", "tee"): "readable-stream-pair",
    ("writable-stream", "getWriter"): "any",
    ("readable-stream-byob-reader", "read"): "any",
    ("readable-stream-default-reader", "read"): "any",
    # Storage
    ("window", "getLocalStorage"): "storage",
    ("window", "getSessionStorage"): "storage",
    # CSS
    ("css-style-sheet", "getMedia"): "media-list",
    ("css-style-sheet", "getCssRules"): "css-rule-list",
    ("css-style-sheet", "getOwnerRule"): "css-rule",
    ("css-rule", "getParentRule"): "css-rule",
    ("css-rule", "getParentStyleSheet"): "css-style-sheet",
    ("css-import-rule", "getStyleSheet"): "css-style-sheet",
    ("css-import-rule", "getMedia"): "media-list",
    ("css-grouping-rule", "getCssRules"): "css-rule-list",
    ("css-media-rule", "getMedia"): "media-list",
    ("css-style-rule", "getStyle"): "css-style-declaration",
    ("css-keyframe-rule", "getStyle"): "css-style-declaration",
    ("css-keyframes-rule", "getCssRules"): "css-rule-list",
    ("css-page-rule", "getStyle"): "css-style-declaration",
    ("css-margin-rule", "getStyle"): "css-style-declaration",
    ("css-font-face-rule", "getStyle"): "css-style-declaration",
    ("media-list", "item"): "string",
    # HTML elements
    ("html-element", "getDataset"): "dom-string-map",
    ("html-element", "getClassList"): "dom-token-list",
    ("html-element", "getStyle"): "css-style-declaration",
    ("html-element", "getOffsetParent"): "element",
    ("html-slot-element", "getAssignedElements"): "element-list",
    ("html-iframe-element", "getContentDocument"): "document",
    ("html-iframe-element", "getContentWindow"): "window",
    ("html-object-element", "getContentDocument"): "document",
    ("html-object-element", "getContentWindow"): "window",
    # XMLHttpRequest
    ("xml-http-request", "getUpload"): "xml-http-request-upload",
    ("xml-http-request", "getResponseXML"): "document",
    # DOMRect
    ("range", "getBoundingClientRect"): "dom-rect",
    ("range", "getClientRects"): "dom-rect-list",
    ("element", "getBoundingClientRect"): "dom-rect",
    ("element", "getClientRects"): "dom-rect-list",
    # ShadowRoot
    ("shadow-root", "getHost"): "element",
    # TreeWalker
    ("tree-walker", "getRoot"): "node",
    ("tree-walker", "getCurrentNode"): "node",
    ("tree-walker", "parentNode"): "node",
    ("tree-walker", "firstChild"): "node",
    ("tree-walker", "lastChild"): "node",
    ("tree-walker", "nextSibling"): "node",
    ("tree-walker", "previousSibling"): "node",
    ("tree-walker", "nextNode"): "node",
    ("tree-walker", "previousNode"): "node",
    ("tree-walker", "getFilter"): "node-filter",
    # NodeIterator
    ("node-iterator", "getRoot"): "node",
    ("node-iterator", "getReferenceNode"): "node",
    ("node-iterator", "getFilter"): "node-filter",
    ("node-iterator", "nextNode"): "node",
    ("node-iterator", "previousNode"): "node",
    # Node methods that return Node
    ("node", "cloneNode"): "node",
    ("node", "insertBefore"): "node",
    ("node", "appendChild"): "node",
    ("node", "replaceChild"): "node",
    ("node", "removeChild"): "node",
    # ParentNode
    ("parent-node", "getChildren"): "html-collection",
    # AbstractRange
    ("abstract-range", "getStartContainer"): "node",
    ("abstract-range", "getEndContainer"): "node",
    # NamedNodeMap
    ("named-node-map", "getNamedItemNs"): "attr",
    ("named-node-map", "getNamedItem"): "attr",
    ("named-node-map", "setNamedItem"): "attr",
    ("named-node-map", "setNamedItemNs"): "attr",
    ("named-node-map", "removeNamedItem"): "attr",
    ("named-node-map", "removeNamedItemNs"): "attr",
    # XPathResult
    ("xpath-result", "getSingleNodeValue"): "node",
    ("xpath-result", "iterateNext"): "node",
    ("xpath-result", "snapshotItem"): "node",
    ("x-path-result", "getSingleNodeValue"): "node",
    ("x-path-result", "iterateNext"): "node",
    ("x-path-result", "snapshotItem"): "node",
    # XPathEvaluatorBase
    ("xpath-evaluator-base", "createNsResolver"): "xpath-ns-resolver",
    ("xpath-evaluator-base", "evaluate"): "xpath-result",
    ("x-path-evaluator-base", "createNsResolver"): "xpath-ns-resolver",
    ("x-path-evaluator-base", "evaluate"): "xpath-result",
    # CaretPosition
    ("caret-position", "getOffsetNode"): "node",
    ("caret-position", "getClientRect"): "dom-rect",
    # Document - additional methods
    ("document", "getApplets"): "html-collection",
    ("document", "getElementsByTagNameNs"): "html-collection",
    ("document", "createElementNs"): "element",
    ("document", "createAttributeNs"): "attr",
    # Element - additional methods
    ("element", "getAttributeNodeNs"): "attr",
    ("element", "getElementsByTagNameNs"): "html-collection",
    # HTMLElement
    ("html-element", "attachInternals"): "element-internals",
    # StyleSheet
    ("style-sheet", "getMedia"): "media-list",
    # CSSStyleSheet
    ("css-style-sheet", "getRules"): "css-rule-list",
    # StyleSheetList
    ("style-sheet-list", "item"): "css-style-sheet",
    # HTMLMapElement
    ("html-map-element", "getAreas"): "html-collection",
    # HTMLTableSectionElement
    ("html-table-section-element", "getRows"): "html-collection",
    # HTMLTableRowElement
    ("html-table-row-element", "getCells"): "html-collection",
    # HTMLDataListElement
    ("html-data-list-element", "getOptions"): "html-collection",
    # Labels getters (return NodeList)
    ("html-button-element", "getLabels"): "node-list",
    ("html-select-element", "getLabels"): "node-list",
    ("html-text-area-element", "getLabels"): "node-list",
    ("html-output-element", "getLabels"): "node-list",
    ("html-progress-element", "getLabels"): "node-list",
    ("html-meter-element", "getLabels"): "node-list",
    ("element-internals", "getLabels"): "node-list",
    # SubmitEvent
    ("submit-event", "getSubmitter"): "html-element",
    # HTMLSlotElement
    ("html-slot-element", "assignedNodes"): "node-list",
    ("html-slot-element", "assignedElements"): "element-list",
    # IntersectionObserverEntry
    ("intersection-observer-entry", "getRootBounds"): "dom-rect-read-only",
    ("intersection-observer-entry", "getBoundingClientRect"): "dom-rect-read-only",
    ("intersection-observer-entry", "getIntersectionRect"): "dom-rect-read-only",
    # ResizeObserverEntry
    ("resize-observer-entry", "getContentRect"): "dom-rect-read-only",
    # Window/Document global getters
    ("window-or-worker-global-scope", "getPerformance"): "performance",
    ("window-or-worker-global-scope", "getCaches"): "cache-storage",
    ("window-or-worker-global-scope", "getCrypto"): "crypto",
    # Request methods
    ("request", "getHeaders"): "headers",
    ("request", "clone"): "request",
    # Response static methods
    ("response", "error"): "response",
    ("response", "redirect"): "response",
    ("response", "json"): "response",
    ("response", "clone"): "response",
    # DOM methods
    ("dom-implementation", "createHtmlDocument"): "document",
    ("text", "splitText"): "text",
    ("x-path-expression", "evaluate"): "xpath-result",
    ("x-path-evaluator-base", "createExpression"): "xpath-expression",
    ("x-slt-processor", "transformToFragment"): "document-fragment",
    ("x-slt-processor", "transformToDocument"): "document",
    # Range methods
    ("range", "extractContents"): "document-fragment",
    ("range", "cloneRange"): "range",
    # ElementCSSInlineStyle
    ("element-css-inline-style", "getStyle"): "css-style-declaration",
    # GamepadEvent
    ("gamepad-event", "getGamepad"): "gamepad",
    # Touch
    ("touch", "getTarget"): "event-target",
    # TouchEvent
    ("touch-event", "getView"): "window",
    # ReadableStream
    ("readable-stream", "pipeThrough"): "readable-stream",
    # TransformStream
    ("transform-stream", "getReadable"): "readable-stream",
    ("transform-stream", "getWritable"): "writable-stream",
    # GenericTransformStream
    ("generic-transform-stream", "getReadable"): "readable-stream",
    ("generic-transform-stream", "getWritable"): "writable-stream",
    # WritableStreamDefaultController
    ("writable-stream-default-controller", "getSignal"): "abort-signal",
    # HTMLOrSVGElement
    ("html-or-svg-element", "getDataset"): "dom-string-map",
    # HTMLLinkElement
    ("html-link-element", "getRelList"): "dom-token-list",
    ("html-link-element", "getSizes"): "dom-token-list",
    ("html-link-element", "getBlocking"): "dom-token-list",
    # Document global getters
    ("document", "getNavigation"): "navigator",
    ("document", "createCdataSection"): "c-data-section",
    ("document", "getAll"): "html-all-collection",
    # HTMLStyleElement
    ("html-style-element", "getBlocking"): "dom-token-list",
    # HTMLAnchorElement
    ("html-anchor-element", "getRelList"): "dom-token-list",
    # HTMLIFrameElement sandbox
    ("htmli-frame-element", "getSandbox"): "dom-token-list",
    # HTMLAreaElement
    ("html-area-element", "getRelList"): "dom-token-list",
    # HTMLFormElement
    ("html-form-element", "getRelList"): "dom-token-list",
    # HTMLOutputElement htmlFor
    ("html-output-element", "getHtmlFor"): "dom-token-list",
    # HTMLScriptElement
    ("html-script-element", "getBlocking"): "dom-token-list",
    # XSLTProcessor
    ("xslt-processor", "transformToFragment"): "document-fragment",
    ("xslt-processor", "transformToDocument"): "document",
    # MessageChannel
    ("message-channel", "getPort1"): "message-port",
    ("message-channel", "getPort2"): "message-port",
    # CanvasRenderingContext2D
    ("canvas-rendering-context2-d", "createLinearGradient"): "canvas-gradient",
    ("canvas-rendering-context2-d", "createRadialGradient"): "canvas-gradient",
    ("canvas-rendering-context2-d", "createConicGradient"): "canvas-gradient",
    # HTMLTableElement
    ("html-table-element", "getCaption"): "html-table-caption-element",
    ("html-table-element", "getTHead"): "html-table-section-element",
    ("html-table-element", "getTFoot"): "html-table-section-element",
    # HTMLMediaElement
    ("html-media-element", "getTextTrack"): "text-track",
    # ValidityState
    ("html-element", "getValidity"): "validity-state",
    ("html-input-element", "getValidity"): "validity-state",
    ("html-text-area-element", "getValidity"): "validity-state",
    ("html-select-element", "getValidity"): "validity-state",
    ("html-button-element", "getValidity"): "validity-state",
    # Storage
    ("window", "getLocalStorage"): "storage",
    ("window", "getSessionStorage"): "storage",
    # ViewTransition
    ("document", "startViewTransition"): "view-transition",
    # MimeTypeArray
    ("navigator", "getMimeTypes"): "mime-type-array",
    # IntersectionObserver
    ("intersection-observer", "getRoot"): "event-target",
    # Window - getNavigation returns Navigator
    ("window", "getNavigation"): "navigator",
    # Element - attribute getters that return Attr
    ("element", "getAttributeNode"): "attr",
    ("element", "getAttributeNodeNs"): "attr",
    # MouseEvent - relatedTarget
    ("mouse-event", "getRelatedTarget"): "event-target",
    # Event - target getters
    ("event", "getTarget"): "event-target",
    ("event", "getCurrentTarget"): "event-target",
    ("event", "getSrcElement"): "event-target",
    # DocumentOrShadowRoot - element getters
    ("document-or-shadow-root", "getFullscreenElement"): "element",
    ("document-or-shadow-root", "getActiveElement"): "element",
    ("document-or-shadow-root", "getPointerLockElement"): "element",
    ("document-or-shadow-root", "getStyleSheetList"): "css-style-sheet-list",
    # CSSRule - parent getters
    ("css-rule", "getParentRule"): "css-rule",
    ("css-rule", "getParentStyleSheet"): "css-style-sheet",
    # Body - getBody
    ("body", "getBody"): "readable-stream",
    # ReadableByteStreamController
    ("readable-byte-stream-controller", "getByobRequest"): "readable-stream-byob-request",
    # XMLHttpRequest
    ("xml-http-request", "getResponseXml"): "document",
    # ClipboardEvent
    ("clipboard-event", "getClipboardData"): "data-transfer",
    # Touch
    ("touch", "getTarget"): "event-target",
    # TouchList - item
    ("touch-list", "item"): "touch",
    # UIEvent - view
    ("ui-event", "getView"): "window",
    # DragEvent
    ("drag-event", "getDataTransfer"): "data-transfer",
    # PointerEvent - getCoalescedEvents/getPredictedEvents
    ("pointer-event", "getCoalescedEvents"): "pointer-event-list",
    ("pointer-event", "getPredictedEvents"): "pointer-event-list",
    # Event - composedPath
    ("event", "composedPath"): "event-target-list",
    # HTMLElement - offsetParent
    ("html-element", "getOffsetParent"): "element",
    # HTMLIFrameElement
    ("html-iframe-element", "getContentDocument"): "document",
    ("html-iframe-element", "getContentWindow"): "window",
    # HTMLObjectElement
    ("html-object-element", "getContentDocument"): "document",
    ("html-object-element", "getContentWindow"): "window",
    # HTMLSlotElement
    ("html-slot-element", "getAssignedElements"): "element-list",
    ("html-slot-element", "assignedElements"): "element-list",
    ("html-slot-element", "assignedNodes"): "node-list",
    # TextTrackList
    ("text-track-list", "item"): "text-track",
    ("text-track-list", "getTextTrack"): "text-track",
    # TextTrackCueList
    ("text-track-cue-list", "item"): "text-track-cue",
    ("text-track-cue-list", "getCueById"): "text-track-cue",
    # TextTrack
    ("text-track", "getCues"): "text-track-cue-list",
    ("text-track", "getActiveCues"): "text-track-cue-list",
    # DataTransferItemList
    ("data-transfer-item-list", "item"): "data-transfer-item",
    # DataTransfer
    ("data-transfer", "getItems"): "data-transfer-item-list",
    ("data-transfer", "getFiles"): "file-list",
    # FileList
    ("file-list", "item"): "file",
    # Selection
    ("selection", "getAnchorNode"): "node",
    ("selection", "getFocusNode"): "node",
    ("selection", "getRangeAt"): "range",
    # Node
    ("node", "getRootNode"): "node",
    ("node", "getParentNode"): "node",
    ("node", "getParentElement"): "element",
    ("node", "getFirstChild"): "node",
    ("node", "getLastChild"): "node",
    ("node", "getNextSibling"): "node",
    ("node", "getPreviousSibling"): "node",
    ("node", "getOwnerDocument"): "document",
    ("node", "getChildNodes"): "node-list",
    ("node", "cloneNode"): "node",
    ("node", "insertBefore"): "node",
    ("node", "appendChild"): "node",
    ("node", "replaceChild"): "node",
    ("node", "removeChild"): "node",
    # Attr
    ("attr", "getOwnerElement"): "element",
    # Element
    ("element", "closest"): "element",
    ("element", "matches"): "boolean",
    ("element", "webkitMatchesSelector"): "boolean",
    ("element", "getElementsByTagName"): "html-collection",
    ("element", "getElementsByTagNameNs"): "html-collection",
    ("element", "getElementsByClassName"): "html-collection",
    ("element", "querySelector"): "element",
    ("element", "querySelectorAll"): "node-list",
    ("element", "getBoundingClientRect"): "dom-rect",
    ("element", "getClientRects"): "dom-rect-list",
    ("element", "getAttribute"): "string",
    ("element", "getAttributeNs"): "string",
    ("element", "getAttributeNames"): "string-list",
    ("element", "hasAttribute"): "boolean",
    ("element", "hasAttributeNs"): "boolean",
    ("element", "hasAttributes"): "boolean",
    ("element", "getShadowRoot"): "shadow-root",
    ("element", "getInternals"): "element-internals",
    # Document
    ("document", "getElementById"): "element",
    ("document", "getElementsByTagName"): "html-collection",
    ("document", "getElementsByTagNameNs"): "html-collection",
    ("document", "getElementsByClassName"): "html-collection",
    ("document", "getElementsByName"): "node-list",
    ("document", "querySelector"): "element",
    ("document", "querySelectorAll"): "node-list",
    ("document", "createElement"): "element",
    ("document", "createElementNs"): "element",
    ("document", "createDocumentFragment"): "document-fragment",
    ("document", "createTextNode"): "text",
    ("document", "createComment"): "comment",
    ("document", "createAttribute"): "attr",
    ("document", "createAttributeNs"): "attr",
    ("document", "createRange"): "range",
    ("document", "createNodeIterator"): "node-iterator",
    ("document", "createTreeWalker"): "tree-walker",
    ("document", "importNode"): "node",
    ("document", "adoptNode"): "node",
    ("document", "getSelection"): "selection",
    ("document", "getElementsByTag"): "html-collection",
    ("document", "getElementsByTagNs"): "html-collection",
    ("document", "getBody"): "html-element",
    ("document", "getHead"): "html-element",
    ("document", "getDocumentElement"): "element",
    ("document", "getDoctype"): "document-type",
    ("document", "getImplementation"): "dom-implementation",
    ("document", "getActiveElement"): "element",
    ("document", "getStyleSheets"): "style-sheet-list",
    ("document", "getScrollingElement"): "element",
    # DOMRectList
    ("dom-rect-list", "item"): "dom-rect",
    # DOMTokenList
    ("dom-token-list", "item"): "string",
    # HTMLCollection
    ("html-collection", "item"): "element",
    ("html-collection", "namedItem"): "element",
    # NodeList
    ("node-list", "item"): "node",
    # NamedNodeMap
    ("named-node-map", "item"): "attr",
    ("named-node-map", "getNamedItem"): "attr",
    ("named-node-map", "getNamedItemNs"): "attr",
    # Range
    ("range", "getStartContainer"): "node",
    ("range", "getEndContainer"): "node",
    ("range", "getCommonAncestorContainer"): "node",
    ("range", "getBoundingClientRect"): "dom-rect",
    ("range", "getClientRects"): "dom-rect-list",
    ("range", "cloneContents"): "document-fragment",
    ("range", "createContextualFragment"): "document-fragment",
    ("range", "extractContents"): "document-fragment",
    ("range", "cloneRange"): "range",
    # ShadowRoot
    ("shadow-root", "getHost"): "element",
    # TreeWalker
    ("tree-walker", "getRoot"): "node",
    ("tree-walker", "getCurrentNode"): "node",
    ("tree-walker", "parentNode"): "node",
    ("tree-walker", "firstChild"): "node",
    ("tree-walker", "lastChild"): "node",
    ("tree-walker", "nextSibling"): "node",
    ("tree-walker", "previousSibling"): "node",
    ("tree-walker", "nextNode"): "node",
    ("tree-walker", "previousNode"): "node",
    ("tree-walker", "getFilter"): "node-filter",
    # NodeIterator
    ("node-iterator", "getRoot"): "node",
    ("node-iterator", "getReferenceNode"): "node",
    ("node-iterator", "getFilter"): "node-filter",
    ("node-iterator", "nextNode"): "node",
    ("node-iterator", "previousNode"): "node",
    # Canvas
    ("canvas-rendering-context", "createLinearGradient"): "canvas-gradient",
    ("canvas-rendering-context", "createRadialGradient"): "canvas-gradient",
    ("canvas-rendering-context", "createConicGradient"): "canvas-gradient",
    ("canvas-rendering-context", "createPattern"): "canvas-pattern",
    ("canvas-rendering-context", "createImageData"): "image-data",
    ("canvas-rendering-context", "getImageData"): "image-data",
    ("canvas-rendering-context", "measureText"): "text-metrics",
    ("canvas-rendering-context", "getLineDash"): "float-32-list",
    ("canvas-rendering-context", "getTransform"): "dom-matrix",
    ("canvas-rendering-context", "getCanvas"): "any",
    # OffscreenCanvas
    ("offscreencanvas", "getContext"): "any",
    ("offscreencanvas", "transferToImageBitmap"): "image-bitmap",
    # HTMLCanvasElement
    ("html-canvas-element", "getContext"): "any",
    # DOMParser
    ("dom-parser", "parseFromString"): "document",
    # IntersectionObserverEntry
    ("intersection-observer-entry", "getTarget"): "element",
    ("intersection-observer-entry", "target"): "element",
    # HTMLMediaElement methods
    ("html-media-element", "addTextTrack"): "text-track",
    # HTMLTableElement methods
    ("html-table-element", "createCaption"): "html-table-caption-element",
    ("html-table-element", "createTHead"): "html-table-section-element",
    ("html-table-element", "createTFoot"): "html-table-section-element",
    ("html-table-element", "createTBody"): "html-table-section-element",
    ("html-table-element", "insertRow"): "html-table-row-element",
    # HTMLTableSectionElement methods
    ("html-table-section-element", "insertRow"): "html-table-row-element",
    # HTMLTableRowElement methods
    ("html-table-row-element", "insertCell"): "html-table-cell-element",
    # HTMLFieldSetElement/HTMLObjectElement validity
    ("html-field-set-element", "getValidity"): "validity-state",
    ("html-object-element", "getValidity"): "validity-state",
    # HTMLElement validity (for elements with validation)
    ("html-element", "getValidity"): "validity-state",
    # HTMLFormElement methods
    ("html-form-element", "getElements"): "html-form-controls-collection",
    # HTMLInputElement validity
    ("html-input-element", "getValidity"): "validity-state",
    # HTMLTextAreaElement validity
    ("html-text-area-element", "getValidity"): "validity-state",
    # HTMLSelectElement validity
    ("html-select-element", "getValidity"): "validity-state",
    # HTMLButtonElement validity
    ("html-button-element", "getValidity"): "validity-state",
    # HTMLOutputElement validity
    ("html-output-element", "getValidity"): "validity-state",
    # ElementInternals
    ("html-element", "getInternals"): "element-internals",
    ("element-internals", "getValidity"): "validity-state",
    ("element-internals", "getForm"): "html-form-element",
    ("element-internals", "getCustomStateSet"): "custom-state-set",
    # Canvas methods - using mixin interface names from WIT
    ("html-canvas-element", "transferControlToOffscreen"): "offscreencanvas",
    ("canvas-fill-stroke-styles", "createLinearGradient"): "canvas-gradient",
    ("canvas-fill-stroke-styles", "createRadialGradient"): "canvas-gradient",
    ("canvas-fill-stroke-styles", "createConicGradient"): "canvas-gradient",
    ("canvas-fill-stroke-styles", "createPattern"): "canvas-pattern",
    ("canvas-image-data", "createImageData"): "image-data",
    ("canvas-image-data", "getImageData"): "image-data",
    ("canvas-text", "measureText"): "text-metrics",
    ("canvas-path-drawing-styles", "getLineDash"): "float-32-list",
    ("canvas-transform", "getTransform"): "dom-matrix",
    ("canvas-state", "getCanvas"): "any",
    ("offscreen-canvas-fill-stroke-styles", "createLinearGradient"): "canvas-gradient",
    ("offscreen-canvas-fill-stroke-styles", "createRadialGradient"): "canvas-gradient",
    ("offscreen-canvas-fill-stroke-styles", "createConicGradient"): "canvas-gradient",
    ("offscreen-canvas-fill-stroke-styles", "createPattern"): "canvas-pattern",
    ("offscreen-canvas-image-data", "createImageData"): "image-data",
    ("offscreen-canvas-image-data", "getImageData"): "image-data",
    ("offscreen-canvas-text", "measureText"): "text-metrics",
    ("offscreen-canvas-path-drawing-styles", "getLineDash"): "float-32-list",
    ("offscreen-canvas-transform", "getTransform"): "dom-matrix",
    ("offscreen-canvas-state", "getCanvas"): "any",
    # Canvas settings
    ("canvas-settings", "getContextAttributes"): "any",
    # OffscreenCanvas methods
    ("offscreencanvas", "getContext"): "any",
    ("offscreencanvas", "transferToImageBitmap"): "image-bitmap",
    # ResizeObserverEntry
    ("resize-observer-entry", "getContentRect"): "dom-rect-read-only",
    ("resize-observer-entry", "getBorderBoxSize"): "resize-observer-size-list",
    ("resize-observer-entry", "getContentBoxSize"): "resize-observer-size-list",
    ("resize-observer-entry", "getDevicePixelContentBoxSize"): "resize-observer-size-list",
    # Navigation
    ("navigation", "getCurrentEntry"): "navigation-history-entry",
    ("navigation", "getActivation"): "navigation-activation",
    ("navigation", "getTransition"): "view-transition",
    # Navigator plugins
    ("navigator", "getPlugins"): "plugin-array",
    ("navigator", "getMimeTypes"): "mime-type-array",
    # PluginArray/MimeTypeArray
    ("plugin-array", "item"): "plugin",
    ("plugin-array", "namedItem"): "plugin",
    ("mime-type-array", "item"): "mime-type",
    ("mime-type-array", "namedItem"): "mime-type",
    # Plugin
    ("plugin", "item"): "mime-type",
    ("plugin", "namedItem"): "mime-type",
    # MessageChannel
    ("message-channel", "getPort1"): "message-port",
    ("message-channel", "getPort2"): "message-port",
    # Window storage
    ("window", "getLocalStorage"): "storage",
    ("window", "getSessionStorage"): "storage",
    # Document/Window for iframes
    ("htmli-frame-element", "getContentDocument"): "document",
    ("htmli-frame-element", "getContentWindow"): "window",
    ("html-object-element", "getContentDocument"): "document",
    ("html-object-element", "getContentWindow"): "window",
    # HTMLSelectElement
    ("html-select-element", "getOptions"): "html-options-collection",
    ("html-select-element", "getSelectedOptions"): "html-collection",
    # HTMLOptionsCollection
    ("html-options-collection", "item"): "html-option-element",
    ("html-options-collection", "namedItem"): "html-option-element",
    # HTMLFormElement
    ("html-form-element", "item"): "element",
    ("html-form-element", "namedItem"): "element",
    # DataTransfer
    ("data-transfer", "getItems"): "data-transfer-item-list",
    ("data-transfer", "getFiles"): "file-list",
    ("data-transfer-item-list", "item"): "data-transfer-item",
    # FileList
    ("file-list", "item"): "file",
    # MediaStream methods
    ("media-stream", "getTrackById"): "media-stream-track",
    ("media-stream-track", "getCapabilities"): "any",
    ("media-stream-track", "getConstraints"): "any",
    ("media-stream-track", "getSettings"): "any",
    # MediaRecorder
    ("media-recorder", "getStream"): "media-stream",
    # Blob
    ("blob", "slice"): "blob",
    # SpeechRecognition
    ("speech-recognition-result-list", "item"): "speech-recognition-result",
    ("speech-recognition-result", "item"): "speech-recognition-alternative",
    # SpeechSynthesis
    ("speech-synthesis", "getVoices"): "speech-synthesis-voice-list",
    ("speech-synthesis-voice-list", "item"): "speech-synthesis-voice",
    # ServiceWorker
    ("service-worker-container", "getController"): "service-worker",
    ("service-worker-container", "getReady"): "service-worker-registration",
    ("service-worker-registration", "getActive"): "service-worker",
    ("service-worker-registration", "getWaiting"): "service-worker",
    ("service-worker-registration", "getInstalling"): "service-worker",
    ("service-worker-registration", "getNavigationPreload"): "navigation-preload-manager",
    # Performance
    ("performance", "getTiming"): "performance-timing",
    ("performance", "getNavigation"): "performance-navigation",
    ("performance", "mark"): "performance-mark",
    ("performance", "measure"): "performance-measure",
    # StorageManager
    ("navigator", "getStorage"): "storage-manager",
    # RTC
    ("rtc-rtp-sender", "getTrack"): "media-stream-track",
    ("rtc-rtp-sender", "getTransport"): "rtc-dtls-transport",
    ("rtc-rtp-receiver", "getTrack"): "media-stream-track",
    ("rtc-rtp-receiver", "getTransport"): "rtc-dtls-transport",
    ("rtc-rtp-transceiver", "getSender"): "rtc-rtp-sender",
    ("rtc-rtp-transceiver", "getReceiver"): "rtc-rtp-receiver",
    ("rtc-data-channel", "getTransport"): "rtc-sctp-transport",
    # PaymentRequest
    ("payment-response", "getShippingAddress"): "payment-address",
    ("payment-request", "getShippingAddress"): "payment-address",
    # FormData
    ("html-form-element", "getFormData"): "form-data",
    ("form-data-event", "getFormData"): "form-data",
    # IntersectionObserver
    ("intersection-observer", "getRoot"): "event-target",
    # ResizeObserverEntry target
    ("resize-observer-entry", "getTarget"): "element",
    # ResizeObserverSize
    ("resize-observer-entry", "getContentBoxSize"): "resize-observer-size-list",
    ("resize-observer-entry", "getBorderBoxSize"): "resize-observer-size-list",
    # DocumentFragment for templates
    ("html-template-element", "getContent"): "document-fragment",
    # ValidityState for HTML elements
    ("html-element", "getValidity"): "validity-state",
    # FormDataEvent
    ("form-data-event", "getFormData"): "form-data",
    # SubmitEvent
    ("submit-event", "getSubmitter"): "html-element",
    # Navigator
    ("navigator", "getPlugins"): "plugin-array",
    ("navigator", "getMimeTypes"): "mime-type-array",
    # PluginArray/MimeTypeArray
    ("plugin-array", "item"): "plugin",
    ("plugin-array", "namedItem"): "plugin",
    ("mime-type-array", "item"): "mime-type",
    ("mime-type-array", "namedItem"): "mime-type",
    # Plugin
    ("plugin", "item"): "mime-type",
    ("plugin", "namedItem"): "mime-type",
    # Navigator storage
    ("navigator", "getStorage"): "storage-manager",
    # MessageChannel
    ("message-channel", "getPort1"): "message-port",
    ("message-channel", "getPort2"): "message-port",
    # Window storage
    ("window", "getLocalStorage"): "storage",
    ("window", "getSessionStorage"): "storage",
    # MessageEvent source
    ("message-event", "getSource"): "message-event-source",
    # Canvas/ImageBitmap
    ("image-bitmap-context", "getCanvas"): "any",
    ("offscreencanvas", "transferToImageBitmap"): "image-bitmap",
    # CustomStateSet
    ("element-internals", "getCustomStateSet"): "custom-state-set",
    # Navigation
    ("navigation", "getCurrentEntry"): "navigation-history-entry",
    ("navigation", "getActivation"): "navigation-activation",
    ("navigation", "getTransition"): "view-transition",
    # MediaStream
    ("media-stream", "getTrackById"): "media-stream-track",
    ("media-stream", "getTracks"): "any",
    ("media-stream", "getAudioTracks"): "any",
    ("media-stream", "getVideoTracks"): "any",
    # MediaStreamTrack
    ("media-stream-track", "getCapabilities"): "any",
    ("media-stream-track", "getConstraints"): "any",
    ("media-stream-track", "getSettings"): "any",
    # MediaDevices
    ("media-devices", "enumerateDevices"): "any",
    # MediaRecorder
    ("media-recorder", "getStream"): "media-stream",
    # Blob
    ("blob", "slice"): "blob",
    # SpeechRecognition
    ("speech-recognition-result-list", "item"): "speech-recognition-result",
    ("speech-recognition-result", "item"): "speech-recognition-alternative",
    # SpeechSynthesis
    ("speech-synthesis", "getVoices"): "speech-synthesis-voice-list",
    ("speech-synthesis-voice-list", "item"): "speech-synthesis-voice",
    # ServiceWorker
    ("service-worker-container", "getController"): "service-worker",
    ("service-worker-container", "getReady"): "service-worker-registration",
    ("service-worker-registration", "getActive"): "service-worker",
    ("service-worker-registration", "getWaiting"): "service-worker",
    ("service-worker-registration", "getInstalling"): "service-worker",
    ("service-worker-registration", "getNavigationPreload"): "navigation-preload-manager",
    # Performance
    ("performance", "getTiming"): "performance-timing",
    ("performance", "getNavigation"): "performance-navigation",
    ("performance", "mark"): "performance-mark",
    ("performance", "measure"): "performance-measure",
    ("performance", "getEntries"): "any",
    ("performance", "getEntriesByType"): "any",
    ("performance", "getEntriesByName"): "any",
    # RTC
    ("rtc-peer-connection", "getLocalDescription"): "any",
    ("rtc-peer-connection", "getRemoteDescription"): "any",
    ("rtc-peer-connection", "getCurrentLocalDescription"): "any",
    ("rtc-peer-connection", "getPendingLocalDescription"): "any",
    ("rtc-peer-connection", "getCurrentRemoteDescription"): "any",
    ("rtc-peer-connection", "getPendingRemoteDescription"): "any",
    ("rtc-peer-connection", "getConfiguration"): "any",
    ("rtc-rtp-sender", "getTrack"): "media-stream-track",
    ("rtc-rtp-sender", "getTransport"): "rtc-dtls-transport",
    ("rtc-rtp-sender", "getParameters"): "any",
    ("rtc-rtp-receiver", "getTrack"): "media-stream-track",
    ("rtc-rtp-receiver", "getTransport"): "rtc-dtls-transport",
    ("rtc-rtp-receiver", "getParameters"): "any",
    ("rtc-rtp-receiver", "getSynchronizationSources"): "any",
    ("rtc-rtp-transceiver", "getSender"): "rtc-rtp-sender",
    ("rtc-rtp-transceiver", "getReceiver"): "rtc-rtp-receiver",
    ("rtc-data-channel", "getTransport"): "rtc-sctp-transport",
    ("rtc-ice-transport", "getComponent"): "any",
    ("rtc-dtls-transport", "getTransport"): "rtc-ice-transport",
    ("rtc-sctp-transport", "getTransport"): "rtc-ice-transport",
    # PaymentRequest
    ("payment-response", "getShippingAddress"): "payment-address",
    # CSSKeyframesRule findRule returns CSSKeyframeRule | null
    ("css-keyframes-rule", "findRule"): "css-keyframe-rule",
    # Window getEvent returns Event | undefined
    ("window", "getEvent"): "event",
    # Element getAttributeNames returns string[]
    ("element", "getAttributeNames"): "string-list",
    # Gamepad getAxes/getButtons
    ("gamepad", "getAxes"): "float-32-list",
    ("gamepad", "getButtons"): "gamepad-button-list",
    # PointerEvent getCoalescedEvents/getPredictedEvents
    ("pointer-event", "getCoalescedEvents"): "pointer-event-list",
    ("pointer-event", "getPredictedEvents"): "pointer-event-list",
    # CSSStyleDeclaration getPropertyPriority returns string
    ("css-style-declaration", "getPropertyPriority"): "string",
    # CSSStyleDeclaration item returns string
    ("css-style-declaration", "item"): "string",
    # CSSRuleList item returns CSSRule
    ("css-rule-list", "item"): "css-rule",
    # Document createEvent returns Event
    ("document", "createEvent"): "event",
    # Event initEvent returns void
    ("event", "composedPath"): "event-target-list",

    # ParentNode firstElementChild/lastElementChild return Element
    ("parent-node", "getFirstElementChild"): "element",
    ("parent-node", "getLastElementChild"): "element",
    # Slottable assignedSlot returns HTMLSlotElement
    ("slottable", "getAssignedSlot"): "html-slot-element",
    # HTMLIFrameElement getSVGDocument/getContentDocument return Document
    ("htmli-frame-element", "getSvgDocument"): "document",
    ("htmli-frame-element", "getContentDocument"): "document",
    # HTMLObjectElement getSVGDocument/getContentDocument return Document
    ("html-object-element", "getSvgDocument"): "document",
    ("html-object-element", "getContentDocument"): "document",
    # HTMLEmbedElement getSVGDocument returns Document
    ("html-embed-element", "getSvgDocument"): "document",
    # HTMLFrameElement getContentDocument returns Document
    ("html-frame-element", "getContentDocument"): "document",
    # HTMLSelectElement item/namedItem return HTMLOptionElement
    ("html-select-element", "item"): "html-option-element",
    ("html-select-element", "namedItem"): "html-option-element",
    # HTMLOptionElement getForm returns HTMLFormElement
    ("html-option-element", "getForm"): "html-form-element",
    # HTMLLabelElement getForm returns HTMLFormElement
    ("html-label-element", "getForm"): "html-form-element",
    # TextTrackList item/getTextTrack return TextTrack
    ("text-track-list", "item"): "text-track",
    ("text-track-list", "getTextTrack"): "text-track",
    # Event getTarget/getCurrentTarget return EventTarget
    ("event", "getTarget"): "event-target",
    ("event", "getCurrentTarget"): "event-target",
    # FocusEvent getRelatedTarget returns EventTarget
    ("focus-event", "getRelatedTarget"): "event-target",
    # MouseEvent getRelatedTarget returns EventTarget
    ("mouse-event", "getRelatedTarget"): "event-target",
    # Element setAttributeNodeNS returns Attr
    ("element", "setAttributeNodeNs"): "attr",
    # HTMLCanvasElement getCanvas returns HTMLCanvasElement
    ("canvas-state", "getCanvas"): "any",
    ("offscreen-canvas-state", "getCanvas"): "any",
    # CountQueuingStrategy size returns function, need any
    ("count-queuing-strategy", "getSize"): "any",
    # ByteLengthQueuingStrategy size returns function, need any
    ("byte-length-queuing-strategy", "getSize"): "any",
    # Performance getEntries returns PerformanceEntryList
    ("performance", "getEntries"): "any",
    ("performance", "getEntriesByType"): "any",
    ("performance", "getEntriesByName"): "any",
    # MediaRecorder getStream returns MediaStream
    ("media-recorder", "getStream"): "media-stream",
    # PluginArray item/namedItem return Plugin
    ("plugin-array", "item"): "plugin",
    ("plugin-array", "namedItem"): "plugin",
    # MimeTypeArray item/namedItem return MimeType
    ("mime-type-array", "item"): "mime-type",
    ("mime-type-array", "namedItem"): "mime-type",
    # Plugin item/namedItem return MimeType
    ("plugin", "item"): "mime-type",
    ("plugin", "namedItem"): "mime-type",
    # MessageChannel getPort1/getPort2 return MessagePort
    ("message-channel", "getPort1"): "message-port",
    ("message-channel", "getPort2"): "message-port",
    # Storage getLocalStorage/getSessionStorage return Storage
    ("window", "getLocalStorage"): "storage",
    ("window", "getSessionStorage"): "storage",
    # NavigatorStorage getStorage returns StorageManager
    ("navigator-storage", "getStorage"): "storage-manager",
    # HTMLMediaElement getTextTrack returns TextTrack
    ("html-media-element", "getTextTrack"): "text-track",
    # HTMLMediaElement addTextTrack returns TextTrack
    ("html-media-element", "addTextTrack"): "text-track",
    # HTMLTableElement createCaption/createTHead/createTFoot/createTBody/insertRow
    ("html-table-element", "createCaption"): "html-table-caption-element",
    ("html-table-element", "createTHead"): "html-table-section-element",
    ("html-table-element", "createTFoot"): "html-table-section-element",
    ("html-table-element", "createTBody"): "html-table-section-element",
    ("html-table-element", "insertRow"): "html-table-row-element",
    # HTMLTableSectionElement insertRow returns HTMLTableRowElement
    ("html-table-section-element", "insertRow"): "html-table-row-element",
    # HTMLTableRowElement insertCell returns HTMLTableCellElement
    ("html-table-row-element", "insertCell"): "html-table-cell-element",
    # DataTransfer getItems/getFiles return handles
    ("data-transfer", "getItems"): "data-transfer-item-list",
    ("data-transfer", "getFiles"): "file-list",
    # DataTransferItemList item returns DataTransferItem
    ("data-transfer-item-list", "item"): "data-transfer-item",
    # FileList item returns File
    ("file-list", "item"): "file",
    # MediaStream getTrackById returns MediaStreamTrack
    ("media-stream", "getTrackById"): "media-stream-track",
    # ViewTransition startViewTransition returns ViewTransition
    ("document", "startViewTransition"): "view-transition",
    # IntersectionObserver getRoot returns Element
    ("intersection-observer", "getRoot"): "event-target",
    # ClipboardItem getTypes returns string array as handle
    ("clipboard-item", "getTypes"): "string-list",
    # SpeechSynthesis getVoices returns SpeechSynthesisVoiceList
    ("speech-synthesis", "getVoices"): "speech-synthesis-voice-list",
    # SpeechSynthesisVoiceList item returns SpeechSynthesisVoice
    ("speech-synthesis-voice-list", "item"): "speech-synthesis-voice",
    # RTCRtpSender replaceTrack returns Promise
    ("rtc-rtp-sender", "replaceTrack"): "promise-void",
    # HTMLTemplateElement getContent returns DocumentFragment
    ("html-template-element", "getContent"): "document-fragment",
    # ResizeObserverEntry getTarget returns Element
    ("resize-observer-entry", "getTarget"): "element",
    # SubmitEvent getSubmitter returns HTMLElement
    ("submit-event", "getSubmitter"): "html-element",
    # Navigation getCurrentEntry/getActivation/getTransition return handles
    ("navigation", "getCurrentEntry"): "navigation-history-entry",
    ("navigation", "getActivation"): "navigation-activation",
    ("navigation", "getTransition"): "view-transition",
    # HTMLElement attachInternals returns ElementInternals
    ("html-element", "attachInternals"): "element-internals",
    # ClipboardItem getType should return Promise<Blob>
    ("clipboard-item", "getType"): "promise-any",
    # OffscreenCanvas transferToImageBitmap returns ImageBitmap
    ("offscreencanvas", "transferToImageBitmap"): "image-bitmap",
    # HTMLCanvasElement transferControlToOffscreen returns OffscreenCanvas
    ("html-canvas-element", "transferControlToOffscreen"): "offscreencanvas",

    # htmlGlue.ts fixes - HANDLE_RETURNING_FUNCTIONS for getters returning objects
    # CanvasRenderingContext2D.getCanvas returns HTMLCanvasElement
    ("canvas-rendering-context2-d", "getCanvas"): "html-canvas-element",
    # ImageBitmapRenderingContext.getCanvas returns HTMLCanvasElement | OffscreenCanvas
    ("image-bitmap-rendering-context", "getCanvas"): "any",
    # OffscreenCanvas.transferToImageBitmap returns ImageBitmap
    ("offscreencanvas", "transferToImageBitmap"): "image-bitmap",
    # ElementInternals.getStates returns CustomStateSet
    ("element-internals", "getStates"): "custom-state-set",
    # ElementInternals.getForm returns HTMLFormElement
    ("element-internals", "getForm"): "html-form-element",
    # DataTransferItemList.add returns DataTransferItem
    ("data-transfer-item-list", "add"): "data-transfer-item",
    # NavigationActivation.getFrom returns NavigationHistoryEntry | undefined
    ("navigation-activation", "getFrom"): "navigation-history-entry",
    # NavigationActivation.getEntry returns NavigationHistoryEntry
    ("navigation-activation", "getEntry"): "navigation-history-entry",
    # NavigateEvent.getDestination returns NavigationHistoryEntry
    ("navigate-event", "getDestination"): "navigation-history-entry",
    # Navigation.getCurrentEntry returns NavigationHistoryEntry
    ("navigation", "getCurrentEntry"): "navigation-history-entry",
    # Navigation.getActivation returns NavigationActivation
    ("navigation", "getActivation"): "navigation-activation",
    # Navigation.getTransition returns ViewTransition
    ("navigation", "getTransition"): "view-transition",
    # Document.startViewTransition returns ViewTransition
    ("document", "startViewTransition"): "view-transition",
    # Navigator.getPlugins returns PluginArray
    ("navigator", "getPlugins"): "plugin-array",
    # Navigator.getMimeTypes returns MimeTypeArray
    ("navigator", "getMimeTypes"): "mime-type-array",
    # PluginArray.item/namedItem return Plugin
    ("plugin-array", "item"): "plugin",
    ("plugin-array", "namedItem"): "plugin",
    # MimeTypeArray.item/namedItem return MimeType
    ("mime-type-array", "item"): "mime-type",
    ("mime-type-array", "namedItem"): "mime-type",
    # Plugin.item/namedItem return MimeType
    ("plugin", "item"): "mime-type",
    ("plugin", "namedItem"): "mime-type",
    # Event.getTarget returns EventTarget
    ("event", "getTarget"): "event-target",
    # Window.getLocalStorage/getSessionStorage return Storage
    ("window", "getLocalStorage"): "storage",
    ("window", "getSessionStorage"): "storage",
    # NavigateEvent canIntercept/userInitiated/hashChange/hasUAVisualTransition need type assertion
    ("navigate-event", "getCanIntercept"): "any",
    ("navigate-event", "getUserInitiated"): "any",
    ("navigate-event", "getHashChange"): "any",
    ("navigate-event", "getHasUaVisualTransition"): "any",
    # StorageEvent.storageArea
    ("storage-event", "getStorageArea"): "storage",
    # MessageEvent.source
    ("message-event", "getSource"): "message-event-source",

    # CSSRuleList item returns CSSRule
    ("css-rule-list", "item"): "css-rule",
    # Event getTarget/getCurrentTarget return EventTarget
    ("event", "getTarget"): "event-target",
    ("event", "getCurrentTarget"): "event-target",
    # FocusEvent getRelatedTarget returns EventTarget
    ("focus-event", "getRelatedTarget"): "event-target",
    # MouseEvent getRelatedTarget returns EventTarget
    ("mouse-event", "getRelatedTarget"): "event-target",
    # TextTrackList item returns TextTrack
    ("text-track-list", "item"): "text-track",
    # TextTrackCueList item/getCueById return TextTrackCue
    ("text-track-cue-list", "item"): "text-track-cue",
    ("text-track-cue-list", "getCueById"): "text-track-cue",
    # FileList item returns File
    ("file-list", "item"): "file",
    # MediaStream getTrackById returns MediaStreamTrack
    ("media-stream", "getTrackById"): "media-stream-track",
    # ViewTransition startViewTransition returns ViewTransition
    ("document", "startViewTransition"): "view-transition",
    # IntersectionObserver getRoot returns Element
    ("intersection-observer", "getRoot"): "event-target",
    # ClipboardItem getTypes returns string array as handle
    ("clipboard-item", "getTypes"): "string-list",
    # SpeechSynthesis getVoices returns SpeechSynthesisVoiceList
    ("speech-synthesis", "getVoices"): "speech-synthesis-voice-list",
    # SpeechSynthesisVoiceList item returns SpeechSynthesisVoice
    ("speech-synthesis-voice-list", "item"): "speech-synthesis-voice",
    # RTCRtpSender replaceTrack returns Promise
    ("rtc-rtp-sender", "replaceTrack"): "promise-void",
    # HTMLTemplateElement getContent returns DocumentFragment
    ("html-template-element", "getContent"): "document-fragment",
    # ResizeObserverEntry getTarget returns Element
    ("resize-observer-entry", "getTarget"): "element",
    # SubmitEvent getSubmitter returns HTMLElement
    ("submit-event", "getSubmitter"): "html-element",
    # Navigation getCurrentEntry/getActivation/getTransition return handles
    ("navigation", "getCurrentEntry"): "navigation-history-entry",
    ("navigation", "getActivation"): "navigation-activation",
    ("navigation", "getTransition"): "view-transition",
    # HTMLElement attachInternals returns ElementInternals
    ("html-element", "attachInternals"): "element-internals",
    # ClipboardItem getType should return Promise<Blob>
    ("clipboard-item", "getType"): "promise-any",
    # OffscreenCanvas transferToImageBitmap returns ImageBitmap
    ("offscreencanvas", "transferToImageBitmap"): "image-bitmap",
    # HTMLCanvasElement transferControlToOffscreen returns OffscreenCanvas
    ("html-canvas-element", "transferControlToOffscreen"): "offscreencanvas",
    # CSSRuleList item returns CSSRule
    ("css-rule-list", "item"): "css-rule",
    # Event getTarget/getCurrentTarget return EventTarget
    ("event", "getTarget"): "event-target",
    ("event", "getCurrentTarget"): "event-target",
    # FocusEvent getRelatedTarget returns EventTarget
    ("focus-event", "getRelatedTarget"): "event-target",
    # MouseEvent getRelatedTarget returns EventTarget
    ("mouse-event", "getRelatedTarget"): "event-target",
    # HTMLMediaElement getTextTrack returns TextTrack (for multiple track getters)
    ("html-media-element", "getTextTrack1"): "text-track",
    ("html-media-element", "getTextTrack2"): "text-track",
    ("html-media-element", "getTextTrack3"): "text-track",
    # HTMLMediaElement addTextTrack returns TextTrack
    ("html-media-element", "addTextTrack"): "text-track",
    # HTMLInputElement getForm returns HTMLFormElement
    ("html-input-element", "getForm"): "html-form-element",
    # HTMLSelectElement getForm returns HTMLFormElement
    ("html-select-element", "getForm"): "html-form-element",
    # HTMLTextAreaElement getForm returns HTMLFormElement
    ("html-text-area-element", "getForm"): "html-form-element",
    # HTMLButtonElement getForm returns HTMLFormElement
    ("html-button-element", "getForm"): "html-form-element",
    # HTMLFieldSetElement getForm returns HTMLFormElement
    ("html-field-set-element", "getForm"): "html-form-element",
    # HTMLOutputElement getForm returns HTMLFormElement
    ("html-output-element", "getForm"): "html-form-element",
    # HTMLObjectElement getForm returns HTMLFormElement
    ("html-object-element", "getForm"): "html-form-element",
    # HTMLCanvasElement getCanvas returns HTMLCanvasElement
    ("html-canvas-element", "getCanvas"): "html-canvas-element",
    # CanvasRenderingContext2D getCanvas returns HTMLCanvasElement | OffscreenCanvas
    ("canvas-rendering-context", "getCanvas"): "any",
    # OffscreenCanvasRenderingContext2D getCanvas returns OffscreenCanvas
    ("offscreencanvas-rendering-context", "getCanvas"): "offscreencanvas",
    # TextTrackList getTrackById returns TextTrack
    ("text-track-list", "getTrackById"): "text-track",
    # HTMLInputElement getFiles returns FileList
    ("html-input-element", "getFiles"): "file-list",
    # CSSRuleList item returns CSSRule
    ("css-rule-list", "item"): "css-rule",
    # CSSGroupingRule getCssRules returns CSSRuleList
    ("css-grouping-rule", "getCssRules"): "css-rule-list",
    # CSSStyleSheet getCssRules returns CSSRuleList
    ("css-style-sheet", "getCssRules"): "css-rule-list",
    # TextTrackCue getTrack returns TextTrack
    ("text-track-cue", "getTrack"): "text-track",
    # TrackEvent getTrack returns TextTrack
    ("track-event", "getTrack"): "text-track",
    # HTMLInputElement getValueAsDate returns Date
    ("html-input-element", "getValueAsDate"): "date",
    # ElementInternals getShadowRoot returns ShadowRoot
    ("element-internals", "getShadowRoot"): "shadow-root",
    # HTMLLegendElement getForm returns HTMLFormElement
    ("html-legend-element", "getForm"): "html-form-element",
    # MediaStream clone returns MediaStream
    ("media-stream", "clone"): "media-stream",
    # MediaStreamTrack clone returns MediaStreamTrack
    ("media-stream-track", "clone"): "media-stream-track",
    # MediaStreamTrackEvent getTrack returns MediaStreamTrack
    ("media-stream-track-event", "getTrack"): "media-stream-track",
    # RTCPeerConnection addTrack returns RTCRtpSender
    ("rtc-peer-connection", "addTrack"): "rtc-rtp-sender",
    # RTCPeerConnection addTransceiver returns RTCRtpTransceiver
    ("rtc-peer-connection", "addTransceiver"): "rtc-rtp-transceiver",
    # RTCPeerConnection createDataChannel returns RTCDataChannel
    ("rtc-peer-connection", "createDataChannel"): "rtc-data-channel",
    # RTCIceCandidatePair getLocal/getRemote return RTCIceCandidate
    ("rtc-ice-candidate-pair", "getLocal"): "rtc-ice-candidate",
    ("rtc-ice-candidate-pair", "getRemote"): "rtc-ice-candidate",
    # RTCTrackEvent getReceiver/getTransceiver
    ("rtc-track-event", "getReceiver"): "rtc-rtp-receiver",
    ("rtc-track-event", "getTransceiver"): "rtc-rtp-transceiver",
    # MediaDevices getSupportedConstraints returns MediaTrackSupportedConstraints
    ("media-devices", "getSupportedConstraints"): "any",
    # MediaSession setMicrophoneActive/setCameraActive return Promise<void>
    ("media-session", "setMicrophoneActive"): "promise-void",
    ("media-session", "setCameraActive"): "promise-void",
    # PerformanceObserver takeRecords returns PerformanceEntryList
    ("performance-observer", "takeRecords"): "any",
    # Performance getEntries returns PerformanceEntryList
    ("performance", "getEntries"): "any",
    # URL getSearchParams returns URLSearchParams
    ("url", "getSearchParams"): "url-search-params",
    # Headers getSetCookie returns string[]
    ("headers", "getSetCookie"): "string-list",
    # IntersectionObserver takeRecords returns IntersectionObserverEntry[]
    ("intersection-observer", "takeRecords"): "any",
    # SharedWorker getPort returns MessagePort
    ("shared-worker", "getPort"): "message-port",
    # CSSGroupingRule item returns CSSRule
    ("css-grouping-rule", "item"): "css-rule",
    # RTCRtpReceiver getContributingSources returns RTCRtpContributingSource[]
    ("rtc-rtp-receiver", "getContributingSources"): "any",
    # RTCRtpSender getDtmf returns RTCDTMFSender
    ("rtc-rtp-sender", "getDtmf"): "any",
    # RTCIceTransport getLocalCandidates/getRemoteCandidates return RTCIceCandidate[]
    ("rtc-ice-transport", "getLocalCandidates"): "any",
    ("rtc-ice-transport", "getRemoteCandidates"): "any",
    # RTCDtlsTransport getRemoteCertificates returns ArrayBuffer[]
    ("rtc-dtls-transport", "getRemoteCertificates"): "any",
    # Navigation getCurrentEntry currentEntry returns NavigationHistoryEntry
    ("navigation", "currentEntry"): "navigation-history-entry",
    # BlobEvent data returns Blob
    ("blob-event", "getData"): "blob",
    # SpeechSynthesisEvent utterance returns SpeechSynthesisUtterance
    ("speech-synthesis-event", "getUtterance"): "speech-synthesis-utterance",
    # SpeechSynthesis getVoices returns SpeechSynthesisVoiceList
    ("speech-synthesis", "getVoices"): "speech-synthesis-voice-list",
    # SpeechSynthesisUtterance getVoice returns SpeechSynthesisVoice
    ("speech-synthesis-utterance", "getVoice"): "speech-synthesis-voice",
    # SpeechSynthesisEvent getUtterance returns SpeechSynthesisUtterance
    ("speech-synthesis-event", "getUtterance"): "speech-synthesis-utterance",
    # InputDeviceInfo getCapabilities returns MediaTrackCapabilities
    ("input-device-info", "getCapabilities"): "any",
    # WebRTC getters that return objects needing handle wrapping
    ("rtc-peer-connection", "getSctp"): "rtc-sctp-transport",
    ("rtc-session-description", "toJson"): "any",
    ("rtc-ice-candidate", "toJson"): "any",
    ("rtc-certificate", "getFingerprints"): "any",
    ("rtc-rtp-sender", "setParameters"): "promise-void",
    ("rtc-rtp-receiver", "getTransport"): "rtc-dtls-transport",
    ("rtc-rtp-transceiver", "getSender"): "rtc-rtp-sender",
    ("rtc-rtp-transceiver", "getReceiver"): "rtc-rtp-receiver",
    ("rtc-dtls-transport", "getIceTransport"): "rtc-ice-transport",
    ("rtc-ice-transport", "getSelectedCandidatePair"): "rtc-ice-candidate-pair",
    ("rtc-track-event", "getTrack"): "media-stream-track",
    ("rtc-sctp-transport", "getTransport"): "rtc-dtls-transport",
    ("rtc-data-channel-event", "getChannel"): "rtc-data-channel",
    ("rtc-error-event", "getError"): "rtc-error",
    # htmlGlue.ts HANDLE_RETURNING_FUNCTIONS fixes
    # NavigateEvent getSrcElement returns EventTarget
    ("navigate-event", "getSrcElement"): "event-target",
    ("navigate-event", "getSourceElement"): "event-target",
    # PageSwapEvent getActivation/getViewTransition
    ("page-swap-event", "getActivation"): "navigation-activation",
    ("page-swap-event", "getViewTransition"): "view-transition",
    # PageRevealEvent getViewTransition
    ("page-reveal-event", "getViewTransition"): "view-transition",
    # HTMLFrameElement getContentWindow returns Window
    ("html-frame-element", "getContentWindow"): "window",
    # WindowLocalStorage/SessionStorage getLocalStorage/getSessionStorage return Storage
    ("window-local-storage", "getLocalStorage"): "storage",
    ("window-session-storage", "getSessionStorage"): "storage",
    # ImageBitmapRenderingContext getCanvas returns HTMLCanvasElement | OffscreenCanvas
    ("image-bitmap-rendering-context", "getCanvas"): "any",
    # CanvasTransform getTransform returns DOMMatrix
    ("canvas-transform", "getTransform"): "dom-matrix",
    # CanvasFillStrokeStyles getStrokeStyle/getFillStyle return string | CanvasGradient | CanvasPattern
    ("canvas-fill-stroke-styles", "getStrokeStyle"): "any",
    ("canvas-fill-stroke-styles", "getFillStyle"): "any",
    # CanvasPattern getTransform returns DOMMatrix | null
    ("canvas-pattern", "getTransform"): "dom-matrix",
    # OffscreenCanvasRenderingContext2D getCanvas returns OffscreenCanvas
    ("offscreen-canvas-rendering-context2-d", "getCanvas"): "offscreencanvas",
    # NavigatorPlugins getPlugins/getMimeTypes return PluginArray/MimeTypeArray
    ("navigator-plugins", "getPlugins"): "plugin-array",
    ("navigator-plugins", "getMimeTypes"): "mime-type-array",
    # MimeType getEnabledPlugin returns Plugin
    ("mime-type", "getEnabledPlugin"): "plugin",
    # ImageData getData returns ImageDataArray (Uint8ClampedArray)
    ("image-data", "getData"): "image-data-array",
    # OffscreenCanvas transferToImageBitmap returns ImageBitmap
    ("offscreen-canvas", "transferToImageBitmap"): "image-bitmap",
    # RTCPeerConnectionIceEvent getCandidate returns RTCIceCandidate | undefined
    ("rtc-peer-connection-ice-event", "getCandidate"): "rtc-ice-candidate",
    # URL parse returns URL | null
    ("url", "parse"): "url",
    # ExtendableMessageEvent getSource returns MessageEventSource | undefined
    ("extendable-message-event", "getSource"): "message-event-source",
    # PerformanceObserverEntryList getEntriesByType/getEntriesByName return PerformanceEntryList
    ("performance-observer-entry-list", "getEntriesByType"): "performance-entry-list",
    ("performance-observer-entry-list", "getEntriesByName"): "performance-entry-list",
    # CSSStyleDeclaration getParentRule returns CSSRule | undefined
    ("css-style-declaration", "getParentRule"): "css-rule",
}


# Parameters that are handles and need to be looked up
# Maps (interface, function, param_name) -> (target_interface, target_type)
# NOTE: Only add entries here for actual handle types, not dictionary types
PARAMETER_HANDLE_MAPPING = {
    ("audio-decoder", "decode", "chunk"): ("encoded-audio-chunk", "EncodedAudioChunk"),
    ("video-decoder", "decode", "chunk"): ("encoded-video-chunk", "EncodedVideoChunk"),
    ("audio-encoder", "encode", "data"): ("audio-data", "AudioData"),
    ("video-encoder", "encode", "frame"): ("video-frame", "VideoFrame"),
    ("credentials-container", "store", "credential"): ("credential", "Credential"),
    ("tree-walker", "set-current-node", "value"): ("node", "Node"),
    ("resize-observer", "unobserve", "target"): ("element", "Element"),
    ("crypto", "get-random-values", "array"): ("uint8-array", "Uint8Array"),
    # SubtleCrypto encrypt key parameter
    ("subtle-crypto", "encrypt", "key"): ("crypto-key", "CryptoKey"),
    # SpeechSynthesisUtterance voice setter
    ("speech-synthesis-utterance", "set-voice", "value"): ("speech-synthesis-voice", "SpeechSynthesisVoice"),
    # SubtleCrypto deriveBits baseKey parameter
    ("subtle-crypto", "derive-bits", "base-key"): ("crypto-key", "CryptoKey"),
    # KeyboardEvent initKeyboardEvent view parameter
    ("keyboard-event", "init-keyboard-event", "view-arg"): ("window", "Window"),
    # CompositionEvent initCompositionEvent view parameter
    ("composition-event", "init-composition-event", "view-arg"): ("window", "Window"),
}

# Parameters that are dictionary types (not handles) - should be passed directly as objects
# Maps (interface, function, param_name) -> TypeScript type
DICTIONARY_PARAMETER_TYPES = {
    ("audio-decoder", "configure", "config"): "AudioDecoderConfig",
    ("audio-decoder", "is-config-supported", "config"): "AudioDecoderConfig",
    ("video-decoder", "configure", "config"): "VideoDecoderConfig",
    ("video-decoder", "is-config-supported", "config"): "VideoDecoderConfig",
    ("audio-encoder", "configure", "config"): "AudioEncoderConfig",
    ("audio-encoder", "is-config-supported", "config"): "AudioEncoderConfig",
    ("audio-encoder", "encode", "options"): "AudioEncoderEncodeOptions",
    ("video-encoder", "configure", "config"): "VideoEncoderConfig",
    ("video-encoder", "is-config-supported", "config"): "VideoEncoderConfig",
    ("video-encoder", "encode", "options"): "VideoEncoderEncodeOptions",
    ("credentials-container", "get", "options"): "any",
    ("credentials-container", "create", "options"): "any",
    # WebCodecs
    ("audio-data", "copy-to", "options"): "any",
    ("audio-data", "allocation-size", "options"): "any",
    ("video-frame", "copy-to", "options"): "any",
    ("video-frame", "allocation-size", "options"): "any",
    ("image-decoder", "decode", "options"): "ImageDecodeOptions | undefined",
    ("image-decoder", "parse", "options"): "ImageParseOptions",
    # Canvas
    ("html-canvas-element", "get-context", "options"): "any",
    ("offscreencanvas", "get-context", "options"): "any",
    # Element
    ("element", "check-visibility", "options"): "CheckVisibilityOptions | undefined",
    ("element", "scroll", "options"): "ScrollToOptions | undefined",
    ("element", "scroll-to", "options"): "ScrollToOptions | undefined",
    ("element", "scroll-by", "options"): "ScrollToOptions | undefined",
    ("element", "request-fullscreen", "options"): "FullscreenOptions | undefined",
    ("element", "attach-shadow", "options"): "ShadowRootInit",
    # ServiceWorkerRegistration
    ("service-worker-registration", "get-notifications", "filter"): "GetNotificationOptions | undefined",
    # PaymentResponse
    ("payment-response", "retry", "error-fields"): "PaymentValidationErrors | undefined",
    # RTCPeerConnection
    ("rtc-peer-connection", "create-offer", "options"): "RTCOfferOptions | undefined",
    # Geolocation watchPosition options
    ("geolocation", "watch-position", "options"): "PositionOptions | undefined",
}

# Parameters that need bigint to number conversion
# Maps (interface, function, param_name) -> boolean (true if number conversion needed)
PARAMETER_BIGINT_TO_NUMBER = {
    ("web-gl-rendering-context-base", "bind-buffer", "target"): True,
    ("web-gl-rendering-context-base", "bind-framebuffer", "target"): True,
    ("web-gl-rendering-context-base", "bind-renderbuffer", "target"): True,
    ("web-gl-rendering-context-base", "bind-texture", "target"): True,
    ("web-gl-rendering-context-base", "disable", "cap"): True,
    ("web-gl-rendering-context-base", "enable", "cap"): True,
    ("web-gl-rendering-context-base", "get-attribute", "index"): True,
    ("web-gl-rendering-context-base", "get-boolean", "pname"): True,
    ("web-gl-rendering-context-base", "get-buffer-parameter", "target"): True,
    ("web-gl-rendering-context-base", "get-framebuffer-attachment-parameter", "target"): True,
    ("web-gl-rendering-context-base", "get-program-parameter", "pname"): True,
    ("web-gl-rendering-context-base", "get-renderbuffer-parameter", "target"): True,
    ("web-gl-rendering-context-base", "get-shader-parameter", "pname"): True,
    ("web-gl-rendering-context-base", "get-tex-parameter", "target"): True,
    ("web-gl-rendering-context-base", "get-uniform", "location"): True,
    ("web-gl-rendering-context-base", "get-vertex-attrib", "index"): True,
    ("web-gl-rendering-context-base", "vertex-attrib-pointer", "index"): True,
    ("web-gl-rendering-context-base", "create-shader", "type"): True,
    ("web-gl-rendering-context-base", "tex-image-2d", "target"): True,
    ("web-gl-rendering-context-base", "tex-image-2d", "level"): True,
    ("web-gl-rendering-context-base", "tex-sub-image-2d", "target"): True,
    ("web-gl-rendering-context-base", "tex-sub-image-2d", "level"): True,
    ("web-gl-rendering-context-base", "copy-tex-image-2d", "target"): True,
    ("web-gl-rendering-context-base", "copy-tex-image-2d", "level"): True,
    ("web-gl-rendering-context-base", "copy-tex-image-2d", "internalformat"): True,
    ("web-gl-rendering-context-base", "copy-tex-image-2d", "x"): True,
    ("web-gl-rendering-context-base", "copy-tex-image-2d", "y"): True,
    ("web-gl-rendering-context-base", "copy-tex-image-2d", "width"): True,
    ("web-gl-rendering-context-base", "copy-tex-image-2d", "height"): True,
    ("web-gl-rendering-context-base", "copy-tex-image-2d", "border"): True,
    ("web-gl-rendering-context-base", "copy-tex-sub-image-2d", "target"): True,
    ("web-gl-rendering-context-base", "copy-tex-sub-image-2d", "level"): True,
    ("web-gl-rendering-context-base", "copy-tex-sub-image-2d", "xoffset"): True,
    ("web-gl-rendering-context-base", "copy-tex-sub-image-2d", "yoffset"): True,
    ("web-gl-rendering-context-base", "copy-tex-sub-image-2d", "x"): True,
    ("web-gl-rendering-context-base", "copy-tex-sub-image-2d", "y"): True,
    ("web-gl-rendering-context-base", "copy-tex-sub-image-2d", "width"): True,
    ("web-gl-rendering-context-base", "copy-tex-sub-image-2d", "height"): True,
    ("web-gl-rendering-context-base", "compressed-tex-image-2d", "target"): True,
    ("web-gl-rendering-context-base", "compressed-tex-image-2d", "level"): True,
    ("web-gl-rendering-context-base", "compressed-tex-sub-image-2d", "target"): True,
    ("web-gl-rendering-context-base", "compressed-tex-sub-image-2d", "level"): True,
    ("web-gl-rendering-context-base", "renderbuffer-storage", "target"): True,
    ("web-gl-rendering-context-base", "framebuffer-renderbuffer", "target"): True,
    ("web-gl-rendering-context-base", "framebuffer-renderbuffer", "attachment"): True,
    ("web-gl-rendering-context-base", "framebuffer-renderbuffer", "renderbuffertarget"): True,
    ("web-gl-rendering-context-base", "framebuffer-renderbuffer", "renderbuffer"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "framebuffer-texture-2d", "target"): True,
    ("web-gl-rendering-context-base", "framebuffer-texture-2d", "attachment"): True,
    ("web-gl-rendering-context-base", "framebuffer-texture-2d", "textarget"): True,
    ("web-gl-rendering-context-base", "framebuffer-texture-2d", "texture"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "framebuffer-texture-2d", "level"): True,
    ("web-gl-rendering-context-base", "generate-mipmap", "target"): True,
    ("web-gl-rendering-context-base", "depth-range", "z-near"): True,
    ("web-gl-rendering-context-base", "depth-range", "z-far"): True,
    ("web-gl-rendering-context-base", "get-active-attrib", "program"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "get-active-attrib", "index"): True,
    ("web-gl-rendering-context-base", "get-active-uniform", "program"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "get-active-uniform", "index"): True,
    ("web-gl-rendering-context-base", "get-attached-shaders", "program"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "get-parameter", "pname"): True,
    ("web-gl-rendering-context-base", "tex-parameter-f", "target"): True,
    ("web-gl-rendering-context-base", "tex-parameter-f", "pname"): True,
    ("web-gl-rendering-context-base", "tex-parameter-i", "target"): True,
    ("web-gl-rendering-context-base", "tex-parameter-i", "pname"): True,
    ("web-gl-rendering-context-base", "tex-parameter-i", "param"): True,
    ("web-gl-rendering-context-base", "hint", "target"): True,
    ("web-gl-rendering-context-base", "hint", "mode"): True,
    ("web-gl-rendering-context-base", "cull-face", "mode"): True,
    ("web-gl-rendering-context-base", "front-face", "mode"): True,
    ("web-gl-rendering-context-base", "depth-func", "func"): True,
    ("web-gl-rendering-context-base", "stencil-func", "func"): True,
    ("web-gl-rendering-context-base", "stencil-op", "fail"): True,
    ("web-gl-rendering-context-base", "blend-func", "factor"): True,
    ("web-gl-rendering-context-base", "blend-func-separate", "src-rgb"): True,
    ("web-gl-rendering-context-base", "blend-equation", "mode"): True,
    ("web-gl-rendering-context-base", "blend-color", "red"): True,
    ("web-gl-rendering-context-base", "blend-color", "green"): True,
    ("web-gl-rendering-context-base", "blend-color", "blue"): True,
    ("web-gl-rendering-context-base", "blend-color", "alpha"): True,
    ("web-gl-rendering-context-base", "blend-equation-separate", "mode-rgb"): True,
    ("web-gl-rendering-context-base", "blend-equation-separate", "mode-alpha"): True,
    ("web-gl-rendering-context-base", "blend-func", "sfactor"): True,
    ("web-gl-rendering-context-base", "blend-func", "dfactor"): True,
    ("web-gl-rendering-context-base", "blend-func-separate", "dst-rgb"): True,
    ("web-gl-rendering-context-base", "blend-func-separate", "src-alpha"): True,
    ("web-gl-rendering-context-base", "blend-func-separate", "dst-alpha"): True,
    ("web-gl-rendering-context-base", "active-texture", "texture"): True,
    ("web-gl-rendering-context-base", "check-framebuffer-status", "target"): True,
    ("web-gl-rendering-context-base", "bind-attrib-location", "program"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "bind-attrib-location", "index"): True,
    ("web-gl-rendering-context-base", "stencil-op-separate", "face"): True,
    ("web-gl-rendering-context-base", "stencil-op-separate", "fail"): True,
    ("web-gl-rendering-context-base", "stencil-op-separate", "zfail"): True,
    ("web-gl-rendering-context-base", "stencil-op-separate", "zpass"): True,
    ("web-gl-rendering-context-base", "stencil-op", "zfail"): True,
    ("web-gl-rendering-context-base", "stencil-op", "zpass"): True,
    ("web-gl-rendering-context-base", "tex-parameter-f", "param"): True,
    ("web-gl-rendering-context-base", "tex-parameter-i", "param"): True,
    ("web-gl-rendering-context-base", "clear", "mask"): True,
    ("web-gl-rendering-context-base", "clear-color", "red"): True,
    ("web-gl-rendering-context-base", "clear-color", "green"): True,
    ("web-gl-rendering-context-base", "clear-color", "blue"): True,
    ("web-gl-rendering-context-base", "clear-color", "alpha"): True,
    ("web-gl-rendering-context-base", "clear-depth", "depth"): True,
    ("web-gl-rendering-context-base", "clear-stencil", "s"): True,
    ("web-gl-rendering-context-base", "color-mask", "red"): "boolean",
    ("web-gl-rendering-context-base", "color-mask", "green"): "boolean",
    ("web-gl-rendering-context-base", "color-mask", "blue"): "boolean",
    ("web-gl-rendering-context-base", "color-mask", "alpha"): "boolean",
    ("web-gl-rendering-context-base", "depth-mask", "flag"): "boolean",
    # WebGLRenderingContextOverloads
    ("web-gl-rendering-context-overloads", "uniform-matrix2fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-overloads", "uniform-matrix2fv", "transpose"): "boolean",
    ("web-gl-rendering-context-overloads", "uniform-matrix3fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-overloads", "uniform-matrix3fv", "transpose"): "boolean",
    ("web-gl-rendering-context-overloads", "uniform-matrix4fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-overloads", "uniform-matrix4fv", "transpose"): "boolean",
    ("web-gl-rendering-context-base", "stencil-mask", "mask"): True,
    ("web-gl-rendering-context-base", "draw-arrays", "mode"): True,
    ("web-gl-rendering-context-base", "draw-arrays", "first"): True,
    ("web-gl-rendering-context-base", "draw-arrays", "count"): True,
    ("web-gl-rendering-context-base", "draw-elements", "mode"): True,
    ("web-gl-rendering-context-base", "draw-elements", "count"): True,
    ("web-gl-rendering-context-base", "draw-elements", "type"): True,
    ("web-gl-rendering-context-base", "draw-elements", "offset"): True,
    ("web-gl-rendering-context-base", "use-program", "program"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "attach-shader", "program"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "attach-shader", "shader"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "bind-buffer", "buffer"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "bind-framebuffer", "framebuffer"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "bind-renderbuffer", "renderbuffer"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "bind-texture", "texture"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "delete-buffer", "buffer"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "delete-framebuffer", "framebuffer"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "delete-program", "program"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "delete-renderbuffer", "renderbuffer"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "delete-shader", "shader"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "delete-texture", "texture"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "detach-shader", "program"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "detach-shader", "shader"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "get-attrib-location", "program"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "get-uniform-location", "program"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "link-program", "program"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "shader-source", "shader"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "compile-shader", "shader"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "get-shader-parameter", "shader"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "get-program-parameter", "program"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "get-shader-info-log", "shader"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "get-program-info-log", "program"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "get-shader-source", "shader"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "is-buffer", "buffer"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "is-framebuffer", "framebuffer"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "is-program", "program"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "is-renderbuffer", "renderbuffer"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "is-shader", "shader"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "is-texture", "texture"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "uniform1i", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform1i", "x"): True,
    ("web-gl-rendering-context-base", "uniform2i", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform2i", "x"): True,
    ("web-gl-rendering-context-base", "uniform2i", "y"): True,
    ("web-gl-rendering-context-base", "uniform3i", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform3i", "x"): True,
    ("web-gl-rendering-context-base", "uniform3i", "y"): True,
    ("web-gl-rendering-context-base", "uniform3i", "z"): True,
    ("web-gl-rendering-context-base", "uniform4i", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform4i", "x"): True,
    ("web-gl-rendering-context-base", "uniform4i", "y"): True,
    ("web-gl-rendering-context-base", "uniform4i", "z"): True,
    ("web-gl-rendering-context-base", "uniform4i", "w"): True,
    ("web-gl-rendering-context-base", "uniform1f", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform1f", "x"): True,
    ("web-gl-rendering-context-base", "uniform2f", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform2f", "x"): True,
    ("web-gl-rendering-context-base", "uniform2f", "y"): True,
    ("web-gl-rendering-context-base", "uniform3f", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform3f", "x"): True,
    ("web-gl-rendering-context-base", "uniform3f", "y"): True,
    ("web-gl-rendering-context-base", "uniform3f", "z"): True,
    ("web-gl-rendering-context-base", "uniform4f", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform4f", "x"): True,
    ("web-gl-rendering-context-base", "uniform4f", "y"): True,
    ("web-gl-rendering-context-base", "uniform4f", "z"): True,
    ("web-gl-rendering-context-base", "uniform4f", "w"): True,
    ("web-gl-rendering-context-base", "uniform1iv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform2iv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform3iv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform4iv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform1fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform2fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform3fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform4fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform-matrix-2fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform-matrix-3fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "uniform-matrix-4fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-base", "vertex-attrib-1f", "index"): True,
    ("web-gl-rendering-context-base", "vertex-attrib-2f", "index"): True,
    ("web-gl-rendering-context-base", "vertex-attrib-3f", "index"): True,
    ("web-gl-rendering-context-base", "vertex-attrib-4f", "index"): True,
        ("web-gl-rendering-context-base", "vertex-attrib1fv", "index"): True,
    ("web-gl-rendering-context-base", "vertex-attrib2fv", "index"): True,
    ("web-gl-rendering-context-base", "vertex-attrib3fv", "index"): True,
    ("web-gl-rendering-context-base", "vertex-attrib4fv", "index"): True,
    ("web-gl-rendering-context-base", "enable-vertex-attrib-array", "index"): True,
    ("web-gl-rendering-context-base", "disable-vertex-attrib-array", "index"): True,
    ("web-gl-rendering-context-base", "buffer-data", "target"): True,
    ("web-gl-rendering-context-base", "buffer-sub-data", "target"): True,
    ("web-gl-rendering-context-base", "get-buffer-parameter", "pname"): True,
    ("web-gl-rendering-context-base", "get-framebuffer-attachment-parameter", "attachment"): True,
    ("web-gl-rendering-context-base", "get-framebuffer-attachment-parameter", "pname"): True,
    ("web-gl-rendering-context-base", "get-renderbuffer-parameter", "pname"): True,
    ("web-gl-rendering-context-base", "get-tex-parameter", "pname"): True,
    ("web-gl-rendering-context-base", "get-vertex-attrib", "pname"): True,
    ("web-gl-rendering-context-base", "get-vertex-attrib-offset", "index"): True,
    ("web-gl-rendering-context-base", "get-vertex-attrib-offset", "pname"): True,
    ("web-gl-rendering-context-base", "tex-parameter-f", "target"): True,
    ("web-gl-rendering-context-base", "tex-parameter-f", "pname"): True,
    ("web-gl-rendering-context-base", "tex-parameter-i", "target"): True,
    ("web-gl-rendering-context-base", "tex-parameter-i", "pname"): True,
    # WebGL2
    ("web-gl2-rendering-context-base", "bind-buffer", "target"): True,
    ("web-gl2-rendering-context-base", "bind-framebuffer", "target"): True,
    ("web-gl2-rendering-context-base", "bind-renderbuffer", "target"): True,
    ("web-gl2-rendering-context-base", "bind-texture", "target"): True,
    ("web-gl2-rendering-context-base", "bind-vertex-array", "array"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "disable", "cap"): True,
    ("web-gl2-rendering-context-base", "enable", "cap"): True,
    ("web-gl2-rendering-context-base", "get-buffer-parameter", "target"): True,
    ("web-gl2-rendering-context-base", "get-framebuffer-attachment-parameter", "target"): True,
    ("web-gl2-rendering-context-base", "get-program-parameter", "pname"): True,
    ("web-gl2-rendering-context-base", "get-renderbuffer-parameter", "target"): True,
    ("web-gl2-rendering-context-base", "get-tex-parameter", "target"): True,
    ("web-gl2-rendering-context-base", "get-uniform", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "get-vertex-attrib", "index"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-pointer", "index"): True,
    ("web-gl2-rendering-context-base", "create-query", "target"): True,
    ("web-gl2-rendering-context-base", "delete-query", "query"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "is-query", "query"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "begin-query", "target"): True,
    ("web-gl2-rendering-context-base", "begin-query", "query"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "end-query", "target"): True,
    ("web-gl2-rendering-context-base", "get-query", "target"): True,
    ("web-gl2-rendering-context-base", "get-query", "pname"): True,
    ("web-gl2-rendering-context-base", "get-query-parameter", "query"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "create-sampler", "sampler"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "delete-sampler", "sampler"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "is-sampler", "sampler"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "bind-sampler", "unit"): True,
    ("web-gl2-rendering-context-base", "bind-sampler", "sampler"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "sampler-parameteri", "sampler"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "sampler-parameteri", "pname"): True,
    ("web-gl2-rendering-context-base", "sampler-parameteri", "param"): True,
    ("web-gl2-rendering-context-base", "sampler-parameterf", "sampler"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "sampler-parameterf", "pname"): True,
    ("web-gl2-rendering-context-base", "sampler-parameterf", "param"): True,
    ("web-gl2-rendering-context-base", "get-sampler-parameter", "sampler"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "get-sampler-parameter", "pname"): True,
    ("web-gl2-rendering-context-base", "fence-sync", "condition"): True,
    ("web-gl2-rendering-context-base", "fence-sync", "flags"): True,
    ("web-gl2-rendering-context-base", "delete-sync", "sync"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "is-sync", "sync"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "client-wait-sync", "sync"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "wait-sync", "sync"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "get-sync-parameter", "sync"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "create-transform-feedback", "tf"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "delete-transform-feedback", "tf"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "is-transform-feedback", "tf"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "bind-transform-feedback", "target"): True,
    ("web-gl2-rendering-context-base", "bind-transform-feedback", "id"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "begin-transform-feedback", "primitiveMode"): True,
    ("web-gl2-rendering-context-base", "transform-feedback-varyings", "program"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "get-transform-feedback-varying", "program"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "pause-transform-feedback", "tf"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "resume-transform-feedback", "tf"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "bind-buffer-base", "target"): True,
    ("web-gl2-rendering-context-base", "bind-buffer-base", "buffer"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "bind-buffer-range", "target"): True,
    ("web-gl2-rendering-context-base", "bind-buffer-range", "buffer"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "get-indexed-parameter", "target"): True,
    ("web-gl2-rendering-context-base", "get-uniform-indices", "program"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "get-active-uniforms", "program"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "get-uniform-block-index", "program"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "get-active-uniform-block-parameter", "program"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "get-active-uniform-block-name", "program"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "uniform-block-binding", "program"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "delete-vertex-array", "vertexArray"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "is-vertex-array", "vertexArray"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "draw-arrays-instanced", "mode"): True,
    ("web-gl2-rendering-context-base", "draw-elements-instanced", "mode"): True,
    ("web-gl2-rendering-context-base", "draw-range-elements", "mode"): True,
    ("web-gl2-rendering-context-base", "draw-buffers", "buffers"): "array",
    ("web-gl2-rendering-context-base", "clear-buffer-f", "buffer"): True,
    ("web-gl2-rendering-context-base", "clear-buffer-i", "buffer"): True,
    ("web-gl2-rendering-context-base", "clear-buffer-ui", "buffer"): True,
    ("web-gl2-rendering-context-base", "clear-buffer-fv", "buffer"): True,
    ("web-gl2-rendering-context-base", "clear-buffer-iv", "buffer"): True,
    ("web-gl2-rendering-context-base", "clear-buffer-uiv", "buffer"): True,
    ("web-gl2-rendering-context-base", "tex-image-3d", "target"): True,
    ("web-gl2-rendering-context-base", "tex-image-3d", "level"): True,
    ("web-gl2-rendering-context-base", "tex-sub-image-3d", "target"): True,
    ("web-gl2-rendering-context-base", "tex-sub-image-3d", "level"): True,
    ("web-gl2-rendering-context-base", "copy-tex-sub-image-3d", "target"): True,
    ("web-gl2-rendering-context-base", "copy-tex-sub-image-3d", "level"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-image-3d", "target"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-image-3d", "level"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-sub-image-3d", "target"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-sub-image-3d", "level"): True,
    ("web-gl2-rendering-context-base", "get-buffer-sub-data", "target"): True,
    ("web-gl2-rendering-context-base", "get-internalformat-parameter", "target"): True,
    ("web-gl2-rendering-context-base", "get-internalformat-parameter", "pname"): True,
    ("web-gl2-rendering-context-base", "invalidate-framebuffer", "target"): True,
    ("web-gl2-rendering-context-base", "invalidate-sub-framebuffer", "target"): True,
    ("web-gl2-rendering-context-base", "read-buffer", "src"): True,
    ("web-gl2-rendering-context-base", "renderbuffer-storage-multisample", "target"): True,
    ("web-gl2-rendering-context-base", "tex-storage-2d", "target"): True,
    ("web-gl2-rendering-context-base", "tex-storage-3d", "target"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-divisor", "index"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i-pointer", "index"): True,
    ("web-gl2-rendering-context-base", "uniform1ui", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "uniform2ui", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "uniform3ui", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "uniform4ui", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "uniform1uiv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "uniform2uiv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "uniform3uiv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "uniform4uiv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "uniform-matrix2x3fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "uniform-matrix2x4fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "uniform-matrix3x2fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "uniform-matrix3x4fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "uniform-matrix4x2fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "uniform-matrix4x3fv", "location"): "optional-handle:web-gl-uniform-location",
    # WebGL2 uniformMatrix* transpose parameters
    ("web-gl2-rendering-context-base", "uniform-matrix2x3fv", "transpose"): "boolean",
    ("web-gl2-rendering-context-base", "uniform-matrix2x4fv", "transpose"): "boolean",
    ("web-gl2-rendering-context-base", "uniform-matrix3x2fv", "transpose"): "boolean",
    ("web-gl2-rendering-context-base", "uniform-matrix3x4fv", "transpose"): "boolean",
    ("web-gl2-rendering-context-base", "uniform-matrix4x2fv", "transpose"): "boolean",
    ("web-gl2-rendering-context-base", "uniform-matrix4x3fv", "transpose"): "boolean",
    # WebGL2RenderingContextOverloads
    ("web-gl2-rendering-context-overloads", "uniform-matrix2fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-overloads", "uniform-matrix2fv", "transpose"): "boolean",
    ("web-gl2-rendering-context-overloads", "uniform-matrix3fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-overloads", "uniform-matrix3fv", "transpose"): "boolean",
    ("web-gl2-rendering-context-overloads", "uniform-matrix4fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-overloads", "uniform-matrix4fv", "transpose"): "boolean",
    # WebGLRenderingContextBase - vertexAttrib*fv values parameter
    ("web-gl-rendering-context-base", "vertex-attrib1fv", "values"): "handle:float-32-list",
    ("web-gl-rendering-context-base", "vertex-attrib2fv", "values"): "handle:float-32-list",
    ("web-gl-rendering-context-base", "vertex-attrib3fv", "values"): "handle:float-32-list",
    ("web-gl-rendering-context-base", "vertex-attrib4fv", "values"): "handle:float-32-list",
    # WebGLRenderingContextOverloads - uniform*fv/uniform*iv location parameter
    ("web-gl-rendering-context-overloads", "uniform1fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-overloads", "uniform2fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-overloads", "uniform3fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-overloads", "uniform4fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-overloads", "uniform1iv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-overloads", "uniform2iv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-overloads", "uniform3iv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl-rendering-context-overloads", "uniform4iv", "location"): "optional-handle:web-gl-uniform-location",
    # WebGLRenderingContextOverloads - uniform*fv/uniform*iv v parameter
    ("web-gl-rendering-context-overloads", "uniform1fv", "v"): "handle:float-32-list",
    ("web-gl-rendering-context-overloads", "uniform2fv", "v"): "handle:float-32-list",
    ("web-gl-rendering-context-overloads", "uniform3fv", "v"): "handle:float-32-list",
    ("web-gl-rendering-context-overloads", "uniform4fv", "v"): "handle:float-32-list",
    ("web-gl-rendering-context-overloads", "uniform1iv", "v"): "handle:int-32-list",
    ("web-gl-rendering-context-overloads", "uniform2iv", "v"): "handle:int-32-list",
    ("web-gl-rendering-context-overloads", "uniform3iv", "v"): "handle:int-32-list",
    ("web-gl-rendering-context-overloads", "uniform4iv", "v"): "handle:int-32-list",
    ("web-gl-rendering-context-overloads", "uniform-matrix2fv", "value"): "handle:float-32-list",
    ("web-gl-rendering-context-overloads", "uniform-matrix3fv", "value"): "handle:float-32-list",
    ("web-gl-rendering-context-overloads", "uniform-matrix4fv", "value"): "handle:float-32-list",
    ("web-gl-rendering-context-overloads", "buffer-data", "target"): True,
    ("web-gl-rendering-context-overloads", "buffer-data", "size"): True,
    ("web-gl-rendering-context-overloads", "buffer-data", "usage"): True,
    ("web-gl-rendering-context-overloads", "buffer-sub-data", "target"): True,
    ("web-gl-rendering-context-overloads", "buffer-sub-data", "offset"): True,
    ("web-gl-rendering-context-overloads", "compressed-tex-image2-d", "target"): True,
    ("web-gl-rendering-context-overloads", "compressed-tex-image2-d", "level"): True,
    ("web-gl-rendering-context-overloads", "compressed-tex-image2-d", "internalformat"): True,
    ("web-gl-rendering-context-overloads", "compressed-tex-image2-d", "width"): True,
    ("web-gl-rendering-context-overloads", "compressed-tex-image2-d", "height"): True,
    ("web-gl-rendering-context-overloads", "compressed-tex-image2-d", "border"): True,
    ("web-gl-rendering-context-overloads", "compressed-tex-sub-image2-d", "target"): True,
    ("web-gl-rendering-context-overloads", "compressed-tex-sub-image2-d", "level"): True,
    ("web-gl-rendering-context-overloads", "compressed-tex-sub-image2-d", "xoffset"): True,
    ("web-gl-rendering-context-overloads", "compressed-tex-sub-image2-d", "yoffset"): True,
    ("web-gl-rendering-context-overloads", "compressed-tex-sub-image2-d", "width"): True,
    ("web-gl-rendering-context-overloads", "compressed-tex-sub-image2-d", "height"): True,
    ("web-gl-rendering-context-overloads", "compressed-tex-sub-image2-d", "format"): True,
    ("web-gl-rendering-context-overloads", "tex-image2-d", "target"): True,
    ("web-gl-rendering-context-overloads", "tex-image2-d", "level"): True,
    ("web-gl-rendering-context-overloads", "tex-image2-d", "internalformat"): True,
    ("web-gl-rendering-context-overloads", "tex-image2-d", "width"): True,
    ("web-gl-rendering-context-overloads", "tex-image2-d", "height"): True,
    ("web-gl-rendering-context-overloads", "tex-image2-d", "border"): True,
    ("web-gl-rendering-context-overloads", "tex-image2-d", "format"): True,
    ("web-gl-rendering-context-overloads", "tex-image2-d", "type"): True,
    ("web-gl-rendering-context-overloads", "tex-sub-image2-d", "target"): True,
    ("web-gl-rendering-context-overloads", "tex-sub-image2-d", "level"): True,
    ("web-gl-rendering-context-overloads", "tex-sub-image2-d", "xoffset"): True,
    ("web-gl-rendering-context-overloads", "tex-sub-image2-d", "yoffset"): True,
    ("web-gl-rendering-context-overloads", "tex-sub-image2-d", "width"): True,
    ("web-gl-rendering-context-overloads", "tex-sub-image2-d", "height"): True,
    ("web-gl-rendering-context-overloads", "tex-sub-image2-d", "format"): True,
    ("web-gl-rendering-context-overloads", "tex-sub-image2-d", "type"): True,
    ("web-gl-rendering-context-overloads", "read-pixels", "x"): True,
    ("web-gl-rendering-context-overloads", "read-pixels", "y"): True,
    ("web-gl-rendering-context-overloads", "read-pixels", "width"): True,
    ("web-gl-rendering-context-overloads", "read-pixels", "height"): True,
    ("web-gl-rendering-context-overloads", "read-pixels", "format"): True,
    ("web-gl-rendering-context-overloads", "read-pixels", "type"): True,
    # WebGL2 - more missing conversions
    ("web-gl2-rendering-context-base", "tex-image3-d", "target"): True,
    ("web-gl2-rendering-context-base", "tex-image3-d", "level"): True,
    ("web-gl2-rendering-context-base", "tex-image3-d", "internalformat"): True,
    ("web-gl2-rendering-context-base", "tex-image3-d", "width"): True,
    ("web-gl2-rendering-context-base", "tex-image3-d", "height"): True,
    ("web-gl2-rendering-context-base", "tex-image3-d", "depth"): True,
    ("web-gl2-rendering-context-base", "tex-image3-d", "border"): True,
    ("web-gl2-rendering-context-base", "tex-image3-d", "format"): True,
    ("web-gl2-rendering-context-base", "tex-image3-d", "type"): True,
    ("web-gl2-rendering-context-base", "tex-image3-d", "pbo-offset"): True,
    ("web-gl2-rendering-context-base", "tex-sub-image3-d", "target"): True,
    ("web-gl2-rendering-context-base", "tex-sub-image3-d", "level"): True,
    ("web-gl2-rendering-context-base", "tex-sub-image3-d", "xoffset"): True,
    ("web-gl2-rendering-context-base", "tex-sub-image3-d", "yoffset"): True,
    ("web-gl2-rendering-context-base", "tex-sub-image3-d", "zoffset"): True,
    ("web-gl2-rendering-context-base", "tex-sub-image3-d", "width"): True,
    ("web-gl2-rendering-context-base", "tex-sub-image3-d", "height"): True,
    ("web-gl2-rendering-context-base", "tex-sub-image3-d", "depth"): True,
    ("web-gl2-rendering-context-base", "tex-sub-image3-d", "format"): True,
    ("web-gl2-rendering-context-base", "tex-sub-image3-d", "type"): True,
    ("web-gl2-rendering-context-base", "tex-sub-image3-d", "pbo-offset"): True,
    ("web-gl2-rendering-context-base", "copy-tex-sub-image3-d", "target"): True,
    ("web-gl2-rendering-context-base", "copy-tex-sub-image3-d", "level"): True,
    ("web-gl2-rendering-context-base", "copy-tex-sub-image3-d", "xoffset"): True,
    ("web-gl2-rendering-context-base", "copy-tex-sub-image3-d", "yoffset"): True,
    ("web-gl2-rendering-context-base", "copy-tex-sub-image3-d", "zoffset"): True,
    ("web-gl2-rendering-context-base", "copy-tex-sub-image3-d", "x"): True,
    ("web-gl2-rendering-context-base", "copy-tex-sub-image3-d", "y"): True,
    ("web-gl2-rendering-context-base", "copy-tex-sub-image3-d", "width"): True,
    ("web-gl2-rendering-context-base", "copy-tex-sub-image3-d", "height"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-image3-d", "target"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-image3-d", "level"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-image3-d", "internalformat"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-image3-d", "width"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-image3-d", "height"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-image3-d", "depth"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-image3-d", "border"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-image3-d", "image-size"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-image3-d", "offset"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-sub-image3-d", "target"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-sub-image3-d", "level"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-sub-image3-d", "xoffset"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-sub-image3-d", "yoffset"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-sub-image3-d", "zoffset"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-sub-image3-d", "width"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-sub-image3-d", "height"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-sub-image3-d", "depth"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-sub-image3-d", "format"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-sub-image3-d", "image-size"): True,
    ("web-gl2-rendering-context-base", "compressed-tex-sub-image3-d", "offset"): True,
    ("web-gl2-rendering-context-base", "framebuffer-texture-layer", "target"): True,
    ("web-gl2-rendering-context-base", "framebuffer-texture-layer", "attachment"): True,
    ("web-gl2-rendering-context-base", "framebuffer-texture-layer", "texture"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "framebuffer-texture-layer", "level"): True,
    ("web-gl2-rendering-context-base", "framebuffer-texture-layer", "layer"): True,
    ("web-gl2-rendering-context-base", "renderbuffer-storage-multisample", "samples"): True,
    ("web-gl2-rendering-context-base", "renderbuffer-storage-multisample", "internalformat"): True,
    ("web-gl2-rendering-context-base", "renderbuffer-storage-multisample", "width"): True,
    ("web-gl2-rendering-context-base", "renderbuffer-storage-multisample", "height"): True,
    ("web-gl2-rendering-context-base", "tex-storage2-d", "target"): True,
    ("web-gl2-rendering-context-base", "tex-storage2-d", "levels"): True,
    ("web-gl2-rendering-context-base", "tex-storage2-d", "internalformat"): True,
    ("web-gl2-rendering-context-base", "tex-storage2-d", "width"): True,
    ("web-gl2-rendering-context-base", "tex-storage2-d", "height"): True,
    ("web-gl2-rendering-context-base", "tex-storage3-d", "target"): True,
    ("web-gl2-rendering-context-base", "tex-storage3-d", "levels"): True,
    ("web-gl2-rendering-context-base", "tex-storage3-d", "internalformat"): True,
    ("web-gl2-rendering-context-base", "tex-storage3-d", "width"): True,
    ("web-gl2-rendering-context-base", "tex-storage3-d", "height"): True,
    ("web-gl2-rendering-context-base", "tex-storage3-d", "depth"): True,
    ("web-gl2-rendering-context-base", "get-frag-data-location", "program"): "handle:web-gl-object",
    ("web-gl2-rendering-context-base", "uniform1ui", "v0"): True,
    ("web-gl2-rendering-context-base", "uniform2ui", "v0"): True,
    ("web-gl2-rendering-context-base", "uniform2ui", "v1"): True,
    ("web-gl2-rendering-context-base", "uniform3ui", "v0"): True,
    ("web-gl2-rendering-context-base", "uniform3ui", "v1"): True,
    ("web-gl2-rendering-context-base", "uniform3ui", "v2"): True,
    ("web-gl2-rendering-context-base", "uniform4ui", "v0"): True,
    ("web-gl2-rendering-context-base", "uniform4ui", "v1"): True,
    ("web-gl2-rendering-context-base", "uniform4ui", "v2"): True,
    ("web-gl2-rendering-context-base", "uniform4ui", "v3"): True,
    ("web-gl2-rendering-context-base", "uniform1uiv", "data"): "handle:uint-32-list",
    ("web-gl2-rendering-context-base", "uniform1uiv", "src-offset"): True,
    ("web-gl2-rendering-context-base", "uniform1uiv", "src-length"): True,
    ("web-gl2-rendering-context-base", "uniform2uiv", "data"): "handle:uint-32-list",
    ("web-gl2-rendering-context-base", "uniform2uiv", "src-offset"): True,
    ("web-gl2-rendering-context-base", "uniform2uiv", "src-length"): True,
    ("web-gl2-rendering-context-base", "uniform3uiv", "data"): "handle:uint-32-list",
    ("web-gl2-rendering-context-base", "uniform3uiv", "src-offset"): True,
    ("web-gl2-rendering-context-base", "uniform3uiv", "src-length"): True,
    ("web-gl2-rendering-context-base", "uniform4uiv", "data"): "handle:uint-32-list",
    ("web-gl2-rendering-context-base", "uniform4uiv", "src-offset"): True,
    ("web-gl2-rendering-context-base", "uniform4uiv", "src-length"): True,
    ("web-gl2-rendering-context-base", "uniform-matrix3x2fv", "data"): "handle:float-32-list",
    ("web-gl2-rendering-context-base", "uniform-matrix3x2fv", "src-offset"): True,
    ("web-gl2-rendering-context-base", "uniform-matrix3x2fv", "src-length"): True,
    ("web-gl2-rendering-context-base", "uniform-matrix4x2fv", "data"): "handle:float-32-list",
    ("web-gl2-rendering-context-base", "uniform-matrix4x2fv", "src-offset"): True,
    ("web-gl2-rendering-context-base", "uniform-matrix4x2fv", "src-length"): True,
    ("web-gl2-rendering-context-base", "uniform-matrix2x3fv", "data"): "handle:float-32-list",
    ("web-gl2-rendering-context-base", "uniform-matrix2x3fv", "src-offset"): True,
    ("web-gl2-rendering-context-base", "uniform-matrix2x3fv", "src-length"): True,
    ("web-gl2-rendering-context-base", "uniform-matrix4x3fv", "data"): "handle:float-32-list",
    ("web-gl2-rendering-context-base", "uniform-matrix4x3fv", "src-offset"): True,
    ("web-gl2-rendering-context-base", "uniform-matrix4x3fv", "src-length"): True,
    ("web-gl2-rendering-context-base", "uniform-matrix2x4fv", "data"): "handle:float-32-list",
    ("web-gl2-rendering-context-base", "uniform-matrix2x4fv", "src-offset"): True,
    ("web-gl2-rendering-context-base", "uniform-matrix2x4fv", "src-length"): True,
    ("web-gl2-rendering-context-base", "uniform-matrix3x4fv", "data"): "handle:float-32-list",
    ("web-gl2-rendering-context-base", "uniform-matrix3x4fv", "src-offset"): True,
    ("web-gl2-rendering-context-base", "uniform-matrix3x4fv", "src-length"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i4i", "index"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i4i", "x"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i4i", "y"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i4i", "z"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i4i", "w"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i4iv", "index"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i4iv", "values"): "handle:int-32-list",
    ("web-gl2-rendering-context-base", "vertex-attrib-i4ui", "index"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i4ui", "x"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i4ui", "y"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i4ui", "z"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i4ui", "w"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i4uiv", "index"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i4uiv", "values"): "handle:uint-32-list",
    ("web-gl2-rendering-context-base", "vertex-attrib-i-pointer", "size"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i-pointer", "type"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i-pointer", "stride"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i-pointer", "offset"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-divisor", "divisor"): True,
    ("web-gl2-rendering-context-base", "draw-arrays-instanced", "first"): True,
    ("web-gl2-rendering-context-base", "draw-arrays-instanced", "count"): True,
    ("web-gl2-rendering-context-base", "draw-arrays-instanced", "instance-count"): True,
    ("web-gl2-rendering-context-base", "draw-elements-instanced", "count"): True,
    ("web-gl2-rendering-context-base", "draw-elements-instanced", "type"): True,
    ("web-gl2-rendering-context-base", "draw-elements-instanced", "offset"): True,
    ("web-gl2-rendering-context-base", "draw-elements-instanced", "instance-count"): True,
    ("web-gl2-rendering-context-base", "draw-range-elements", "start"): True,
    ("web-gl2-rendering-context-base", "draw-range-elements", "end"): True,
    ("web-gl2-rendering-context-base", "draw-range-elements", "count"): True,
    ("web-gl2-rendering-context-base", "draw-range-elements", "type"): True,
    ("web-gl2-rendering-context-base", "draw-range-elements", "offset"): True,
    ("web-gl2-rendering-context-base", "clear-bufferfi", "buffer"): True,
    ("web-gl2-rendering-context-base", "clear-bufferfi", "drawbuffer"): True,
    ("web-gl2-rendering-context-base", "clear-bufferfi", "depth"): True,
    ("web-gl2-rendering-context-base", "clear-bufferfi", "stencil"): True,
    ("web-gl2-rendering-context-base", "clear-bufferfv", "buffer"): True,
    ("web-gl2-rendering-context-base", "clear-bufferfv", "drawbuffer"): True,
    ("web-gl2-rendering-context-base", "clear-bufferfv", "values"): "handle:float-32-list",
    ("web-gl2-rendering-context-base", "clear-bufferfv", "src-offset"): True,
    ("web-gl2-rendering-context-base", "clear-bufferiv", "buffer"): True,
    ("web-gl2-rendering-context-base", "clear-bufferiv", "drawbuffer"): True,
    ("web-gl2-rendering-context-base", "clear-bufferiv", "values"): "handle:int-32-list",
    ("web-gl2-rendering-context-base", "clear-bufferiv", "src-offset"): True,
    ("web-gl2-rendering-context-base", "clear-bufferuiv", "buffer"): True,
    ("web-gl2-rendering-context-base", "clear-bufferuiv", "drawbuffer"): True,
    ("web-gl2-rendering-context-base", "clear-bufferuiv", "values"): "handle:uint-32-list",
    ("web-gl2-rendering-context-base", "clear-bufferuiv", "src-offset"): True,
    ("web-gl2-rendering-context-base", "get-buffer-sub-data", "target"): True,
    ("web-gl2-rendering-context-base", "get-buffer-sub-data", "src-byte-offset"): True,
    ("web-gl2-rendering-context-base", "get-buffer-sub-data", "dst-offset"): True,
    ("web-gl2-rendering-context-base", "get-buffer-sub-data", "length"): True,
    # WebGL missing conversions
    ("web-gl-rendering-context-base", "copy-tex-image2-d", "target"): True,
    ("web-gl-rendering-context-base", "copy-tex-image2-d", "level"): True,
    ("web-gl-rendering-context-base", "copy-tex-image2-d", "internalformat"): True,
    ("web-gl-rendering-context-base", "copy-tex-image2-d", "x"): True,
    ("web-gl-rendering-context-base", "copy-tex-image2-d", "y"): True,
    ("web-gl-rendering-context-base", "copy-tex-image2-d", "width"): True,
    ("web-gl-rendering-context-base", "copy-tex-image2-d", "height"): True,
    ("web-gl-rendering-context-base", "copy-tex-image2-d", "border"): True,
    ("web-gl-rendering-context-base", "copy-tex-sub-image2-d", "target"): True,
    ("web-gl-rendering-context-base", "copy-tex-sub-image2-d", "level"): True,
    ("web-gl-rendering-context-base", "copy-tex-sub-image2-d", "xoffset"): True,
    ("web-gl-rendering-context-base", "copy-tex-sub-image2-d", "yoffset"): True,
    ("web-gl-rendering-context-base", "copy-tex-sub-image2-d", "x"): True,
    ("web-gl-rendering-context-base", "copy-tex-sub-image2-d", "y"): True,
    ("web-gl-rendering-context-base", "copy-tex-sub-image2-d", "width"): True,
    ("web-gl-rendering-context-base", "copy-tex-sub-image2-d", "height"): True,
    ("web-gl-rendering-context-base", "framebuffer-texture2-d", "target"): True,
    ("web-gl-rendering-context-base", "framebuffer-texture2-d", "attachment"): True,
    ("web-gl-rendering-context-base", "framebuffer-texture2-d", "textarget"): True,
    ("web-gl-rendering-context-base", "framebuffer-texture2-d", "texture"): "optional-handle:web-gl-object",
    ("web-gl-rendering-context-base", "framebuffer-texture2-d", "level"): True,
    ("web-gl-rendering-context-base", "is-enabled", "cap"): True,
    ("web-gl-rendering-context-base", "line-width", "width"): True,
    ("web-gl-rendering-context-base", "pixel-storei", "pname"): True,
    ("web-gl-rendering-context-base", "pixel-storei", "param"): True,
    ("web-gl-rendering-context-base", "polygon-offset", "factor"): True,
    ("web-gl-rendering-context-base", "polygon-offset", "units"): True,
    ("web-gl-rendering-context-base", "get-query-parameter", "pname"): True,
    # WebGL2 missing conversions
    ("web-gl2-rendering-context-base", "copy-buffer-sub-data", "read-target"): True,
    ("web-gl2-rendering-context-base", "copy-buffer-sub-data", "write-target"): True,
    ("web-gl2-rendering-context-base", "copy-buffer-sub-data", "read-offset"): True,
    ("web-gl2-rendering-context-base", "copy-buffer-sub-data", "write-offset"): True,
    ("web-gl2-rendering-context-base", "copy-buffer-sub-data", "size"): True,
    ("web-gl2-rendering-context-base", "blit-framebuffer", "src-x0"): True,
    ("web-gl2-rendering-context-base", "blit-framebuffer", "src-y0"): True,
    ("web-gl2-rendering-context-base", "blit-framebuffer", "src-x1"): True,
    ("web-gl2-rendering-context-base", "blit-framebuffer", "src-y1"): True,
    ("web-gl2-rendering-context-base", "blit-framebuffer", "dst-x0"): True,
    ("web-gl2-rendering-context-base", "blit-framebuffer", "dst-y0"): True,
    ("web-gl2-rendering-context-base", "blit-framebuffer", "dst-x1"): True,
    ("web-gl2-rendering-context-base", "blit-framebuffer", "dst-y1"): True,
    ("web-gl2-rendering-context-base", "blit-framebuffer", "mask"): True,
    ("web-gl2-rendering-context-base", "blit-framebuffer", "filter"): True,
    ("web-gl2-rendering-context-base", "framebuffer-texture-layer", "level"): True,
    ("web-gl2-rendering-context-base", "framebuffer-texture-layer", "layer"): True,
    ("web-gl2-rendering-context-base", "bind-transform-feedback", "target"): True,
    ("web-gl2-rendering-context-base", "begin-transform-feedback", "primitive-mode"): True,
    ("web-gl2-rendering-context-base", "bind-vertex-array", "array"): "optional-handle:web-gl-object",
    # WebGL missing conversions for methods still using bigint
    ("web-gl-rendering-context-base", "get-shader-precision-format", "shadertype"): True,
    ("web-gl-rendering-context-base", "get-shader-precision-format", "precisiontype"): True,
    ("web-gl-rendering-context-base", "renderbuffer-storage", "internalformat"): True,
    ("web-gl-rendering-context-base", "renderbuffer-storage", "width"): True,
    ("web-gl-rendering-context-base", "renderbuffer-storage", "height"): True,
    ("web-gl-rendering-context-base", "sample-coverage", "value"): True,
    ("web-gl-rendering-context-base", "sample-coverage", "invert"): "boolean",
    ("web-gl-rendering-context-base", "scissor", "x"): True,
    ("web-gl-rendering-context-base", "scissor", "y"): True,
    ("web-gl-rendering-context-base", "scissor", "width"): True,
    ("web-gl-rendering-context-base", "scissor", "height"): True,
    ("web-gl-rendering-context-base", "stencil-func", "ref"): True,
    ("web-gl-rendering-context-base", "stencil-func", "mask"): True,
    ("web-gl-rendering-context-base", "stencil-func-separate", "face"): True,
    ("web-gl-rendering-context-base", "stencil-func-separate", "func"): True,
    ("web-gl-rendering-context-base", "stencil-func-separate", "ref"): True,
    ("web-gl-rendering-context-base", "stencil-func-separate", "mask"): True,
    ("web-gl-rendering-context-base", "validate-program", "program"): "handle:web-gl-object",
    ("web-gl-rendering-context-base", "vertex-attrib1f", "index"): True,
    ("web-gl-rendering-context-base", "vertex-attrib1f", "x"): True,
    ("web-gl-rendering-context-base", "vertex-attrib2f", "index"): True,
    ("web-gl-rendering-context-base", "vertex-attrib2f", "x"): True,
    ("web-gl-rendering-context-base", "vertex-attrib2f", "y"): True,
    ("web-gl-rendering-context-base", "vertex-attrib3f", "index"): True,
    ("web-gl-rendering-context-base", "vertex-attrib3f", "x"): True,
    ("web-gl-rendering-context-base", "vertex-attrib3f", "y"): True,
    ("web-gl-rendering-context-base", "vertex-attrib3f", "z"): True,
    ("web-gl-rendering-context-base", "vertex-attrib4f", "index"): True,
    ("web-gl-rendering-context-base", "vertex-attrib4f", "x"): True,
    ("web-gl-rendering-context-base", "vertex-attrib4f", "y"): True,
    ("web-gl-rendering-context-base", "vertex-attrib4f", "z"): True,
    ("web-gl-rendering-context-base", "vertex-attrib4f", "w"): True,
    ("web-gl-rendering-context-base", "stencil-mask-separate", "face"): True,
    ("web-gl-rendering-context-base", "stencil-mask-separate", "mask"): True,
    ("web-gl-rendering-context-base", "vertex-attrib-pointer", "index"): True,
    ("web-gl-rendering-context-base", "vertex-attrib-pointer", "size"): True,
    ("web-gl-rendering-context-base", "vertex-attrib-pointer", "type"): True,
    ("web-gl-rendering-context-base", "vertex-attrib-pointer", "normalized"): "boolean",
    ("web-gl-rendering-context-base", "vertex-attrib-pointer", "stride"): True,
    ("web-gl-rendering-context-base", "vertex-attrib-pointer", "offset"): True,
    ("web-gl-rendering-context-base", "viewport", "x"): True,
    ("web-gl-rendering-context-base", "viewport", "y"): True,
    ("web-gl-rendering-context-base", "viewport", "width"): True,
    ("web-gl-rendering-context-base", "viewport", "height"): True,
    ("web-gl-rendering-context-base", "tex-parameterf", "target"): True,
    ("web-gl-rendering-context-base", "tex-parameterf", "pname"): True,
    ("web-gl-rendering-context-base", "tex-parameterf", "param"): True,
    ("web-gl-rendering-context-base", "tex-parameteri", "target"): True,
    ("web-gl-rendering-context-base", "tex-parameteri", "pname"): True,
    ("web-gl-rendering-context-base", "tex-parameteri", "param"): True,
    # WebGL2 missing Number() conversions
    ("web-gl2-rendering-context-base", "get-internalformat-parameter", "target"): True,
    ("web-gl2-rendering-context-base", "get-internalformat-parameter", "internalformat"): True,
    ("web-gl2-rendering-context-base", "get-internalformat-parameter", "pname"): True,
    ("web-gl2-rendering-context-base", "uniform-1ui", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "uniform-2ui", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "uniform-3ui", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "uniform-4ui", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-base", "delete-transform-feedback", "tf"): "optional-handle:web-gl-object",
    # More WebGL2 Number() conversions
    ("web-gl2-rendering-context-base", "get-query-parameter", "pname"): True,
    ("web-gl2-rendering-context-base", "get-sync-parameter", "pname"): True,
    ("web-gl2-rendering-context-base", "bind-buffer-base", "index"): True,
    ("web-gl2-rendering-context-base", "bind-buffer-range", "index"): True,
    ("web-gl2-rendering-context-base", "bind-buffer-range", "offset"): True,
    ("web-gl2-rendering-context-base", "bind-buffer-range", "size"): True,
    ("web-gl2-rendering-context-base", "get-indexed-parameter", "index"): True,
    ("web-gl2-rendering-context-base", "bind-vertex-array", "array"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "delete-vertex-array", "vertex-array"): "optional-handle:web-gl-object",
    ("web-gl2-rendering-context-base", "begin-query", "target"): True,
    ("web-gl2-rendering-context-base", "end-query", "target"): True,
    ("web-gl2-rendering-context-base", "get-query", "target"): True,
    ("web-gl2-rendering-context-base", "get-query", "pname"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-divisor", "index"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-divisor", "divisor"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i-pointer", "index"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i-pointer", "size"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i-pointer", "type"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i-pointer", "stride"): True,
    ("web-gl2-rendering-context-base", "vertex-attrib-i-pointer", "offset"): True,
    # More WebGL2 Number() conversions for sync and uniform block operations
    ("web-gl2-rendering-context-base", "client-wait-sync", "flags"): True,
    ("web-gl2-rendering-context-base", "client-wait-sync", "timeout"): True,
    ("web-gl2-rendering-context-base", "wait-sync", "flags"): True,
    ("web-gl2-rendering-context-base", "wait-sync", "timeout"): True,
    ("web-gl2-rendering-context-base", "get-active-uniform-block-parameter", "uniform-block-index"): True,
    ("web-gl2-rendering-context-base", "get-active-uniform-block-parameter", "pname"): True,
    ("web-gl2-rendering-context-base", "get-active-uniform-block-name", "uniform-block-index"): True,
    ("web-gl2-rendering-context-base", "uniform-block-binding", "uniform-block-index"): True,
    ("web-gl2-rendering-context-base", "uniform-block-binding", "uniform-block-binding"): True,
    ("web-gl2-rendering-context-base", "draw-range-elements", "start"): True,
    ("web-gl2-rendering-context-base", "draw-range-elements", "end"): True,
    ("web-gl2-rendering-context-base", "draw-range-elements", "count"): True,
    ("web-gl2-rendering-context-base", "draw-range-elements", "type"): True,
    ("web-gl2-rendering-context-base", "draw-range-elements", "offset"): True,
    ("web-gl2-rendering-context-base", "draw-arrays-instanced", "first"): True,
    ("web-gl2-rendering-context-base", "draw-arrays-instanced", "count"): True,
    ("web-gl2-rendering-context-base", "draw-arrays-instanced", "instance-count"): True,
    ("web-gl2-rendering-context-base", "draw-elements-instanced", "count"): True,
    ("web-gl2-rendering-context-base", "draw-elements-instanced", "type"): True,
    ("web-gl2-rendering-context-base", "draw-elements-instanced", "offset"): True,
    ("web-gl2-rendering-context-base", "draw-elements-instanced", "instance-count"): True,
    ("web-gl2-rendering-context-base", "clear-bufferfi", "buffer"): True,
    ("web-gl2-rendering-context-base", "clear-bufferfi", "drawbuffer"): True,
    ("web-gl2-rendering-context-base", "clear-bufferfi", "depth"): True,
    ("web-gl2-rendering-context-base", "clear-bufferfi", "stencil"): True,
    ("web-gl2-rendering-context-base", "read-pixels", "x"): True,
    ("web-gl2-rendering-context-base", "read-pixels", "y"): True,
    ("web-gl2-rendering-context-base", "read-pixels", "width"): True,
    ("web-gl2-rendering-context-base", "read-pixels", "height"): True,
    ("web-gl2-rendering-context-base", "read-pixels", "format"): True,
    ("web-gl2-rendering-context-base", "read-pixels", "type"): True,
    # WebGL2 methods still missing Number() conversions
    ("web-gl2-rendering-context-base", "get-transform-feedback-varying", "index"): True,
    ("web-gl2-rendering-context-base", "bind-buffer-base", "index"): True,
    ("web-gl2-rendering-context-base", "bind-buffer-range", "index"): True,
    ("web-gl2-rendering-context-base", "bind-buffer-range", "offset"): True,
    ("web-gl2-rendering-context-base", "bind-buffer-range", "size"): True,
    ("web-gl2-rendering-context-base", "begin-transform-feedback", "primitive-mode"): True,
    ("web-gl2-rendering-context-base", "draw-buffers", "buffers"): "array",
    # WebGL2RenderingContextOverloads - buffer sub data needs Number()
    ("web-gl2-rendering-context-overloads", "buffer-sub-data", "target"): True,
    ("web-gl2-rendering-context-overloads", "buffer-sub-data", "dst-byte-offset"): True,
    ("web-gl2-rendering-context-overloads", "tex-image2-d", "target"): True,
    ("web-gl2-rendering-context-overloads", "tex-image2-d", "level"): True,
    ("web-gl2-rendering-context-overloads", "tex-image2-d", "internalformat"): True,
    ("web-gl2-rendering-context-overloads", "tex-image2-d", "width"): True,
    ("web-gl2-rendering-context-overloads", "tex-image2-d", "height"): True,
    ("web-gl2-rendering-context-overloads", "tex-image2-d", "border"): True,
    ("web-gl2-rendering-context-overloads", "tex-image2-d", "format"): True,
    ("web-gl2-rendering-context-overloads", "tex-image2-d", "type"): True,
    # Document methods - node parameter needs handle lookup
    ("document", "import-node", "node"): "handle:node",
    ("document", "adopt-node", "node"): "handle:node",
    # Element methods - attr parameter needs handle lookup
    ("element", "set-attribute-node", "attr"): "handle:attr",
    ("element", "set-attribute-node-ns", "attr"): "handle:attr",
    ("element", "remove-attribute-node", "attr"): "handle:attr",
    # Element attachShadow - init is a dictionary
    ("element", "attach-shadow", "init"): "dictionary:ShadowRootInit",
    # Range methods - node parameter needs handle lookup
    ("range", "set-start", "node"): "handle:node",
    ("range", "set-end", "node"): "handle:node",
    ("range", "set-start-before", "node"): "handle:node",
    ("range", "set-start-after", "node"): "handle:node",
    ("range", "set-end-before", "node"): "handle:node",
    ("range", "set-end-after", "node"): "handle:node",
    ("range", "insert-node", "node"): "handle:node",
    ("range", "surround-contents", "new-parent"): "handle:node",
    ("range", "clone-contents"): "handle:document-fragment",
    # Node methods - node parameters need handle lookup
    ("node", "insert-before", "node"): "handle:node",
    ("node", "append-child", "node"): "handle:node",
    ("node", "replace-child", "node"): "handle:node",
    ("node", "remove-child", "node"): "handle:node",
    # Window scroll methods - options is a dictionary
    ("window", "scroll", "options"): "dictionary:ScrollToOptions | undefined",
    ("window", "scroll-to", "options"): "dictionary:ScrollToOptions | undefined",
    ("window", "scroll-by", "options"): "dictionary:ScrollToOptions | undefined",
    # Document write methods - text is string from bigint[]
    ("document", "write", "text"): "string-from-array",
    ("document", "write-ln", "text"): "string-from-array",
    # DOMImplementation createDocumentType
    ("dom-implementation", "create-document-type", "qualified-name"): "string",
    ("dom-implementation", "create-document-type", "public-id"): "string",
    ("dom-implementation", "create-document-type", "system-id"): "string",
    # Canvas texImage2D - source is AllowSharedBufferSource
    ("canvas-rendering-context", "tex-image-2d", "source"): "buffer-source",
    # EncodedAudioChunk/EncodedVideoChunk copyTo - destination is buffer
    ("encoded-audio-chunk", "copy-to", "destination"): "buffer-source",
    ("encoded-video-chunk", "copy-to", "destination"): "buffer-source",
    # AudioData/VideoFrame copyTo - destination is buffer
    ("audio-data", "copy-to", "destination"): "buffer-source",
    ("video-frame", "copy-to", "destination"): "buffer-source",
    # SubtleCrypto methods - algorithm is dictionary
    ("subtle-crypto", "encrypt", "algorithm"): "dictionary:AlgorithmIdentifier",
    ("subtle-crypto", "encrypt", "key"): "handle:crypto-key",
    ("subtle-crypto", "decrypt", "algorithm"): "dictionary:AlgorithmIdentifier",
    ("subtle-crypto", "decrypt", "key"): "handle:crypto-key",
    ("subtle-crypto", "sign", "algorithm"): "dictionary:AlgorithmIdentifier",
    ("subtle-crypto", "sign", "key"): "handle:crypto-key",
    ("subtle-crypto", "verify", "algorithm"): "dictionary:AlgorithmIdentifier",
    ("subtle-crypto", "verify", "key"): "handle:crypto-key",
    ("subtle-crypto", "digest", "algorithm"): "dictionary:AlgorithmIdentifier",
    ("subtle-crypto", "generate-key", "algorithm"): "dictionary:AlgorithmIdentifier",
    ("subtle-crypto", "derive-key", "algorithm"): "dictionary:AlgorithmIdentifier",
    ("subtle-crypto", "derive-key", "base-key"): "optional-handle:crypto-key",
    ("subtle-crypto", "derive-bits", "algorithm"): "dictionary:AlgorithmIdentifier",
    ("subtle-crypto", "derive-bits", "base-key"): "handle:crypto-key",
    ("subtle-crypto", "import-key", "format"): "enum:KeyFormat",
    ("subtle-crypto", "import-key", "algorithm"): "dictionary:AlgorithmIdentifier",
    ("subtle-crypto", "import-key", "key-data"): "any",
    ("subtle-crypto", "export-key", "format"): "enum:KeyFormat",
    ("subtle-crypto", "export-key", "key"): "handle:crypto-key",
    ("subtle-crypto", "wrap-key", "format"): "enum:KeyFormat",
    ("subtle-crypto", "wrap-key", "key"): "handle:crypto-key",
    ("subtle-crypto", "wrap-key", "wrapping-key"): "handle:crypto-key",
    ("subtle-crypto", "unwrap-key", "format"): "enum:KeyFormat",
    ("subtle-crypto", "unwrap-key", "wrapped-key"): "any",
    ("subtle-crypto", "unwrap-key", "unwrapping-key"): "handle:crypto-key",
    ("subtle-crypto", "unwrap-key", "algorithm"): "dictionary:AlgorithmIdentifier",
    # MediaQueryList addListener/removeListener - callback is event handler
    ("media-query-list", "add-listener", "callback"): "event-listener",
    ("media-query-list", "remove-listener", "callback"): "event-listener",
    # EventTarget addEventListener/removeEventListener
    ("event-target", "add-event-listener", "callback"): "event-listener",
    ("event-target", "remove-event-listener", "callback"): "event-listener",
    # Document createNodeIterator/createTreeWalker - root is Node, filter is NodeFilter
    ("document", "create-node-iterator", "root"): "handle:node",
    ("document", "create-node-iterator", "filter"): "optional-handle:node-filter",
    ("document", "create-tree-walker", "root"): "handle:node",
    ("document", "create-tree-walker", "filter"): "optional-handle:node-filter",
    # WebGL2 invalidateFramebuffer attachments array
    ("web-gl2-rendering-context-base", "invalidate-framebuffer", "attachments"): "array",
    ("web-gl2-rendering-context-base", "invalidate-sub-framebuffer", "attachments"): "array",
    ("web-gl2-rendering-context-base", "invalidate-sub-framebuffer", "x"): True,
    ("web-gl2-rendering-context-base", "invalidate-sub-framebuffer", "y"): True,
    ("web-gl2-rendering-context-base", "invalidate-sub-framebuffer", "width"): True,
    ("web-gl2-rendering-context-base", "invalidate-sub-framebuffer", "height"): True,
    ("web-gl2-rendering-context-base", "transform-feedback-varyings", "buffer-mode"): True,
    # WebGL2 getActiveUniforms
    ("web-gl2-rendering-context-base", "get-active-uniforms", "uniform-indices"): "array",
    ("web-gl2-rendering-context-base", "get-active-uniforms", "pname"): True,
    # WebGL2RenderingContextOverloads - tex-sub-image2-d
    ("web-gl2-rendering-context-overloads", "tex-sub-image2-d", "target"): True,
    ("web-gl2-rendering-context-overloads", "tex-sub-image2-d", "level"): True,
    ("web-gl2-rendering-context-overloads", "tex-sub-image2-d", "xoffset"): True,
    ("web-gl2-rendering-context-overloads", "tex-sub-image2-d", "yoffset"): True,
    ("web-gl2-rendering-context-overloads", "tex-sub-image2-d", "width"): True,
    ("web-gl2-rendering-context-overloads", "tex-sub-image2-d", "height"): True,
    ("web-gl2-rendering-context-overloads", "tex-sub-image2-d", "format"): True,
    ("web-gl2-rendering-context-overloads", "tex-sub-image2-d", "type"): True,
    # WebGL2RenderingContextOverloads - compressed-tex-image2-d
    ("web-gl2-rendering-context-overloads", "compressed-tex-image2-d", "target"): True,
    ("web-gl2-rendering-context-overloads", "compressed-tex-image2-d", "level"): True,
    ("web-gl2-rendering-context-overloads", "compressed-tex-image2-d", "internalformat"): True,
    ("web-gl2-rendering-context-overloads", "compressed-tex-image2-d", "width"): True,
    ("web-gl2-rendering-context-overloads", "compressed-tex-image2-d", "height"): True,
    ("web-gl2-rendering-context-overloads", "compressed-tex-image2-d", "border"): True,
    ("web-gl2-rendering-context-overloads", "compressed-tex-image2-d", "image-size"): True,
    ("web-gl2-rendering-context-overloads", "compressed-tex-image2-d", "offset"): True,
    # WebGL2RenderingContextOverloads - compressed-tex-sub-image2-d
    ("web-gl2-rendering-context-overloads", "compressed-tex-sub-image2-d", "target"): True,
    ("web-gl2-rendering-context-overloads", "compressed-tex-sub-image2-d", "level"): True,
    ("web-gl2-rendering-context-overloads", "compressed-tex-sub-image2-d", "xoffset"): True,
    ("web-gl2-rendering-context-overloads", "compressed-tex-sub-image2-d", "yoffset"): True,
    ("web-gl2-rendering-context-overloads", "compressed-tex-sub-image2-d", "width"): True,
    ("web-gl2-rendering-context-overloads", "compressed-tex-sub-image2-d", "height"): True,
    ("web-gl2-rendering-context-overloads", "compressed-tex-sub-image2-d", "format"): True,
    ("web-gl2-rendering-context-overloads", "compressed-tex-sub-image2-d", "image-size"): True,
    ("web-gl2-rendering-context-overloads", "compressed-tex-sub-image2-d", "offset"): True,
    # WebGL2RenderingContextOverloads - buffer-data
    ("web-gl2-rendering-context-overloads", "buffer-data", "target"): True,
    ("web-gl2-rendering-context-overloads", "buffer-data", "size"): True,
    ("web-gl2-rendering-context-overloads", "buffer-data", "usage"): True,
    # WebGL2RenderingContextOverloads - read-pixels
    ("web-gl2-rendering-context-overloads", "read-pixels", "x"): True,
    ("web-gl2-rendering-context-overloads", "read-pixels", "y"): True,
    ("web-gl2-rendering-context-overloads", "read-pixels", "width"): True,
    ("web-gl2-rendering-context-overloads", "read-pixels", "height"): True,
    ("web-gl2-rendering-context-overloads", "read-pixels", "format"): True,
    ("web-gl2-rendering-context-overloads", "read-pixels", "type"): True,
    # WebGL2RenderingContextOverloads - uniform*fv location/data/srcOffset/srcLength
    ("web-gl2-rendering-context-overloads", "uniform1fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-overloads", "uniform1fv", "data"): "handle:float-32-list",
    ("web-gl2-rendering-context-overloads", "uniform1fv", "src-offset"): True,
    ("web-gl2-rendering-context-overloads", "uniform1fv", "src-length"): True,
    ("web-gl2-rendering-context-overloads", "uniform2fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-overloads", "uniform2fv", "data"): "handle:float-32-list",
    ("web-gl2-rendering-context-overloads", "uniform2fv", "src-offset"): True,
    ("web-gl2-rendering-context-overloads", "uniform2fv", "src-length"): True,
    ("web-gl2-rendering-context-overloads", "uniform3fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-overloads", "uniform3fv", "data"): "handle:float-32-list",
    ("web-gl2-rendering-context-overloads", "uniform3fv", "src-offset"): True,
    ("web-gl2-rendering-context-overloads", "uniform3fv", "src-length"): True,
    ("web-gl2-rendering-context-overloads", "uniform4fv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-overloads", "uniform4fv", "data"): "handle:float-32-list",
    ("web-gl2-rendering-context-overloads", "uniform4fv", "src-offset"): True,
    ("web-gl2-rendering-context-overloads", "uniform4fv", "src-length"): True,
    # WebGL2RenderingContextOverloads - uniform*iv location/data/srcOffset/srcLength
    ("web-gl2-rendering-context-overloads", "uniform1iv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-overloads", "uniform1iv", "data"): "handle:int-32-list",
    ("web-gl2-rendering-context-overloads", "uniform1iv", "src-offset"): True,
    ("web-gl2-rendering-context-overloads", "uniform1iv", "src-length"): True,
    ("web-gl2-rendering-context-overloads", "uniform2iv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-overloads", "uniform2iv", "data"): "handle:int-32-list",
    ("web-gl2-rendering-context-overloads", "uniform2iv", "src-offset"): True,
    ("web-gl2-rendering-context-overloads", "uniform2iv", "src-length"): True,
    ("web-gl2-rendering-context-overloads", "uniform3iv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-overloads", "uniform3iv", "data"): "handle:int-32-list",
    ("web-gl2-rendering-context-overloads", "uniform3iv", "src-offset"): True,
    ("web-gl2-rendering-context-overloads", "uniform3iv", "src-length"): True,
    ("web-gl2-rendering-context-overloads", "uniform4iv", "location"): "optional-handle:web-gl-uniform-location",
    ("web-gl2-rendering-context-overloads", "uniform4iv", "data"): "handle:int-32-list",
    ("web-gl2-rendering-context-overloads", "uniform4iv", "src-offset"): True,
    ("web-gl2-rendering-context-overloads", "uniform4iv", "src-length"): True,
    # WebGL2RenderingContextOverloads - uniformMatrix*fv data/srcOffset/srcLength
    ("web-gl2-rendering-context-overloads", "uniform-matrix2fv", "data"): "handle:float-32-list",
    ("web-gl2-rendering-context-overloads", "uniform-matrix2fv", "src-offset"): True,
    ("web-gl2-rendering-context-overloads", "uniform-matrix2fv", "src-length"): True,
    ("web-gl2-rendering-context-overloads", "uniform-matrix3fv", "data"): "handle:float-32-list",
    ("web-gl2-rendering-context-overloads", "uniform-matrix3fv", "src-offset"): True,
    ("web-gl2-rendering-context-overloads", "uniform-matrix3fv", "src-length"): True,
    ("web-gl2-rendering-context-overloads", "uniform-matrix4fv", "data"): "handle:float-32-list",
    ("web-gl2-rendering-context-overloads", "uniform-matrix4fv", "src-offset"): True,
    ("web-gl2-rendering-context-overloads", "uniform-matrix4fv", "src-length"): True,
    # Document methods - node parameter needs handle lookup
    ("document", "import-node", "node"): "handle:node",
    ("document", "adopt-node", "node"): "handle:node",
    # Canvas methods - fillRule needs conversion
    ("canvas-draw-path", "fill", "fill-rule"): True,
    ("canvas-draw-path", "clip", "fill-rule"): True,
    ("canvas-draw-path", "is-point-in-path", "fill-rule"): True,
    ("canvas-draw-path", "is-point-in-stroke", "fill-rule"): True,
    # CSS methods
    ("css-style-declaration", "set-property", "priority"): True,
    # HTML methods
    ("html-form-element", "reset"): True,
    # WebRTC methods
    ("rtc-rtp-transceiver", "set-codec-preferences", "codecs"): "array",
    # Workers methods
    ("dedicated-worker-global-scope", "get"): True,
    ("worker-global-scope", "get"): True,
    # WebGL buffer-source parameters (bigint → AllowSharedBufferSource)
    ("web-gl-rendering-context-overloads", "buffer-sub-data", "data"): "buffer-source",
    ("web-gl2-rendering-context-overloads", "buffer-sub-data", "src-data"): "buffer-source",
    # WebGL nullable-buffer-source parameters (Uint8Array | undefined → ArrayBufferView | null)
    ("web-gl-rendering-context-overloads", "read-pixels", "pixels"): "nullable-buffer-source",
    ("web-gl-rendering-context-overloads", "tex-image2-d", "pixels"): "nullable-buffer-source",
    ("web-gl-rendering-context-overloads", "tex-sub-image2-d", "pixels"): "nullable-buffer-source",
    # WebGL2 object type casts (WebGLObject | null → specific type | null)
    ("web-gl2-rendering-context-base", "bind-transform-feedback", "tf"): "any",
    ("web-gl2-rendering-context-base", "is-vertex-array", "vertex-array"): "any",
    # SubtleCrypto buffer-source parameters (already typed arrays, just need cast)
    ("subtle-crypto", "encrypt", "data"): "any",
    ("subtle-crypto", "decrypt", "data"): "any",
    ("subtle-crypto", "sign", "data"): "any",
    ("subtle-crypto", "verify", "signature"): "any",
    ("subtle-crypto", "verify", "data"): "any",
    ("subtle-crypto", "digest", "data"): "any",
    ("subtle-crypto", "derive-key", "algorithm"): "dictionary:AlgorithmIdentifier",
    ("subtle-crypto", "derive-bits", "algorithm"): "dictionary:AlgorithmIdentifier",
    ("subtle-crypto", "wrap-key", "algorithm"): "dictionary:AlgorithmIdentifier",
    ("subtle-crypto", "unwrap-key", "algorithm"): "dictionary:AlgorithmIdentifier",
    # DOM methods - handle lookups
    ("event-target", "dispatch-event", "event"): "handle:event",
    ("range", "compare-boundary-points", "source-range"): "handle:range",
    ("range", "select-node", "node"): "handle:node",
    ("range", "select-node-contents", "node"): "handle:node",
    ("range", "is-point-in-range", "node"): "handle:node",
    ("range", "compare-point", "node"): "handle:node",
    ("range", "intersects-node", "node"): "handle:node",
    ("range", "insert-node", "node"): "handle:node",
    ("range", "surround-contents", "new-parent"): "handle:node",
    ("range", "set-start", "node"): "handle:node",
    ("range", "set-end", "node"): "handle:node",
    ("range", "set-start-before", "node"): "handle:node",
    ("range", "set-start-after", "node"): "handle:node",
    ("range", "set-end-before", "node"): "handle:node",
    ("range", "set-end-after", "node"): "handle:node",
    ("range", "clone-range", "range"): "handle:range",
    # NodeIterator and TreeWalker
    ("node-iterator", "next-node"): True,
    ("node-iterator", "previous-node"): True,
    ("tree-walker", "next-node"): True,
    ("tree-walker", "previous-node"): True,
    # Node methods - handle lookups
    ("node", "append-child", "node"): "handle:node",
    ("node", "insert-before", "node"): "handle:node",
    ("node", "replace-child", "node"): "handle:node",
    ("node", "remove-child", "node"): "handle:node",
    ("node", "clone-node", "deep"): "boolean",
    ("node", "is-default-namespace", "namespace"): "string",
    ("node", "lookup-namespace-uri", "prefix"): "string",
    ("node", "lookup-prefix", "namespace"): "string",
    ("node", "normalize"): True,
    ("node", "contains", "other"): "optional-handle:node",
    ("node", "compare-document-position", "other"): "handle:node",
    ("node", "is-equal-node", "other"): "optional-handle:node",
    # Element methods - handle lookups and enum conversions
    ("element", "get-attribute-node", "qualified-name"): "string",
    ("element", "set-attribute-node", "attr"): "handle:attr",
    ("element", "set-attribute-node-ns", "attr"): "handle:attr",
    ("element", "remove-attribute-node", "attr"): "handle:attr",
    ("element", "insert-adjacent-element", "where"): "enum:InsertPosition",
    ("element", "insert-adjacent-text", "where"): "enum:InsertPosition",
    ("element", "insert-adjacent-html", "position"): "enum:InsertPosition",
    ("element", "insert-adjacent-element", "element"): "handle:element",
    ("element", "set-pointer-capture", "pointer-id"): True,
    ("element", "release-pointer-capture", "pointer-id"): True,
    ("element", "has-pointer-capture", "pointer-id"): True,
    ("element", "scroll", "options"): "any",
    ("element", "scroll-to", "options"): "any",
    ("element", "scroll-by", "options"): "any",
    ("element", "check-visibility", "options"): "any",
    ("element", "request-pointer-lock"): True,
    ("element", "request-fullscreen", "options"): "any",
    # ParentNode methods - variadic (string | Node)[] - use any for complex union spread
    ("parent-node", "prepend", "nodes"): "any",
    ("parent-node", "append", "nodes"): "any",
    ("parent-node", "replace-children", "nodes"): "any",
    # Document methods - node lookups and string conversions
    ("document", "import-node", "node"): "handle:node",
    ("document", "adopt-node", "node"): "handle:node",
    ("document", "create-event", "event-interface-name"): "string",
    ("document", "create-range", "range"): "handle:range",
    ("document", "get-elements-by-tag-name-ns", "namespace-uri"): "string",
    ("document", "get-elements-by-tag-name-ns", "local-name"): "string",
    ("document", "create-element-ns", "namespace"): "string",
    ("document", "create-element-ns", "qualified-name"): "string",
    ("document", "create-attribute-ns", "namespace"): "string",
    ("document", "create-attribute-ns", "qualified-name"): "string",
    ("document", "create-cdata-section", "data"): "string",
    ("document", "create-processing-instruction", "target"): "string",
    ("document", "create-processing-instruction", "data"): "string",
    ("document", "create-comment", "data"): "string",
    ("document", "create-text-node", "data"): "string",
    ("document", "element-from-point", "x"): True,
    ("document", "element-from-point", "y"): True,
    # CharacterData methods
    ("character-data", "append-data", "data"): "string",
    ("character-data", "insert-data", "offset"): True,
    ("character-data", "insert-data", "data"): "string",
    ("character-data", "delete-data", "offset"): True,
    ("character-data", "delete-data", "count"): True,
    ("character-data", "replace-data", "offset"): True,
    ("character-data", "replace-data", "count"): True,
    ("character-data", "replace-data", "data"): "string",
    ("character-data", "substring-data", "offset"): True,
    ("character-data", "substring-data", "count"): True,
    # Attr methods
    ("attr", "get-owner-element", "element"): "handle:element",
    # NamedNodeMap methods
    ("named-node-map", "get-named-item", "qualified-name"): "string",
    ("named-node-map", "get-named-item-ns", "namespace"): "string",
    ("named-node-map", "get-named-item-ns", "local-name"): "string",
    ("named-node-map", "set-named-item", "attr"): "handle:attr",
    ("named-node-map", "set-named-item-ns", "attr"): "handle:attr",
    ("named-node-map", "remove-named-item", "qualified-name"): "string",
    ("named-node-map", "remove-named-item-ns", "namespace"): "string",
    ("named-node-map", "remove-named-item-ns", "local-name"): "string",
    # DOMTokenList methods
    ("dom-token-list", "add", "tokens"): "any",
    ("dom-token-list", "remove", "tokens"): "any",
    ("dom-token-list", "toggle", "token"): "string",
    ("dom-token-list", "replace", "token"): "string",
    ("dom-token-list", "replace", "new-token"): "string",
    ("dom-token-list", "supports", "token"): "string",
    ("dom-token-list", "contains", "token"): "string",
    # Event listener options
    ("event-target", "add-event-listener", "options"): "event-listener-options",
    ("event-target", "remove-event-listener", "options"): "event-listener-options",
    # AbortController
    ("abort-controller", "abort", "reason"): "any",
    # Gamepad
    ("gamepad-haptic-actuator", "play-effect", "type"): "enum:GamepadHapticEffectType",
    ("gamepad-haptic-actuator", "reset", "type"): "enum:GamepadHapticEffectType",
    # Geolocation
    ("geolocation", "get-current-position", "success-callback"): "handle:position-callback",
    ("geolocation", "watch-position", "success-callback"): "handle:position-callback",
    ("geolocation", "get-current-position", "error-callback"): "optional-handle:position-error-callback",
    ("geolocation", "watch-position", "error-callback"): "optional-handle:position-error-callback",
    ("geolocation", "watch-position", "options"): "any",
    # TypedArray returns - need to wrap in handle
    ("gamepad", "get-axes", "axes"): "handle:float-32-list",
    ("gamepad", "get-buttons", "buttons"): "handle:gamepad-button-list",
    # CSS methods
    ("css-style-declaration", "set-property", "property"): "string",
    ("css-style-declaration", "set-property", "value"): "string",
    ("css-style-declaration", "set-property", "priority"): "string",
    ("css-style-declaration", "remove-property", "property"): "string",
    ("css-style-sheet", "insert-rule", "rule"): "string",
    ("css-style-sheet", "insert-rule", "index"): True,
    ("css-style-sheet", "delete-rule", "index"): True,
    ("css-style-sheet", "add-rule", "selector"): "string",
    ("css-style-sheet", "add-rule", "style"): "string",
    ("css-style-sheet", "add-rule", "index"): True,
    ("css-style-sheet", "remove-rule", "index"): True,
    ("css-keyframes-rule", "append-rule", "rule"): "string",
    ("css-keyframes-rule", "delete-rule", "key"): "string",
    ("css-keyframes-rule", "find-rule", "key"): "string",
    ("css-grouping-rule", "insert-rule", "rule"): "string",
    ("css-grouping-rule", "insert-rule", "index"): True,
    ("css-grouping-rule", "delete-rule", "index"): True,
    ("css-media-rule", "insert-rule", "rule"): "string",
    ("css-media-rule", "insert-rule", "index"): True,
    ("css-media-rule", "delete-rule", "index"): True,
    # Window methods
    ("window", "get-computed-style", "elt"): "handle:element",
    ("window", "get-computed-style", "pseudo"): "string",
    ("window", "scroll", "options"): "any",
    ("window", "scroll-to", "options"): "any",
    ("window", "scroll-by", "options"): "any",
    ("window", "open", "url"): "string",
    ("window", "open", "target"): "string",
    ("window", "open", "features"): "string",
    ("window", "post-message", "message"): "any",
    ("window", "post-message", "options"): "any",
    # Selection methods
    ("selection", "add-range", "range"): "handle:range",
    ("selection", "remove-range", "range"): "handle:range",
    ("selection", "collapse", "node"): "optional-handle:node",
    ("selection", "collapse", "offset"): True,
    ("selection", "collapse-to-start"): True,
    ("selection", "collapse-to-end"): True,
    ("selection", "extend", "node"): "handle:node",
    ("selection", "extend", "offset"): True,
    ("selection", "set-base-and-extent", "anchor-node"): "handle:node",
    ("selection", "set-base-and-extent", "anchor-offset"): True,
    ("selection", "set-base-and-extent", "focus-node"): "handle:node",
    ("selection", "set-base-and-extent", "focus-offset"): True,
    ("selection", "contains-node", "node"): "handle:node",
    ("selection", "contains-node", "partial-containment"): "boolean",
    # DOMImplementation methods
    ("dom-implementation", "create-document-type", "qualified-name"): "string",
    ("dom-implementation", "create-document-type", "public-id"): "string",
    ("dom-implementation", "create-document-type", "system-id"): "string",
    ("dom-implementation", "create-document", "namespace"): "string",
    ("dom-implementation", "create-document", "qualified-name"): "string",
    ("dom-implementation", "create-document", "doctype"): "optional-handle:document-type",
    ("dom-implementation", "create-html-document", "title"): "string",
    ("dom-implementation", "has-feature"): True,
    # Crypto algorithm parameters - need any cast
    ("subtle-crypto", "wrap-key", "wrap-algorithm"): "any",
    # Document methods
    ("document", "caret-position-from-point", "options"): "any",
    ("document", "writeln", "text"): "string",
    ("document", "write", "text"): "string",
    # HTMLElement togglePopover
    ("html-element", "toggle-popover", "options"): "boolean",
    # Gamepad methods
    ("gamepad-haptic-actuator", "play-effect", "params"): "any",
    # Element methods - string parameters
    ("element", "set-attribute", "value"): "string",
    ("element", "set-attribute-ns", "value"): "string",
    ("element", "set-html-unsafe", "html"): "string",
    ("element", "insert-adjacent-html", "string"): "string",
    # AbortSignal methods
    ("abort-signal", "timeout", "milliseconds"): True,
    ("abort-signal", "any", "signals"): "handle-array:abort-signal",
    # MutationObserver
    ("mutation-observer", "observe", "target"): "handle:node",
    ("mutation-observer", "observe", "options"): "any",
    # ChildNode methods - nodes array to spread
    ("child-node", "before", "nodes"): "any",
    ("child-node", "after", "nodes"): "any",
    ("child-node", "replace-with", "nodes"): "any",
    # Range methods
    ("range", "create-contextual-fragment", "string"): "string",
    # Node methods - handle conversions
    ("node", "replace-child", "child"): "handle:node",
    ("node", "is-equal-node", "other-node"): "optional-handle:node",
    ("node", "is-same-node", "other-node"): "optional-handle:node",
    ("node", "insert-before", "child"): "optional-handle:node",
    ("node", "remove-child", "child"): "handle:node",
    # XPath methods
    ("x-path-expression", "evaluate", "context-node"): "handle:node",
    ("x-path-expression", "evaluate", "result"): "optional-handle:xpath-result",
    ("x-path-evaluator-base", "create-ns-resolver", "node-resolver"): "handle:node",
    ("x-path-evaluator-base", "create-expression", "resolver"): "any",
    ("x-path-evaluator-base", "evaluate", "context-node"): "handle:node",
    ("x-path-evaluator-base", "evaluate", "resolver"): "any",
    ("x-path-evaluator-base", "evaluate", "result"): "optional-handle:xpath-result",
    # XSLTProcessor methods
    ("xslt-processor", "import-stylesheet", "style"): "handle:node",
    ("xslt-processor", "transform-to-fragment", "source"): "handle:node",
    ("xslt-processor", "transform-to-fragment", "output"): "handle:document",
    ("xslt-processor", "transform-to-document", "source"): "handle:node",
    # UIEvent methods
    ("ui-event", "init-ui-event", "type-arg"): "string",
    ("ui-event", "init-ui-event", "bubbles-arg"): "boolean-or-undefined",
    ("ui-event", "init-ui-event", "cancelable-arg"): "boolean-or-undefined",
    ("ui-event", "init-ui-event", "view-arg"): "optional-handle:window",
    ("ui-event", "init-ui-event", "detail-arg"): True,
    # KeyboardEvent methods
    ("keyboard-event", "init-keyboard-event", "type-arg"): "string",
    ("keyboard-event", "init-keyboard-event", "bubbles-arg"): "boolean-or-undefined",
    ("keyboard-event", "init-keyboard-event", "cancelable-arg"): "boolean-or-undefined",
    ("keyboard-event", "init-keyboard-event", "view-arg"): "optional-handle:window",
    ("keyboard-event", "init-keyboard-event", "key-arg"): "string",
    ("keyboard-event", "init-keyboard-event", "alt-key"): "boolean-or-undefined",
    # ReadableStream methods
    ("readable-stream", "pipe-through", "transform"): "any",
    ("readable-stream", "pipe-through", "options"): "any",
    ("readable-stream", "pipe-to", "destination"): "any",
    ("readable-stream", "pipe-to", "options"): "any",
    # WindowOrWorkerGlobalScope methods
    ("window-or-worker-global-scope", "fetch", "input"): "any",
    ("window-or-worker-global-scope", "fetch", "init"): "any",
    ("window-or-worker-global-scope", "create-image-bitmap", "image"): "any",
    ("window-or-worker-global-scope", "create-image-bitmap", "options"): "any",
    ("window-or-worker-global-scope", "structured-clone", "value"): "any",
    ("window-or-worker-global-scope", "structured-clone", "options"): "any",
    # Response methods
    ("response", "json", "init"): "any",
    # CompositionEvent methods
    ("composition-event", "init-composition-event", "type-arg"): "string",
    # Clipboard methods
    ("clipboard", "write", "data"): "any",
    # HTMLOptionsCollection methods
    ("html-options-collection", "add", "element"): "any",
    ("html-options-collection", "add", "before"): "any",
    # Headers methods - string parameters
    ("headers", "delete", "name"): "string",
    ("headers", "get", "name"): "string",
    ("headers", "has", "name"): "string",
    ("headers", "set", "name"): "string",
    # HTMLMediaElement methods
    ("html-media-element", "add-text-track", "kind"): "enum:TextTrackKind",
    # ShadowRoot methods
    ("shadow-root", "set-html-unsafe", "html"): "string",
    # TextTrack methods - cue parameter needs handle lookup
    ("text-track", "add-cue", "cue"): "handle:text-track-cue",
    ("text-track", "remove-cue", "cue"): "handle:text-track-cue",
    # HTMLSelectElement methods - element parameter needs handle lookup
    ("html-select-element", "add", "element"): "handle:html-option-element",
    ("html-select-element", "add", "before"): "any",
    # HTMLTableElement methods - index parameter needs Number()
    ("html-table-element", "insert-row", "index"): True,
    ("html-table-section-element", "insert-row", "index"): True,
    ("html-table-row-element", "insert-cell", "index"): True,
    # Canvas methods - image parameter needs handle lookup
    ("canvas-fill-stroke-styles", "create-pattern", "image"): "handle:canvas-image-source",
    # HTMLSlotElement methods
    ("html-slot-element", "assign", "nodes"): "node-array",
    # XMLHttpRequest methods
    ("xml-http-request", "send", "body"): "any",
    # HTMLElement focus options
    ("html-element", "focus", "options"): "any",
    # FormData append - value can be string or blob
    ("form-data", "append", "value"): "any",
    ("form-data", "append", "blob-name"): "string",
    # Canvas drawImage - image needs handle lookup
    ("canvas-draw-image", "draw-image", "image"): "handle:canvas-image-source",
    # Canvas createImageData - imagedata parameter needs handle lookup
    ("canvas-image-data", "put-image-data", "image-data"): "handle:image-data",
    # Canvas fill/stroke with Path2D
    ("canvas-draw-path", "fill", "path"): "optional-handle:path-2-d",
    ("canvas-draw-path", "stroke", "path"): "optional-handle:path-2-d",
    ("canvas-draw-path", "clip", "path"): "optional-handle:path-2-d",
    ("canvas-draw-path", "is-point-in-path", "path"): "optional-handle:path-2-d",
    ("canvas-draw-path", "is-point-in-stroke", "path"): "optional-handle:path-2-d",
    # EventTarget dispatchEvent
    ("event-target", "dispatch-event", "event"): "handle:event",
    # CustomElementRegistry define
    ("custom-element-registry", "define", "constructor"): "any",
    # MutationRecord target
    ("mutation-observer", "observe", "target"): "handle:node",
    ("mutation-observer", "observe", "options"): "any",
    # Request body
    ("request", "new-request", "body"): "any",
    # OffscreenCanvas convertToBlob
    ("offscreencanvas", "convert-to-blob", "options"): "any",
    # ResizeObserver observe
    ("resize-observer", "observe", "target"): "handle:element",
    ("resize-observer", "observe", "options"): "any",
    # IntersectionObserver observe
    ("intersection-observer", "observe", "target"): "handle:element",
    # HTMLCanvasElement toBlob callback
    ("html-canvas-element", "to-blob", "callback"): "any",
    # HTMLCanvasElement toDataURL
    ("html-canvas-element", "to-data-url", "type"): "string",
    ("html-canvas-element", "to-data-url", "quality"): True,
    # Window requestAnimationFrame
    ("window", "request-animation-frame", "callback"): "any",
    # Worklet addModule
    ("worklet", "add-module", "module-url"): "string",
    ("worklet", "add-module", "options"): "any",
    # StorageEvent initStorageEvent
    ("storage-event", "init-storage-event", "type-arg"): "string",
    # Event initEvent
    ("event", "init-event", "type-arg"): "string",
    # MediaRecorder start
    ("media-recorder", "start", "timeslice"): True,
    # Element getAttributeNS - namespace can be undefined, convert to null
    ("element", "get-attribute-ns", "namespace"): "string-or-null",
    # HTMLMediaElement srcObject - bigint handle to MediaProvider
    ("html-media-element", "set-src-object", "value"): "optional-handle:media-provider",
    # HTMLTableElement caption/tHead/tFoot - bigint handle to elements
    ("html-table-element", "set-caption", "value"): "optional-handle:html-table-caption-element",
    ("html-table-element", "set-t-head", "value"): "optional-handle:html-table-section-element",
    ("html-table-element", "set-t-foot", "value"): "optional-handle:html-table-section-element",
    # HTMLInputElement files - bigint handle to FileList
    ("html-input-element", "set-files", "value"): "optional-handle:file-list",
    # HTMLInputElement valueAsDate - bigint handle to Date
    ("html-input-element", "set-value-as-date", "value"): "optional-handle:date",
    # MediaStream addTrack/removeTrack - bigint handle to MediaStreamTrack
    ("media-stream", "add-track", "track"): "handle:media-stream-track",
    ("media-stream", "remove-track", "track"): "handle:media-stream-track",
    # Canvas createPattern - image needs handle lookup
    ("canvas-fill-stroke-styles", "create-pattern", "image"): "handle:canvas-image-source",
    # Canvas drawImage - image needs handle lookup
    ("canvas-draw-image", "draw-image", "image"): "handle:canvas-image-source",
    # Canvas putImageData - imageData needs handle lookup
    ("canvas-image-data", "put-image-data", "image-data"): "handle:image-data",
    # TextTrack addCue/removeCue - cue needs handle lookup
    ("text-track", "add-cue", "cue"): "handle:text-track-cue",
    ("text-track", "remove-cue", "cue"): "handle:text-track-cue",
    # HTMLElement focus - options is dictionary
    ("html-element", "focus", "options"): "any",
    # HTMLSlotElement assign - nodes array
    ("html-slot-element", "assign", "nodes"): "node-array",
    # HTMLSlotElement assignedNodes/assignedElements - options
    ("html-slot-element", "assigned-nodes", "options"): "any",
    ("html-slot-element", "assigned-elements", "options"): "any",
    # Window requestAnimationFrame - callback
    ("window", "request-animation-frame", "callback"): "any",
    # Window postMessage
    ("window", "post-message", "message"): "any",
    ("window", "post-message", "options"): "any",
    # MessagePort postMessage
    ("message-port", "post-message", "message"): "any",
    ("message-port", "post-message", "transfer"): "any",
    # URL createObjectURL
    ("url", "create-object-url", "object"): "any",
    # Worklet addModule
    ("worklet", "add-module", "module-url"): "string",
    ("worklet", "add-module", "options"): "any",
    # MutationObserver observe
    ("mutation-observer", "observe", "target"): "handle:node",
    ("mutation-observer", "observe", "options"): "any",
    # IntersectionObserver observe
    ("intersection-observer", "observe", "target"): "handle:element",
    # MouseEvent initMouseEvent
    ("mouse-event", "init-mouse-event", "type-arg"): "string",
    ("mouse-event", "init-mouse-event", "bubbles-arg"): "boolean-or-false",
    ("mouse-event", "init-mouse-event", "cancelable-arg"): "boolean-or-false",
    ("mouse-event", "init-mouse-event", "view-arg"): "optional-handle-strict:window",
    ("mouse-event", "init-mouse-event", "detail-arg"): True,
    ("mouse-event", "init-mouse-event", "screen-x-arg"): True,
    ("mouse-event", "init-mouse-event", "screen-y-arg"): True,
    ("mouse-event", "init-mouse-event", "client-x-arg"): True,
    ("mouse-event", "init-mouse-event", "client-y-arg"): True,
    ("mouse-event", "init-mouse-event", "ctrl-key-arg"): "boolean-or-false",
    ("mouse-event", "init-mouse-event", "alt-key-arg"): "boolean-or-false",
    ("mouse-event", "init-mouse-event", "shift-key-arg"): "boolean-or-false",
    ("mouse-event", "init-mouse-event", "meta-key-arg"): "boolean-or-false",
    ("mouse-event", "init-mouse-event", "button-arg"): True,
    ("mouse-event", "init-mouse-event", "related-target-arg"): "optional-handle:event-target",
    # UIEvent initUIEvent
    ("ui-event", "init-ui-event", "type-arg"): "string",
    ("ui-event", "init-ui-event", "bubbles-arg"): "boolean-or-undefined",
    ("ui-event", "init-ui-event", "cancelable-arg"): "boolean-or-undefined",
    ("ui-event", "init-ui-event", "view-arg"): "optional-handle:window",
    ("ui-event", "init-ui-event", "detail-arg"): True,
    # MediaSession setActionHandler
    ("media-session", "set-action-handler", "action"): "any",
    ("media-session", "set-action-handler", "handler"): "any",
    # CanvasImageData getImageData settings
    ("canvas-image-data", "get-image-data", "settings"): "any",
    # OffscreenCanvas convertToBlob
    ("offscreencanvas", "convert-to-blob", "options"): "any",
    # HTMLCanvasElement toDataURL
    ("html-canvas-element", "to-data-url", "type"): "string",
    ("html-canvas-element", "to-data-url", "quality"): True,
    # Request body
    ("request", "new-request", "body"): "any",
    # MediaCapabilities
    ("media-capabilities", "decoding-info", "configuration"): "any",
    ("media-capabilities", "encoding-info", "configuration"): "any",
    # MediaDevices
    ("media-devices", "get-user-media", "constraints"): "any",
    ("media-devices", "get-display-media", "options"): "any",
    # MediaStream
    ("media-stream", "add-track", "track"): "handle:media-stream-track",
    ("media-stream", "remove-track", "track"): "handle:media-stream-track",
    # EventTarget dispatchEvent
    ("event-target", "dispatch-event", "event"): "handle:event",
    # Document importNode/adoptNode
    ("document", "import-node", "node"): "handle:node",
    ("document", "adopt-node", "node"): "handle:node",
    # Node methods
    ("node", "append-child", "node"): "handle:node",
    ("node", "insert-before", "node"): "handle:node",
    ("node", "replace-child", "node"): "handle:node",
    ("node", "remove-child", "node"): "handle:node",
    # Element methods
    ("element", "set-attribute-node", "attr"): "handle:attr",
    ("element", "set-attribute-node-ns", "attr"): "handle:attr",
    ("element", "remove-attribute-node", "attr"): "handle:attr",
    # Range methods
    ("range", "set-start", "node"): "handle:node",
    ("range", "set-end", "node"): "handle:node",
    ("range", "set-start-before", "node"): "handle:node",
    ("range", "set-start-after", "node"): "handle:node",
    ("range", "set-end-before", "node"): "handle:node",
    ("range", "set-end-after", "node"): "handle:node",
    ("range", "insert-node", "node"): "handle:node",
    ("range", "surround-contents", "new-parent"): "handle:node",
    ("range", "select-node", "node"): "handle:node",
    ("range", "select-node-contents", "node"): "handle:node",
    ("range", "compare-boundary-points", "source-range"): "handle:range",
    # Selection methods
    ("selection", "add-range", "range"): "handle:range",
    ("selection", "remove-range", "range"): "handle:range",
    ("selection", "collapse", "node"): "optional-handle:node",
    ("selection", "extend", "node"): "handle:node",
    ("selection", "set-base-and-extent", "anchor-node"): "handle:node",
    ("selection", "set-base-and-extent", "focus-node"): "handle:node",
    ("selection", "contains-node", "node"): "handle:node",
    # Gamepad methods
    ("gamepad", "get-axes", "axes"): "handle:float-32-list",
    ("gamepad", "get-buttons", "buttons"): "handle:gamepad-button-list",
    # Element setAttribute value - needs string conversion
    ("element", "set-attribute", "value"): "string",
    ("element", "set-attribute-ns", "value"): "string",
    # Canvas methods
    ("canvas-draw-path", "fill", "path"): "optional-handle:path-2-d",
    ("canvas-draw-path", "stroke", "path"): "optional-handle:path-2-d",
    ("canvas-draw-path", "clip", "path"): "optional-handle:path-2-d",
    ("canvas-draw-path", "is-point-in-path", "path"): "optional-handle:path-2-d",
    ("canvas-draw-path", "is-point-in-stroke", "path"): "optional-handle:path-2-d",
    ("canvas-image-data", "put-image-data", "image-data"): "handle:image-data",
    ("canvas-draw-image", "draw-image", "image"): "handle:canvas-image-source",
    ("canvas-fill-stroke-styles", "create-pattern", "image"): "handle:canvas-image-source",
    # HTMLCanvasElement
    ("html-canvas-element", "to-blob", "callback"): "any",
    ("html-canvas-element", "to-data-url", "type"): "string",
    # Node methods
    ("node", "append-child", "node"): "handle:node",
    ("node", "insert-before", "node"): "handle:node",
    ("node", "replace-child", "node"): "handle:node",
    ("node", "remove-child", "node"): "handle:node",
    # HTMLElement focus
    ("html-element", "focus", "options"): "any",
    # HTMLSelectElement add
    ("html-select-element", "add", "element"): "any",
    # HTMLSlotElement
    ("html-slot-element", "assign", "nodes"): "node-array",
    ("html-slot-element", "assigned-nodes", "options"): "any",
    ("html-slot-element", "assigned-elements", "options"): "any",
    # MediaDevices
    ("media-devices", "get-user-media", "constraints"): "any",
    ("media-devices", "get-display-media", "options"): "any",
    # MediaStream
    ("media-stream", "add-track", "track"): "handle:media-stream-track",
    ("media-stream", "remove-track", "track"): "handle:media-stream-track",
    # EventTarget dispatchEvent
    ("event-target", "dispatch-event", "event"): "handle:event",
    # MutationObserver observe
    ("mutation-observer", "observe", "target"): "handle:node",
    ("mutation-observer", "observe", "options"): "any",
    # IntersectionObserver observe
    ("intersection-observer", "observe", "target"): "handle:element",
    # ResizeObserver observe
    ("resize-observer", "observe", "target"): "handle:element",
    ("resize-observer", "observe", "options"): "any",
    # CustomElementRegistry define
    ("custom-element-registry", "define", "name"): "string",
    ("custom-element-registry", "define", "constructor"): "any",
    ("custom-element-registry", "define", "options"): "any",
    # TextTrack
    ("text-track", "add-cue", "cue"): "handle:text-track-cue",
    ("text-track", "remove-cue", "cue"): "handle:text-track-cue",
    # MediaSession
    ("media-session", "set-action-handler", "action"): "any",
    ("media-session", "set-action-handler", "handler"): "any",
    # Window requestAnimationFrame
    ("window", "request-animation-frame", "callback"): "any",
    # Worklet addModule
    ("worklet", "add-module", "module-url"): "string",
    ("worklet", "add-module", "options"): "any",
    # MediaCapabilities
    ("media-capabilities", "decoding-info", "configuration"): "any",
    ("media-capabilities", "encoding-info", "configuration"): "any",
    # HTMLElement togglePopover
    ("html-element", "toggle-popover", "force"): "boolean-or-undefined",
    # Element checkVisibility
    ("element", "check-visibility", "options"): "any",
    # Element requestFullscreen
    ("element", "request-fullscreen", "options"): "any",
    # MouseEvent initMouseEvent
    ("mouse-event", "init-mouse-event", "type"): "string",
    ("mouse-event", "init-mouse-event", "bubbles"): "boolean-or-undefined",
    ("mouse-event", "init-mouse-event", "cancelable"): "boolean-or-undefined",
    ("mouse-event", "init-mouse-event", "view"): "optional-handle:window",
    ("mouse-event", "init-mouse-event", "detail"): True,
    ("mouse-event", "init-mouse-event", "screen-x"): True,
    ("mouse-event", "init-mouse-event", "screen-y"): True,
    ("mouse-event", "init-mouse-event", "client-x"): True,
    ("mouse-event", "init-mouse-event", "client-y"): True,
    ("mouse-event", "init-mouse-event", "ctrl-key"): "boolean-or-undefined",
    ("mouse-event", "init-mouse-event", "alt-key"): "boolean-or-undefined",
    ("mouse-event", "init-mouse-event", "shift-key"): "boolean-or-undefined",
    ("mouse-event", "init-mouse-event", "meta-key"): "boolean-or-undefined",
    ("mouse-event", "init-mouse-event", "button"): True,
    ("mouse-event", "init-mouse-event", "related-target"): "optional-handle:event-target",
    # KeyboardEvent initKeyboardEvent
    ("keyboard-event", "init-keyboard-event", "type"): "string",
    ("keyboard-event", "init-keyboard-event", "bubbles"): "boolean-or-undefined",
    ("keyboard-event", "init-keyboard-event", "cancelable"): "boolean-or-undefined",
    ("keyboard-event", "init-keyboard-event", "view"): "optional-handle:window",
    ("keyboard-event", "init-keyboard-event", "key"): "string",
    ("keyboard-event", "init-keyboard-event", "location"): True,
    ("keyboard-event", "init-keyboard-event", "ctrl-key"): "boolean-or-undefined",
    ("keyboard-event", "init-keyboard-event", "shift-key"): "boolean-or-undefined",
    ("keyboard-event", "init-keyboard-event", "alt-key"): "boolean-or-undefined",
    ("keyboard-event", "init-keyboard-event", "meta-key"): "boolean-or-undefined",
    # UIEvent initUIEvent
    ("ui-event", "init-ui-event", "type"): "string",
    ("ui-event", "init-ui-event", "bubbles"): "boolean-or-undefined",
    ("ui-event", "init-ui-event", "cancelable"): "boolean-or-undefined",
    ("ui-event", "init-ui-event", "view"): "optional-handle:window",
    ("ui-event", "init-ui-event", "detail"): True,
    # Event initEvent
    ("event", "init-event", "type"): "string",
    ("event", "init-event", "bubbles"): "boolean-or-undefined",
    ("event", "init-event", "cancelable"): "boolean-or-undefined",
    # StorageEvent initStorageEvent
    ("storage-event", "init-storage-event", "type"): "string",
    ("storage-event", "init-storage-event", "key"): "string-or-null",
    ("storage-event", "init-storage-event", "old-value"): "string-or-null",
    ("storage-event", "init-storage-event", "new-value"): "string-or-null",
    ("storage-event", "init-storage-event", "url"): "string",
    ("storage-event", "init-storage-event", "storage-area"): "any",
    # SubtleCrypto deriveBits - length needs Number()
    ("subtle-crypto", "derive-bits", "length"): True,
    # ReadableStreamBYOBRequest respond - bytesWritten needs Number()
    ("readable-stream-byob-request", "respond", "bytes-written"): True,
    # ReadableByteStreamController enqueue - chunk needs type assertion
    ("readable-byte-stream-controller", "enqueue", "chunk"): "any",
    # Element NS methods - namespace needs null conversion
    ("element", "set-attribute-ns", "namespace"): "string-or-null",
    ("element", "remove-attribute-ns", "namespace"): "string-or-null",
    ("element", "has-attribute-ns", "namespace"): "string-or-null",
    ("element", "get-attribute-node-ns", "namespace"): "string-or-null",
    # CanvasUserInterface drawFocusIfNeeded - element needs handle lookup
    ("canvas-user-interface", "draw-focus-if-needed", "element"): "handle:element",
    # Path2D addPath - path and transform need handle lookup
    ("path2-d", "add-path", "path"): "handle:path2-d",
    ("path2-d", "add-path", "transform"): "optional-handle-strict:dom-matrix",
    # DOMParser parseFromString - string and type need string conversion
    ("dom-parser", "parse-from-string", "string"): "string",
    ("dom-parser", "parse-from-string", "type"): "string",
    # XMLSerializer serializeToString - root needs handle lookup
    ("xml-serializer", "serialize-to-string", "root"): "handle:node",
    # ImageBitmapRenderingContext transferFromImageBitmap - bitmap needs handle lookup
    ("image-bitmap-rendering-context", "transfer-from-image-bitmap", "bitmap"): "optional-handle:image-bitmap",
    # XSLTProcessor getParameter - localName needs string conversion
    ("xslt-processor", "get-parameter", "local-name"): "string",
    # Canvas drawImage - image needs handle lookup (additional variants)
    ("canvas-draw-image", "draw-image", "image"): "handle:canvas-image-source",
    # Canvas putImageData - imageData needs handle lookup
    ("canvas-image-data", "put-image-data", "image-data"): "handle:image-data",
    # Canvas createPattern - image needs handle lookup
    ("canvas-fill-stroke-styles", "create-pattern", "image"): "handle:canvas-image-source",
    # Window getEvent - can return undefined
    ("window", "get-event", "event"): "any",
    # Document currentScript - HTMLOrSVGScriptElement needs cast
    ("document", "get-current-script", "script"): "any",
    # CSS insertRule - rule index
    ("css-style-sheet", "insert-rule", "index"): True,
    # ClipboardItem getType - returns Promise
    ("clipboard-item", "get-type", "type"): "string",
    # PaymentRequest updateWith - details is dictionary
    ("payment-request", "update-with", "details"): "any",
    # PaymentResponse complete - result is enum
    ("payment-response", "complete", "result"): "any",
    # Performance mark/measure - options are dictionaries
    ("performance", "mark", "options"): "any",
    ("performance", "measure", "options"): "any",
    ("performance", "measure", "start-or-options"): "any",
    ("performance", "measure", "start-or-measure-options"): "string | PerformanceMeasureOptions | undefined",
    # Performance clearMeasures - measureName is string | undefined
    ("performance", "clear-measures", "measure-name"): "string | undefined",
    # Performance setResourceTimingBufferSize - maxSize needs number conversion
    ("performance", "set-resource-timing-buffer-size", "max-size"): True,
    # ResizeObserverEntry box sizes - need array spread
    ("resize-observer-entry", "get-border-box-size", "sizes"): "any",
    ("resize-observer-entry", "get-content-box-size", "sizes"): "any",
    ("resize-observer-entry", "get-device-pixel-content-box-size", "size"): "any",
    # IntersectionObserver observe - target needs handle lookup
    ("intersection-observer", "observe", "target"): "handle:element",
    # ResizeObserver observe - target needs handle lookup
    ("resize-observer", "observe", "target"): "handle:element",
    # Node methods - node parameters need handle lookup
    ("node", "append-child", "node"): "handle:node",
    ("node", "insert-before", "node"): "handle:node",
    ("node", "replace-child", "node"): "handle:node",
    ("node", "remove-child", "node"): "handle:node",
    # MediaStreamTrack applyConstraints - constraints is dictionary
    ("media-stream-track", "apply-constraints", "constraints"): "any",
    # MediaStream addTrack/removeTrack - track needs handle lookup
    ("media-stream", "add-track", "track"): "handle:media-stream-track",
    ("media-stream", "remove-track", "track"): "handle:media-stream-track",
    # RTC methods - various dictionary/handle parameters
    ("rtc-peer-connection", "set-local-description", "description"): "any",
    ("rtc-peer-connection", "set-remote-description", "description"): "any",
    ("rtc-peer-connection", "add-ice-candidate", "candidate"): "any",
    ("rtc-rtp-sender", "set-parameters", "parameters"): "any",
    ("rtc-rtp-sender", "replace-track", "track"): "optional-handle:media-stream-track",
    ("rtc-rtp-transceiver", "set-direction", "direction"): "any",
    ("rtc-peer-connection", "set-configuration", "configuration"): "optional-handle:any",
    ("rtc-rtp-receiver", "get-parameters", "parameters"): "any",
    # URL createObjectURL - object needs any
    ("url", "create-object-url", "object"): "any",
    # WebSocket send - data can be various types
    ("ws", "send", "data"): "any",
    # RTCDataChannel send - data can be string or binary
    ("rtc-data-channel", "send", "data"): "string | Blob | ArrayBuffer | ArrayBufferView",
    # Worker/MessagePort postMessage - message/transfer need any
    ("dedicated-worker-global-scope", "post-message", "message"): "any",
    ("dedicated-worker-global-scope", "post-message", "transfer"): "any",
    ("worker", "post-message", "message"): "any",
    ("worker", "post-message", "transfer"): "any",
    ("message-port", "post-message", "message"): "any",
    ("message-port", "post-message", "transfer"): "any",
    ("service-worker", "post-message", "message"): "any",
    ("service-worker", "post-message", "transfer"): "any",
    ("client", "post-message", "message"): "any",
    ("client", "post-message", "transfer"): "any",
    # Fetch Request/Response - init/body need any
    ("window-or-worker-global-scope", "fetch", "input"): "any",
    ("window-or-worker-global-scope", "fetch", "init"): "any",
    ("request", "new-request", "input"): "any",
    ("request", "new-request", "init"): "any",
    ("response", "json", "body"): "any",
    ("response", "json", "init"): "any",
    # FileReader readAsText - encoding needs any
    ("file-reader", "read-as-text", "encoding"): "any",
    # Notification requestPermission - callback deprecated
    ("notification", "request-permission", "callback"): "any",
    # HTMLFormControlsCollection item - needs cast to HTMLCollection
    ("html-form-element", "get-elements", "elements"): "any",
    # StructuredSerializeOptions for postMessage
    ("message-event", "init-message-event", "ports"): "any",
    # NotificationPermissionCallback deprecated
    ("notification", "request-permission", "deprecated-callback"): "any",
    # MediaSession setPositionState
    ("media-session", "set-position-state", "state"): "any",
    # Clipboard read/write
    ("clipboard", "read", "data"): "any",
    ("clipboard", "write", "data"): "any",
    # PerformanceObserver observe
    ("performance-observer", "observe", "options"): "any",
    # MediaRecorder start
    ("media-recorder", "start", "timeslice"): True,
    # SpeechSynthesisUtterance needs handle lookup
    ("speech-synthesis", "speak", "utterance"): "handle:speech-synthesis-utterance",
    # DocumentFragment children
    ("html-form-element", "get-elements", "collection"): "any",
    # Window requestAnimationFrame - callback parameter
    ("window", "request-animation-frame", "callback"): "any",
    # HTMLElement focus options
    ("html-element", "focus", "options"): "any",
    # Canvas draw focus
    ("canvas-user-interface", "draw-focus-if-needed", "element"): "handle:element",
    # SpeechSynthesis speak
    ("speech-synthesis", "speak", "utterance"): "handle:speech-synthesis-utterance",
    # Optional string setters that need null conversion
    ("html-element", "set-access-key", "value"): "string-or-null",
    ("html-element", "set-autocapitalize", "value"): "string-or-null",
    ("html-element", "set-dir", "value"): "string-or-null",
    ("html-element", "set-inner-text", "value"): "string-or-null",
    ("html-element", "set-lang", "value"): "string-or-null",
    ("html-element", "set-title", "value"): "string-or-null",
    ("html-element", "set-translate", "value"): "string-or-null",
    ("html-element", "set-popover", "value"): "string-or-null",
    ("html-element", "set-outer-text", "value"): "string-or-null",
    ("html-element", "set-hidden", "value"): "boolean-or-false",
    ("html-anchor-element", "set-download", "value"): "string-or-null",
    ("html-anchor-element", "set-href", "value"): "string-or-null",
    ("html-anchor-element", "set-hreflang", "value"): "string-or-null",
    ("html-anchor-element", "set-ping", "value"): "string-or-null",
    ("html-anchor-element", "set-rel", "value"): "string-or-null",
    ("html-anchor-element", "set-target", "value"): "string-or-null",
    ("html-media-element", "set-src", "value"): "string-or-null",
    ("html-image-element", "set-src", "value"): "string-or-null",
    ("html-image-element", "set-cross-origin", "value"): "string-or-null",
    ("html-image-element", "set-decoding", "value"): "enum-string",
    ("html-link-element", "set-cross-origin", "value"): "string-or-null",
    ("html-media-element", "set-cross-origin", "value"): "string-or-null",
    ("html-script-element", "set-cross-origin", "value"): "string-or-null",
    ("html-source-element", "set-src", "value"): "string-or-null",
    ("html-track-element", "set-src", "value"): "string-or-null",
    ("html-iframe-element", "set-src", "value"): "string-or-null",
    ("html-embed-element", "set-src", "value"): "string-or-null",
    # Enum string setters that need type assertion
    ("html-media-element", "set-preload", "value"): "enum-string",
    ("html-image-element", "set-loading", "value"): "enum-string",
    ("html-image-element", "set-fetch-priority", "value"): "enum-string",
    ("html-link-element", "set-fetch-priority", "value"): "enum-string",
    ("html-link-element", "set-loading", "value"): "enum-string",
    ("html-script-element", "set-fetch-priority", "value"): "enum-string",
    ("html-button-element", "set-type", "value"): "enum-string",
    ("html-input-element", "set-form-enctype", "value"): "enum-string",
    ("html-input-element", "set-form-method", "value"): "enum-string",
    ("html-input-element", "set-enter-key-hint", "value"): "enum-string",
    ("html-text-area-element", "set-enter-key-hint", "value"): "enum-string",
    ("html-text-area-element", "set-wrap", "value"): "enum-string",
    ("html-input-element", "set-selection-start", "value"): "number-or-null",
    ("html-input-element", "set-selection-end", "value"): "number-or-null",
    ("html-form-element", "set-enctype", "value"): "enum-string",
    ("html-form-element", "set-method", "value"): "enum-string",
    ("html-form-element", "set-autocomplete", "value"): "enum-string",
    ("html-input-element", "set-autocomplete", "value"): "enum-string",
    ("html-select-element", "set-autocomplete", "value"): "enum-string",
    ("html-text-area-element", "set-autocomplete", "value"): "enum-string",
    ("html-style-element", "set-media", "value"): "enum-string",
    # Event handler setters - need type assertion (on* properties)
    ("window-event-handlers", "set-ongamepadconnected", "value"): "event-handler",
    ("window-event-handlers", "set-ongamepaddisconnected", "value"): "event-handler",
    ("global-event-handlers", "set-onclick", "value"): "event-handler",
    ("global-event-handlers", "set-ondblclick", "value"): "event-handler",
    ("global-event-handlers", "set-onmousedown", "value"): "event-handler",
    ("global-event-handlers", "set-onmouseup", "value"): "event-handler",
    ("global-event-handlers", "set-onmouseover", "value"): "event-handler",
    ("global-event-handlers", "set-onmousemove", "value"): "event-handler",
    ("global-event-handlers", "set-onmouseout", "value"): "event-handler",
    ("global-event-handlers", "set-onkeydown", "value"): "event-handler",
    ("global-event-handlers", "set-onkeyup", "value"): "event-handler",
    ("global-event-handlers", "set-onfocus", "value"): "event-handler",
    ("global-event-handlers", "set-onblur", "value"): "event-handler",
    ("global-event-handlers", "set-onchange", "value"): "event-handler",
    ("global-event-handlers", "set-onsubmit", "value"): "event-handler",
    ("global-event-handlers", "set-onreset", "value"): "event-handler",
    ("global-event-handlers", "set-oninput", "value"): "event-handler",
    ("screen-orientation", "set-onchange", "value"): "event-handler",
    ("rtc-peer-connection", "set-onconnectionstatechange", "value"): "event-handler",
    ("rtc-peer-connection", "set-ondatachannel", "value"): "event-handler",
    ("rtc-peer-connection", "set-onicecandidate", "value"): "event-handler",
    ("rtc-peer-connection", "set-oniceconnectionstatechange", "value"): "event-handler",
    ("rtc-peer-connection", "set-onicegatheringstatechange", "value"): "event-handler",
    ("rtc-peer-connection", "set-onnegotiationneeded", "value"): "event-handler",
    ("rtc-peer-connection", "set-onsignalingstatechange", "value"): "event-handler",
    ("rtc-peer-connection", "set-ontrack", "value"): "event-handler",
    ("rtc-data-channel", "set-onopen", "value"): "event-handler",
    ("rtc-data-channel", "set-onclose", "value"): "event-handler",
    ("rtc-data-channel", "set-onclosing", "value"): "event-handler",
    ("rtc-data-channel", "set-onerror", "value"): "event-handler",
    ("rtc-data-channel", "set-onmessage", "value"): "event-handler",
    ("rtc-data-channel", "set-onbufferedamountlow", "value"): "event-handler",
    # RTCIceTransport event handlers
    ("rtc-ice-transport", "set-onstatechange", "value"): "event-handler",
    ("rtc-ice-transport", "set-ongatheringstatechange", "value"): "event-handler",
    ("rtc-ice-transport", "set-onselectedcandidatepairchange", "value"): "event-handler",
    # RTCDtlsTransport event handlers
    ("rtc-dtls-transport", "set-onstatechange", "value"): "event-handler",
    ("rtc-dtls-transport", "set-onerror", "value"): "event-handler",
    # RTCSctpTransport event handlers
    ("rtc-sctp-transport", "set-onstatechange", "value"): "event-handler",
    # RTCDTMFSender event handlers
    ("rtcdtmf-sender", "set-ontonechange", "value"): "event-handler",
    ("web-socket", "set-onopen", "value"): "event-handler",
    ("web-socket", "set-onclose", "value"): "event-handler",
    ("web-socket", "set-onerror", "value"): "event-handler",
    ("web-socket", "set-onmessage", "value"): "event-handler",
    ("web-socket", "set-binary-type", "value"): "enum-string",
    # MediaStream event handlers
    ("media-stream", "set-onaddtrack", "value"): "event-handler",
    ("media-stream", "set-onremovetrack", "value"): "event-handler",
    # MediaRecorder event handlers
    ("media-recorder", "set-onstart", "value"): "event-handler",
    ("media-recorder", "set-onstop", "value"): "event-handler",
    ("media-recorder", "set-ondataavailable", "value"): "event-handler",
    ("media-recorder", "set-onpause", "value"): "event-handler",
    ("media-recorder", "set-onresume", "value"): "event-handler",
    ("media-recorder", "set-onerror", "value"): "event-handler",
    # SpeechSynthesis event handlers
    ("speech-synthesis", "set-onvoiceschanged", "value"): "event-handler",
    # SpeechSynthesisUtterance event handlers
    ("speech-synthesis-utterance", "set-onstart", "value"): "event-handler",
    ("speech-synthesis-utterance", "set-onend", "value"): "event-handler",
    ("speech-synthesis-utterance", "set-onerror", "value"): "event-handler",
    ("speech-synthesis-utterance", "set-onpause", "value"): "event-handler",
    ("speech-synthesis-utterance", "set-onresume", "value"): "event-handler",
    ("speech-synthesis-utterance", "set-onmark", "value"): "event-handler",
    ("speech-synthesis-utterance", "set-onboundary", "value"): "event-handler",
    # MediaDevices event handlers
    ("media-devices", "set-ondevicechange", "value"): "event-handler",
    # RTCDtlsTransport event handlers (onerror already has type assertion)
    ("rtc-dtls-transport", "set-onerror", "value"): "event-handler",
    # ServiceWorkerContainer event handlers
    ("service-worker-container", "set-onmessage", "value"): "event-handler",
    ("service-worker-container", "set-onmessageerror", "value"): "event-handler",
    ("service-worker-container", "set-oncontrollerchange", "value"): "event-handler",
    # ServiceWorkerRegistration event handlers
    ("service-worker-registration", "set-onupdatefound", "value"): "event-handler",
    # PaymentRequest event handlers
    ("payment-request", "set-onshippingaddresschange", "value"): "event-handler",
    ("payment-request", "set-onshippingoptionchange", "value"): "event-handler",
    ("payment-request", "set-onpaymentmethodchange", "value"): "event-handler",
    # Performance event handlers
    ("performance", "set-onresourcetimingbufferfull", "value"): "event-handler",
    # Notification event handlers
    ("notification", "set-onclick", "value"): "event-handler",
    ("notification", "set-onshow", "value"): "event-handler",
    ("notification", "set-onerror", "value"): "event-handler",
    ("notification", "set-onclose", "value"): "event-handler",
    # WindowEventHandlers event handlers
    ("window-event-handlers", "set-onpagereveal", "value"): "event-handler",
    # WebCodecs event handlers
    ("audio-decoder", "set-ondequeue", "value"): "event-handler",
    ("video-decoder", "set-ondequeue", "value"): "event-handler",
    ("audio-encoder", "set-ondequeue", "value"): "event-handler",
    ("video-encoder", "set-ondequeue", "value"): "event-handler",
    # GlobalEventHandlers - missing event handlers
    ("global-event-handlers", "set-onbeforeinput", "value"): "event-handler",
    ("global-event-handlers", "set-onbeforematch", "value"): "event-handler",
    ("global-event-handlers", "set-onbeforetoggle", "value"): "event-handler",
    ("global-event-handlers", "set-onresize", "value"): "event-handler",
    ("global-event-handlers", "set-ontouchstart", "value"): "event-handler",
    ("global-event-handlers", "set-ontouchend", "value"): "event-handler",
    ("global-event-handlers", "set-ontouchmove", "value"): "event-handler",
    ("global-event-handlers", "set-ontouchcancel", "value"): "event-handler",
    # MediaQueryList event handler
    ("media-query-list", "set-onchange", "value"): "event-handler",
    # Document event handlers
    ("document", "set-onfullscreenchange", "value"): "event-handler",
    ("document", "set-onfullscreenerror", "value"): "event-handler",
    ("document", "set-onreadystatechange", "value"): "event-handler",
    ("document", "set-onvisibilitychange", "value"): "event-handler",
    # Element event handlers
    ("element", "set-onfullscreenchange", "value"): "event-handler",
    ("element", "set-onfullscreenerror", "value"): "event-handler",
    # VisualViewport event handlers
    ("visual-viewport", "set-onresize", "value"): "event-handler",
    ("visual-viewport", "set-onscroll", "value"): "event-handler",
    # AbortSignal event handler
    ("abort-signal", "set-onabort", "value"): "event-handler",
    # ShadowRoot event handler
    ("shadow-root", "set-onslotchange", "value"): "event-handler",
    # XMLHttpRequestEventTarget event handlers
    ("xml-http-request-event-target", "set-onloadstart", "value"): "event-handler",
    ("xml-http-request-event-target", "set-onprogress", "value"): "event-handler",
    ("xml-http-request-event-target", "set-onabort", "value"): "event-handler",
    ("xml-http-request-event-target", "set-onerror", "value"): "event-handler",
    ("xml-http-request-event-target", "set-onload", "value"): "event-handler",
    ("xml-http-request-event-target", "set-ontimeout", "value"): "event-handler",
    ("xml-http-request-event-target", "set-onloadend", "value"): "event-handler",
    # XMLHttpRequest event handlers
    ("xml-http-request", "set-onreadystatechange", "value"): "event-handler",
    # TextTrackCue event handlers
    ("text-track-cue", "set-onenter", "value"): "event-handler",
    ("text-track-cue", "set-onexit", "value"): "event-handler",
    # TextTrack event handlers
    ("text-track", "set-oncuechange", "value"): "event-handler",
    # TextTrackList event handlers
    ("text-track-list", "set-onchange", "value"): "event-handler",
    ("text-track-list", "set-onaddtrack", "value"): "event-handler",
    ("text-track-list", "set-onremovetrack", "value"): "event-handler",
    # OffscreenCanvas event handlers
    ("offscreen-canvas", "set-oncontextlost", "value"): "event-handler",
    ("offscreen-canvas", "set-oncontextrestored", "value"): "event-handler",
    # NavigationHistoryEntry event handlers
    ("navigation-history-entry", "set-ondispose", "value"): "event-handler",
    # EventSource event handlers
    ("event-source", "set-onopen", "value"): "event-handler",
    ("event-source", "set-onmessage", "value"): "event-handler",
    ("event-source", "set-onerror", "value"): "event-handler",
    # MessagePort event handlers
    ("message-port", "set-onmessage", "value"): "event-handler",
    ("message-port", "set-onmessageerror", "value"): "event-handler",
    # BroadcastChannel event handlers
    ("broadcast-channel", "set-onmessage", "value"): "event-handler",
    ("broadcast-channel", "set-onmessageerror", "value"): "event-handler",
    # MessageEventTarget event handlers
    ("message-event-target", "set-onmessage", "value"): "event-handler",
    ("message-event-target", "set-onmessageerror", "value"): "event-handler",
    # AbstractWorker event handlers
    ("abstract-worker", "set-onerror", "value"): "event-handler",
    # SpeechSynthesisUtterance voice handle conversion
    ("speech-synthesis-utterance", "set-voice", "value"): "optional-handle:speech-synthesis-voice",
    # MediaSession playbackState enum setter
    ("media-session", "set-playback-state", "value"): "enum-string",
    # MediaMetadata artwork handle array setter
    ("media-metadata", "set-artwork", "value"): "handle-array:media-image",
    # DocumentOrShadowRoot adoptedStyleSheets handle array setter
    ("document-or-shadow-root", "set-adopted-style-sheets", "value"): "handle-array:css-style-sheet",
    # RTCRtpReceiver jitter buffer target
    ("rtc-rtp-receiver", "set-jitter-buffer-target", "value"): "number-or-null",
    # Document body setter
    ("document", "set-body", "value"): "optional-handle-strict:html-element",
    # Node setters
    ("node", "set-node-value", "value"): "string-or-null",
    ("node", "set-text-content", "value"): "string-or-null",
    # HTMLElement focus options
    ("html-or-svg-element", "focus", "options"): "dictionary:focus-options",
    # HTMLFormElement requestSubmit
    ("html-form-element", "request-submit", "submitter"): "optional-handle:html-element",
    # HTMLInputElement/TextAreaElement setSelectionRange direction
    ("html-input-element", "set-selection-range", "direction"): "enum-string",
    ("html-text-area-element", "set-selection-range", "direction"): "enum-string",
    # htmlGlue.ts fixes - additional enum setters
    ("htmli-frame-element", "set-referrer-policy", "value"): "enum-string",
    ("htmli-frame-element", "set-loading", "value"): "enum-string",
    ("html-input-element", "set-selection-direction", "value"): "enum-string",
    ("html-text-area-element", "set-selection-direction", "value"): "enum-string",
    ("canvas-compositing", "set-global-composite-operation", "value"): "enum-string",
    ("canvas-fill-stroke-styles", "set-stroke-style", "value"): "any",
    ("canvas-fill-stroke-styles", "set-fill-style", "value"): "any",
    ("data-transfer", "set-drop-effect", "value"): "enum-string",
    ("data-transfer", "set-effect-allowed", "value"): "enum-string",
    # CanvasImageData settings - any type
    ("canvas-image-data", "get-image-data", "settings"): "any",
    # Path2D addPath
    ("path-2d", "add-path", "path"): "handle:path-2-d",
    # CanvasImageData createImageData settings
    ("canvas-image-data", "create-image-data", "settings"): "dictionary:image-data-settings",
    # Path2D addPath
    ("path-2d", "add-path", "path"): "handle:path-2-d",
    # htmlGlue.ts PARAMETER_BIGINT_TO_NUMBER fixes
    # OffscreenCanvas width/height setters need number conversion
    ("offscreencanvas", "set-width", "value"): True,
    ("offscreencanvas", "set-height", "value"): True,
    # CanvasPattern setTransform parameter
    ("canvas-pattern", "set-transform", "transform"): "optional-handle-strict:dom-matrix",
    # ElementInternals setValidity anchor - null vs undefined
    ("element-internals", "set-validity", "anchor"): "optional-handle-strict:html-element",
    # Path2D addPath transform
    ("path-2d", "add-path", "transform"): "optional-handle-strict:dom-matrix",
    # OffscreenCanvas width/height setters need number conversion
    ("offscreencanvas", "set-width", "value"): True,
    ("offscreencanvas", "set-height", "value"): True,
    # HTMLIFrameElement srcdoc setter - enum property (HTMLSrcdoc)
    ("htmli-frame-element", "set-srcdoc", "value"): "string",
    # HTMLFormElement getElements - need any cast for HTMLFormControlsCollection
    ("html-form-element", "get-elements", "elements"): "any",
    # HTMLInputElement/TextAreaElement setSelectionRange direction - needs string conversion
    ("html-input-element", "set-selection-range", "direction"): "string",
    ("html-text-area-element", "set-selection-range", "direction"): "string",
    # HTMLOrSVGElement nonce getter - returns string | undefined
    ("html-or-svg-element", "get-nonce", "nonce"): "string-or-null",
    ("path-2d", "add-path", "transform"): "dictionary:dom-matrix",
    # OffscreenCanvas convertToBlob options
    ("offscreen-canvas", "convert-to-blob", "options"): "dictionary:image-encode-options",
    # OffscreenCanvas width/height setters - need number conversion
    ("offscreen-canvas", "set-width", "value"): True,
    ("offscreen-canvas", "set-height", "value"): True,
    # CustomElementRegistry upgrade
    ("custom-element-registry", "upgrade", "root"): "handle:node",
    # ElementInternals setFormValue
    ("element-internals", "set-form-value", "value"): "any",
    ("element-internals", "set-form-value", "state"): "any",
    # ElementInternals setValidity
    ("element-internals", "set-validity", "flags"): "dictionary:validity-state-flags",
    ("element-internals", "set-validity", "anchor"): "optional-handle-strict:html-element",
    # DataTransfer setDragImage
    ("data-transfer", "set-drag-image", "image"): "handle:element",
    # AnimationFrameProvider requestAnimationFrame
    ("animation-frame-provider", "request-animation-frame", "callback"): "dictionary:frame-request-callback",
    # IntersectionObserver unobserve target
    ("intersection-observer", "unobserve", "target"): "handle:element",
    # ResizeObserver observe options
    ("resize-observer", "observe", "options"): "dictionary:resize-observer-options",
    # Return type casts - properties that return values that need "as any" cast
    # Properties that return readonly arrays that need spreading
    ("data-transfer", "types"): "readonly-array",
    ("clipboard-item", "types"): "readonly-array",
    ("performance", "entryList"): "readonly-array",
    ("navigator-language", "languages"): "readonly-array",
    ("performance-observer", "supportedEntryTypes"): "readonly-array",
    ("gamepad", "axes"): "readonly-array",
    ("gamepad", "buttons"): "readonly-array",
    # Location ancestorOrigins
    ("location", "ancestorOrigins"): "readonly-array",
    # IntersectionObserverEntry root
    ("intersection-observer-entry", "rootBounds"): "readonly-array",
    # ResizeObserverEntry borderBoxSize/contentBoxSize/devicePixelContentBoxSize
    ("resize-observer-entry", "devicePixelContentBoxSize"): "readonly-array",
    ("resize-observer-size-list", "borderBoxSize/contentBoxSize/devicePixelContentBoxSize"): "readonly-array",

    # RTCSctpTransport.transport - wrong handle table
    ("rtc-sctp-transport", "getTransport"): "rtc-dtls-transport",
    # Window clearInterval/clearTimeout - id needs number conversion
    ("window-or-worker-global-scope", "clear-interval", "id"): True,
    ("window-or-worker-global-scope", "clear-timeout", "id"): True,
    # URLSearchParams.has - value needs string conversion
    ("url-search-params", "has", "value"): "string",
    # URL.canParse - base needs string conversion
    ("url", "can-parse", "base"): "string-or-url",
    # RTCDTMFSender.insertDTMF - duration/interToneGap need number conversion
    ("rtc-dtmf-sender", "insert-dtmf", "duration"): True,
    ("rtc-dtmf-sender", "insert-dtmf", "inter-tone-gap"): True,
    # Cache.keys - request needs any (RequestInfo)
    ("cache", "keys", "request"): "any",
    ("cache", "keys", "options"): "any",
    # CacheStorage.has/delete/open - cacheName needs string conversion
    ("cache-storage", "has", "cache-name"): "string",
    ("cache-storage", "delete", "cache-name"): "string",
    ("cache-storage", "open", "cache-name"): "string",
    ("cache-storage", "match", "request"): "any",
    ("cache-storage", "match", "options"): "any",
    # IDBCursor.continue - key needs any
    ("idb-cursor", "continue", "key"): "any",
    # IntersectionObserver root - can be Document, need cast
    ("intersection-observer", "get-root", "root"): "any",
    # StyleSheet ownerNode - can be ProcessingInstruction, need cast
    ("style-sheet", "get-owner-node", "owner-node"): "any",
    # Document currentScript - HTMLOrSVGScriptElement, need cast
    ("document", "get-current-script", "script"): "any",
    # Event srcElement - EventTarget, need cast to Element
    ("event", "get-src-element", "src-element"): "any",
    # Window event - Event | undefined
    ("window", "get-event", "event"): "any",
    # TextEvent initTextEvent - view param needs optional window handle
    ("text-event", "init-text-event", "view"): "optional-handle:window",
    # TouchList item - index needs number conversion
    ("touch-list", "item", "index"): True,
    # CompositionEvent initCompositionEvent - view-arg needs window lookup
    ("composition-event", "init-composition-event", "view-arg"): "optional-handle:window",
    # Headers set - value needs any cast
    ("headers", "set", "value"): "any",
    # Response redirect - status needs number conversion
    ("response", "redirect", "status"): True,
    # RTCPeerConnection methods
    ("rtc-peer-connection", "generate-certificate", "keygen-algorithm"): "any",
    ("rtc-peer-connection", "add-track", "track"): "optional-handle-strict:media-stream-track",
    ("rtc-peer-connection", "remove-track", "sender"): "handle:rtc-rtp-sender",
    ("rtc-peer-connection", "add-transceiver", "track-or-kind"): "any",
    ("rtc-peer-connection", "create-data-channel", "data-channel-dict"): "dictionary:any",
    ("rtc-peer-connection", "get-stats", "selector"): "optional-handle:media-stream-track",
    # RTCRtpSender replaceTrack
    ("rtc-rtp-sender", "replace-track", "with-track"): "optional-handle:media-stream-track",
    # RTCPeerConnection createDataChannel - label is string not bigint
    ("rtc-peer-connection", "create-data-channel", "label"): "string",
    # RTCDTMFSender insertDTMF - duration needs number conversion
    ("rtc-dtmf-sender", "insert-dtmf", "duration"): True,
    # WebSocket send - data needs any cast
    ("ws", "send", "data"): "any",
    # WebSocket close - code needs number conversion
    ("ws", "close", "code"): True,
    # PerformanceObserverEntryList getEntriesByType - type is string not boolean
    ("performance-observer-entry-list", "get-entries-by-type", "type"): "string",
    # PerformanceObserverEntryList getEntriesByName - name is string not boolean
    ("performance-observer-entry-list", "get-entries-by-name", "name"): "string",
    # URLSearchParams delete - name is string not number
    ("url-search-params", "delete", "name"): "string",
    # URLSearchParams has - value param
    ("url-search-params", "has", "value"): "string",
    # Cache put - request needs any, response needs handle lookup
    ("cache", "put", "request"): "any",
    ("cache", "put", "response"): "handle:response",
    # RTCPeerConnection createDataChannel - label is string not bigint
    ("rtc-peer-connection", "create-data-channel", "label"): "string",
    # PerformanceObserverEntryList getEntriesByName - name is string not boolean
    ("performance-observer-entry-list", "get-entries-by-name", "name"): "string",
    # PerformanceObserverEntryList getEntriesByType - type is string not boolean
    ("performance-observer-entry-list", "get-entries-by-type", "type"): "string",
    # MediaStream getTrackById - trackId is string not boolean
    ("media-stream", "get-track-by-id", "track-id"): "string",
    # HtmlElement setPopoverTargetElement - element handle with null
    ("html-element", "set-popover-target-element", "element"): "optional-handle-strict:html-element",
    # Path2D addPath transform - DOMMatrix with null
    ("path-2d", "add-path", "transform"): "optional-handle-strict:dom-matrix",
    # RTCDataChannel createDataChannel - label is string not bigint
    ("rtc-peer-connection", "create-data-channel", "label"): "string",
    # RTCPeerConnection addTrack - streams is array of MediaStream (spread as rest params)
    ("rtc-peer-connection", "add-track", "streams"): "spread-handle-array:media-stream",
    # RTCPeerConnection addTransceiver - init is dictionary
    ("rtc-peer-connection", "add-transceiver", "init"): "dictionary:any",
    # RTCSctpTransport.transport - wrong handle table
    ("rtc-sctp-transport", "getTransport"): "rtc-dtls-transport",
    # Performance mark - markOptions is dictionary
    ("performance", "mark", "mark-options"): "dictionary:any",
    # ServiceWorkerContainer register - options is dictionary
    ("service-worker-container", "register", "options"): "dictionary:any",
    # Cache match - options is dictionary
    ("cache", "match", "options"): "dictionary:any",
    # Cache matchAll - options is dictionary
    ("cache", "match-all", "options"): "dictionary:any",
    # Cache delete - options is dictionary
    ("cache", "delete", "options"): "dictionary:any",
    # Cache put - request is any
    ("cache", "put", "request"): "any",
    # Cache add - request is any
    ("cache", "add", "request"): "any",
    # Cache addAll - requests is any
    ("cache", "add-all", "requests"): "any",
    # WebSocket send - data is any (BufferSource)
    ("ws", "send", "data"): "any",
    # WebSocket close - code is number
    ("ws", "close", "code"): True,
    # Worker constructor - scriptUrl is any
    ("worker", "constructor", "script-url"): "any",
    ("worker", "constructor", "options"): "dictionary:any",
    # URLSearchParams set - name is string not bigint
    ("url-search-params", "set", "name"): "string",
    # URLSearchParams append - name is string not bigint
    ("url-search-params", "append", "name"): "string",
    # ClipboardItem supports - type is string not boolean
    ("clipboard-item", "supports", "type"): "string",
    # TextEvent initTextEvent - view needs optional handle lookup
    ("text-event", "init-text-event", "view"): "optional-handle:window",
    # TextEvent initTextEvent - bubbles is boolean not string
    ("text-event", "init-text-event", "bubbles"): "boolean",
    # CompositionEvent initCompositionEvent - view-arg needs handle lookup
    ("composition-event", "init-composition-event", "view-arg"): "optional-handle:window",
    # SpeechSynthesis speak - utterance needs handle lookup
    ("speech-synthesis", "speak", "utterance"): "handle:speech-synthesis-utterance",
    # PaymentRequest show - detailsPromise is any
    ("payment-request", "show", "details-promise"): "any",
    # PaymentResponse retry - errorFields is dictionary
    ("payment-response", "retry", "error-fields"): "dictionary:any",
    # ServiceWorkerContainer register - scriptUrl is string
    ("service-worker-container", "register", "script-url"): "string",
    # Headers append - value is string
    ("headers", "append", "value"): "string",
    # ExtendableMessageEvent getSource - source is any
    ("extendable-message-event", "get-source", "source"): "any",
    # Notification constructor - options is dictionary
    ("notification", "constructor", "options"): "dictionary:any",
    # Cache matchAll - request needs any
    ("cache", "match-all", "request"): "any",
    # Cache delete - request needs any
    ("cache", "delete", "request"): "any",
    # Geolocation watchPosition - success-callback needs handle lookup
    ("geolocation", "watch-position", "success-callback"): "handle:position-callback",
    # CompositionEvent initCompositionEvent - view-arg needs window handle
    ("composition-event", "init-composition-event", "view-arg"): "optional-handle:window",
    # TextEvent initTextEvent - view needs window handle
    ("text-event", "init-text-event", "view"): "optional-handle:window",
    # WebSocket close - reason needs string conversion
    ("web-socket", "close", "reason"): "string",
    # WebSocket close - code needs number conversion
    ("web-socket", "close", "code"): True,
    # Ws close - reason needs string conversion
    ("ws", "close", "reason"): "string",
    # Ws close - code needs number conversion
    ("ws", "close", "code"): True,
    # ReadableStreamBYOBRequest respondWithNewView - view needs any
    ("readable-stream-byob-request", "respond-with-new-view", "view"): "any",
    # RTCPeerConnection addTransceiver - streams needs spread handle array
    ("rtc-peer-connection", "add-transceiver", "streams"): "spread-handle-array:media-stream",
    # RTCRtpReceiver setJitterBufferTarget - value needs number-or-null
    ("rtc-rtp-receiver", "set-jitter-buffer-target", "value"): "number-or-null",
    # Performance clearMarks - mark-name needs string
    ("performance", "clear-marks", "mark-name"): "string",
    # PerformanceObserverEntryList getEntriesByName - name needs string
    ("performance-observer-entry-list", "get-entries-by-name", "name"): "string",
    # PerformanceObserverEntryList getEntriesByType - entry-type needs string
    ("performance-observer-entry-list", "get-entries-by-type", "entry-type"): "string",
    # IntersectionObserverEntry getBoundingClientRect - rect needs any
    ("intersection-observer-entry", "get-bounding-client-rect", "rect"): "any",
    # PaymentResponse onpayerdetailchange setter - value needs event handler
    ("payment-response", "set-onpayerdetailchange", "value"): "event-handler",
    # Animation updatePlaybackRate - playback-rate needs number conversion
    ("animation", "update-playback-rate", "playback-rate"): True,
    # Cache delete - options needs any
    ("cache", "delete", "options"): "any",
    # URL.canParse - url is bigint but needs string conversion
    ("url", "can-parse", "url"): "string",
    # URL.setHost - value is bigint but needs string conversion
    ("url", "set-host", "value"): "string",
    # URL.setPort - value is boolean but needs string conversion
    ("url", "set-port", "value"): "string",
    # URLSearchParams.delete - value is bigint but needs string conversion
    ("url-search-params", "delete", "value"): "string",
    # IntersectionObserver.observe - target is number but needs handle lookup
    ("intersection-observer", "observe", "target"): "handle:element",
    # Response.redirect - url is bigint but needs string conversion
    ("response", "redirect", "url"): "string",
    # ReadableStreamBYOBReader.read - view is bigint but needs any
    ("readable-stream-byob-reader", "read", "view"): "any",
    # SubtleCrypto.decrypt - algorithm is bigint but needs string conversion
    ("subtle-crypto", "decrypt", "algorithm"): "string",
    # ServiceWorkerRegistration.showNotification - title is boolean but needs string conversion
    ("service-worker-registration", "show-notification", "title"): "string",
    # SpeechSynthesisUtterance.setVoice - value is boolean but needs handle lookup
    ("speech-synthesis-utterance", "set-voice", "value"): "optional-handle:speech-synthesis-voice",
    # SubtleCrypto.decrypt - algorithm is any
    ("subtle-crypto", "decrypt", "algorithm"): "any",
    # SubtleCrypto.decrypt - key is boolean but needs handle lookup
    ("subtle-crypto", "decrypt", "key"): "handle:crypto-key",
    # Geolocation.getCurrentPosition - callbacks need handle lookup
    ("geolocation", "get-current-position", "success-callback"): "handle:position-callback",
    ("geolocation", "get-current-position", "error-callback"): "optional-handle:position-error-callback",
    # Geolocation.getCurrentPosition - options is dictionary
    ("geolocation", "get-current-position", "options"): "dictionary:any",
    # Geolocation.clearWatch - watchId is string but needs number conversion
    ("geolocation", "clear-watch", "watch-id"): True,
    # KeyboardEvent.initKeyboardEvent - viewArg is string but needs handle lookup
    ("keyboard-event", "init-keyboard-event", "view-arg"): "optional-handle:window",
    # KeyboardEvent.initKeyboardEvent - ctrlKey/shiftKey are string but need boolean conversion
    ("keyboard-event", "init-keyboard-event", "ctrl-key"): "boolean",
    ("keyboard-event", "init-keyboard-event", "shift-key"): "boolean",
    # CompositionEvent.initCompositionEvent - dataArg is boolean but needs string conversion
    ("composition-event", "init-composition-event", "data-arg"): "string",
    # TextEvent.initTextEvent - view is boolean but needs handle lookup
    ("text-event", "init-text-event", "view"): "optional-handle:window",
    # WindowOrWorkerGlobalScope.queueMicrotask - callback is bigint but needs VoidFunction
    ("window-or-worker-global-scope", "queue-microtask", "callback"): "any",
    # MediaSession.setMicrophoneActive - active is number but needs boolean
    ("media-session", "set-microphone-active", "active"): "boolean",
    # MediaMetadata.setAlbum - value is number but needs string
    ("media-metadata", "set-album", "value"): "string",
    # MediaMetadata.setArtwork - value is string[] but needs handle array lookup
    ("media-metadata", "set-artwork", "value"): "handle-array:media-image",
    # ServiceWorkerRegistration.showNotification - options is bigint but needs dictionary
    ("service-worker-registration", "show-notification", "options"): "dictionary:any",
    # IntersectionObserver.unobserve - target is bigint | undefined but needs optional handle lookup
    ("intersection-observer", "unobserve", "target"): "optional-handle-strict:element",
    # ResizeObserver.observe - target is string but needs handle lookup
    ("resize-observer", "observe", "target"): "handle:element",
    # ResizeObserver.observe - options is EventHandlerRecord but needs dictionary
    ("resize-observer", "observe", "options"): "dictionary:any",
    # URLSearchParams.set - value is bigint but needs string
    ("url-search-params", "set", "value"): "string",
    # RTCPeerConnection.getStats - selector is string but needs handle lookup
    ("rtc-peer-connection", "get-stats", "selector"): "optional-handle:media-stream-track",
    # RTCIceCandidate sdpMLineIndex - is number | undefined but needs bigint
    ("rtc-ice-candidate", "get-sdp-m-line-index", "sdp-m-line-index"): True,
    # RTCDTMFSender.insertDTMF - duration is string but needs number conversion
    ("rtc-dtmf-sender", "insert-dtmf", "duration"): True,
    # IdbObjectStore.setName - value is bigint but needs string
    ("idb-object-store", "set-name", "value"): "string",
    # SubtleCrypto.deriveBits - algorithm is number but needs any
    ("subtle-crypto", "derive-bits", "algorithm"): "any",
    # SubtleCrypto.deriveBits - baseKey is number but needs handle lookup
    ("subtle-crypto", "derive-bits", "base-key"): "handle:crypto-key",
    # Geolocation.watchPosition - options is bigint but needs dictionary
    ("geolocation", "watch-position", "options"): "dictionary:any",
    # ClipboardItem.supports - returns boolean but needs string
    ("clipboard-item", "supports", "type"): "string",
    # WindowOrWorkerGlobalScope.atob - data is bigint but needs string
    ("window-or-worker-global-scope", "atob", "data"): "string",
    # SpeechSynthesisUtterance.setLang - value is boolean but needs string
    ("speech-synthesis-utterance", "set-lang", "value"): "string",
    # PaymentRequest.getId - returns string but needs number
    ("payment-request", "get-id", "id"): True,
    # Performance.getEntriesByName - type is number but needs string
    ("performance", "get-entries-by-name", "type"): "string",
    # URL.parse - base is boolean but needs string conversion
    ("url", "parse", "base"): "string-or-url",
    # Geolocation.watchPosition - errorCallback is string but needs handle lookup
    ("geolocation", "watch-position", "error-callback"): "optional-handle:position-error-callback",
    # KeyboardEvent.initKeyboardEvent - viewArg is boolean but needs handle lookup
    ("keyboard-event", "init-keyboard-event", "view-arg"): "optional-handle:window",
    # KeyboardEvent.initKeyboardEvent - altKey is string but needs boolean conversion
    ("keyboard-event", "init-keyboard-event", "alt-key"): "boolean",
    # TextEvent.initTextEvent - cancelable is string but needs boolean conversion
    ("text-event", "init-text-event", "cancelable"): "boolean",
    # SpeechSynthesisUtterance.setRate - value is string but needs number conversion
    ("speech-synthesis-utterance", "set-rate", "value"): True,
    # URL.setSearch - value is bigint but needs string conversion
    ("url", "set-search", "value"): "string",
    # RTCDTMFSender.insertDTMF - duration is bigint but needs number conversion
    ("rtc-dtmf-sender", "insert-dtmf", "duration"): True,
    # KeyboardEvent.initKeyboardEvent - locationArg is string but needs number conversion
    ("keyboard-event", "init-keyboard-event", "location-arg"): True,
    # TextEvent.initTextEvent - view is boolean but needs handle lookup
    ("text-event", "init-text-event", "view-arg"): "optional-handle:window",
    # SpeechRecognitionResultList item - index is string but needs number conversion
    ("speech-recognition-result-list", "item", "index"): True,
    # SpeechRecognitionResult item - index is string but needs number conversion
    ("speech-recognition-result", "item", "index"): True,
    # IDBCursor.advance - count is string but needs number conversion
    ("idb-cursor", "advance", "count"): True,
    # WebSocket.send - data is string | undefined but needs any cast
    ("ws", "send", "data"): "any",
    # SubtleCrypto deriveKey algorithm - needs any cast
    ("subtle-crypto", "derive-key", "algorithm"): "any",
    ("subtle-crypto", "derive-key", "derived-key-type"): "any",
    # XPathExpression evaluate type parameter
    ("x-path-expression", "evaluate", "type"): True,
    # eventsGlue.ts fixes
    # Clipboard.writeText - data is bigint but needs string conversion
    ("clipboard", "write-text", "data"): "string",
    # KeyboardEvent.getModifierState - keyArg is boolean but needs string conversion
    ("keyboard-event", "get-modifier-state", "key-arg"): "string",
    # TextEvent.initTextEvent - data is bigint but needs string conversion
    ("text-event", "init-text-event", "data"): "string",
    # SpeechSynthesisUtterance - text needs string conversion
    ("speech-synthesis-utterance", "set-text", "value"): "string",
    # CompositionEvent initCompositionEvent - viewArg needs window handle lookup
    ("composition-event", "init-composition-event", "view-arg"): "optional-handle:window",
    # URL setUsername - value needs string conversion
    ("url", "set-username", "value"): "string",
    # RTCDataChannel bufferedAmountLowThreshold - value needs number conversion
    ("rtc-data-channel", "set-buffered-amount-low-threshold", "value"): True,
}

# Properties that are enums (string in DOM, bigint in WIT)
# Maps (interface, property) to enum type name
ENUM_PROPERTIES = {
    ("audio-decoder", "state"): "CodecState",
    ("video-decoder", "state"): "CodecState",
    ("audio-encoder", "state"): "CodecState",
    ("video-encoder", "state"): "CodecState",
    ("encoded-audio-chunk", "type"): "EncodedAudioChunkType",
    ("encoded-video-chunk", "type"): "EncodedVideoChunkType",
    ("audio-data", "format"): "AudioSampleFormat",
    ("video-frame", "format"): "VideoPixelFormat",
    ("video-color-space", "primaries"): "VideoColorPrimaries",
    ("video-color-space", "transfer"): "VideoTransferCharacteristics",
    ("video-color-space", "matrix"): "VideoMatrixCoefficients",
    # WebGL color space
    ("web-gl-rendering-context-base", "drawingBufferColorSpace"): "PredefinedColorSpace",
    ("web-gl-rendering-context-base", "unpackColorSpace"): "PredefinedColorSpace",
    # Gamepad mapping
    ("gamepad", "mapping"): "GamepadMappingType",
    # ShadowRoot mode
    ("shadow-root", "mode"): "ShadowRootMode",
    # Document visibility
    ("document", "visibilityState"): "VisibilityState",
    # ShadowRoot slot assignment
    ("shadow-root", "slotAssignment"): "SlotAssignmentMode",
    # DOMTokenList value
    ("dom-token-list", "value"): "DOMTokenListValue",
    # ClipboardItem presentation style
    ("clipboard-item", "presentationStyle"): "PresentationStyle",
    # Request enums
    ("request", "credentials"): "RequestCredentials",
    ("request", "cache"): "RequestCache",
    ("request", "redirect"): "RequestRedirect",
    ("request", "duplex"): "RequestDuplex",
    # Response type
    ("response", "type"): "ResponseType",
    # XMLHttpRequest response type
    ("xml-http-request", "responseType"): "XMLHttpRequestResponseType",
    # HTMLIFrameElement srcdoc (note: WIT uses htmli-frame-element)
    ("htmli-frame-element", "srcdoc"): "HTMLSrcdoc",
    # Canvas enums
    ("canvas-image-smoothing", "imageSmoothingQuality"): "ImageSmoothingQuality",
    ("canvas-path-drawing-styles", "lineCap"): "CanvasLineCap",
    ("canvas-path-drawing-styles", "lineJoin"): "CanvasLineJoin",
    ("canvas-text-drawing-styles", "textAlign"): "CanvasTextAlign",
    ("canvas-text-drawing-styles", "textBaseline"): "CanvasTextBaseline",
    ("canvas-text-drawing-styles", "direction"): "CanvasDirection",
    ("canvas-text-drawing-styles", "fontKerning"): "CanvasFontKerning",
    ("canvas-text-drawing-styles", "fontStretch"): "CanvasFontStretch",
    ("canvas-text-drawing-styles", "fontVariantCaps"): "CanvasFontVariantCaps",
    ("canvas-text-drawing-styles", "textRendering"): "CanvasTextRendering",
    # ImageData color space
    ("image-data", "colorSpace"): "PredefinedColorSpace",
    # Navigation activation type
    ("navigation-activation", "navigationType"): "NavigationType",
    # Location href
    ("location", "href"): "LocationHref",
    # URL href - removed, href is just a string property, not an enum
    # HTMLHyperlinkElementUtils href
    ("html-hyperlink-element-utils", "href"): "HTMLHyperlinkHref",
    # MediaStreamTrack ready state
    ("media-stream-track", "readyState"): "MediaStreamTrackState",
    # MediaDeviceInfo kind
    ("media-device-info", "kind"): "MediaDeviceKind",
    # History scroll restoration
    ("history", "scrollRestoration"): "ScrollRestoration",
    # TextTrack mode
    ("text-track", "mode"): "TextTrackMode",
    # MediaList mediaText
    ("media-list", "mediaText"): "MediaText",
    # Request enums (additional)
    ("request", "destination"): "RequestDestination",
    ("request", "referrerPolicy"): "ReferrerPolicy",
    ("request", "mode"): "RequestMode",
    # Response url
    ("response", "url"): "ResponseUrl",
    # HTMLMediaElement canPlayType (method returns enum)
    ("html-media-element", "canPlayType"): "CanPlayTypeResult",
    # MediaRecorder state
    ("media-recorder", "state"): "MediaRecorderState",
    # SpeechRecognitionAlternative transcript
    ("speech-recognition-alternative", "transcript"): "TranscriptString",
    # SpeechSynthesisUtterance text
    ("speech-synthesis-utterance", "text"): "TextString",
    # SpeechSynthesisVoice name
    ("speech-synthesis-voice", "name"): "VoiceName",
    # ServiceWorkerRegistration updateViaCache
    ("service-worker-registration", "updateViaCache"): "ServiceWorkerUpdateViaCache",
    # PerformanceNavigationTiming type
    ("performance-navigation-timing", "type"): "NavigationTimingType",
    # PermissionStatus state/name
    ("permission-status", "state"): "PermissionState",
    ("permission-status", "name"): "PermissionName",
    # URL host
    ("url", "host"): "UrlHost",
    # RTC enums
    ("rtc-rtp-transceiver", "direction"): "RTCRtpTransceiverDirection",
    ("rtc-dtls-transport", "state"): "RTCDtlsTransportState",
    ("rtc-ice-transport", "state"): "RTCIceTransportState",
    ("rtc-ice-transport", "gatheringState"): "RTCIceGatheringState",
    ("rtc-sctp-transport", "state"): "RTCSctpTransportState",
    ("rtc-data-channel", "readyState"): "RTCDataChannelState",
    ("rtc-data-channel", "binaryType"): "RTCDataChannelBinaryType",
    # WebSocket binaryType (note: WIT uses web-socket)
    ("web-socket", "binaryType"): "WebSocketBinaryType",
    # ServiceWorker state
    ("service-worker", "state"): "ServiceWorkerState",
    # Request method/referrer (string→bigint enums)
    ("request", "method"): "HttpMethod",
    ("request", "referrer"): "ReferrerUrl",
    # Gamepad id (string → bigint)
    ("gamepad", "id"): "GamepadId",
    # HTMLMediaElement canPlayType/preload
    ("html-media-element", "canPlayType"): "CanPlayTypeResult",
    ("html-media-element", "preload"): "PreloadString",
    # Notification tag
    ("notification", "tag"): "NotificationTag",
    ("notification", "title"): "NotificationTitle",
    # Request URL (string → bigint)
    ("request", "getUrl"): "RequestUrl",
    # XMLHttpRequest responseURL
    ("xml-http-request", "responseURL"): "ResponseUrl",
    # Notification permission
    ("notification", "permission"): "NotificationPermission",
    # RTCSessionDescription type
    ("rtc-session-description", "type"): "RTCSdpType",
    # RTCIceCandidate type
    ("rtc-ice-candidate", "type"): "RTCIceCandidateType",
    # RTCIceCandidate tcpType
    ("rtc-ice-candidate", "tcpType"): "RTCIceTcpCandidateType",
    # RTCDataChannel protocol
    ("rtc-data-channel", "protocol"): "DataChannelProtocol",
    # SpeechSynthesisErrorCode
    ("speech-synthesis-error-event", "error"): "SpeechSynthesisErrorCode",
    # PointerEvent pointerType
    ("pointer-event", "pointerType"): "PointerType",
    # InputEvent inputType
    ("input-event", "inputType"): "InputType",
    # Crypto randomUUID returns UUID string
    ("crypto", "random-uuid"): "UUID",
    # PerformanceEntry entryType returns string
    ("performance-entry", "entryType"): "PerformanceEntryType",
    # MediaSession playbackState returns string
    ("media-session", "playbackState"): "MediaSessionPlaybackState",
    # RTCError errorDetail returns string
    ("rtc-error", "errorDetail"): "RTCErrorDetailType",
    # KeyboardEvent key returns string (needs enum conversion)
    ("keyboard-event", "key"): "KeyString",
    # CompositionEvent data returns string (needs enum conversion)
    ("composition-event", "data"): "CompositionData",
    # MediaStreamTrack kind returns string
    ("media-stream-track", "kind"): "MediaStreamTrackKind",
    # RTCIceCandidate enum properties
    ("rtc-ice-candidate", "protocol"): "RTCIceProtocol",
    ("rtc-ice-candidate", "relayProtocol"): "RTCIceProtocol",
    ("rtc-ice-candidate", "usernameFragment"): "RTCIceUsernameFragment",
    ("rtc-ice-candidate", "component"): "RTCIceComponent",
    # RTCIceCandidate candidate/sdpMid (string → bigint)
    ("rtc-ice-candidate", "candidate"): "IceCandidateString",
    ("rtc-ice-candidate", "sdpMid"): "SdpMidString",
    # RTCRtpTransceiver currentDirection
    ("rtc-rtp-transceiver", "currentDirection"): "RTCRtpTransceiverDirection",
    # Notification dir (string → bigint)
    ("notification", "dir"): "NotificationDirection",
    # IntersectionObserver rootMargin (string → number config)
    ("intersection-observer", "rootMargin"): "RootMarginString",
    # RTCIceCandidate foundation (string → bigint)
    ("rtc-ice-candidate", "foundation"): "IceCandidateFoundation",
    # RTCDTMFSender toneBuffer (string → bigint)
    ("rtcdtmf-sender", "toneBuffer"): "ToneBufferString",
    # PaymentRequest shippingType (string enum → bigint)
    ("payment-request", "shippingType"): "PaymentShippingType",
    # PaymentMethodChangeEvent methodName (string → bigint)
    ("payment-method-change-event", "methodName"): "MethodNameString",
    # PaymentResponse payerEmail (string → bigint)
    ("payment-response", "payerEmail"): "PayerEmailString",
    # PaymentResponse methodName (string → bigint)
    ("payment-response", "methodName"): "MethodNameString",
    # SpeechSynthesisUtterance lang (string → bigint)
    ("speech-synthesis-utterance", "lang"): "SpeechLang",
    # GeolocationPositionError message (string → bigint)
    ("geolocation-position-error", "message"): "GeolocationErrorMessage",
    # RTCIceCandidate address (string → boolean)
    ("rtc-ice-candidate", "address"): "RTCIceCandidateAddress",
    # PerformanceNavigation redirectCount - removed, it's a number not an enum
    # URL protocol (string → bigint)
    ("url", "protocol"): "UrlProtocol",
    # URL toJSON (string → bigint)
    ("url", "to-json"): "UrlToJsonString",
    # RTCIceCandidate relatedAddress (string → bigint)
    ("rtc-ice-candidate", "related-address"): "RTCIceRelatedAddress",
    # RTCIceCandidate sdpMLineIndex (number → string)
    ("rtc-ice-candidate", "sdp-m-line-index"): "SdpMLineIndexString",
    # URL search (string → bigint)
    ("url", "search"): "UrlSearchString",
    # WheelEvent deltaX (number → string)
    ("wheel-event", "delta-x"): "WheelDeltaXString",
    # MediaRecorder audioBitsPerSecond (number → string)
    ("media-recorder", "audio-bits-per-second"): "AudioBitsPerSecondString",
    # SpeechRecognitionResultList length (number → boolean)

    # PerformanceNavigationTiming unloadEventEnd (number → string)
    ("performance-navigation-timing", "unload-event-end"): "UnloadEventEndString",
    # SpeechSynthesisVoice lang (string → bigint)
    ("speech-synthesis-voice", "lang"): "SpeechSynthesisVoiceLang",
    # PerformanceResourceTiming connectEnd (number → string)
    ("performance-resource-timing", "connect-end"): "PerformanceConnectEndString",
    # PerformanceResourceTiming responseStart (number → string)
    ("performance-resource-timing", "response-start"): "PerformanceResponseStartString",
    # PerformanceEntry name (string → bigint)
    ("performance-entry", "name"): "PerformanceEntryName",
    # SpeechSynthesisEvent name (string → number)
    ("speech-synthesis-event", "name"): "SpeechSynthesisEventName",
    # PaymentRequest id (string → number)
    ("payment-request", "id"): "PaymentRequestId",
    # RTCIceCandidate relatedAddress (string | undefined → bigint | undefined)
    ("rtc-ice-candidate", "related-address"): "RTCIceRelatedAddress",
    # RTCDataChannel bufferedAmountLowThreshold (number → string)
    ("rtc-data-channel", "buffered-amount-low-threshold"): "BufferedAmountLowThresholdString",
    # ExtendableMessageEvent lastEventId (string → number)
    ("extendable-message-event", "last-event-id"): "ExtendableMessageEventLastEventId",
}

# Enum value mappings (string → bigint)
# Values are Python integers that will be rendered as bigint literals (e.g., 0n)
ENUM_VALUE_MAPPINGS = {
    "CodecState": {
        "unconfigured": 0,
        "configured": 1,
        "closed": 2,
    },
    "EncodedAudioChunkType": {
        "key": 0,
        "delta": 1,
    },
    "EncodedVideoChunkType": {
        "key": 0,
        "delta": 1,
    },
    "AudioSampleFormat": {
        "u8": 0,
        "s16": 1,
        "s32": 2,
        "f32": 3,
        "u8-planar": 4,
        "s16-planar": 5,
        "s32-planar": 6,
        "f32-planar": 7,
    },
    "VideoPixelFormat": {
        "I420": 0,
        "NV12": 1,
        "RGBA": 2,
        "RGBX": 3,
        "BGRA": 4,
        "BGRX": 5,
    },
    "VideoColorPrimaries": {
        "bt709": 0,
        "bt470bg": 1,
        "smpte170m": 2,
    },
    "VideoTransferCharacteristics": {
        "bt709": 0,
        "smpte170m": 1,
        "iec61966-2-1": 2,
    },
    "VideoMatrixCoefficients": {
        "bt709": 0,
        "bt470bg": 1,
        "smpte170m": 2,
    },
    "PredefinedColorSpace": {
        "srgb": 0,
        "display-p3": 1,
    },
    "GamepadMappingType": {
        "": 0,
        "standard": 1,
        "xr-standard": 2,
    },
    "ShadowRootMode": {
        "open": 0,
        "closed": 1,
    },
    "VisibilityState": {
        "visible": 0,
        "hidden": 1,
    },
    "SlotAssignmentMode": {
        "manual": 0,
        "named": 1,
    },
    "DOMTokenListValue": {
        "": 0,
    },
    "PresentationStyle": {
        "unspecified": 0,
        "inline": 1,
        "attachment": 2,
    },
    "RequestCredentials": {
        "omit": 0,
        "same-origin": 1,
        "include": 2,
    },
    "RequestCache": {
        "default": 0,
        "no-store": 1,
        "reload": 2,
        "no-cache": 3,
        "force-cache": 4,
        "only-if-cached": 5,
    },
    "RequestRedirect": {
        "follow": 0,
        "error": 1,
        "manual": 2,
    },
    "RequestDuplex": {
        "half": 0,
    },
    "ResponseType": {
        "basic": 0,
        "cors": 1,
        "default": 2,
        "error": 3,
        "opaque": 4,
        "opaqueredirect": 5,
    },
    "XMLHttpRequestResponseType": {
        "": 0,
        "arraybuffer": 1,
        "blob": 2,
        "document": 3,
        "json": 4,
        "text": 5,
    },
    "HTMLSrcdoc": {
        "": 0,
    },
    "ImageSmoothingQuality": {
        "low": 0,
        "medium": 1,
        "high": 2,
    },
    "CanvasLineCap": {
        "butt": 0,
        "round": 1,
        "square": 2,
    },
    "CanvasLineJoin": {
        "round": 0,
        "bevel": 1,
        "miter": 2,
    },
    "CanvasTextAlign": {
        "left": 0,
        "right": 1,
        "center": 2,
        "start": 3,
        "end": 4,
    },
    "CanvasTextBaseline": {
        "top": 0,
        "hanging": 1,
        "middle": 2,
        "alphabetic": 3,
        "ideographic": 4,
        "bottom": 5,
    },
    "CanvasDirection": {
        "ltr": 0,
        "rtl": 1,
        "inherit": 2,
    },
    "CanvasFontKerning": {
        "auto": 0,
        "normal": 1,
        "none": 2,
    },
    "CanvasFontStretch": {
        "": 0,
    },
    "CanvasFontVariantCaps": {
        "": 0,
    },
    "CanvasTextRendering": {
        "auto": 0,
        "optimizeSpeed": 1,
        "optimizeLegibility": 2,
        "geometricPrecision": 3,
    },
    "NavigationType": {
        "push": 0,
        "replace": 1,
        "traverse": 2,
        "reload": 3,
    },
    "LocationHref": {
        "": 0,
    },
    "URLHref": {
        "": 0,
    },
    "HTMLHyperlinkHref": {
        "": 0,
    },
    "MediaStreamTrackState": {
        "live": 0,
        "ended": 1,
    },
    "MediaDeviceKind": {
        "audioinput": 0,
        "audiooutput": 1,
        "videoinput": 2,
    },
    "ScrollRestoration": {
        "auto": 0,
        "manual": 1,
    },
    "TextTrackMode": {
        "disabled": 0,
        "hidden": 1,
        "showing": 2,
    },
    "MediaText": {
        "": 0,
    },
    # New enum value mappings
    "RequestDestination": {
        "": 0,
        "audio": 1,
        "audioworklet": 2,
        "document": 3,
        "embed": 4,
        "font": 5,
        "image": 6,
        "manifest": 7,
        "object": 8,
        "paintworklet": 9,
        "report": 10,
        "script": 11,
        "sharedworker": 12,
        "style": 13,
        "track": 14,
        "video": 15,
        "worker": 16,
        "xslt": 17,
    },
    "ReferrerPolicy": {
        "": 0,
        "no-referrer": 1,
        "no-referrer-when-downgrade": 2,
        "origin": 3,
        "origin-when-cross-origin": 4,
        "same-origin": 5,
        "strict-origin": 6,
        "strict-origin-when-cross-origin": 7,
        "unsafe-url": 8,
    },
    "RequestMode": {
        "navigate": 0,
        "same-origin": 1,
        "no-cors": 2,
        "cors": 3,
    },
    "ResponseUrl": {
        "": 0,
    },
    "CanPlayTypeResult": {
        "": 0,
        "probably": 1,
        "maybe": 2,
    },
    "MediaRecorderState": {
        "inactive": 0,
        "recording": 1,
        "paused": 2,
    },
    "TranscriptString": {
        "": 0,
    },
    "TextString": {
        "": 0,
    },
    "VoiceName": {
        "": 0,
    },
    "ServiceWorkerUpdateViaCache": {
        "all": 0,
        "imports": 1,
        "none": 2,
    },
    "NavigationTimingType": {
        "navigate": 0,
        "reload": 1,
        "back_forward": 2,
        "prerender": 3,
    },
    "PermissionState": {
        "granted": 0,
        "denied": 1,
        "prompt": 2,
    },
    "PermissionName": {
        "": 0,
    },
    "UrlHost": {
        "": 0,
    },
    "RTCRtpTransceiverDirection": {
        "sendrecv": 0,
        "sendonly": 1,
        "recvonly": 2,
        "inactive": 3,
        "stopped": 4,
    },
    "RTCDtlsTransportState": {
        "new": 0,
        "connecting": 1,
        "connected": 2,
        "closed": 3,
        "failed": 4,
    },
    "RTCIceTransportState": {
        "new": 0,
        "checking": 1,
        "connected": 2,
        "completed": 3,
        "disconnected": 4,
        "closed": 5,
        "failed": 6,
    },
    "RTCIceGatheringState": {
        "new": 0,
        "gathering": 1,
        "complete": 2,
    },
    "RTCSctpTransportState": {
        "connecting": 0,
        "connected": 1,
        "closed": 2,
    },
    "RTCDataChannelState": {
        "connecting": 0,
        "open": 1,
        "closing": 2,
        "closed": 3,
    },
    "RTCDataChannelBinaryType": {
        "blob": 0,
        "arraybuffer": 1,
    },
    "WebSocketBinaryType": {
        "blob": 0,
        "arraybuffer": 1,
    },
    "ServiceWorkerState": {
        "installing": 0,
        "installed": 1,
        "activating": 2,
        "activated": 3,
        "redundant": 4,
    },
    # HTTP methods as enum
    "HttpMethod": {
        "": 0,
        "GET": 1,
        "POST": 2,
        "PUT": 3,
        "DELETE": 4,
        "HEAD": 5,
        "OPTIONS": 6,
        "PATCH": 7,
    },
    # Referrer URL as enum
    "ReferrerUrl": {
        "": 0,
    },
    # Gamepad ID as enum
    "GamepadId": {
        "": 0,
    },
    # Notification tag/title
    "NotificationTag": {
        "": 0,
    },
    "NotificationTitle": {
        "": 0,
    },
    # Request URL
    "RequestUrl": {
        "": 0,
    },
    "ResponseUrl": {
        "": 0,
    },
    # HTTP Method
    "HttpMethod": {
        "": 0,
        "GET": 1,
        "POST": 2,
        "PUT": 3,
        "DELETE": 4,
        "HEAD": 5,
        "OPTIONS": 6,
        "PATCH": 7,
    },
    # CanPlayType result
    "CanPlayTypeResult": {
        "": 0,
        "probably": 1,
        "maybe": 2,
    },
    # Preload string
    "PreloadString": {
        "": 0,
        "none": 1,
        "metadata": 2,
        "auto": 3,
    },
    # Notification permission
    "NotificationPermission": {
        "default": 0,
        "denied": 1,
        "granted": 2,
    },
    # RTC SDP type
    "RTCSdpType": {
        "offer": 0,
        "pranswer": 1,
        "answer": 2,
        "rollback": 3,
    },
    # RTC Ice candidate type
    "RTCIceCandidateType": {
        "host": 0,
        "srflx": 1,
        "prflx": 2,
        "relay": 3,
    },
    # RTC Ice TCP candidate type
    "RTCIceTcpCandidateType": {
        "active": 0,
        "passive": 1,
        "so": 2,
    },
    # RTC Ice protocol
    "RTCIceProtocol": {
        "udp": 0,
        "tcp": 1,
    },
    # RTC Ice component
    "RTCIceComponent": {
        "rtp": 0,
        "rtcp": 1,
    },
    # RTC Ice username fragment (opaque string)
    "RTCIceUsernameFragment": {
        "": 0,
    },
    # DataChannel label (opaque string)
    "DataChannelLabel": {
        "": 0,
    },
    # DataChannel protocol
    "DataChannelProtocol": {
        "": 0,
    },
    # SpeechSynthesis error code
    "SpeechSynthesisErrorCode": {
        "canceled": 0,
        "interrupted": 1,
        "audio-busy": 2,
        "audio-hardware": 3,
        "network": 4,
        "synthesis-unavailable": 5,
        "synthesis-failed": 6,
        "language-unavailable": 7,
        "voice-unavailable": 8,
        "text-too-long": 9,
        "invalid-argument": 10,
    },
    # PointerType
    "PointerType": {
        "mouse": 0,
        "pen": 1,
        "touch": 2,
    },
    # InputType (partial - most common ones)
    "InputType": {
        "insertText": 0,
        "insertReplacementText": 1,
        "insertLineBreak": 2,
        "insertParagraph": 3,
        "insertFromPaste": 4,
        "deleteContentBackward": 5,
        "deleteContentForward": 6,
        "deleteByCut": 7,
        "historyUndo": 8,
        "historyRedo": 9,
    },
    # MediaSessionPlaybackState
    "MediaSessionPlaybackState": {
        "none": 0,
        "paused": 1,
        "playing": 2,
    },
    # RTCErrorDetailType
    "RTCErrorDetailType": {
        "data-channel-failure": 0,
        "dtls-failure": 1,
        "fingerprint-failure": 2,
        "sctp-failure": 3,
    },
    # IceCandidateString (opaque string for randomUUID)
    "IceCandidateString": {
        "": 0,
    },
    # SdpMidString (opaque string)
    "SdpMidString": {
        "": 0,
    },
    # RTCIceUsernameFragment (opaque string)
    "RTCIceUsernameFragment": {
        "": 0,
    },
    # NotificationDirection
    "NotificationDirection": {
        "auto": 0,
        "ltr": 2,
        "rtl": 3,
    },
    # RootMarginString (opaque string)
    "RootMarginString": {
        "": 0,
    },
    # IceCandidateFoundation (opaque string)
    "IceCandidateFoundation": {
        "": 0,
    },
    # ToneBufferString (opaque string)
    "ToneBufferString": {
        "": 0,
    },
    # PaymentShippingType enum
    "PaymentShippingType": {
        "shipping": 0,
        "delivery": 1,
        "pickup": 2,
    },
    # MethodNameString (opaque string)
    "MethodNameString": {
        "": 0,
    },
    # PayerEmailString (opaque string)
    "PayerEmailString": {
        "": 0,
    },
    # SpeechLang (opaque string)
    "SpeechLang": {
        "": 0,
    },
    # GeolocationErrorMessage (opaque string)
    "GeolocationErrorMessage": {
        "": 0,
    },
    # RTCIceCandidateAddress (opaque string)
    "RTCIceCandidateAddress": {
        "": 0,
    },
    # PerformanceRedirectCount (number as string)
    "PerformanceRedirectCount": {
        "0": 0,
    },
    # UrlProtocol (opaque string)
    "UrlProtocol": {
        "": 0,
    },
    # UrlHostname (opaque string)
    "UrlHostname": {
        "": 0,
    },
    # UrlToJsonString (opaque string)
    "UrlToJsonString": {
        "": 0,
    },
    # RTCIceRelatedAddress (opaque string)
    "RTCIceRelatedAddress": {
        "": 0,
    },
    # SdpMLineIndexString (opaque number)
    "SdpMLineIndexString": {
        "0": 0,
    },
    # SpeechRate (opaque number)
    "SpeechRate": {
        "0": 0,
    },
    # KeyCodeString (opaque number)
    "KeyCodeString": {
        "0": 0,
    },
    # IsComposingNumber (boolean as number)
    "IsComposingNumber": {
        "false": 0,
        "true": 1,
    },
    # WorkerStartString (opaque number)
    "WorkerStartString": {
        "0": 0,
    },
    # DomainLookupStartString (opaque number)
    "DomainLookupStartString": {
        "0": 0,
    },
    # RequestStartString (opaque number)
    "RequestStartString": {
        "0": 0,
    },
    # ClipboardSupportsNumber (boolean as number)
    "ClipboardSupportsNumber": {
        "false": 0,
        "true": 1,
    },
    # UrlSearchString (opaque string)
    "UrlSearchString": {
        "": 0,
    },
    # WheelDeltaXString (opaque number)
    "WheelDeltaXString": {
        "0": 0,
    },
    # AudioBitsPerSecondString (opaque number)
    "AudioBitsPerSecondString": {
        "0": 0,
    },
    # UnloadEventEndString (opaque number)
    "UnloadEventEndString": {
        "0": 0,
    },
    # SpeechSynthesisVoiceLang (opaque string)
    "SpeechSynthesisVoiceLang": {
        "": 0,
    }
}

# Setters that accept enum values (bigint in WIT, string in DOM)
# Maps (interface, property_name) to enum type name
# NOTE: property_name should be in camelCase
ENUM_SETTER_PROPERTIES = {
    # WebGL color space setters
    ("web-gl-rendering-context-base", "drawingBufferColorSpace"): "PredefinedColorSpace",
    ("web-gl-rendering-context-base", "unpackColorSpace"): "PredefinedColorSpace",
    # Canvas setters
    ("canvas-image-smoothing", "imageSmoothingQuality"): "ImageSmoothingQuality",
    ("canvas-path-drawing-styles", "lineCap"): "CanvasLineCap",
    ("canvas-path-drawing-styles", "lineJoin"): "CanvasLineJoin",
    ("canvas-text-drawing-styles", "textAlign"): "CanvasTextAlign",
    ("canvas-text-drawing-styles", "textBaseline"): "CanvasTextBaseline",
    ("canvas-text-drawing-styles", "direction"): "CanvasDirection",
    ("canvas-text-drawing-styles", "fontKerning"): "CanvasFontKerning",
    ("canvas-text-drawing-styles", "fontStretch"): "CanvasFontStretch",
    ("canvas-text-drawing-styles", "fontVariantCaps"): "CanvasFontVariantCaps",
    ("canvas-text-drawing-styles", "textRendering"): "CanvasTextRendering",
    # History setter
    ("history", "scrollRestoration"): "ScrollRestoration",
    # TextTrack setter
    ("text-track", "mode"): "TextTrackMode",
    # XMLHttpRequest setter
    ("xml-http-request", "responseType"): "XMLHttpRequestResponseType",
    # URL setter - href removed, it's just a string property
    # HTMLIFrameElement setter
    ("html-iframe-element", "srcdoc"): "HTMLSrcdoc",
    # WebRTC setters
    ("rtc-rtp-transceiver", "direction"): "RTCRtpTransceiverDirection",
    ("rtc-data-channel", "binaryType"): "RTCDataChannelBinaryType",
}

# Properties that return readonly arrays that need to be converted to mutable arrays
# Maps (interface, property) to True
BOOLEAN_TO_BIGINT_PROPERTIES = {
    # WebGLRenderingContext
    ("web-gl-rendering-context-base", "is-buffer"): True,
    ("web-gl-rendering-context-base", "is-enabled"): True,
    ("web-gl-rendering-context-base", "is-framebuffer"): True,
    ("web-gl-rendering-context-base", "is-program"): True,
    ("web-gl-rendering-context-base", "is-renderbuffer"): True,
    ("web-gl-rendering-context-base", "is-shader"): True,
    ("web-gl-rendering-context-base", "is-texture"): True,
    # WebGL2RenderingContext
    ("web-gl2-rendering-context-base", "is-query"): True,
    ("web-gl2-rendering-context-base", "is-sampler"): True,
    ("web-gl2-rendering-context-base", "is-sync"): True,
    ("web-gl2-rendering-context-base", "is-transform-feedback"): True,
    ("web-gl2-rendering-context-base", "is-vertex-array"): True,
    # AbortSignal
    ("abort-signal", "get-aborted"): True,
    # URL
    ("url", "can-parse"): True,
    # Event
    ("event", "get-bubbles"): True,
    ("event", "get-return-value"): True,
    ("event", "get-composed"): True,
    ("event", "get-cancel-bubble"): True,
    ("event", "get-is-trusted"): True,
    # Notification
    ("notification", "get-require-interaction"): True,
    # PointerEvent
    ("pointer-event", "get-is-primary"): True,
    # SpeechSynthesis
    ("speech-synthesis", "get-pending"): True,
    # Navigator
    ("navigator", "vibrate"): True,
    # SpeechRecognitionResult
    ("speech-recognition-result", "get-is-final"): True,
    # SpeechSynthesisVoice
    ("speech-synthesis-voice", "get-default"): True,
    ("speech-synthesis-voice", "get-local-service"): True,
    # CloseEvent
    ("close-event", "get-was-clean"): True,
    # CryptoKey
    ("crypto-key", "get-extractable"): True,
    # Gamepad
    ("gamepad", "get-connected"): True,
    # GamepadButton
    ("gamepad-button", "get-pressed"): True,
    ("gamepad-button", "get-touched"): True,
    # XMLHttpRequest
    ("xml-http-request", "get-with-credentials"): True,
    # Request
    ("request", "get-is-history-navigation"): True,
    # Body - bodyUsed returns boolean
    ("body", "get-body-used"): True,
    # TouchEvent
    ("touch-event", "get-meta-key"): True,
    # Response
    ("response", "get-ok"): True,
    # Event getBubbles/getCancelable/getComposed/getIsTrusted
    ("event", "get-bubbles"): True,
    ("event", "get-cancelable"): True,
    ("event", "get-composed"): True,
    ("event", "get-is-trusted"): True,
    # Clipboard has types method returns boolean
    ("clipboard-item", "has-type"): True,
    # DOMTokenList contains returns boolean
    ("dom-token-list", "contains"): True,
    # DOMTokenList supports returns boolean
    ("dom-token-list", "supports"): True,
    # TouchEvent getAltKey/getCtrlKey/getShiftKey return boolean
    ("touch-event", "get-alt-key"): True,
    ("touch-event", "get-ctrl-key"): True,
    ("touch-event", "get-shift-key"): True,
    # WebRTC boolean to bigint
    ("rtc-peer-connection", "get-can-trickle-ice-candidates"): True,
    ("rtc-data-channel", "get-ordered"): True,
    ("rtc-data-channel", "get-negotiated"): True,
    # DOMTokenList toggle returns boolean
    ("dom-token-list", "toggle"): True,
    # InputEvent isComposing returns boolean
    ("input-event", "get-is-composing"): True,
    # KeyboardEvent repeat returns boolean
    ("keyboard-event", "get-repeat"): True,
    # SpeechSynthesisEvent elapsedTime returns number (not boolean)
    ("speech-synthesis-event", "get-elapsed-time"): True,
    # KeyboardEvent ctrlKey returns boolean
    ("keyboard-event", "get-ctrl-key"): True,
    # MediaRecorder isTypeSupported returns boolean
    ("media-recorder", "is-type-supported"): True,
    # KeyboardEvent altKey returns boolean
    ("keyboard-event", "get-alt-key"): True,
    # NodeIterator pointerBeforeReferenceNode returns boolean
    ("node-iterator", "get-pointer-before-reference-node"): True,
    # Response redirected returns boolean
    ("response", "get-redirected"): True,
    # KeyboardEvent isComposing returns boolean
    ("keyboard-event", "get-is-composing"): True,
    # ClipboardItem supports returns boolean
    ("clipboard-item", "supports"): True,
    # Headers.has returns boolean
    ("headers", "has"): True,
    # KeyboardEvent.getModifierState returns boolean
    ("keyboard-event", "get-modifier-state"): True,
    # KeyboardEvent metaKey returns boolean
    ("keyboard-event", "get-meta-key"): True,
    # KeyboardEvent shiftKey returns boolean
    ("keyboard-event", "get-shift-key"): True,
    # IntersectionObserverEntry isIntersecting returns boolean
    ("intersection-observer-entry", "get-is-intersecting"): True,
    # WindowOrWorkerGlobalScope crossOriginIsolated returns boolean
    ("window-or-worker-global-scope", "get-cross-origin-isolated"): True,
    # WindowOrWorkerGlobalScope isSecureContext returns boolean
    ("window-or-worker-global-scope", "get-is-secure-context"): True,
    # Request keepalive returns boolean
    ("request", "get-keepalive"): True,
    # MediaStream active returns boolean
    ("media-stream", "get-active"): True,
    # XPathResult invalidIteratorState returns boolean
    ("x-path-result", "get-invalid-iterator-state"): True,
}

# Getters that return optional types in DOM but non-optional in WIT
# Maps (interface, getter_name) to the default value to use for coalescing
GETTER_RETURN_COALESCING = {
    # HTMLOrSVGElement.nonce returns string | undefined in DOM, but string in WIT
    ("html-or-svg-element", "get-nonce"): "",
}

# Handle-returning getters that need non-null assertion (!) when storing
# Maps (interface, getter_name) to True
GETTER_HANDLE_NON_NULL_ASSERTION = {
    # window.event returns Event | undefined in DOM but WIT expects non-optional
    ("window", "get-event"): True,
}

# Properties that return number but need to be converted to bigint
# Maps (interface, property) to True
# NOTE: property names should be in camelCase (the format used by the code generator)
NUMBER_TO_BIGINT_PROPERTIES = {
    ("encoded-audio-chunk", "timestamp"): True,
    ("encoded-audio-chunk", "duration"): True,
    ("encoded-video-chunk", "timestamp"): True,
    ("encoded-video-chunk", "duration"): True,
    ("audio-data", "timestamp"): True,
    ("audio-data", "duration"): True,
    ("video-frame", "timestamp"): True,
    ("video-frame", "duration"): True,
    ("video-frame", "codedWidth"): True,
    ("video-frame", "codedHeight"): True,
    ("video-frame", "displayWidth"): True,
    ("video-frame", "displayHeight"): True,
    ("image-track", "frameCount"): True,
    ("image-track", "repetitionCount"): True,
    # WebGL properties
    ("web-gl-active-info", "size"): True,
    ("web-gl-active-info", "type"): True,
    ("web-gl-shader-precision-format", "precision"): True,
    ("web-gl-shader-precision-format", "rangeMin"): True,
    ("web-gl-shader-precision-format", "rangeMax"): True,
    ("web-gl-rendering-context-base", "drawingBufferWidth"): True,
    ("web-gl-rendering-context-base", "drawingBufferHeight"): True,
    # WebGL methods that return number but WIT expects bigint
    ("web-gl-rendering-context-base", "check-framebuffer-status"): True,
    ("web-gl-rendering-context-base", "get-attrib-location"): True,
    ("web-gl-rendering-context-base", "get-vertex-attrib-offset"): True,
    ("web-gl2-rendering-context-base", "get-frag-data-location"): True,
    ("web-gl2-rendering-context-base", "get-uniform-block-index"): True,
    ("web-gl2-rendering-context-base", "client-wait-sync"): True,
    # Gamepad
    ("gamepad", "timestamp"): True,
    ("gamepad", "index"): True,
    # GamepadButton
    ("gamepad-button", "value"): True,
    # GeolocationCoordinates
    ("geolocation-coordinates", "latitude"): True,
    ("geolocation-coordinates", "longitude"): True,
    ("geolocation-coordinates", "altitude"): True,
    ("geolocation-coordinates", "accuracy"): True,
    ("geolocation-coordinates", "altitudeAccuracy"): True,
    ("geolocation-coordinates", "heading"): True,
    ("geolocation-coordinates", "speed"): True,
    # GeolocationPosition
    ("geolocation-position", "timestamp"): True,
    # Canvas
    ("canvas-rendering-context", "lineWidth"): True,
    ("canvas-rendering-context", "miterLimit"): True,
    ("canvas-rendering-context", "shadowBlur"): True,
    ("canvas-rendering-context", "shadowOffsetX"): True,
    ("canvas-rendering-context", "shadowOffsetY"): True,
    ("canvas-rendering-context", "globalAlpha"): True,
    ("text-metrics", "width"): True,
    ("text-metrics", "actualBoundingBoxAscent"): True,
    ("text-metrics", "actualBoundingBoxDescent"): True,
    ("text-metrics", "fontBoundingBoxAscent"): True,
    ("text-metrics", "fontBoundingBoxDescent"): True,
    ("text-metrics", "ideographicBaseline"): True,
    ("text-metrics", "hangingBaseline"): True,
    ("dom-rect", "x"): True,
    ("dom-rect", "y"): True,
    ("dom-rect", "width"): True,
    ("dom-rect", "height"): True,
    ("dom-rect", "top"): True,
    ("dom-rect", "right"): True,
    ("dom-rect", "bottom"): True,
    ("dom-rect", "left"): True,
    # Performance timing
    ("performance-timing-confidence", "value"): True,
    ("performance", "timeOrigin"): True,
    ("performance-navigation-timing", "startTime"): True,
    ("performance-navigation-timing", "duration"): True,
    ("performance-navigation-timing", "redirectStart"): True,
    ("performance-navigation-timing", "redirectEnd"): True,
    ("performance-navigation-timing", "fetchStart"): True,
    ("performance-navigation-timing", "domainLookupStart"): True,
    ("performance-navigation-timing", "domainLookupEnd"): True,
    ("performance-navigation-timing", "connectStart"): True,
    ("performance-navigation-timing", "connectEnd"): True,
    ("performance-navigation-timing", "secureConnectionStart"): True,
    ("performance-navigation-timing", "requestStart"): True,
    ("performance-navigation-timing", "responseStart"): True,
    ("performance-navigation-timing", "responseEnd"): True,
    ("performance-navigation-timing", "domInteractive"): True,
    ("performance-navigation-timing", "domContentLoadedEventStart"): True,
    ("performance-navigation-timing", "domContentLoadedEventEnd"): True,
    ("performance-navigation-timing", "domComplete"): True,
    ("performance-navigation-timing", "loadEventStart"): True,
    ("performance-navigation-timing", "loadEventEnd"): True,
    ("performance-navigation-timing", "unloadEventStart"): True,
    ("performance-navigation-timing", "unloadEventEnd"): True,
    ("performance-navigation-timing", "redirectCount"): True,
    # PerformanceNavigation
    ("performance-navigation", "redirectCount"): True,
    # PerformanceResourceTiming
    ("performance-resource-timing", "redirectStart"): True,
    ("performance-resource-timing", "responseStatus"): True,
    # Events
    ("event", "eventPhase"): True,
    ("event", "timeStamp"): True,
    # WebSocket
    ("ws", "readyState"): True,
    ("ws", "bufferedAmount"): True,
    ("close-event", "code"): True,
    # HTML Elements
    ("html-input-element", "selectionStart"): True,
    ("html-input-element", "selectionEnd"): True,
    ("html-text-area-element", "selectionStart"): True,
    ("html-text-area-element", "selectionEnd"): True,
    ("html-text-area-element", "textLength"): True,
    ("html-media-element", "currentTime"): True,
    ("html-media-element", "duration"): True,
    ("html-media-element", "playbackRate"): True,
    ("html-media-element", "volume"): True,
    ("html-media-element", "defaultPlaybackRate"): True,
    # SpeechSynthesisUtterance
    ("speech-synthesis-utterance", "volume"): True,
    ("speech-synthesis-utterance", "rate"): True,
    ("speech-synthesis-utterance", "pitch"): True,
    ("html-progress-element", "value"): True,
    ("html-progress-element", "max"): True,
    ("html-meter-element", "value"): True,
    ("html-meter-element", "min"): True,
    ("html-meter-element", "max"): True,
    ("html-meter-element", "low"): True,
    ("html-meter-element", "high"): True,
    ("html-meter-element", "optimum"): True,
    # WebAnimations
    ("animation", "startTime"): True,
    ("animation", "currentTime"): True,
    ("keyframe-effect", "iterationStart"): True,
    ("keyframe-effect", "iterations"): True,
    ("keyframe-effect", "duration"): True,
    # WebRTC
    ("rtc-peer-connection", "localDescription"): True,
    # Touch
    ("touch", "radiusX"): True,
    ("touch", "radiusY"): True,
    ("touch", "rotationAngle"): True,
    ("touch", "force"): True,
    # OffscreenCanvas
    ("offscreen-canvas", "width"): True,
    ("offscreen-canvas", "height"): True,
    # Navigator
    ("navigator-concurrent-hardware", "hardwareConcurrency"): True,
    # Navigation
    ("navigation-history-entry", "index"): True,
    # PerformanceTiming
    ("performance-timing", "navigationStart"): True,
    ("performance-timing", "unloadEventStart"): True,
    ("performance-timing", "unloadEventEnd"): True,
    ("performance-timing", "redirectStart"): True,
    ("performance-timing", "redirectEnd"): True,
    ("performance-timing", "fetchStart"): True,
    ("performance-timing", "domainLookupStart"): True,
    ("performance-timing", "domainLookupEnd"): True,
    ("performance-timing", "connectStart"): True,
    ("performance-timing", "connectEnd"): True,
    ("performance-timing", "secureConnectionStart"): True,
    ("performance-timing", "requestStart"): True,
    ("performance-timing", "responseStart"): True,
    ("performance-timing", "responseEnd"): True,
    ("performance-timing", "domLoading"): True,
    ("performance-timing", "domInteractive"): True,
    ("performance-timing", "domContentLoadedEventStart"): True,
    ("performance-timing", "domContentLoadedEventEnd"): True,
    ("performance-timing", "domComplete"): True,
    ("performance-timing", "loadEventStart"): True,
    ("performance-timing", "loadEventEnd"): True,
    # PerformanceEntry
    ("performance-entry", "startTime"): True,
    ("performance-entry", "duration"): True,
    # PerformanceResourceTiming
    ("performance-resource-timing", "transferSize"): True,
    ("performance-resource-timing", "encodedBodySize"): True,
    ("performance-resource-timing", "decodedBodySize"): True,
    ("performance-resource-timing", "connectStart"): True,
    ("performance-resource-timing", "workerStart"): True,
    ("performance-resource-timing", "responseEnd"): True,
    ("performance-resource-timing", "secureConnectionStart"): True,
    ("performance-resource-timing", "requestStart"): True,
    # XPathResult stringValue returns string but WIT expects bigint
    ("x-path-result", "stringValue"): True,
    # PointerEvent
    ("pointer-event", "pointerId"): True,
    ("pointer-event", "width"): True,
    ("pointer-event", "height"): True,
    ("pointer-event", "pressure"): True,
    ("pointer-event", "tangentialPressure"): True,
    ("pointer-event", "tiltX"): True,
    ("pointer-event", "tiltY"): True,
    ("pointer-event", "twist"): True,
    ("pointer-event", "altitudeAngle"): True,
    ("pointer-event", "azimuthAngle"): True,
    # TouchEvent
    ("touch-event", "touchPointId"): True,
    # WebSocket
    ("ws", "readyState"): True,
    ("ws", "bufferedAmount"): True,
    # RTC
    ("rtc-data-channel", "maxMessageSize"): True,
    # Gamepad
    ("gamepad", "timestamp"): True,
    ("gamepad", "index"): True,
    # Navigator
    ("navigator", "maxTouchPoints"): True,
    # Touch
    ("touch", "pageX"): True,
    ("touch", "pageY"): True,
    ("touch", "clientX"): True,
    ("touch", "clientY"): True,
    ("touch", "screenX"): True,
    ("touch", "screenY"): True,
    # BlobEvent
    ("blob-event", "timecode"): True,
    # WebAssembly Table
    ("table", "length"): True,
    # Additional number→bigint for TS2322 fixes
    ("touch", "identifier"): True,
    ("response", "status"): True,
    # AbstractRange
    ("abstract-range", "endOffset"): True,
    ("abstract-range", "startOffset"): True,
    # WheelEvent
    ("wheel-event", "deltaMode"): True,
    # Performance
    ("performance-entry", "startTime"): True,
    ("performance-entry", "duration"): True,
    ("performance-resource-timing", "transferSize"): True,
    ("performance-resource-timing", "encodedBodySize"): True,
    ("performance-resource-timing", "decodedBodySize"): True,
    # PointerEvent
    ("pointer-event", "pointerId"): True,
    ("pointer-event", "width"): True,
    ("pointer-event", "height"): True,
    ("pointer-event", "pressure"): True,
    ("pointer-event", "tangentialPressure"): True,
    ("pointer-event", "tiltX"): True,
    ("pointer-event", "tiltY"): True,
    ("pointer-event", "twist"): True,
    ("pointer-event", "altitudeAngle"): True,
    ("pointer-event", "azimuthAngle"): True,
    # TouchEvent
    ("touch-event", "touchPointId"): True,
    # WebSocket
    ("ws", "readyState"): True,
    ("ws", "bufferedAmount"): True,
    # Gamepad
    ("gamepad", "timestamp"): True,
    ("gamepad", "index"): True,
    # RTC
    ("rtc-data-channel", "maxMessageSize"): True,
    # HTMLMediaElement
    ("html-media-element", "readyState"): True,
    ("html-media-element", "networkState"): True,
    # HTMLImageElement
    ("html-image-element", "naturalWidth"): True,
    ("html-image-element", "naturalHeight"): True,
    ("html-image-element", "width"): True,
    ("html-image-element", "height"): True,
    # ScreenOrientation angle
    ("screen-orientation", "angle"): True,
    # GeolocationPositionError code
    ("geolocation-position-error", "code"): True,
    # TouchList length
    ("touch-list", "length"): True,
    # IntersectionObserver thresholds
    ("intersection-observer", "get-thresholds"): True,
    # MediaStreamTrack getSettings capabilities number values
    ("media-stream-track", "get-settings"): True,
    # WebRTC - number to bigint
    ("rtc-ice-candidate", "relatedPort"): True,
    ("rtc-ice-candidate", "port"): True,
    ("rtc-ice-candidate", "priority"): True,
    ("rtc-certificate", "expires"): True,
    ("rtc-data-channel", "maxPacketLifeTime"): True,
    ("rtc-data-channel", "maxRetransmits"): True,
    ("rtc-data-channel", "id"): True,
    ("rtc-data-channel", "bufferedAmount"): True,
    ("rtc-rtp-receiver", "jitterBufferTarget"): True,
    ("rtc-error", "sdpLineNumber"): True,
    ("rtc-error", "sctpCauseCode"): True,
    ("rtc-error", "receivedAlert"): True,
    ("rtc-error", "sentAlert"): True,
    # SpeechRecognition
    ("speech-recognition-alternative", "confidence"): True,
    ("speech-recognition-result", "length"): True,
    ("speech-recognition-result-list", "length"): True,
    # ReadableByteStreamController desiredSize
    ("readable-byte-stream-controller", "desiredSize"): True,
    # ReadableStreamDefaultController desiredSize
    ("readable-stream-default-controller", "desiredSize"): True,
    # UIEvent - which returns number
    ("ui-event", "which"): True,
    # UIEvent - detail returns number
    ("ui-event", "detail"): True,
    # RTCDataChannel - bufferedAmountLowThreshold returns number
    ("rtc-data-channel", "buffered-amount-low-threshold"): True,
    # RTCDataChannel - bufferedAmount returns number
    ("rtc-data-channel", "buffered-amount"): True,
    # IntersectionObserverEntry - time returns DOMHighResTimeStamp (number)
    ("intersection-observer-entry", "time"): True,
    # ResizeObserverSize - blockSize/inlineSize return number
    ("resize-observer-size", "blockSize"): True,
    ("resize-observer-size", "inlineSize"): True,
    # ReadableStreamDefaultController desiredSize returns number
    ("readable-stream-default-controller", "desired-size"): True,
    # KeyboardEvent location returns number
    ("keyboard-event", "location"): True,
}

# Event handler properties that return EventHandler | null but WIT expects bigint
# Maps (interface_wit_name, property_name) to True
# These getters need to convert event handlers to bigint handles
EVENT_HANDLER_PROPERTIES = {
    # WebCodecs
    ("audio-decoder", "ondequeue"): True,
    ("video-decoder", "ondequeue"): True,
    ("audio-encoder", "ondequeue"): True,
    ("video-encoder", "ondequeue"): True,
    # GlobalEventHandlers
    ("global-event-handlers", "onbeforeinput"): True,
    ("global-event-handlers", "onbeforematch"): True,
    ("global-event-handlers", "onbeforetoggle"): True,
    ("global-event-handlers", "onresize"): True,
    ("global-event-handlers", "ontouchstart"): True,
    ("global-event-handlers", "ontouchend"): True,
    ("global-event-handlers", "ontouchmove"): True,
    ("global-event-handlers", "ontouchcancel"): True,
    ("global-event-handlers", "onpointerrawupdate"): True,
    # MediaQueryList
    ("media-query-list", "onchange"): True,
    # Document
    ("document", "onfullscreenchange"): True,
    ("document", "onfullscreenerror"): True,
    ("document", "onreadystatechange"): True,
    ("document", "onvisibilitychange"): True,
    # Element
    ("element", "onfullscreenchange"): True,
    ("element", "onfullscreenerror"): True,
    # VisualViewport
    ("visual-viewport", "onresize"): True,
    ("visual-viewport", "onscroll"): True,
    # AbortSignal
    ("abort-signal", "onabort"): True,
    # ShadowRoot
    ("shadow-root", "onslotchange"): True,
    # XMLHttpRequestEventTarget
    ("xml-http-request-event-target", "onloadstart"): True,
    ("xml-http-request-event-target", "onprogress"): True,
    ("xml-http-request-event-target", "onabort"): True,
    ("xml-http-request-event-target", "onerror"): True,
    ("xml-http-request-event-target", "onload"): True,
    ("xml-http-request-event-target", "ontimeout"): True,
    ("xml-http-request-event-target", "onloadend"): True,
    # XMLHttpRequest
    ("xml-http-request", "onreadystatechange"): True,
    # TextTrackList
    ("text-track-list", "onchange"): True,
    ("text-track-list", "onaddtrack"): True,
    ("text-track-list", "onremovetrack"): True,
    # TextTrack
    ("text-track", "oncuechange"): True,
    # TextTrackCue
    ("text-track-cue", "onenter"): True,
    ("text-track-cue", "onexit"): True,
    # TextTrack
    ("text-track", "oncuechange"): True,
    # TextTrackList
    ("text-track-list", "onchange"): True,
    ("text-track-list", "onaddtrack"): True,
    ("text-track-list", "onremovetrack"): True,
    # OffscreenCanvas
    ("offscreen-canvas", "oncontextlost"): True,
    ("offscreen-canvas", "oncontextrestored"): True,
    # NavigationHistoryEntry
    ("navigation-history-entry", "ondispose"): True,
    # EventSource
    ("event-source", "onopen"): True,
    ("event-source", "onmessage"): True,
    ("event-source", "onerror"): True,
    # MessagePort
    ("message-port", "onmessage"): True,
    ("message-port", "onmessageerror"): True,
    # BroadcastChannel
    ("broadcast-channel", "onmessage"): True,
    ("broadcast-channel", "onmessageerror"): True,
    # MessageEventTarget
    ("message-event-target", "onmessage"): True,
    ("message-event-target", "onmessageerror"): True,
    # AbstractWorker
    ("abstract-worker", "onerror"): True,
    # PaymentResponse
    ("payment-response", "onpayerdetailchange"): True,
    # RTCDTMFSender
    ("rtcdtmf-sender", "ontonechange"): True,
}

# Interface-specific browser attribute name overrides
# Maps (interface_name, wit_attr_name) -> browser_attr_name
# This is used when the same WIT attribute name maps to different browser names on different interfaces
INTERFACE_ATTR_OVERRIDES = {
    # Document.url should be URL (uppercase), but other interfaces use url (lowercase)
    ("document", "url"): "URL",
    # IntersectionObserverEntry.boundingClientRect is a property, not a method
    ("intersection-observer-entry", "bounding-client-rect"): "boundingClientRect",
    # Window.navigation should be navigator
    ("window", "navigation"): "navigator",
    # NamedNodeMap.namedItem should be getNamedItem
    ("named-node-map", "named-item"): "getNamedItem",
    # ClipboardItem.type should be types (plural) 
    ("clipboard-item", "type"): "types",
    # WebGLRenderingContextBase.shader-source getter should call getShaderSource
    ("web-gl-rendering-context-base", "shader-source"): "getShaderSource",
}

# Functions that are defined as getters in WIT but are actually methods in DOM API
GETTER_BUT_ACTUALLY_METHOD = {
    "context-attributes", "supported-extensions", "extension", "active-attrib",
    "active-uniform", "attached-shaders", "attrib-location", "buffer-parameter",
    "parameter", "framebuffer-attachment-parameter", "program-parameter",
    "program-info-log", "renderbuffer-parameter", "shader-parameter",
    "shader-precision-format", "shader-info-log", "shader-source",
    "named-item", "tex-parameter", "uniform", "uniform-location", "vertex-attrib",
    "vertex-attrib-offset",
    "property-value", "property-priority", "svg-document",
    "fingerprints", "parameters", "contributing-sources", "synchronization-sources",
    "remote-certificates", "selected-candidate-pair",
    "buffer-sub-data", "internalformat-parameter", "frag-data-location",
    "query", "query-parameter", "sampler-parameter", "sync-parameter",
    "indexed-parameter", "uniform-indices", "active-uniforms",
    "uniform-block-index", "active-uniform-block-parameter", "active-uniform-block-name",
    "response-header", "all-response-headers", "can-insert-dtmf",
    "transform-feedback-varying", "bounding-client-rect",
    # DOM methods that start with get- but are actually methods with parameters
    "attribute", "attribute-ns", "attribute-node", "attribute-node-ns",
    "elements-by-tag-name", "elements-by-tag-name-ns", "elements-by-class-name",
    "element-by-id", "elements-by-name",
    "client-rects", "client-rect",
    "named-item-ns", "named-item",
    "computed-style", "html", "selection", "animations",
    "element-from-point", "elements-from-point", "caret-position-from-point", "caret-range-from-point",
    "item", "root-node", "custom-validity", "range-text", "selection-range",
    "html-document", "svg-document",
    "track-by-id", "audio-tracks", "video-tracks",
    "constraints", "supported-constraints", "capabilities", "configuration",
    "user-media", "display-media", "action-handler", "position-state",
    "microphone-active", "camera-active", "notifications", "registrations",
    "entries-by-type", "entries-by-name", "entries",
    "receivers", "transceivers", "senders", "stats",
    "header-value", "set-cookie", "resource-timing-buffer-size",
    "ready", "controller", "metadata", "voices", "voices-and-voice",
    # Canvas methods that are getters in WIT but methods in DOM
    "image-data", "transform",
    # Methods that return arrays and need to be called as methods
    "coalesced-events", "predicted-events",
    # Attribute names method
    "attribute-names",
    # ClipboardItem getType takes a parameter
    "type",
    # KeyboardEvent.getModifierState takes a parameter
    "modifier-state",
    # Crypto.getRandomValues takes a parameter
    "random-values",
}

# Functions that are defined as setters in WIT but are actually methods in DOM API
# These should call a method instead of setting a property
# Parameters to skip when calling browser methods (WIT has them but browser API doesn't)
# Maps (interface, function, param_name) -> True to skip the parameter
PARAMS_TO_SKIP = {
    # HTMLElement.showPopover() doesn't take options
    ("html-element", "show-popover", "options"): True,
    # Clipboard.read() doesn't take formats
    ("clipboard", "read", "formats"): True,
    # ReadableStreamBYOBReader.read() only takes view, not options
    ("readable-stream-byob-reader", "read", "options"): True,
    # PaymentResponse.complete() only takes result, not details
    ("payment-response", "complete", "details"): True,
    # ResizeObserver.observe() requires target parameter - handled separately
}

# Methods that need different browser method name for setters
# Key is (interface_name, wit_name[4:]) - the part after "set-"
SETTER_METHOD_NAMES = {
    ("named-node-map", "named-item"): "setNamedItem",
    ("named-node-map", "named-item-ns"): "setNamedItemNS",
    ("window-or-worker-global-scope", "timeout"): "setTimeout",
    ("window-or-worker-global-scope", "interval"): "setInterval",
    ("rtc-rtp-sender", "parameters"): "setParameters",
    ("element-internals", "validity"): "setValidity",
    ("html-object-element", "custom-validity"): "setCustomValidity",
    ("html-input-element", "custom-validity"): "setCustomValidity",
    ("html-button-element", "custom-validity"): "setCustomValidity",
    ("html-select-element", "custom-validity"): "setCustomValidity",
    ("html-text-area-element", "custom-validity"): "setCustomValidity",
    ("html-output-element", "custom-validity"): "setCustomValidity",
    ("html-field-set-element", "custom-validity"): "setCustomValidity",
    ("rtc-peer-connection", "configuration"): "setConfiguration",
}

SETTER_BUT_ACTUALLY_METHOD = {
    # Format: (interface_name, property_name) tuples for interface-specific method handling
    # Element methods
    ("element", "attribute"),
    ("element", "attribute-ns"),
    ("element", "attribute-node"),
    ("element", "attribute-node-ns"),
    ("element", "html-unsafe"),
    ("element", "pointer-capture"),
    # Range methods
    ("range", "start"),
    ("range", "start-before"),
    ("range", "start-after"),
    ("range", "end"),
    ("range", "end-before"),
    ("range", "end-after"),
    # HTMLInputElement/HTMLTextAreaElement methods
    ("html-input-element", "selection-range"),
    ("html-input-element", "range-text"),
    ("html-text-area-element", "selection-range"),
    ("html-text-area-element", "range-text"),
    # ElementInternals methods
    ("element-internals", "custom-validity"),
    ("element-internals", "validity"),
    # XMLHttpRequest methods
    ("xml-http-request", "request-header"),
    # Headers methods
    ("headers", "header-value"),
    # MediaSession methods
    ("media-session", "action-handler"),
    ("media-session", "position-state"),
    ("media-session", "microphone-active"),
    ("media-session", "camera-active"),
    # Performance methods
    ("performance", "resource-timing-buffer-size"),
    # RTCRtpTransceiver methods
    ("rtc-rtp-transceiver", "codec-preferences"),
    # ElementInternals methods
    ("element-internals", "form-value"),
    # DataTransfer methods
    ("data-transfer", "drag-image"),
    # HTMLButtonElement methods
    ("html-button-element", "popover-target-element"),
    # Canvas methods - setTransform is a method, property name without "set-" prefix
    ("canvas-transform", "transform"),
    # CanvasPattern setTransform is a method
    ("canvas-pattern", "transform"),
    # NamedNodeMap methods that return replaced Attr
    ("named-node-map", "named-item"),
    ("named-node-map", "named-item-ns"),
    # Window timer methods that return timeout/interval ID
    ("window-or-worker-global-scope", "timeout"),
    ("window-or-worker-global-scope", "interval"),
    # RTCRtpSender - setParameters is a method
    ("rtc-rtp-sender", "parameters"),
    # RTCPeerConnection - setConfiguration is a method
    ("rtc-peer-connection", "configuration"),
    # MediaSession - screenshareActive setter returns void but WIT expects return
    ("media-session", "screenshare-active"),
    # Element methods that return Attr
    ("element", "set-attribute-node"),
    ("element", "set-attribute-node-ns"),
    ("element", "remove-attribute-node"),
    # setCustomValidity is a method, not a property setter
    ("html-object-element", "custom-validity"),
    ("html-input-element", "custom-validity"),
    ("html-button-element", "custom-validity"),
    ("html-select-element", "custom-validity"),
    ("html-text-area-element", "custom-validity"),
    ("html-output-element", "custom-validity"),
    ("html-field-set-element", "custom-validity"),
    # XSLTProcessor - setParameter is a method
    ("xslt-processor", "parameter"),
}

# Synthetic handle types - types that need handle tables but don't have WIT interfaces
# Maps wit_type_name -> (ts_type, handle_var_suffix, handle_pascal)
# handle_var_suffix: the suffix for the handle variable (e.g., "Handles" -> "_stringHandles")
# handle_pascal: the PascalCase name for the counter (e.g., "String" -> _nextString)
SYNTHETIC_HANDLE_TYPES = {
    "string": ("string", "stringHandles", "String"),
    "uint8-array": ("Uint8Array", "uint8ArrayHandles", "Uint8Array"),
    "void": ("void", "voidHandles", "Void"),
    "boolean": ("boolean", "booleanHandles", "Boolean"),
    "number": ("number", "numberHandles", "Number"),
    "any": ("any", "anyHandles", "Any"),
    "web-gl-object": ("WebGLObject", "webGlObjectHandles", "WebGlObject"),
    "element": ("Element", "elementHandles", "Element"),
    "css-keyframe-rule": ("CSSKeyframeRule", "cssKeyframeRuleHandles", "CssKeyframeRule"),
    "css-style-declaration": ("CSSStyleDeclaration", "cssStyleDeclarationHandles", "CssStyleDeclaration"),
    "window": ("Window", "windowHandles", "Window"),
    "element-list": ("Element[]", "elementListHandles", "ElementList"),
    "html-collection": ("HTMLCollection", "htmlCollectionHandles", "HtmlCollection"),
    "html-form-controls-collection": ("HTMLFormControlsCollection", "htmlFormControlsCollectionHandles", "HtmlFormControlsCollection"),
    "document-fragment": ("DocumentFragment", "documentFragmentHandles", "DocumentFragment"),
    "text": ("Text", "textHandles", "Text"),
    "comment": ("Comment", "commentHandles", "Comment"),
    "processing-instruction": ("ProcessingInstruction", "processingInstructionHandles", "ProcessingInstruction"),
    "node": ("Node", "nodeHandles", "Node"),
    "attr": ("Attr", "attrHandles", "Attr"),
    "event": ("Event", "eventHandles", "Event"),
    "node-iterator": ("NodeIterator", "nodeIteratorHandles", "NodeIterator"),
    "tree-walker": ("TreeWalker", "treeWalkerHandles", "TreeWalker"),
    "node-list": ("NodeList", "nodeListHandles", "NodeList"),
    "document": ("Document", "documentHandles", "Document"),
    "dom-rect-list": ("DOMRectList", "domRectListHandles", "DomRectList"),
    "string-list": ("string[]", "stringListHandles", "StringList"),
    "shadow-root": ("ShadowRoot", "shadowRootHandles", "ShadowRoot"),
    "css-rule": ("CSSRule", "cssRuleHandles", "CssRule"),
    "web-gl-uniform-location": ("WebGLUniformLocation", "webGlUniformLocationHandles", "WebGlUniformLocation"),
    "web-gl-shader-list": ("WebGLShader[]", "webGlShaderListHandles", "WebGlShaderList"),
    "dom-rect": ("DOMRect", "domRectHandles", "DomRect"),
    "dom-rect-read-only": ("DOMRectReadOnly", "domRectReadOnlyHandles", "DomRectReadOnly"),
    "web-gl-active-info": ("WebGLActiveInfo", "webGlActiveInfoHandles", "WebGlActiveInfo"),
    "web-gl-shader-precision-format": ("WebGLShaderPrecisionFormat", "webGlShaderPrecisionFormatHandles", "WebGlShaderPrecisionFormat"),
    "number-list": ("number[]", "numberListHandles", "NumberList"),
    "media-list": ("MediaList", "mediaListHandles", "MediaList"),
    "promise-void": ("Promise<void>", "promiseVoidHandles", "PromiseVoid"),
    "promise-string": ("Promise<string>", "promiseStringHandles", "PromiseString"),
    "promise-any": ("Promise<any>", "promiseAnyHandles", "PromiseAny"),
    "promise-permission-status": ("Promise<PermissionStatus>", "promisePermissionStatusHandles", "PromisePermissionStatus"),
    "geolocation-coordinates": ("GeolocationCoordinates", "geolocationCoordinatesHandles", "GeolocationCoordinates"),
    "headers": ("Headers", "headersHandles", "Headers"),
    "html-options-collection": ("HTMLOptionsCollection", "htmlOptionsCollectionHandles", "HtmlOptionsCollection"),
    "validity-state": ("ValidityState", "validityStateHandles", "ValidityState"),
    "html-table-caption-element": ("HTMLTableCaptionElement", "htmlTableCaptionElementHandles", "HtmlTableCaptionElement"),
    "html-table-section-element": ("HTMLTableSectionElement", "htmlTableSectionElementHandles", "HtmlTableSectionElement"),
    # Additional DOM types
    "bar-prop": ("BarProp", "barPropHandles", "BarProp"),
    "abort-signal": ("AbortSignal", "abortSignalHandles", "AbortSignal"),
    "validity-state": ("ValidityState", "validityStateHandles", "ValidityState"),
    "dom-token-list": ("DOMTokenList", "domTokenListHandles", "DomTokenList"),
    "location": ("Location", "locationHandles", "Location"),
    "history": ("History", "historyHandles", "History"),
    "custom-element-registry": ("CustomElementRegistry", "customElementRegistryHandles", "CustomElementRegistry"),
    "navigator": ("Navigator", "navigatorHandles", "Navigator"),
    "screen": ("Screen", "screenHandles", "Screen"),
    "visual-viewport": ("VisualViewport", "visualViewportHandles", "VisualViewport"),
    "screen-orientation": ("ScreenOrientation", "screenOrientationHandles", "ScreenOrientation"),
    "css-style-sheet": ("CSSStyleSheet", "cssStyleSheetHandles", "CssStyleSheet"),
    "css-rule-list": ("CSSRuleList", "cssRuleListHandles", "CssRuleList"),
    "media-list": ("MediaList", "mediaListHandles", "MediaList"),
    "document-type": ("DocumentType", "documentTypeHandles", "DocumentType"),
    "dom-implementation": ("DOMImplementation", "domImplementationHandles", "DomImplementation"),
    "html-element": ("HTMLElement", "htmlElementHandles", "HtmlElement"),
    "html-form-element": ("HTMLFormElement", "htmlFormElementHandles", "HtmlFormElement"),
    "event-target": ("EventTarget", "eventTargetHandles", "EventTarget"),
    "external": ("External", "externalHandles", "External"),
    "speech-synthesis": ("SpeechSynthesis", "speechSynthesisHandles", "SpeechSynthesis"),
    "mime-type": ("MimeType", "mimeTypeHandles", "MimeType"),
    "rtc-session-description": ("RTCSessionDescription", "rtcSessionDescriptionHandles", "RtcSessionDescription"),
    "response": ("Response", "responseHandles", "Response"),
    "media-stream-track": ("MediaStreamTrack", "mediaStreamTrackHandles", "MediaStreamTrack"),
    "child-node": ("ChildNode", "childNodeHandles", "ChildNode"),
    "time-ranges": ("TimeRanges", "timeRangesHandles", "TimeRanges"),
    "message-port": ("MessagePort", "messagePortHandles", "MessagePort"),
    "c-data-section": ("CDATASection", "cdataSectionHandles", "CdataSection"),
    "named-node-map": ("NamedNodeMap", "namedNodeMapHandles", "NamedNodeMap"),
    "html-all-collection": ("HTMLAllCollection", "htmlAllCollectionHandles", "HtmlAllCollection"),
    "element-internals": ("ElementInternals", "elementInternalsHandles", "ElementInternals"),
    # Additional types needed for handle wrapping
    "clipboard": ("Clipboard", "clipboardHandles", "Clipboard"),
    "credentials-container": ("CredentialsContainer", "credentialsContainerHandles", "CredentialsContainer"),
    "geolocation": ("Geolocation", "geolocationHandles", "Geolocation"),
    "user-activation": ("UserActivation", "userActivationHandles", "UserActivation"),
    "media-capabilities": ("MediaCapabilities", "mediaCapabilitiesHandles", "MediaCapabilities"),
    "media-devices": ("MediaDevices", "mediaDevicesHandles", "MediaDevices"),
    "media-session": ("MediaSession", "mediaSessionHandles", "MediaSession"),
    "permissions": ("Permissions", "permissionsHandles", "Permissions"),
    "service-worker-container": ("ServiceWorkerContainer", "serviceWorkerContainerHandles", "ServiceWorkerContainer"),
    "battery-manager": ("BatteryManager", "batteryManagerHandles", "BatteryManager"),
    "gamepad-list": ("Gamepad[]", "gamepadListHandles", "GamepadList"),
    "plugin-array": ("PluginArray", "pluginArrayHandles", "PluginArray"),
    "mime-type-array": ("MimeTypeArray", "mimeTypeArrayHandles", "MimeTypeArray"),
    "style-sheet-list": ("StyleSheetList", "styleSheetListHandles", "StyleSheetList"),
    "css-style-sheet-list": ("CSSStyleSheet[]", "cssStyleSheetListHandles", "CssStyleSheetList"),
    "text-track": ("TextTrack", "textTrackHandles", "TextTrack"),
    "text-track-list": ("TextTrackList", "textTrackListHandles", "TextTrackList"),
    "text-track-cue-list": ("TextTrackCueList", "textTrackCueListHandles", "TextTrackCueList"),
    "audio-track-list": ("AudioTrackList", "audioTrackListHandles", "AudioTrackList"),
    "video-track-list": ("VideoTrackList", "videoTrackListHandles", "VideoTrackList"),
    "media-error": ("MediaError", "mediaErrorHandles", "MediaError"),
    "media-provider": ("MediaProvider", "mediaProviderHandles", "MediaProvider"),
    "html-table-caption-element": ("HTMLTableCaptionElement", "htmlTableCaptionElementHandles", "HtmlTableCaptionElement"),
    "html-table-section-element": ("HTMLTableSectionElement", "htmlTableSectionElementHandles", "HtmlTableSectionElement"),
    "html-options-collection": ("HTMLOptionsCollection", "htmlOptionsCollectionHandles", "HtmlOptionsCollection"),
    "gamepad-button-list": ("readonly GamepadButton[]", "gamepadButtonListHandles", "GamepadButtonList"),
    "gamepad-haptic-actuator-list": ("GamepadHapticActuator[]", "gamepadHapticActuatorListHandles", "GamepadHapticActuatorList"),
    "float-32-list": ("readonly number[]", "float32ListHandles", "Float32List"),
    "int-32-list": ("Int32Array", "int32ListHandles", "Int32List"),
    "uint-32-list": ("Uint32Array", "uint32ListHandles", "Uint32List"),
    "dom-string-map": ("DOMStringMap", "domStringMapHandles", "DomStringMap"),
    "image-track-list": ("ImageTrackList", "imageTrackListHandles", "ImageTrackList"),
    "image-track": ("ImageTrack", "imageTrackHandles", "ImageTrack"),
    # Additional types for TS2322 fixes
    "event-target-list": ("EventTarget[]", "eventTargetListHandles", "EventTargetList"),
    "mutation-record-list": ("MutationRecord[]", "mutationRecordListHandles", "MutationRecordList"),
    "touch": ("Touch", "touchHandles", "Touch"),
    "touch-list": ("TouchList", "touchListHandles", "TouchList"),
    "pointer-event-list": ("PointerEvent[]", "pointerEventListHandles", "PointerEventList"),
    "data-transfer": ("DataTransfer", "dataTransferHandles", "DataTransfer"),
    "cache-storage": ("CacheStorage", "cacheStorageHandles", "CacheStorage"),
    "crypto": ("Crypto", "cryptoHandles", "Crypto"),
    "readable-stream": ("ReadableStream", "readableStreamHandles", "ReadableStream"),
    "writable-stream": ("WritableStream", "writableStreamHandles", "WritableStream"),
    "storage": ("Storage", "storageHandles", "Storage"),
    "readable-stream-byob-request": ("ReadableStreamBYOBRequest", "readableStreamByobRequestHandles", "ReadableStreamByobRequest"),
    "readable-stream-pair": ("[ReadableStream, ReadableStream]", "readableStreamPairHandles", "ReadableStreamPair"),
    "xpath-expression": ("XPathExpression", "xpathExpressionHandles", "XPathExpression"),
    "xpath-ns-resolver": ("XPathNSResolver", "xpathNsResolverHandles", "XPathNsResolver"),
    "xpath-result": ("XPathResult", "xpathResultHandles", "XPathResult"),
    "node-filter": ("NodeFilter", "nodeFilterHandles", "NodeFilter"),
    "dom-rect-read-only": ("DOMRectReadOnly", "domRectReadOnlyHandles", "DomRectReadOnly"),
    "xml-http-request-upload": ("XMLHttpRequestUpload", "xmlHttpRequestUploadHandles", "XmlHttpRequestUpload"),
    "selection": ("Selection", "selectionHandles", "Selection"),
    "mutation-observer": ("MutationObserver", "mutationObserverHandles", "MutationObserver"),
    "performance": ("Performance", "performanceHandles", "Performance"),
    "request": ("Request", "requestHandles", "Request"),
    "dom-matrix": ("DOMMatrix", "domMatrixhandles", "DomMatrix"),
    # Additional synthetic types for TS2322 fixes
    "range": ("Range", "rangeHandles", "Range"),
    "gamepad": ("Gamepad", "gamepadHandles", "Gamepad"),
    "crypto": ("Crypto", "cryptoHandles", "Crypto"),
    "x-slt-processor": ("XSLTProcessor", "xsltProcessorHandles", "XsltProcessor"),
    "xpath-result": ("XPathResult", "xpathResultHandles", "XpathResult"),
    "xpath-expression": ("XPathExpression", "xpathExpressionHandles", "XpathExpression"),
    "xpath-ns-resolver": ("XPathNSResolver", "xpathNsResolverHandles", "XpathNsResolver"),
    "node-filter": ("NodeFilter", "nodeFilterHandles", "NodeFilter"),
    # Additional types for TS2322 fixes
    "canvas-gradient": ("CanvasGradient", "canvasGradientHandles", "CanvasGradient"),
    "view-transition": ("ViewTransition", "viewTransitionHandles", "ViewTransition"),
    # Types for parameter conversion
    "buffer-source": ("BufferSource", "bufferSourceHandles", "BufferSource"),
    "event-listener": ("EventListener", "eventListenerHandles", "EventListener"),
    # Geolocation callbacks
    "position-callback": ("PositionCallback", "positionCallbackHandles", "PositionCallback"),
    "position-error-callback": ("PositionErrorCallback", "positionErrorCallbackHandles", "PositionErrorCallback"),
    "option-position-error-callback": ("PositionErrorCallback", "optionPositionErrorCallbackHandles", "OptionPositionErrorCallback"),
    # HTML/DOM types
    "offscreen-canvas": ("OffscreenCanvas", "offscreencanvasHandles", "Offscreencanvas"),
    "file-list": ("FileList", "fileListhandles", "FileList"),
    "message-event-source": ("EventTarget", "messageEventSourcehandles", "MessageEventSource"),
    # Media types
    "speech-synthesis-voice-list": ("SpeechSynthesisVoice[]", "speechSynthesisVoiceListhandles", "SpeechSynthesisVoiceList"),
    "speech-synthesis-voice": ("SpeechSynthesisVoice", "speechSynthesisVoiceHandles", "SpeechSynthesisVoice"),
    # Service Worker types
    "service-worker": ("ServiceWorker", "serviceWorkerhandles", "ServiceWorker"),
    "navigation-preload-manager": ("NavigationPreloadManager", "navigationPreloadManagerhandles", "NavigationPreloadManager"),
    # Observer types
    "resize-observer-size-list": ("ResizeObserverSize[]", "resizeObserverSizeListhandles", "ResizeObserverSizeList"),
    # Payment types
    "payment-address": ("PaymentAddress", "paymentAddresshandles", "PaymentAddress"),
    # WebRTC types
    "rtc-dtls-transport": ("RTCDtlsTransport", "rtcDtlsTransportHandles", "RtcDtlsTransport"),
    "rtc-rtp-sender": ("RTCRtpSender", "rtcRtpSenderHandles", "RtcRtpSender"),
    "rtc-rtp-receiver": ("RTCRtpReceiver", "rtcRtpReceiverHandles", "RtcRtpReceiver"),
    "rtc-rtp-transceiver": ("RTCRtpTransceiver", "rtcRtpTransceiverHandles", "RtcRtpTransceiver"),
    "rtc-ice-transport": ("RTCIceTransport", "rtcIceTransportHandles", "RtcIceTransport"),
    # Missing HTML element types
    "html-table-row-element": ("HTMLTableRowElement", "htmlTableRowElementHandles", "HtmlTableRowElement"),
    "html-table-cell-element": ("HTMLTableCellElement", "htmlTableCellElementHandles", "HtmlTableCellElement"),
    "html-option-element": ("HTMLOptionElement", "htmlOptionElementHandles", "HtmlOptionElement"),
    # Missing DOM types
    "html-slot-element": ("HTMLSlotElement", "htmlSlotElementHandles", "HtmlSlotElement"),
    # offscreencanvas (alias without hyphen)
    "offscreencanvas": ("OffscreenCanvas", "offscreencanvasHandles", "Offscreencanvas"),
    # form-data type
    "form-data": ("FormData", "formDataHandles", "FormData"),
    # image-bitmap type
    "image-bitmap": ("ImageBitmap", "imageBitmapHandles", "ImageBitmap"),
    # custom-state-set type
    "custom-state-set": ("CustomStateSet", "customStateSetHandles", "CustomStateSet"),
    # data-transfer-item type
    "data-transfer-item": ("DataTransferItem", "dataTransferItemHandles", "DataTransferItem"),
    # navigation types
    "navigation-history-entry": ("NavigationHistoryEntry", "navigationHistoryEntryHandles", "NavigationHistoryEntry"),
    "navigation-activation": ("NavigationActivation", "navigationActivationHandles", "NavigationActivation"),
    # date type (for valueAsDate)
    "date": ("Date", "dateHandles", "Date"),
    # plugin type
    "plugin": ("Plugin", "pluginHandles", "Plugin"),
    # mime-type
    "mime-type": ("MimeType", "mimeTypeHandles", "MimeType"),
    # image-data-array
    "image-data-array": ("ImageDataSettings", "imageDataArrayHandles", "ImageDataArray"),
    # speech-synthesis-utterance
    "speech-synthesis-utterance": ("SpeechSynthesisUtterance", "speechSynthesisUtteranceHandles", "SpeechSynthesisUtterance"),
    # blob type
    "blob": ("Blob", "blobHandles", "Blob"),
    # media-stream
    "media-stream": ("MediaStream", "mediaStreamHandles", "MediaStream"),
    # media-stream-track
    "media-stream-track": ("MediaStreamTrack", "mediaStreamTrackHandles", "MediaStreamTrack"),
    # queuing-strategy-size
    "queuing-strategy-size": ("any", "queuingStrategySizeHandles", "QueuingStrategySize"),
    # text-track-cue
    "text-track-cue": ("TextTrackCue", "textTrackCueHandles", "TextTrackCue"),
    # path-2-d
    "path-2-d": ("Path2D", "path2dHandles", "Path2d"),
    # image-data
    "image-data": ("ImageData", "imageDataHandles", "ImageData"),
    # canvas-image-source
    "canvas-image-source": ("CanvasImageSource", "canvasImageSourceHandles", "CanvasImageSource"),
    # html-option-or-opt-group
    "html-option-or-opt-group": ("HTMLOptGroupElement | HTMLOptionElement", "htmlOptionOrOptGroupHandles", "HtmlOptionOrOptGroup"),
    # rtc-ice-candidate
    "rtc-ice-candidate": ("RTCIceCandidate", "rtcIceCandidateHandles", "RtcIceCandidate"),
    # url-search-params
    "url-search-params": ("URLSearchParams", "urlSearchParamsHandles", "UrlSearchParams"),
    # media-stream-track-event
    "media-stream-track-event": ("MediaStreamTrackEvent", "mediaStreamTrackEventHandles", "MediaStreamTrackEvent"),
    # custom-state-set
    "custom-state-set": ("CustomStateSet", "customStateSetHandles", "CustomStateSet"),
    # navigation types
    "navigation-history-entry": ("NavigationHistoryEntry", "navigationHistoryEntryHandles", "NavigationHistoryEntry"),
    "navigation-activation": ("NavigationActivation", "navigationActivationHandles", "NavigationActivation"),
    # plugin/mimetype types
    "plugin-array": ("PluginArray", "pluginArrayHandles", "PluginArray"),
    "mime-type-array": ("MimeTypeArray", "mimeTypeArrayHandles", "MimeTypeArray"),
    "plugin": ("Plugin", "pluginHandles", "Plugin"),
    "mime-type": ("MimeType", "mimeTypeHandles", "MimeType"),
    # html-canvas-element (for CanvasRenderingContext2D.getCanvas)
    "html-canvas-element": ("HTMLCanvasElement", "htmlCanvasElementHandles", "HtmlCanvasElement"),
    # ImageDataArray (Uint8ClampedArray)
    "image-data-array": ("Uint8ClampedArray", "imageDataArrayHandles", "ImageDataArray"),
    # OffscreenCanvas (for transferToImageBitmap, getCanvas)
    "offscreencanvas": ("OffscreenCanvas", "offscreencanvasHandles", "Offscreencanvas"),
    # WebRTC list types
    "rtc-rtp-receiver-list": ("RTCRtpReceiver[]", "rtcRtpReceiverListHandles", "RtcRtpReceiverList"),
    "rtc-rtp-transceiver-list": ("RTCRtpTransceiver[]", "rtcRtpTransceiverListHandles", "RtcRtpTransceiverList"),
    "rtc-rtp-sender-list": ("RTCRtpSender[]", "rtcRtpSenderListHandles", "RtcRtpSenderList"),
    # MediaImage type
    "media-image": ("MediaImage", "mediaImageHandles", "MediaImage"),
    # MediaImage list type
    "media-image-list": ("MediaImage[]", "mediaImageListHandles", "MediaImageList"),
    # PerformanceEntryList type
    "performance-entry-list": ("PerformanceEntryList", "performanceEntryListHandles", "PerformanceEntryList"),
    # Event handler type for on* properties
    "event-handler": ("EventHandler", "eventHandlerHandles", "EventHandler"),
}

# Type definitions that need to be generated in glue code
CUSTOM_TYPE_DEFINITIONS = {
    "DOMTokenListValue": "string",
    "EventHandlerRecord": "any",
    "WebGLObject": "any",
    "u64": "bigint",
    "RTCDataChannelBinaryType": "\"blob\" | \"arraybuffer\"",
    "CSSFontFaceDescriptors": "any",
    "CSSFontFeatureValuesMap": "any",
    "OnBeforeUnloadEventHandlerRecord": "OnBeforeUnloadEventHandlerNonNull | null",
    "OnErrorEventHandlerRecord": "OnErrorEventHandlerNonNull | null",
    "VoidFunctionRecord": "VoidFunction",
    "EventHandler": "(this: any, ev: any) => any",
    "GeometryUtils": "any",
    "HyperlinkElementUtils": "any",
    "PopoverTargetAttributes": "any",
    "CSSPageDescriptors": "any",
    "CSSMarginRule": "CSSRule",
    "CSSStyleProperties": "Record<string, string>",
    "Origin": "string",
    "FetchLaterResult": "any",
    "NotRestoredReasonDetails": "any",
    "NotRestoredReasons": "any",
    "DeviceChangeEvent": "Event",
    "ChapterInformation": "any",
    "PerformanceTimingConfidence": "any",
    "NotificationEvent": "Event",
    "VisibilityStateEntry": "PerformanceEntry",
    "CommandEvent": "Event",
    "CloseWatcher": "EventTarget",
    "CaptureController": "any",
    "Navigation": "any",
    "NavigationTransition": "any",
    "NavigateEvent": "Event",
    "NavigationPrecommitController": "any",
    "NavigationDestination": "any",
    "NavigationCurrentEntryChangeEvent": "Event",
    # Battery API
    "BatteryManager": "any",
    # Audio/Video Track types (not in standard DOM lib)
    "AudioTrackList": "any",
    "AudioTrack": "any",
    "VideoTrackList": "any",
    "VideoTrack": "any",
    # Worker types (not in standard DOM lib - interfaces only, no runtime value)
    "WorkerGlobalScope": "any",
    "DedicatedWorkerGlobalScope": "any",
    "SharedWorkerGlobalScope": "any",
    "WorkerNavigator": "any",
    "WorkerLocation": "any",
    # Service Worker types
    "ServiceWorkerGlobalScope": "any",
    "Client": "any",
    "WindowClient": "any",
    "Clients": "any",
    "ExtendableEvent": "Event",
    "InstallEvent": "Event",
    "FetchEvent": "Event",
    "ExtendableMessageEvent": "MessageEvent",
    # Speech Recognition types (experimental)
    "SpeechRecognition": "any",
    "SpeechRecognitionErrorEvent": "Event",
    "SpeechRecognitionEvent": "Event",
    "SpeechGrammar": "any",
    "SpeechGrammarList": "any",
    "SpeechRecognitionPhrase": "any",
    # WebAssembly types (global constructors)
    "Module": "typeof WebAssembly.Module",
    "Instance": "typeof WebAssembly.Instance",
    "Memory": "typeof WebAssembly.Memory",
    "Table": "typeof WebAssembly.Table",
    "Global": "typeof WebAssembly.Global",
    "Exception": "any",
    "HTMLString": "string",
    "MediaText": "string",
    "HTMLHyperlinkHref": "string",
    "LocationHref": "string",
    "URLHref": "string",
}

# Type name casing corrections for TypeScript DOM types
TYPE_NAME_CASING_OVERRIDES = {
    "IdbCursor": "IDBCursor",
    "IdbCursorWithValue": "IDBCursorWithValue",
    "IdbDatabase": "IDBDatabase",
    "IdbFactory": "IDBFactory",
    "IdbIndex": "IDBIndex",
    "IdbKeyRange": "IDBKeyRange",
    "IdbObjectStore": "IDBObjectStore",
    "IdbOpenDBRequest": "IDBOpenDBRequest",
    "IdbRequest": "IDBRequest",
    "IdbTransaction": "IDBTransaction",
    "IdbVersionChangeEvent": "IDBVersionChangeEvent",
    "WebGlObject": "WebGLObject",
    "WebGlBuffer": "WebGLBuffer",
    "WebGlFramebuffer": "WebGLFramebuffer",
    "WebGlProgram": "WebGLProgram",
    "WebGlRenderbuffer": "WebGLRenderbuffer",
    "WebGlShader": "WebGLShader",
    "WebGlTexture": "WebGLTexture",
    "WebGlUniformLocation": "WebGLUniformLocation",
    "WebGlActiveInfo": "WebGLActiveInfo",
    "WebGlShaderPrecisionFormat": "WebGLShaderPrecisionFormat",
    "WebGlContextEvent": "WebGLContextEvent",
    "WebGlRenderingContext": "WebGLRenderingContext",
    "WebGlRenderingContextBase": "WebGLRenderingContextBase",
    "WebGlRenderingContextOverloads": "WebGLRenderingContextOverloads",
    "WebGl2RenderingContext": "WebGL2RenderingContext",
    "WebGl2RenderingContextBase": "WebGL2RenderingContextBase",
    "WebGl2RenderingContextOverloads": "WebGL2RenderingContextOverloads",
    "HtmlElement": "HTMLElement",
    "HtmlAllCollection": "HTMLAllCollection",
    "HtmlCollection": "HTMLCollection",
    "HtmlCollectionBase": "HTMLCollectionBase",
    "HtmlCollectionOf": "HTMLCollectionOf",
    "HtmlFormControlsCollection": "HTMLFormControlsCollection",
    "HtmlOptionsCollection": "HTMLOptionsCollection",
    "HtmlAnchorElement": "HTMLAnchorElement",
    "HtmlAreaElement": "HTMLAreaElement",
    "HtmlAudioElement": "HTMLAudioElement",
    "HtmlBaseElement": "HTMLBaseElement",
    "HtmlBodyElement": "HTMLBodyElement",
    "HtmlBrElement": "HTMLBRElement",
    "HtmlButtonElement": "HTMLButtonElement",
    "HtmlCanvasElement": "HTMLCanvasElement",
    "HtmlDataElement": "HTMLDataElement",
    "HtmlDataListElement": "HTMLDataListElement",
    "HtmlDetailsElement": "HTMLDetailsElement",
    "HtmlDialogElement": "HTMLDialogElement",
    "HtmlDirectoryElement": "HTMLDirectoryElement",
    "HtmlDivElement": "HTMLDivElement",
    "HtmlDListElement": "HTMLDListElement",
    "HtmlEmbedElement": "HTMLEmbedElement",
    "HtmlFieldSetElement": "HTMLFieldSetElement",
    "HtmlFontElement": "HTMLFontElement",
    "HtmlFormElement": "HTMLFormElement",
    "HtmlFrameElement": "HTMLFrameElement",
    "HtmlFrameSetElement": "HTMLFrameSetElement",
    "HtmlHeadElement": "HTMLHeadElement",
    "HtmlHeadingElement": "HTMLHeadingElement",
    "HtmlHrElement": "HTMLHRElement",
    "HtmlHtmlElement": "HTMLHtmlElement",
    "HtmlIFrameElement": "HTMLIFrameElement",
    "HtmlImageElement": "HTMLImageElement",
    "HtmlInputElement": "HTMLInputElement",
    "HtmlLabelElement": "HTMLLabelElement",
    "HtmlLegendElement": "HTMLLegendElement",
    "HtmlLiElement": "HTMLLIElement",
    "HtmlLinkElement": "HTMLLinkElement",
    "HtmlMapElement": "HTMLMapElement",
    "HtmlMarqueeElement": "HTMLMarqueeElement",
    "HtmlMediaElement": "HTMLMediaElement",
    "HtmlMenuElement": "HTMLMenuElement",
    "HtmlMetaElement": "HTMLMetaElement",
    "HtmlMeterElement": "HTMLMeterElement",
    "HtmlModElement": "HTMLModElement",
    "HtmlObjectElement": "HTMLObjectElement",
    "HtmlOListElement": "HTMLOListElement",
    "HtmlOptGroupElement": "HTMLOptGroupElement",
    "HtmlOptionElement": "HTMLOptionElement",
    "HtmlOutputElement": "HTMLOutputElement",
    "HtmlParagraphElement": "HTMLParagraphElement",
    "HtmlParamElement": "HTMLParamElement",
    "HtmlPictureElement": "HTMLPictureElement",
    "HtmlPreElement": "HTMLPreElement",
    "HtmlProgressElement": "HTMLProgressElement",
    "HtmlQuoteElement": "HTMLQuoteElement",
    "HtmlScriptElement": "HTMLScriptElement",
    "HtmlSelectElement": "HTMLSelectElement",
    "HtmlSlotElement": "HTMLSlotElement",
    "HtmlSourceElement": "HTMLSourceElement",
    "HtmlSpanElement": "HTMLSpanElement",
    "HtmlStyleElement": "HTMLStyleElement",
    "HtmlTableCaptionElement": "HTMLTableCaptionElement",
    "HtmlTableCellElement": "HTMLTableCellElement",
    "HtmlTableColElement": "HTMLTableColElement",
    "HtmlTableElement": "HTMLTableElement",
    "HtmlTableRowElement": "HTMLTableRowElement",
    "HtmlTableSectionElement": "HTMLTableSectionElement",
    "HtmlTemplateElement": "HTMLTemplateElement",
    "HtmlTextAreaElement": "HTMLTextAreaElement",
    "HtmlTimeElement": "HTMLTimeElement",
    "HtmlTitleElement": "HTMLTitleElement",
    "HtmlTrackElement": "HTMLTrackElement",
    "HtmlUListElement": "HTMLUListElement",
    "HtmluListElement": "HTMLUListElement",
    "HtmlhrElement": "HTMLHRElement",
    "HtmloListElement": "HTMLOListElement",
    "HtmlliElement": "HTMLLIElement",
    "HtmldListElement": "HTMLDListElement",
    "HtmlbrElement": "HTMLBRElement",
    "HtmliFrameElement": "HTMLIFrameElement",
    "HtmlUnknownElement": "HTMLUnknownElement",
    "HtmlVideoElement": "HTMLVideoElement",
    "HtmlOrSvgElement": "HTMLOrSVGElement",
    "HtmlHyperlinkElementUtils": "HTMLHyperlinkElementUtils",
    "DomImplementation": "DOMImplementation",
    "DomParser": "DOMParser",
    "DomRect": "DOMRect",
    "DomRectList": "DOMRectList",
    "DomRectReadOnly": "DOMRectReadOnly",
    "DomStringList": "DOMStringList",
    "DomStringMap": "DOMStringMap",
    "DomTokenList": "DOMTokenList",
    "DomMatrix": "DOMMatrix",
    "DomMatrixReadOnly": "DOMMatrixReadOnly",
    "DomPoint": "DOMPoint",
    "DomPointReadOnly": "DOMPointReadOnly",
    "DomQuad": "DOMQuad",
    "DomSettableTokenList": "DOMSettableTokenList",
    "CssRule": "CSSRule",
    "CssRuleList": "CSSRuleList",
    "CssStyleDeclaration": "CSSStyleDeclaration",
    "CssStyleRule": "CSSStyleRule",
    "CssStyleSheet": "CSSStyleSheet",
    "CssConditionRule": "CSSConditionRule",
    "CssContainerRule": "CSSContainerRule",
    "CssCounterStyleRule": "CSSCounterStyleRule",
    "CssFontFaceRule": "CSSFontFaceRule",
    "CssFontFeatureValuesRule": "CSSFontFeatureValuesRule",
    "CssFontPaletteValuesRule": "CSSFontPaletteValuesRule",
    "CssGroupingRule": "CSSGroupingRule",
    "CssImageLayerRepetition": "CSSImageLayerRepetition",
    "CssImportRule": "CSSImportRule",
    "CssKeyframeRule": "CSSKeyframeRule",
    "CssKeyframesRule": "CSSKeyframesRule",
    "CssKeywordValue": "CSSKeywordValue",
    "CssMarginRule": "CSSMarginRule",
    "CssMediaRule": "CSSMediaRule",
    "CssNamespaceRule": "CSSNamespaceRule",
    "CssNumericValue": "CSSNumericValue",
    "CssPageRule": "CSSPageRule",
    "CssPerspective": "CSSPerspective",
    "CssPositionValue": "CSSPositionValue",
    "CssPrimitiveValue": "CSSPrimitiveValue",
    "CssPropertyRule": "CSSPropertyRule",
    "CssRotate": "CSSRotate",
    "CssScale": "CSSScale",
    "CssSkew": "CSSSkew",
    "CssStyleValue": "CSSStyleValue",
    "CssSupportsRule": "CSSSupportsRule",
    "CssTransformComponent": "CSSTransformComponent",
    "CssTransformValue": "CSSTransformValue",
    "CssTranslate": "CSSTranslate",
    "CssUnitValue": "CSSUnitValue",
    "CssUnparsedValue": "CSSUnparsedValue",
    "CssValue": "CSSValue",
    "CssValueList": "CSSValueList",
    "CssVariableReferenceValue": "CSSVariableReferenceValue",
    "CssFontFaceDescriptors": "CSSFontFaceDescriptors",
    "CssFontFeatureValuesMap": "CSSFontFeatureValuesMap",
    "CssPageDescriptors": "CSSPageDescriptors",
    "CssStyleProperties": "CSSStyleProperties",
    "ElementCssInlineStyle": "ElementCSSInlineStyle",
    "XmlDocument": "XMLDocument",
    "XmlHttpRequest": "XMLHttpRequest",
    "XmlHttpRequestEventTarget": "XMLHttpRequestEventTarget",
    "XmlHttpRequestUpload": "XMLHttpRequestUpload",
    "XmlSerializer": "XMLSerializer",
    "UiEvent": "UIEvent",
    "RtcCertificate": "RTCCertificate",
    "RtcDataChannel": "RTCDataChannel",
    "RtcDataChannelEvent": "RTCDataChannelEvent",
    "RtcDtlsTransport": "RTCDtlsTransport",
    "RtcEncodedAudioFrame": "RTCEncodedAudioFrame",
    "RtcEncodedVideoFrame": "RTCEncodedVideoFrame",
    "RtcError": "RTCError",
    "RtcErrorEvent": "RTCErrorEvent",
    "RtcIceCandidate": "RTCIceCandidate",
    "RtcIceCandidatePair": "RTCIceCandidatePair",
    "RtcIceTransport": "RTCIceTransport",
    "RtcPeerConnection": "RTCPeerConnection",
    "RtcPeerConnectionIceErrorEvent": "RTCPeerConnectionIceEvent",
    "RtcPeerConnectionIceEvent": "RTCPeerConnectionIceEvent",
    "RtcRtpReceiver": "RTCRtpReceiver",
    "RtcRtpScriptTransform": "RTCRtpScriptTransform",
    "RtcRtpSender": "RTCRtpSender",
    "RtcRtpTransceiver": "RTCRtpTransceiver",
    "RtcSctpTransport": "RTCSctpTransport",
    "RtcSessionDescription": "RTCSessionDescription",
    "RtcStatsReport": "RTCStatsReport",
    "RtcTrackEvent": "RTCTrackEvent",
    "RtcTransformEvent": "RTCTransformEvent",
    "RtcdtmfSender": "RTCDTMFSender",
    "RtcdtmfToneChangeEvent": "RTCDTMFToneChangeEvent",
    "Url": "URL",
    "UrlSearchParams": "URLSearchParams",
    "Origin": "Origin",
    "Module": "Module",
    "Instance": "Instance",
    "Table": "Table",
    "Memory": "Memory",
    "Global": "Global",
    "Exception": "Exception",
    "GeometryUtils": "GeometryUtils",
    "XPathNsResolver": "XPathNSResolver",
    "XsltProcessor": "XSLTProcessor",
    "HyperlinkElementUtils": "HyperlinkElementUtils",
    "SpeechGrammar": "SpeechGrammar",
    "SpeechGrammarList": "SpeechGrammarList",
    "SpeechRecognition": "SpeechRecognition",
    "SpeechRecognitionErrorEvent": "SpeechRecognitionErrorEvent",
    "SpeechRecognitionEvent": "SpeechRecognitionEvent",
    "SpeechRecognitionPhrase": "SpeechRecognitionPhrase",
    "FetchLaterResult": "FetchLaterResult",
    "VisibilityStateEntry": "VisibilityStateEntry",
    "CommandEvent": "CommandEvent",
    "CloseWatcher": "CloseWatcher",
    "PopoverTargetAttributes": "PopoverTargetAttributes",
    "Navigation": "Navigation",
    "NavigationTransition": "NavigationTransition",
    "NavigateEvent": "NavigateEvent",
    "NavigationPrecommitController": "NavigationPrecommitController",
    "NavigationDestination": "NavigationDestination",
    "NavigationCurrentEntryChangeEvent": "NavigationCurrentEntryChangeEvent",
    "NotRestoredReasonDetails": "NotRestoredReasonDetails",
    "NotRestoredReasons": "NotRestoredReasons",
    "NavigatorId": "NavigatorID",
    "WorkerGlobalScope": "WorkerGlobalScope",
    "DedicatedWorkerGlobalScope": "DedicatedWorkerGlobalScope",
    "SharedWorkerGlobalScope": "SharedWorkerGlobalScope",
    "WorkerNavigator": "WorkerNavigator",
    "WorkerLocation": "WorkerLocation",
    "PerformanceTimingConfidence": "PerformanceTimingConfidence",
    "OnErrorEventHandlerRecord": "OnErrorEventHandlerNonNull",
    "OnBeforeUnloadEventHandlerRecord": "OnBeforeUnloadEventHandlerNonNull",
    "VoidFunctionRecord": "VoidFunction",
    "MessageEventTarget": "MessageEventTarget<any>",
    "AudioTrackList": "AudioTrackList",
    "AudioTrack": "AudioTrack",
    "VideoTrackList": "VideoTrackList",
    "VideoTrack": "VideoTrack",
    "TextTrackCue": "TextTrackCue",
    "TextTrackList": "TextTrackList",
    "ReadableStreamByobReader": "ReadableStreamBYOBReader",
    "ReadableStreamByobRequest": "ReadableStreamBYOBRequest",
    "ClipboardChangeEvent": "ClipboardEvent",
}

# Properties/methods that need type assertions (not in TypeScript DOM lib)
# Format: (interface_wit_name, property_name)
PROPERTIES_NEEDING_TYPE_ASSERTION = {
    # VideoFrame
    ("video-frame", "rotation"),
    ("video-frame", "flip"),
    ("video-frame", "metadata"),
    # ImageTrackList
    ("image-track-list", "imageTrack"),
    ("image-track-list", "getReady"),
    # WebGLRenderingContextBase
    ("web-gl-rendering-context-base", "drawingBufferFormat"),
    ("web-gl-rendering-context-base", "drawingBufferStorage"),
    ("web-gl-rendering-context-base", "error"),
    # GlobalEventHandlers
    ("global-event-handlers", "oncommand"),
    # CSSMediaRule/CSSSupportsRule
    ("css-media-rule", "matches"),
    ("css-supports-rule", "matches"),
    # CSSFontFeatureValuesRule
    ("css-font-feature-values-rule", "annotation"),
    ("css-font-feature-values-rule", "ornaments"),
    ("css-font-feature-values-rule", "stylistic"),
    ("css-font-feature-values-rule", "swash"),
    ("css-font-feature-values-rule", "characterVariant"),
    ("css-font-feature-values-rule", "styleset"),
    ("css-font-feature-values-rule", "historicalForms"),
    # Window
    ("window", "fetchLater"),
    ("window", "object"),
    # Document
    ("document", "parseHtmlUnsafe"),
    ("document", "object"),
    # Element
    ("element", "customElementRegistry"),
    ("element", "html"),
    # HTMLElement
    ("html-element", "scrollParent"),
    ("html-element", "headingOffset"),
    ("html-element", "headingReset"),
    # Range
    ("range", "start"),
    ("range", "end"),
    ("range", "endAfter"),
    # DocumentOrShadowRoot
    ("document-or-shadow-root", "customElementRegistry"),
    # CSSRule
    ("css-rule", "name"),
    ("css-rule", "style"),
    # CSSMarginRule
    ("css-margin-rule", "name"),
    ("css-margin-rule", "style"),
    # CSSStyleDeclaration
    ("css-style-declaration", "property"),
    # Navigator
    ("navigator", "battery"),
    ("navigator", "gamepads"),
    # NavigatorContentUtils
    ("navigator-content-utils", "unregisterProtocolHandler"),
    # VisualViewport
    ("visual-viewport", "onscrollend"),
    # MessagePort
    ("message-port", "onclose"),
    # Gamepad
    ("gamepad", "touches"),
    # GamepadHapticActuator
    ("gamepad-haptic-actuator", "effects"),
    # ScreenOrientation
    ("screen-orientation", "lock"),
    # EventListener
    ("event-listener", "handleEvent"),
    # ParentNode
    ("parent-node", "moveBefore"),
    # Node
    ("node", "rootNode"),
    # ShadowRoot
    ("shadow-root", "html"),
    # NodeFilter
    ("node-filter", "acceptNode"),
    # XPathNSResolver
    ("x-path-ns-resolver", "lookupNamespaceURI"),
    # ClipboardEvent
    ("clipboard-change-event", "changeId"),
    ("clipboard-change-event", "types"),
    # PointerEvent
    ("pointer-event", "persistentDeviceId"),
    # Touch
    ("touch", "altitudeAngle"),
    ("touch", "azimuthAngle"),
    ("touch", "touchType"),
    # TouchEvent
    ("touch-event", "getModifierState"),
    # Request
    ("request", "isReloadNavigation"),
    ("request", "isHistoryNavigation"),
    ("request", "duplex"),
    # WindowOrWorkerGlobalScope
    ("window-or-worker-global-scope", "timeout"),
    ("window-or-worker-global-scope", "interval"),
    # ReadableStream
    ("readable-stream", "from"),
    ("readable-stream", "reader"),
    # WritableStream
    ("writable-stream", "writer"),
    # FormData
    ("form-data", "all"),
    # HTMLAllCollection
    ("html-all-collection", "element"),
    # HTMLOptionsCollection
    ("html-options-collection", "undefined"),
    # HTMLMediaElement
    ("html-media-element", "startDate"),
    ("html-media-element", "getAudioTracks"),
    ("html-media-element", "getVideoTracks"),
    # TextTrackList
    ("text-track-list", "textTrack"),
    # TextTrackCueList
    ("text-track-cue-list", "textTrackCue"),
    ("text-track-cue-list", "cueById"),
    # HTMLInputElement
    ("html-input-element", "alpha"),
    ("html-input-element", "colorSpace"),
    # HTMLButtonElement
    ("html-button-element", "command"),
    ("html-button-element", "commandForElement"),
    # HTMLSelectElement
    ("html-select-element", "undefined"),
    # HTMLDialogElement
    ("html-dialog-element", "closedBy"),
    # HTMLTemplateElement
    ("html-template-element", "shadowRootCustomElementRegistry"),
    # HTMLCanvasElement
    ("html-canvas-element", "context"),
    # CanvasPathDrawingStyles
    ("canvas-path-drawing-styles", "lineDash"),
    # CanvasTextDrawingStyles
    ("canvas-text-drawing-styles", "lang"),
    # OffscreenCanvas
    ("offscreencanvas", "context"),
    # CustomElementRegistry
    ("custom-element-registry", "name"),
    ("custom-element-registry", "initialize"),
    # ToggleEvent
    ("toggle-event", "source"),
    # CommandEvent
    ("command-event", "source"),
    ("command-event", "command"),
    # CloseWatcher (EventTarget subtype)
    ("close-watcher", "requestClose"),
    ("close-watcher", "close"),
    ("close-watcher", "destroy"),
    ("close-watcher", "oncancel"),
    ("close-watcher", "onclose"),
    # NavigateEvent
    ("navigate-event", "navigationType"),
    ("navigate-event", "destination"),
    ("navigate-event", "canIntercept"),
    ("navigate-event", "userInitiated"),
    ("navigate-event", "hashChange"),
    ("navigate-event", "signal"),
    ("navigate-event", "formData"),
    ("navigate-event", "downloadRequest"),
    ("navigate-event", "info"),
    ("navigate-event", "hasUAVisualTransition"),
    ("navigate-event", "intercept"),
    ("navigate-event", "scroll"),
    # NavigationCurrentEntryChangeEvent
    ("navigation-current-entry-change-event", "navigationType"),
    ("navigation-current-entry-change-event", "from"),
    # DeviceChangeEvent
    ("device-change-event", "devices"),
    ("device-change-event", "userInsertedDevices"),
    # SpeechRecognitionErrorEvent
    ("speech-recognition-error-event", "error"),
    ("speech-recognition-error-event", "message"),
    # SpeechRecognitionEvent
    ("speech-recognition-event", "resultIndex"),
    ("speech-recognition-event", "results"),
    # NotificationEvent
    ("notification-event", "notification"),
    ("notification-event", "action"),
    # ServiceWorkerContainer
    ("service-worker-container", "getReady"),
    # Animation
    ("animation", "persist"),
    # RTCPeerConnection
    ("rtc-peer-connection", "restartIce"),
    # Geolocation (callback-based, but generator treats as async)
    ("geolocation", "getCurrentPosition"),
    ("extendable-event", "waitUntil"),
    ("extendable-event", "addRoutes"),
    # InstallEvent
    ("install-event", "addRoutes"),
    # FetchEvent
    ("fetch-event", "request"),
    ("fetch-event", "preloadResponse"),
    ("fetch-event", "clientId"),
    ("fetch-event", "resultingClientId"),
    ("fetch-event", "replacesClientId"),
    ("fetch-event", "handled"),
    ("fetch-event", "respondWith"),
    # RTCPeerConnectionIceErrorEvent
    ("rtc-peer-connection-ice-error-event", "address"),
    ("rtc-peer-connection-ice-error-event", "port"),
    ("rtc-peer-connection-ice-error-event", "url"),
    ("rtc-peer-connection-ice-error-event", "errorCode"),
    ("rtc-peer-connection-ice-error-event", "errorText"),
    # EventTarget (keep generic ones)
    ("event-target", "requestClose"),
    ("event-target", "close"),
    ("event-target", "destroy"),
    ("event-target", "oncancel"),
    ("event-target", "onclose"),
    # DataTransfer
    ("data-transfer", "data"),
    # DataTransferItemList
    ("data-transfer-item-list", "dataTransferItem"),
    # DataTransferItem
    ("data-transfer-item", "asString"),
    ("data-transfer-item", "asFile"),
    # NavigationHistoryEntry
    ("navigation-history-entry", "state"),
    # NavigatorID
    ("navigator-id", "taintEnabled"),
    ("navigator-id", "oscpu"),
    # ImageData
    ("image-data", "pixelFormat"),
    # MediaStream
    ("media-stream", "tracks"),
    # MediaStreamTrack
    ("media-stream-track", "settings"),
    # MediaSession
    ("media-session", "screenshareActive"),
    # MediaMetadata
    ("media-metadata", "chapterInfo"),
    # MediaRecorder
    ("media-recorder", "audioBitrateMode"),
    # SpeechSynthesis
    ("speech-synthesis", "voices"),
    # Notification
    ("notification", "navigate"),
    ("notification", "image"),
    ("notification", "vibrate"),
    ("notification", "timestamp"),
    ("notification", "renotify"),
    ("notification", "actions"),
    # IntersectionObserver
    ("intersection-observer", "scrollMargin"),
    ("intersection-observer", "delay"),
    ("intersection-observer", "trackVisibility"),
    # IntersectionObserverEntry
    ("intersection-observer-entry", "isVisible"),
    # Performance
    ("performance", "entries"),
    # PerformanceNavigationTiming
    ("performance-navigation-timing", "criticalChRestart"),
    ("performance-navigation-timing", "notRestoredReasons"),
    ("performance-navigation-timing", "confidence"),
    # PerformanceEntry
    ("performance-entry", "id"),
    ("performance-entry", "navigationId"),
    # PerformanceObserverEntryList
    ("performance-observer-entry-list", "entries"),
    # PerformanceResourceTiming
    ("performance-resource-timing", "deliveryType"),
    ("performance-resource-timing", "finalResponseHeadersStart"),
    ("performance-resource-timing", "firstInterimResponseStart"),
    ("performance-resource-timing", "workerRouterEvaluationStart"),
    ("performance-resource-timing", "workerCacheLookupStart"),
    ("performance-resource-timing", "workerMatchedRouterSource"),
    ("performance-resource-timing", "workerFinalRouterSource"),
    ("performance-resource-timing", "renderBlockingStatus"),
    ("performance-resource-timing", "contentType"),
    ("performance-resource-timing", "contentEncoding"),
    # NavigationPreloadManager
    ("navigation-preload-manager", "state"),
    # RTCIceCandidate
    ("rtc-ice-candidate", "relayProtocol"),
    ("rtc-ice-candidate", "url"),
    # RTCPeerConnectionIceEvent
    ("rtc-peer-connection-ice-event", "url"),
    ("rtc-peer-connection-ice-event", "address"),
    ("rtc-peer-connection-ice-event", "port"),
    ("rtc-peer-connection-ice-event", "errorCode"),
    ("rtc-peer-connection-ice-event", "errorText"),
    # RTCRtpSender
    ("rtc-rtp-sender", "getCapabilities"),
    ("rtc-rtp-sender", "streams"),
    ("rtc-rtp-sender", "stats"),
    # RTCRtpReceiver
    ("rtc-rtp-receiver", "getCapabilities"),
    ("rtc-rtp-receiver", "stats"),
    # RTCIceTransport
    ("rtc-ice-transport", "role"),
    ("rtc-ice-transport", "component"),
    ("rtc-ice-transport", "localCandidates"),
    ("rtc-ice-transport", "remoteCandidates"),
    ("rtc-ice-transport", "localParameters"),
    ("rtc-ice-transport", "remoteParameters"),
    # RTCPeerConnection
    ("rtc-peer-connection", "senders"),
    ("rtc-peer-connection", "stats"),
    # WebSocket
    ("ws", "connect"),
    # WebAssembly Instance/Memory/Table/Global
    ("instance", "exports"),
    ("memory", "grow"),
    ("memory", "toFixedLengthBuffer"),
    ("memory", "toResizableBuffer"),
    ("memory", "buffer"),
    ("table", "grow"),
    ("table", "get"),
    ("table", "set"),
    ("global", "value"),
    # FileReader
    ("file-reader", "newFileReader"),
    # WebGLObject (label property)
    ("web-gl-object", "label"),
    # Origin type (string methods)
    ("origin", "opaque"),
    ("origin", "isSameOrigin"),
    ("origin", "isSameSite"),
    # Notification static
    ("notification", "maxActions"),
    # OffscreenCanvas
    ("offscreen-canvas", "context"),
    # Permissions
    ("permissions", "getQuery"),
    # URLSearchParams
    ("url-search-params", "all"),
    # ImageDecoder
    ("image-decoder", "completed"),
    # CSSKeyframesRule
    ("css-keyframes-rule", "KEYFRAME_RULE"),
    # ReadableStreamGenericReader
    ("readable-stream-generic-reader", "closed"),
    # WritableStreamDefaultWriter
    ("writable-stream-default-writer", "closed"),
    ("writable-stream-default-writer", "ready"),
    # FileReader
    ("file-reader", "readyState"),
    ("file-reader", "result"),
    # FileList
    ("file-list", "length"),
    # DOMStringMap
    ("dom-string-map", "domString"),
    ("dom-string-map", "undefined"),
    # PromiseRejectionEvent
    ("promise-rejection-event", "promise"),
    # IDBDatabase
    ("idb-database", "name"),
    ("idb-database", "version"),
    # IDBIndex
    ("idb-index", "keyPath"),
    ("idb-index", "multiEntry"),
    ("idb-index", "unique"),
    # IDBObjectStore
    ("idb-object-store", "keyPath"),
    ("idb-object-store", "autoIncrement"),
    # IDBRequest
    ("idb-request", "source"),
    # IDBCursor
    ("idb-cursor", "key"),
    ("idb-cursor", "primaryKey"),
    ("idb-cursor", "source"),
    # MediaSession
    ("media-session", "metadata"),
    # IntersectionObserverEntry
    ("intersection-observer-entry", "boundingClientRect"),
    # ServiceWorkerRegistration
    ("service-worker-registration", "scope"),
    # ServiceWorker
    ("service-worker", "scriptURL"),
    # Animation
    ("animation", "pending"),
    # RTCDTMFSender
    ("rtcdtmf-sender", "canInsertDTMF"),
    # WebSocket
    ("web-socket", "url"),
    ("web-socket", "readyState"),
    ("web-socket", "bufferedAmount"),
    ("web-socket", "extensions"),
    ("web-socket", "protocol"),
    # ServiceWorkerContainer
    ("service-worker-container", "controller"),
    # WebGL2RenderingContextOverloads - methods with complex overloads
    ("web-gl2-rendering-context-overloads", "texImage2D"),
    ("web-gl2-rendering-context-overloads", "texSubImage2D"),
    ("web-gl2-rendering-context-overloads", "compressedTexImage2D"),
    ("web-gl2-rendering-context-overloads", "compressedTexSubImage2D"),
    ("web-gl2-rendering-context-overloads", "readPixels"),
    # SubtleCrypto - methods with complex algorithm parameters
    ("subtle-crypto", "deriveKey"),
    ("subtle-crypto", "importKey"),
    ("subtle-crypto", "unwrapKey"),
    # CSSStyleDeclaration - methods with complex parameters
    ("css-style-declaration", "setProperty"),
    ("css-style-sheet", "insertRule"),
    # HTMLFormElement - reset method
    ("html-form-element", "reset"),
    # DedicatedWorkerGlobalScope/WorkerGlobalScope - get method
    ("dedicated-worker-global-scope", "get"),
    ("worker-global-scope", "get"),
    # RTCRtpTransceiver - setCodecPreferences
    ("rtc-rtp-transceiver", "setCodecPreferences"),
    # Document - methods with complex parameter types
    ("document", "createElement"),
    ("document", "createElementNS"),
    ("document", "getElementsByTagNameNS"),
    # Element - methods with complex parameter types
    ("element", "getElementsByTagNameNS"),
    # Navigator - vibrate method
    ("navigator", "vibrate"),
    # ServiceWorkerContainer - getRegistration
    ("service-worker-container", "getRegistration"),
    # Worker - postMessage
    ("worker", "postMessage"),
    # CanvasDrawPath - fill/clip/isPointInPath need type assertion (CanvasFillRule is string type)
    ("canvas-draw-path", "fill"),
    ("canvas-draw-path", "clip"),
    ("canvas-draw-path", "isPointInPath"),
    # MessageEvent - initMessageEvent has complex parameter types
    ("message-event", "initMessageEvent"),
    # Cache - addAll takes RequestInfo[] which is complex
    ("cache", "addAll"),
    # HTMLSlotElement - assign takes variadic nodes
    ("html-slot-element", "assign"),
    # HTMLFormElement - elements returns HTMLFormControlsCollection
    ("html-form-element", "getElements"),
    # TextTrack - addCue/removeCue takes TextTrackCue
    ("text-track", "addCue"),
    ("text-track", "removeCue"),
    # HTMLSelectElement - add takes HTMLOptGroupElement | HTMLOptionElement
    ("html-select-element", "add"),
    # Canvas - createPattern takes CanvasImageSource
    ("canvas-fill-stroke-styles", "createPattern"),
    # Canvas - drawImage takes CanvasImageSource
    ("canvas-draw-image", "drawImage"),
    # Canvas - putImageData takes ImageData
    ("canvas-image-data", "putImageData"),
    # Canvas - fill/stroke take optional Path2D
    ("canvas-draw-path", "fill"),
    ("canvas-draw-path", "stroke"),
    ("canvas-draw-path", "clip"),
    ("canvas-draw-path", "isPointInPath"),
    ("canvas-draw-path", "isPointInStroke"),
    # CustomElementRegistry - define takes CustomElementConstructor
    ("custom-element-registry", "define"),
    # HTMLElement - focus takes FocusOptions
    ("html-element", "focus"),
    # HTMLSlotElement - assignedNodes/assignedElements take AssignedNodesOptions
    ("html-slot-element", "assignedNodes"),
    ("html-slot-element", "assignedElements"),
    # HTMLCanvasElement - toBlob takes BlobCallback
    ("html-canvas-element", "toBlob"),
    # OffscreenCanvas - convertToBlob takes ImageEncodeOptions
    ("offscreencanvas", "convertToBlob"),
    # FormData - append takes string | Blob
    ("form-data", "append"),
    # XMLHttpRequest - send takes body
    ("xml-http-request", "send"),
    # EventTarget - dispatchEvent takes Event
    ("event-target", "dispatchEvent"),
    # Request constructor - takes body
    ("request", "new-request"),
    # ResizeObserver - observe takes options
    ("resize-observer", "observe"),
    # HTMLTableElement - insertRow takes index
    ("html-table-element", "insertRow"),
    ("html-table-section-element", "insertRow"),
    ("html-table-row-element", "insertCell"),
    # HTMLMediaElement - canPlayType returns CanPlayTypeResult
    ("html-media-element", "canPlayType"),
    # Window - requestAnimationFrame takes FrameRequestCallback
    ("window", "requestAnimationFrame"),
    # Worklet - addModule takes options
    ("worklet", "addModule"),
    # Navigator - plugins/mimeTypes item returns Plugin | null
    ("navigator", "getPlugins"),
    ("navigator", "getMimeTypes"),
    # PluginArray/MimeTypeArray - item/namedItem return Plugin | null
    ("plugin-array", "item"),
    ("plugin-array", "namedItem"),
    ("mime-type-array", "item"),
    ("mime-type-array", "namedItem"),
    # Plugin - item/namedItem return MimeType | null
    ("plugin", "item"),
    ("plugin", "namedItem"),
    # StorageEvent - initStorageEvent
    ("storage-event", "initStorageEvent"),
    # MouseEvent - initMouseEvent
    ("mouse-event", "init-mouse-event"),
    # KeyboardEvent - initKeyboardEvent
    ("keyboard-event", "init-keyboard-event"),
    # UIEvent - initUIEvent
    ("ui-event", "init-ui-event"),
    # Event - initEvent
    ("event", "init-event"),
    # MediaRecorder - start takes timeslice
    ("media-recorder", "start"),
    # Blob - slice takes content type string
    ("blob", "slice"),
    # URL - createObjectURL takes Blob | MediaSource
    ("url", "create-object-url"),
    # HTMLAnchorElement - assign takes string
    ("html-anchor-element", "assign"),
    # Document - getSetCookie is non-standard
    ("document", "getSetCookie"),
    # HTMLOListElement - start property setter
    ("htmlo-list-element", "start"),
    # Event handlers - need type assertion for bigint ↔ function conversion
    ("window-event-handlers", "ongamepadconnected"),
    ("window-event-handlers", "ongamepaddisconnected"),
    ("window-event-handlers", "onpagereveal"),
    ("window-event-handlers", "onafterprint"),
    ("window-event-handlers", "onbeforeprint"),
    ("window-event-handlers", "onbeforeunload"),
    ("window-event-handlers", "onhashchange"),
    ("window-event-handlers", "onlanguagechange"),
    ("window-event-handlers", "onmessage"),
    ("window-event-handlers", "onmessageerror"),
    ("window-event-handlers", "onoffline"),
    ("window-event-handlers", "ononline"),
    ("window-event-handlers", "onpagehide"),
    ("window-event-handlers", "onpageshow"),
    ("window-event-handlers", "onpageswap"),
    ("window-event-handlers", "onpopstate"),
    ("window-event-handlers", "onrejectionhandled"),
    ("window-event-handlers", "onstorage"),
    ("window-event-handlers", "onunhandledrejection"),
    ("window-event-handlers", "onunload"),
    ("global-event-handlers", "onabort"),
    ("global-event-handlers", "onanimationcancel"),
    ("global-event-handlers", "onanimationend"),
    ("global-event-handlers", "onanimationiteration"),
    ("global-event-handlers", "onanimationstart"),
    ("global-event-handlers", "onauxclick"),
    ("global-event-handlers", "onblur"),
    ("global-event-handlers", "oncanplay"),
    ("global-event-handlers", "oncanplaythrough"),
    ("global-event-handlers", "oncancel"),
    ("global-event-handlers", "onchange"),
    ("global-event-handlers", "onclick"),
    ("global-event-handlers", "onclose"),
    ("global-event-handlers", "oncontextlost"),
    ("global-event-handlers", "oncontextmenu"),
    ("global-event-handlers", "oncontextrestored"),
    ("global-event-handlers", "oncopy"),
    ("global-event-handlers", "oncuechange"),
    ("global-event-handlers", "oncut"),
    ("global-event-handlers", "ondblclick"),
    ("global-event-handlers", "ondrag"),
    ("global-event-handlers", "ondragend"),
    ("global-event-handlers", "ondragenter"),
    ("global-event-handlers", "ondragleave"),
    ("global-event-handlers", "ondragover"),
    ("global-event-handlers", "ondragstart"),
    ("global-event-handlers", "ondrop"),
    ("global-event-handlers", "ondurationchange"),
    ("global-event-handlers", "onemptied"),
    ("global-event-handlers", "onended"),
    ("global-event-handlers", "onerror"),
    ("global-event-handlers", "onfocus"),
    ("global-event-handlers", "onfocusin"),
    ("global-event-handlers", "onfocusout"),
    ("global-event-handlers", "onformdata"),
    ("global-event-handlers", "ongotpointercapture"),
    ("global-event-handlers", "oninput"),
    ("global-event-handlers", "oninvalid"),
    ("global-event-handlers", "onkeydown"),
    ("global-event-handlers", "onkeypress"),
    ("global-event-handlers", "onkeyup"),
    ("global-event-handlers", "onload"),
    ("global-event-handlers", "onloadeddata"),
    ("global-event-handlers", "onloadedmetadata"),
    ("global-event-handlers", "onloadstart"),
    ("global-event-handlers", "onlostpointercapture"),
    ("global-event-handlers", "onmousedown"),
    ("global-event-handlers", "onmouseenter"),
    ("global-event-handlers", "onmouseleave"),
    ("global-event-handlers", "onmousemove"),
    ("global-event-handlers", "onmouseout"),
    ("global-event-handlers", "onmouseover"),
    ("global-event-handlers", "onmouseup"),
    ("global-event-handlers", "onpaste"),
    ("global-event-handlers", "onpause"),
    ("global-event-handlers", "onplay"),
    ("global-event-handlers", "onplaying"),
    ("global-event-handlers", "onpointercancel"),
    ("global-event-handlers", "onpointerdown"),
    ("global-event-handlers", "onpointerenter"),
    ("global-event-handlers", "onpointerleave"),
    ("global-event-handlers", "onpointermove"),
    ("global-event-handlers", "onpointerout"),
    ("global-event-handlers", "onpointerover"),
    ("global-event-handlers", "onpointerup"),
    ("global-event-handlers", "onprogress"),
    ("global-event-handlers", "onratechange"),
    ("global-event-handlers", "onreset"),
    ("global-event-handlers", "onresize"),
    ("global-event-handlers", "onscroll"),
    ("global-event-handlers", "onscrollend"),
    ("global-event-handlers", "onsecuritypolicyviolation"),
    ("global-event-handlers", "onseeked"),
    ("global-event-handlers", "onseeking"),
    ("global-event-handlers", "onselect"),
    ("global-event-handlers", "onselectionchange"),
    ("global-event-handlers", "onselectstart"),
    ("global-event-handlers", "onslotchange"),
    ("global-event-handlers", "onstalled"),
    ("global-event-handlers", "onsubmit"),
    ("global-event-handlers", "onsuspend"),
    ("global-event-handlers", "ontimeupdate"),
    ("global-event-handlers", "ontoggle"),
    ("global-event-handlers", "ontransitioncancel"),
    ("global-event-handlers", "ontransitionend"),
    ("global-event-handlers", "ontransitionrun"),
    ("global-event-handlers", "ontransitionstart"),
    ("global-event-handlers", "onvolumechange"),
    ("global-event-handlers", "onwaiting"),
    ("global-event-handlers", "onwebkitanimationend"),
    ("global-event-handlers", "onwebkitanimationiteration"),
    ("global-event-handlers", "onwebkitanimationstart"),
    ("global-event-handlers", "onwebkittransitionend"),
    ("global-event-handlers", "onwheel"),
    # ServiceWorker event handlers
    ("service-worker", "onstatechange"),
    ("service-worker", "onerror"),
    # WebSocket event handlers
    ("web-socket", "onopen"),
    ("web-socket", "onclose"),
    ("web-socket", "onerror"),
    ("web-socket", "onmessage"),
    # RTC event handlers
    ("rtc-peer-connection", "onconnectionstatechange"),
    ("rtc-peer-connection", "ondatachannel"),
    ("rtc-peer-connection", "onicecandidate"),
    ("rtc-peer-connection", "onicecandidateerror"),
    ("rtc-peer-connection", "oniceconnectionstatechange"),
    ("rtc-peer-connection", "onicegatheringstatechange"),
    ("rtc-peer-connection", "onnegotiationneeded"),
    ("rtc-peer-connection", "onsignalingstatechange"),
    ("rtc-peer-connection", "ontrack"),
    ("rtc-data-channel", "onopen"),
    ("rtc-data-channel", "onclose"),
    ("rtc-data-channel", "onclosing"),
    ("rtc-data-channel", "onerror"),
    ("rtc-data-channel", "onmessage"),
    ("rtc-data-channel", "onbufferedamountlow"),
    # MediaStream event handlers - need type assertion
    ("media-stream", "onaddtrack"),
    ("media-stream", "onremovetrack"),
    # MediaStreamTrack event handlers - need type assertion
    ("media-stream-track", "onmute"),
    ("media-stream-track", "onunmute"),
    ("media-stream-track", "onended"),
    # MediaRecorder event handlers - need type assertion
    ("media-recorder", "onstart"),
    ("media-recorder", "onstop"),
    ("media-recorder", "ondataavailable"),
    ("media-recorder", "onpause"),
    ("media-recorder", "onresume"),
    ("media-recorder", "onerror"),
    # SpeechSynthesis event handlers - need type assertion
    ("speech-synthesis", "onvoiceschanged"),
    # SpeechSynthesisUtterance event handlers - need type assertion
    ("speech-synthesis-utterance", "onstart"),
    ("speech-synthesis-utterance", "onend"),
    ("speech-synthesis-utterance", "onerror"),
    ("speech-synthesis-utterance", "onpause"),
    ("speech-synthesis-utterance", "onresume"),
    ("speech-synthesis-utterance", "onmark"),
    ("speech-synthesis-utterance", "onboundary"),
    # MediaDevices event handlers
    ("media-devices", "ondevicechange"),
    # RTCDtlsTransport event handlers
    ("rtc-dtls-transport", "onstatechange"),
    ("rtc-dtls-transport", "onerror"),
    # RTCIceTransport event handlers
    ("rtc-ice-transport", "onstatechange"),
    ("rtc-ice-transport", "ongatheringstatechange"),
    ("rtc-ice-transport", "onselectedcandidatepairchange"),
    # RTCSctpTransport event handlers
    ("rtc-sctp-transport", "onstatechange"),
    # ServiceWorkerContainer event handlers
    ("service-worker-container", "onmessage"),
    ("service-worker-container", "onmessageerror"),
    ("service-worker-container", "oncontrollerchange"),
    # ServiceWorkerRegistration event handlers
    ("service-worker-registration", "onupdatefound"),
    # PaymentRequest event handlers
    ("payment-request", "onshippingaddresschange"),
    ("payment-request", "onshippingoptionchange"),
    ("payment-request", "onpaymentmethodchange"),
    # Performance event handlers
    ("performance", "onresourcetimingbufferfull"),
    # Notification event handlers
    ("notification", "onclick"),
    ("notification", "onshow"),
    ("notification", "onerror"),
    ("notification", "onclose"),
    # MouseEvent getModifierState - method not property
    ("mouse-event", "getModifierState"),
    # getType methods - missing from TypeScript DOM types
    ("credential", "type"),
    ("encoded-audio-chunk", "type"),
    ("encoded-video-chunk", "type"),
    ("image-decoder", "type"),
    ("web-gl-active-info", "type"),
    ("crypto-key", "type"),
    ("css-rule", "type"),
    ("style-sheet", "type"),
    ("screen-orientation", "type"),
    ("screen-orientation", "onchange"),
    ("event", "type"),
    ("mutation-record", "type"),
    ("response", "type"),
    ("html-link-element", "type"),
    ("html-style-element", "type"),
    ("htmlo-list-element", "type"),
    ("htmlu-list-element", "type"),
    ("htmlli-element", "type"),
    ("html-anchor-element", "type"),
    ("html-source-element", "type"),
    ("html-embed-element", "type"),
    ("html-object-element", "type"),
    ("html-input-element", "type"),
    ("html-button-element", "type"),
    ("html-select-element", "type"),
    ("html-text-area-element", "type"),
    ("html-output-element", "type"),
    ("html-field-set-element", "type"),
    ("html-script-element", "type"),
    ("data-transfer-item", "type"),
    ("mime-type", "type"),
    ("html-param-element", "type"),
    ("performance-navigation-timing", "type"),
    ("performance-navigation", "type"),
    ("rtc-session-description", "type"),
    ("rtc-ice-candidate", "type"),
    # ReadableStreamBYOBRequest.view returns ArrayBufferView
    ("readable-stream-byob-request", "view"),
    # PaymentRequestUpdateEvent.updateWith takes PaymentDetailsUpdate
    ("payment-request-update-event", "updateWith"),
    # WebAssembly Global.valueOf returns Object
    ("global", "valueOf"),
    # Animation.id setter expects string
    ("animation", "id"),
    # Animation.playbackRate getter returns number
    ("animation", "playbackRate"),
    # PermissionStatus.onchange setter needs function
    ("permission-status", "onchange"),
}

# Properties that return readonly arrays and need spreading
# Format: (interface_wit_name, property_name)
READONLY_ARRAY_PROPERTIES = {
    ("data-transfer", "types"),
    ("performance", "entryList"),
    ("navigator-language", "languages"),
    ("performance-entry-list", "entryList"),
    ("clipboard-item", "types"),
    ("performance-observer", "supportedEntryTypes"),
    # Gamepad axes and buttons are readonly
    ("gamepad", "axes"),
    ("gamepad", "buttons"),
    # Location ancestorOrigins returns DOMStringList
    ("location", "ancestorOrigins"),
    # IntersectionObserver thresholds is readonly
    ("intersection-observer", "thresholds"),
    # ResizeObserverEntry box sizes are readonly arrays
    ("resize-observer-entry", "borderBoxSize"),
    ("resize-observer-entry", "contentBoxSize"),
    ("resize-observer-entry", "devicePixelContentBoxSize"),
}

# Properties that return arrays of objects that need handle conversion
# Format: (interface_wit_name, method_name_in_camelCase) -> (target_type, element_type)
# target_type is the synthetic type for the array, element_type is the type of each element
HANDLE_RETURNING_ARRAY_PROPERTIES = {
    ("message-event", "getPorts"): ("message-port-list", "message-port"),
    ("media-metadata", "getArtwork"): ("media-image-list", "media-image"),
    ("resize-observer-entry", "getBorderBoxSize"): ("resize-observer-size-list", "resize-observer-size"),
    ("resize-observer-entry", "getContentBoxSize"): ("resize-observer-size-list", "resize-observer-size"),
    ("resize-observer-entry", "getDevicePixelContentBoxSize"): ("resize-observer-size-list", "resize-observer-size"),
    ("rtc-rtp-receiver", "getStreams"): ("media-stream-list", "media-stream"),
    ("rtc-peer-connection", "getLocalStreams"): ("media-stream-list", "media-stream"),
    ("rtc-track-event", "getStreams"): ("media-stream-list", "media-stream"),
    ("extendable-message-event", "getPorts"): ("message-port-list", "message-port"),
    # PointerEvent getCoalescedEvents/getPredictedEvents return arrays
    ("pointer-event", "getCoalescedEvents"): ("pointer-event-list", "pointer-event"),
    ("pointer-event", "getPredictedEvents"): ("pointer-event-list", "pointer-event"),
    # WebRTC methods that return arrays of objects
    ("rtc-peer-connection", "getReceivers"): ("rtc-rtp-receiver-list", "rtc-rtp-receiver"),
    ("rtc-peer-connection", "getTransceivers"): ("rtc-rtp-transceiver-list", "rtc-rtp-transceiver"),
    ("rtc-peer-connection", "getSenders"): ("rtc-rtp-sender-list", "rtc-rtp-sender"),
    ("rtc-certificate", "getFingerprints"): ("any", "any"),
    # DocumentOrShadowRoot getAdoptedStyleSheets returns CSSStyleSheet[]
    ("document-or-shadow-root", "getAdoptedStyleSheets"): ("css-style-sheet-list", "css-style-sheet"),
}

# JavaScript/TypeScript reserved keywords
JS_RESERVED_WORDS = {
    "break", "case", "catch", "continue", "debugger", "default", "delete",
    "do", "else", "finally", "for", "function", "if", "in", "instanceof",
    "new", "return", "switch", "this", "throw", "try", "typeof", "var",
    "void", "while", "with", "class", "const", "enum", "export", "extends",
    "import", "super", "implements", "interface", "let", "package", "private",
    "protected", "public", "static", "yield", "null", "true", "false",
    "undefined", "NaN", "Infinity", "await", "async", "of", "get", "set",
    "arguments", "eval",
}


def correct_type_casing(name: str) -> str:
    """Correct the casing of a type name to match TypeScript DOM conventions."""
    if name in TYPE_NAME_CASING_OVERRIDES:
        return TYPE_NAME_CASING_OVERRIDES[name]
    return name


def strip_generic_params(name: str) -> str:
    """Strip generic parameters from a type name for use in variable/function names.
    
    Examples:
        'MessageEventTarget<any>' -> 'MessageEventTarget'
        'Foo<T, U>' -> 'Foo'
    """
    idx = name.find('<')
    if idx >= 0:
        return name[:idx]
    return name
