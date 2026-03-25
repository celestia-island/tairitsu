/**
 * Style glue — implements the `tairitsu-browser:full/style` WIT import interface.
 *
 * This interface is manually defined in browser-full.wit, not auto-generated from WebIDL.
 * DO NOT EDIT MANUALLY - this file provides CSS style manipulation support for WASM components.
 */

/**
 * Handle table for Element instances (shared with other glue modules).
 */
declare const _elementHandles: Map<bigint, Element>;

/**
 * Lookup an Element by handle, throwing if not found.
 */
function lookupElement(handle: bigint): Element {
  const obj = _elementHandles.get(handle);
  if (obj === undefined) {
    throw new Error(`Element handle ${handle} not found`);
  }
  return obj!;
}

/**
 * Set a CSS property on an element.
 */
export function setStyleProperty(element: bigint, property: string, value: string): void | string {
  try {
    const obj = lookupElement(element);
    (obj as any).style.setProperty(property, value);
  } catch (e) {
    return String(e);
  }
}

/**
 * Get a CSS property value from an element.
 */
export function getStyleProperty(element: bigint, property: string): string | undefined {
  const obj = lookupElement(element);
  return (obj as any).style.getPropertyValue(property) || undefined;
}

/**
 * Remove a CSS property from an element.
 */
export function removeStyleProperty(element: bigint, property: string): void | string {
  try {
    const obj = lookupElement(element);
    (obj as any).style.removeProperty(property);
  } catch (e) {
    return String(e);
  }
}
