
export const element_exports = {
  setAttribute(self: bigint, qualifiedName: string, value: string) {
    const name = typeof qualifiedName === 'string' && qualifiedName.startsWith('r#')
      ? qualifiedName.slice(2)
      : qualifiedName;
    globalThis.__lookupElement(self).setAttribute(name, value);
  },
  removeAttribute(self: bigint, qualifiedName: string) {
    const name = typeof qualifiedName === 'string' && qualifiedName.startsWith('r#')
      ? qualifiedName.slice(2)
      : qualifiedName;
    globalThis.__lookupElement(self).removeAttribute(name);
  },
  getBoundingClientRect(element: bigint) {
    const el = globalThis.__elementHandles.get(element);
    if (!el) {
      return { x: 0, y: 0, width: 0, height: 0 };
    }
    const rect = el.getBoundingClientRect();
    return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
  },
  setInnerHtml(self: bigint, html: string) {
    globalThis.__lookupElement(self).innerHTML = html;
  },
  getAttribute(self: bigint, name: string) {
    return globalThis.__lookupElement(self).getAttribute(name);
  },
  getTagName(self: bigint) {
    return globalThis.__lookupElement(self).tagName || '';
  },
  getClassList(self: bigint) {
    return self;
  },
  getClientHeight(self: bigint) {
    return globalThis.__lookupElement(self).clientHeight || 0;
  },
  getScrollHeight(self: bigint) {
    return globalThis.__lookupElement(self).scrollHeight || 0;
  },
  getScrollTop(self: bigint) {
    return globalThis.__lookupElement(self).scrollTop || 0;
  },
  setScrollTop(self: bigint, value: number) {
    globalThis.__lookupElement(self).scrollTop = value;
  },
};
