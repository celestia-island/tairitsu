// @ts-nocheck

export const document_exports = {
  createElement(localName) {
    const el = document.createElement(localName);
    return globalThis.__storeElement(el);
  },
  createTextNode(data) {
    const text = document.createTextNode(data);
    return globalThis.__storeText(text);
  },
  getBody() {
    return globalThis.__storeElement(document.body);
  },
};
