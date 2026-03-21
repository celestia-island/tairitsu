/**
 * Console glue — implements the `tairitsu-browser:full/console` WIT import interface.
 *
 * This interface is manually defined in browser-full.wit, not auto-generated from WebIDL.
 * DO NOT EDIT MANUALLY - this file provides console logging support for WASM components.
 */

/**
 * Log a message to the console.
 */
export function log(message: string): void {
  console.log(message);
}

/**
 * Log a warning to the console.
 */
export function warn(message: string): void {
  console.warn(message);
}

/**
 * Log an error to the console.
 */
export function error(message: string): void {
  console.error(message);
}
