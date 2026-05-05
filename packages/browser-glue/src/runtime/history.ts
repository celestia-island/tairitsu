// @ts-nocheck
// History interface - provides browser history API for WIT browser world

export const history_exports = {
  getLength() {
    return window.history.length;
  },
  getScrollRestoration() {
    const value = window.history.scrollRestoration;
    switch (value) {
      case 'auto': return 0n;
      case 'manual': return 1n;
      default: return 0n;
    }
  },
  setScrollRestoration(value) {
    let enumValue;
    if (value === 0n) { enumValue = 'auto'; }
    else if (value === 1n) { enumValue = 'manual'; }
    else { enumValue = 'auto'; }
    window.history.scrollRestoration = enumValue;
  },
  getState() {
    return window.history.state || null;
  },
  go(delta) {
    window.history.go(delta);
  },
  back() {
    window.history.back();
  },
  forward() {
    window.history.forward();
  },
  pushState(data, unused, url) {
    window.history.pushState(data, unused, url);
  },
  replaceState(data, unused, url) {
    window.history.replaceState(data, unused, url);
  },
};
