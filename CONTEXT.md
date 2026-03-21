# Context: Fixing TS2322 Type Mismatch Errors in browser-glue

## Goal

Fix TS2322 type mismatch errors in browser-glue generated code by modifying the generator scripts (not the generated files directly).

## Progress

- **Initial:** 216 TS2322 errors
- **Current:** 148 TS2322 errors
- **Fixed:** 68 errors

## What Was Done

### 1. Added Conversion Types in `config.py` (`PARAMETER_BIGINT_TO_NUMBER`)

**String setters needing `string-or-null`:**
- `html-element`: `set-access-key`, `set-autocapitalize`, `set-dir`, `set-inner-text`, `set-lang`, `set-title`, `set-translate`, `set-popover`, `set-outer-text`, `set-hidden`
- `html-anchor-element`: `set-download`, `set-href`, `set-hreflang`, `set-ping`, `set-rel`, `set-target`
- `html-media-element`: `set-src`, `set-cross-origin`
- `html-image-element`: `set-src`, `set-cross-origin`
- `html-link-element`: `set-cross-origin`
- `html-script-element`: `set-cross-origin`
- `html-source-element`, `html-track-element`, `html-iframe-element`, `html-embed-element`: `set-src`
- `node`: `set-node-value`, `set-text-content`

**Number setters needing `number-or-null`:**
- `html-input-element`: `set-selection-start`, `set-selection-end`
- `rtc-rtp-receiver`: `set-jitter-buffer-target`

**Enum string setters needing `enum-string`:**
- `html-media-element`: `set-preload`
- `html-image-element`: `set-loading`, `set-fetch-priority`, `set-decoding`
- `html-link-element`: `set-fetch-priority`, `set-loading`
- `html-script-element`: `set-fetch-priority`
- `html-button-element`: `set-type`
- `html-input-element`: `set-form-enctype`, `set-form-method`, `set-enter-key-hint`, `set-autocomplete`
- `html-text-area-element`: `set-enter-key-hint`, `set-wrap`, `set-autocomplete`
- `html-form-element`: `set-enctype`, `set-method`, `set-autocomplete`
- `html-select-element`: `set-autocomplete`
- `html-style-element`: `set-media`
- `web-socket`: `set-binary-type`

**Event handler setters needing `event-handler`:**
- `window-event-handlers`: `set-ongamepadconnected`, `set-ongamepaddisconnected`
- `global-event-handlers`: onclick, ondblclick, onmousedown, onmouseup, onmouseover, onmousemove, onmouseout, onkeydown, onkeyup, onfocus, onblur, onchange, onsubmit, onreset, oninput
- `screen-orientation`: `set-onchange`
- `rtc-peer-connection`: all on* event setters
- `rtc-data-channel`: all on* event setters
- `web-socket`: all on* event setters

**Handle setters needing `optional-handle:*` or `optional-handle-strict:*`:**
- `document`: `set-body` → `optional-handle-strict:html-element`

### 2. Added Conversion Type Handling in `code_gen.py`

Added handling in `_render_setter_body()` for:
- `string-or-null` → `{param} ?? null`
- `number-or-null` → `{param} ?? null`
- `boolean-or-false` → `{param} ?? false`
- `enum-string` → `{param} as any`
- `event-handler` → `{param} as any`
- `optional-handle-strict:*` → `lookupOption{Type}({param}) as any`

### 3. Added Method Setters in `config.py`

Added to `SETTER_BUT_ACTUALLY_METHOD`:
- `html-object-element`, `html-input-element`, `html-button-element`, `html-select-element`, `html-text-area-element`, `html-output-element`, `html-field-set-element`: `custom-validity`

Added to `SETTER_METHOD_NAMES`:
- All the above with method name `setCustomValidity`

## Remaining Errors (148)

Most remaining errors are **return type mismatches** where the DOM API returns a different type than the WIT definition expects. These require WIT definition changes, not generator config changes.

### Error Categories:

1. **DOM returns string, WIT expects bigint (handle)** - 7 errors
2. **DOM returns number, WIT expects bigint (handle)** - 6 errors
3. **DOM returns string | undefined, WIT expects string** - 4 errors
4. **DOM returns number, WIT expects string** - 4 errors
5. **DOM returns Promise<void>, WIT expects bigint (handle)** - 3 errors
6. **DOM returns object, WIT expects bigint (handle)** - Various (TextTrack, ViewTransition, Storage, RTCRtpTransceiver, etc.)

### Files with Most Errors:
- `webrtcGlue.ts` (42)
- `htmlGlue.ts` (42)
- `mediaGlue.ts` (23)

## Relevant Files

- `/mnt/sdb1/tairitsu/scripts/generator/config.py` - Main config file with type mappings
- `/mnt/sdb1/tairitsu/scripts/generator/code_gen.py` - Code generator
- `/mnt/sdb1/tairitsu/scripts/generate_browser_glue.py` - Main generator script
- `/mnt/sdb1/tairitsu/packages/browser-glue/src/generated/` - Generated files (do not edit)

## Commands

```bash
# Regenerate
python3 scripts/generate_browser_glue.py

# Verify
cd packages/browser-glue && npx tsc --noEmit 2>&1 | grep -c "error TS2322"
```
