// @ts-nocheck
// Location interface - provides hash and viewport info for WIT browser world

export const location_exports = {
  getHash() {
    return window.location.hash || '';
  },
  getInnerWidth() {
    return window.innerWidth;
  },
  getPathname() {
    return window.location.pathname || '/';
  },
};
