// @ts-nocheck

export const element_exports = {
  setAttribute(self, qualifiedName, value) {
    const name = typeof qualifiedName === 'string' && qualifiedName.startsWith('r#')
      ? qualifiedName.slice(2)
      : qualifiedName;
    globalThis.__lookupElement(self).setAttribute(name, value);
  },
  removeAttribute(self, qualifiedName) {
    const name = typeof qualifiedName === 'string' && qualifiedName.startsWith('r#')
      ? qualifiedName.slice(2)
      : qualifiedName;
    globalThis.__lookupElement(self).removeAttribute(name);
  },
  getBoundingClientRect(element) {
    const el = globalThis.__elementHandles.get(element);
    if (!el) {
      return { x: 0, y: 0, width: 0, height: 0 };
    }
    const rect = el.getBoundingClientRect();
    return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
  },
  setInnerHtml(self, html) {
    globalThis.__lookupElement(self).innerHTML = html;
  },
  getAttribute(self, name) {
    return globalThis.__lookupElement(self).getAttribute(name);
  },
  getTagName(self) {
    return globalThis.__lookupElement(self).tagName || '';
  },
  getClassList(self) {
    return self;
  },
  getClientHeight(self) {
    return globalThis.__lookupElement(self).clientHeight || 0;
  },
  getScrollHeight(self) {
    return globalThis.__lookupElement(self).scrollHeight || 0;
  },
  getScrollTop(self) {
    return globalThis.__lookupElement(self).scrollTop || 0;
  },
  setScrollTop(self, value) {
    globalThis.__lookupElement(self).scrollTop = value;
  },
};
