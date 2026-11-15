/**
 * `tairitsu-browser:full/html-input-element` glue.
 *
 * This interface is load-bearing for the framework itself: `tairitsu-web` reads
 * a committed `<input>` value through it (`wit_platform.rs`, the
 * `html_input_element` getter that delivers `ChangeEvent.value`), so a component
 * built against `tairitsu-web` imports this interface unconditionally. Before
 * this module existed the runtime registry wired none of it and every browser
 * component failed at instantiation with
 *
 *   TypeError: Cannot destructure property 'getValue' of
 *   'imports.tairitsu-browser:full/html-input-element' as it is undefined.
 *
 * Handles are `HTMLInputElement` entries in the shared `__elementHandles` table
 * (see `document.ts` / `platformHelpers.ts`), exactly like every other element
 * interface in this directory.
 *
 * Coverage: every attribute/operation that maps straight onto the element and
 * therefore needs no extra handle table. The handful that do need one
 * (`get-form`, `get-files`/`set-files`, `get-list`, `get-labels`,
 * `get-validity`) are deliberately left out and pinned as the known gap by
 * `test/registry-completeness.test.mjs`, so adding one there without wiring it
 * here (or vice versa) fails the suite.
 */

/** The host-side element behind an `html-input-element-handle`. */
function inputOf(self: bigint) {
  return globalThis.__lookupElement(self);
}

/** WIT `option<T>` absent-value convention in this runtime is `undefined`. */
function optionOf<T>(value: T | null | undefined): T | undefined {
  return value === null ? undefined : value;
}

