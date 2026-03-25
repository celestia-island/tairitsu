// @ts-nocheck
import { storeElement, storeText } from "./helpers";

export const document_exports = {
  createElement(localName) {
    const el = document.createElement(localName);
    return storeElement(el);
  },
  createTextNode(data) {
    const text = document.createTextNode(data);
    return storeText(text);
  },
  getBody() {
    return storeElement(document.body);
  },
};
