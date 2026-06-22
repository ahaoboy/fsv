/** Parse the current path from the URL hash (e.g. "#/abc/folder" → "abc/folder"). */
export function pathFromHash(): string {
  const raw = window.location.hash.slice(1); // remove leading #
  if (!raw.startsWith("/")) return "";
  // Decode percent-encoded characters (e.g. %E5%AE%88 → 守)
  return decodeURIComponent(raw.slice(1)); // remove leading /
}

/** Set the URL hash to reflect the given path. */
export function setHashPath(path: string): void {
  const newHash = path ? `#/${path}` : "";
  if (window.location.hash !== newHash) {
    window.history.replaceState(null, "", newHash || window.location.pathname);
  }
}
