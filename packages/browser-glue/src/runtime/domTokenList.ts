export const domTokenList_exports = {
  add(self: bigint, tokens: string[]) {
    const el = globalThis.__lookupElement(self);
    if (el && el.classList) {
      el.classList.add(...tokens);
    }
  },
  remove(self: bigint, tokens: string[]) {
    const el = globalThis.__lookupElement(self);
    if (el && el.classList) {
      el.classList.remove(...tokens);
    }
  },
  contains(self: bigint, token: string) {
    const el = globalThis.__lookupElement(self);
    if (el && el.classList) {
      return el.classList.contains(token);
    }
    return false;
  },
};
