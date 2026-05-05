// @ts-nocheck
// Location interface - provides hash and viewport info for WIT browser world

export const location_exports = {
  getHash() { return window.location.hash || ''; },
  setHash(v: string) { window.location.hash = v; },
  getHref() { return window.location.href || ''; },
  setHref(v: string) { window.location.href = v; },
  getPathname() { return window.location.pathname || '/'; },
  setPathname(v: string) { window.location.pathname = v; },
  getOrigin() { return window.location.origin || ''; },
  getProtocol() { return window.location.protocol || ''; },
  setProtocol(v: string) { window.location.protocol = v; },
  getHost() { return window.location.host || ''; },
  setHost(v: string) { window.location.host = v; },
  getHostname() { return window.location.hostname || ''; },
  setHostname(v: string) { window.location.hostname = v; },
  getPort() { return window.location.port || ''; },
  setPort(v: string) { window.location.port = v; },
  getSearch() { return window.location.search || ''; },
  setSearch(v: string) { window.location.search = v; },
  assign(url: string) { window.location.assign(url); },
  replace(url: string) { window.location.replace(url); },
  reload() { window.location.reload(); },
  getAncestorOrigins() { return []; },
  getInnerWidth() { return window.innerWidth; }
};
