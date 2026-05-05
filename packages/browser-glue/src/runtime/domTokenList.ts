// @ts-nocheck

export const domTokenList_exports = {
  add(self, tokens) {
    const el = globalThis.__lookupElement(self);
    if (el && el.classList) {
      el.classList.add(...tokens);
    }
  },
  remove(self, tokens) {
    const el = globalThis.__lookupElement(self);
    if (el && el.classList) {
      el.classList.remove(...tokens);
    }
  },
  contains(self, token) {
    const el = globalThis.__lookupElement(self);
    if (el && el.classList) {
      return el.classList.contains(token);
    }
    return false;
  },
};
