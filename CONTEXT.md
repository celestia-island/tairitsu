# Context: Fixing TS2322 Type Mismatch Errors in browser-glue

## Goal

Fix TS2322 type mismatch errors in browser-glue generated code by modifying the generator scripts (not the generated files directly).

## Progress

- **Initial:** 216 TS2322 errors
- **Current:** 83 TypeScript errors (16 in target files mediaGlue.ts and eventsGlue.ts)
- **Fixed:** 133 errors

## What Was Done

### 1. Fixed ENUM_PROPERTIES entries to use property names

Changed entries from function names (e.g., `"get-pointer-type"`) to property names (e.g., `"pointerType"`):
- `("pointer-event", "pointerType"): "PointerType"`
- `("input-event", "inputType"): "InputType"`
- `("keyboard-event", "key"): "KeyString"`
- `("composition-event", "data"): "CompositionData"`
- And 14 more entries

### 2. Fixed BOOLEAN_TO_BIGINT_PROPERTIES entries to use property names

Changed entries from function names to property names:
- `("event", "bubbles")`, `("event", "cancelable")`, `("event", "composed")`
- `("touch-event", "metaKey")`, `("touch-event", "altKey")`, `("touch-event", "ctrlKey")`
- And 20 more entries

### 3. Added to GETTER_BUT_ACTUALLY_METHOD

Added `"modifier-state"` for `KeyboardEvent.getModifierState()` which is a method, not a property.

### 4. Added to SYNTHETIC_HANDLE_TYPES

Added `"speech-synthesis-voice"` to enable creation of `lookupOptionSpeechSynthesisVoice` function.

### 5. Previous work (from earlier session)

- Added conversion types in `PARAMETER_BIGINT_TO_NUMBER`
- Added handling in `code_gen.py` for various conversion types
- Added method setters in `SETTER_BUT_ACTUALLY_METHOD`

## Remaining Errors (16 in target files)

### eventsGlue.ts (9 errors)
- Line 474: `ClipboardChangeEventGetTypes` - returns `obj.type` but expects `string[]`
- Line 525: readonly string[] to string[] assignment
- Line 1150: boolean not assignable to bigint
- Line 1205: number not assignable to bigint
- Line 1279: string | undefined not assignable to string
- Line 1418: initKeyboardEvent parameter type issues
- Line 1472: initCompositionEvent parameter type issues
- Line 1507: initTextEvent parameter type issues

### mediaGlue.ts (7 errors)
- Line 1154: string not assignable to number
- Line 1309: bigint not assignable to boolean
- Lines 1406, 1414: MediaImage[] handle array conversion issues
- Line 1684: number not assignable to string
- Line 2168: number not assignable to boolean
- Line 2203: string not assignable to number

## Key Config Patterns

1. **ENUM_PROPERTIES**: Use property name (camelCase), not function name
   - Correct: `("pointer-event", "pointerType")`
   - Wrong: `("pointer-event", "get-pointer-type")`

2. **BOOLEAN_TO_BIGINT_PROPERTIES**: Use property name, not function name
   - Correct: `("event", "bubbles")`
   - Wrong: `("event", "get-bubbles")`

3. **GETTER_BUT_ACTUALLY_METHOD**: Use name without "get-" prefix
   - Correct: `"modifier-state"`
   - Wrong: `"get-modifier-state"`

4. **SYNTHETIC_HANDLE_TYPES**: Add types that need `lookupOption*` functions

## Relevant Files

- `/mnt/sdb1/tairitsu/scripts/generator/config.py` - Main config file
- `/mnt/sdb1/tairitsu/scripts/generator/code_gen.py` - Code generator
- `/mnt/sdb1/tairitsu/scripts/generate_browser_glue.py` - Main generator script
- `/mnt/sdb1/tairitsu/packages/browser-glue/src/generated/` - Generated files (do not edit)

## Commands

```bash
# Regenerate
python3 scripts/generate_browser_glue.py

# Verify
cd packages/browser-glue && npx tsc --noEmit 2>&1 | grep -c "error TS"
```