export const htmlInputElement_exports = {
  // -- string attributes ---------------------------------------------------
  getAccept(self: bigint): string {
    return inputOf(self).accept ?? "";
  },
  setAccept(self: bigint, value: string) {
    inputOf(self).accept = value;
  },
  getAlt(self: bigint): string {
    return inputOf(self).alt ?? "";
  },
  setAlt(self: bigint, value: string) {
    inputOf(self).alt = value;
  },
  getAlign(self: bigint): string {
    return inputOf(self).align ?? "";
  },
  setAlign(self: bigint, value: string) {
    inputOf(self).align = value;
  },
  getAutocomplete(self: bigint): string {
    return inputOf(self).autocomplete ?? "";
  },
  setAutocomplete(self: bigint, value: string) {
    inputOf(self).autocomplete = value;
  },
  getDirName(self: bigint): string {
    return inputOf(self).dirName ?? "";
  },
  setDirName(self: bigint, value: string) {
    inputOf(self).dirName = value;
  },
  getColorSpace(self: bigint): string {
    return inputOf(self).colorSpace ?? "";
  },
  setColorSpace(self: bigint, value: string) {
    inputOf(self).colorSpace = value;
  },
  getFormAction(self: bigint): string {
    return inputOf(self).formAction ?? "";
  },
  setFormAction(self: bigint, value: string) {
    inputOf(self).formAction = value;
  },
  getFormEnctype(self: bigint): string {
    return inputOf(self).formEnctype ?? "";
  },
  setFormEnctype(self: bigint, value: string) {
    inputOf(self).formEnctype = value;
  },
  getFormMethod(self: bigint): string {
    return inputOf(self).formMethod ?? "";
  },
  setFormMethod(self: bigint, value: string) {
    inputOf(self).formMethod = value;
  },
  getFormTarget(self: bigint): string {
    return inputOf(self).formTarget ?? "";
  },
  setFormTarget(self: bigint, value: string) {
    inputOf(self).formTarget = value;
  },
  getName(self: bigint): string {
    return inputOf(self).name ?? "";
  },
  setName(self: bigint, value: string) {
    inputOf(self).name = value;
  },
  getPattern(self: bigint): string {
    return inputOf(self).pattern ?? "";
  },
  setPattern(self: bigint, value: string) {
    inputOf(self).pattern = value;
  },
  getPlaceholder(self: bigint): string {
    return inputOf(self).placeholder ?? "";
  },
  setPlaceholder(self: bigint, value: string) {
    inputOf(self).placeholder = value;
  },
  getSrc(self: bigint): string {
    return inputOf(self).src ?? "";
  },
  setSrc(self: bigint, value: string) {
    inputOf(self).src = value;
  },
  getType(self: bigint): string {
    return inputOf(self).type ?? "";
  },
  setType(self: bigint, value: string) {
    inputOf(self).type = value;
  },
  getUseMap(self: bigint): string {
    return inputOf(self).useMap ?? "";
  },
  setUseMap(self: bigint, value: string) {
    inputOf(self).useMap = value;
  },

  // -- numeric attributes --------------------------------------------------
  getMin(self: bigint): string {
    return inputOf(self).min ?? "";
  },
  setMin(self: bigint, value: string) {
    inputOf(self).min = value;
  },
  getMax(self: bigint): string {
    return inputOf(self).max ?? "";
  },
  setMax(self: bigint, value: string) {
    inputOf(self).max = value;
  },
  getStep(self: bigint): string {
    return inputOf(self).step ?? "";
  },
  setStep(self: bigint, value: string) {
    inputOf(self).step = value;
  },
  getMinLength(self: bigint): number {
    return Number(inputOf(self).minLength ?? 0);
  },
  setMinLength(self: bigint, value: number) {
    inputOf(self).minLength = value;
  },
  getMaxLength(self: bigint): number {
    return Number(inputOf(self).maxLength ?? 0);
  },
  setMaxLength(self: bigint, value: number) {
    inputOf(self).maxLength = value;
  },
  getSize(self: bigint): number {
    return Number(inputOf(self).size ?? 0);
  },
  setSize(self: bigint, value: number) {
    inputOf(self).size = value;
  },
  getHeight(self: bigint): number {
    return Number(inputOf(self).height ?? 0);
  },
  setHeight(self: bigint, value: number) {
    inputOf(self).height = value;
  },
  getWidth(self: bigint): number {
    return Number(inputOf(self).width ?? 0);
  },
  setWidth(self: bigint, value: number) {
    inputOf(self).width = value;
  },

  // -- boolean attributes --------------------------------------------------
  getAlpha(self: bigint): boolean {
    return Boolean(inputOf(self).alpha);
  },
  setAlpha(self: bigint, value: boolean) {
    inputOf(self).alpha = value;
  },
  getDefaultChecked(self: bigint): boolean {
    return Boolean(inputOf(self).defaultChecked);
  },
  setDefaultChecked(self: bigint, value: boolean) {
    inputOf(self).defaultChecked = value;
  },
  getChecked(self: bigint): boolean {
    return Boolean(inputOf(self).checked);
  },
  setChecked(self: bigint, value: boolean) {
    inputOf(self).checked = value;
  },
  getDisabled(self: bigint): boolean {
    return Boolean(inputOf(self).disabled);
  },
  setDisabled(self: bigint, value: boolean) {
    inputOf(self).disabled = value;
  },
  getFormNoValidate(self: bigint): boolean {
    return Boolean(inputOf(self).formNoValidate);
  },
  setFormNoValidate(self: bigint, value: boolean) {
    inputOf(self).formNoValidate = value;
  },
  getIndeterminate(self: bigint): boolean {
    return Boolean(inputOf(self).indeterminate);
  },
  setIndeterminate(self: bigint, value: boolean) {
    inputOf(self).indeterminate = value;
  },
  getMultiple(self: bigint): boolean {
    return Boolean(inputOf(self).multiple);
  },
  setMultiple(self: bigint, value: boolean) {
    inputOf(self).multiple = value;
  },
  getReadOnly(self: bigint): boolean {
    return Boolean(inputOf(self).readOnly);
  },
  setReadOnly(self: bigint, value: boolean) {
    inputOf(self).readOnly = value;
  },
  getRequired(self: bigint): boolean {
    return Boolean(inputOf(self).required);
  },
  setRequired(self: bigint, value: boolean) {
    inputOf(self).required = value;
  },

  // -- value state ---------------------------------------------------------
  getDefaultValue(self: bigint): string {
    return inputOf(self).defaultValue ?? "";
  },
  setDefaultValue(self: bigint, value: string) {
    inputOf(self).defaultValue = value;
  },
  getValue(self: bigint): string {
    return inputOf(self).value ?? "";
  },
  setValue(self: bigint, value: string) {
    inputOf(self).value = value;
  },
  getValueAsNumber(self: bigint): number {
    return Number(inputOf(self).valueAsNumber ?? Number.NaN);
  },
  setValueAsNumber(self: bigint, value: number) {
    inputOf(self).valueAsNumber = value;
  },
  /**
   * `valueAsDate` as epoch milliseconds.
   *
   * The WIT contract declares `u64`, but the DOM's date-state range starts at
   * year 1 — `1969-12-31` and every birthdate before 1970 are perfectly valid
   * values whose epoch is NEGATIVE. A negative number cannot cross a `u64`
   * boundary: jco either throws (`invalid u64 value`) or wraps it to a
   * meaningless positive number, both of which are worse than a honest "none".
   *
   * So: pre-1970 dates return `undefined` (WIT `none`), and are NOT clamped to
   * the epoch — a clamped `0` would be indistinguishable from a real
   * `1970-01-01` selection. Callers that need those dates must use the string
   * form (`getValue()` / `getDefaultValue()`), which is lossless.
   */
  getValueAsDate(self: bigint): bigint | undefined {
    const date = optionOf(inputOf(self).valueAsDate);
    if (date === undefined) return undefined;
    const epochMs = date.getTime();
    if (!Number.isFinite(epochMs) || epochMs < 0) return undefined;
    return BigInt(epochMs);
  },
  /** Symmetric limitation: a `u64` cannot express a pre-1970 date. */
  setValueAsDate(self: bigint, value: bigint | undefined) {
    inputOf(self).valueAsDate =
      value === undefined ? null : new Date(Number(value));
  },
  stepUp(self: bigint, n: number | undefined) {
    inputOf(self).stepUp(n ?? undefined);
  },
  stepDown(self: bigint, n: number | undefined) {
    inputOf(self).stepDown(n ?? undefined);
  },

  // -- constraint validation (the parts that need no handle table) ---------
  getWillValidate(self: bigint): boolean {
    return Boolean(inputOf(self).willValidate);
  },
  getValidationMessage(self: bigint): string {
    return inputOf(self).validationMessage ?? "";
  },
  checkValidity(self: bigint): boolean {
    return Boolean(inputOf(self).checkValidity());
  },
  reportValidity(self: bigint): boolean {
    return Boolean(inputOf(self).reportValidity());
  },
  setCustomValidity(self: bigint, error: string) {
    inputOf(self).setCustomValidity(error);
  },

  // -- text selection & picker --------------------------------------------
  select(self: bigint) {
    inputOf(self).select();
  },
  getSelectionStart(self: bigint): number | undefined {
    return optionOf(inputOf(self).selectionStart);
  },
  setSelectionStart(self: bigint, value: number | undefined) {
    inputOf(self).selectionStart = value ?? null;
  },
  getSelectionEnd(self: bigint): number | undefined {
    return optionOf(inputOf(self).selectionEnd);
  },
  setSelectionEnd(self: bigint, value: number | undefined) {
    inputOf(self).selectionEnd = value ?? null;
  },
  getSelectionDirection(self: bigint): string | undefined {
    return optionOf(inputOf(self).selectionDirection);
  },
  setSelectionDirection(self: bigint, value: string | undefined) {
    inputOf(self).selectionDirection = value ?? null;
  },
  setSelectionRange(
    self: bigint,
    start: number,
    end: number,
    direction?: string,
  ) {
    inputOf(self).setSelectionRange(start, end, direction ?? undefined);
  },
  setRangeText(self: bigint, replacement: string) {
    inputOf(self).setRangeText(replacement);
  },
  showPicker(self: bigint) {
    const el = inputOf(self);
    if (typeof el.showPicker === "function") el.showPicker();
  },
};
