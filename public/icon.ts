import fs from "node:fs";
import path from "node:path";

// ── Constants ──────────────────────────────────────────────────────────────
const DIST_DIR = path.resolve("dist");
const ICON_FILE = path.join(DIST_DIR, "icon.ico");
const HTML_FILE = path.join(DIST_DIR, "index.html");

const FAVICON_LINK_RE = /<link[^>]*rel=["']icon["'][^>]*>/i;
const LINK_TEMPLATE = (base64: string) =>
  `<link rel="icon" href="data:image/x-icon;base64,${base64}" />`;

// ── Main ───────────────────────────────────────────────────────────────────
// Already inlined — nothing to do
if (!fs.existsSync(ICON_FILE)) {
  console.log("favicon already inlined, skipping");
  process.exit(0);
}

const iconBase64 = fs.readFileSync(ICON_FILE).toString("base64");
const inlineTag = LINK_TEMPLATE(iconBase64);

let html = fs.readFileSync(HTML_FILE, "utf8");
html = html.replace(FAVICON_LINK_RE, inlineTag);

fs.writeFileSync(HTML_FILE, html);
fs.rmSync(ICON_FILE);

console.log("favicon inlined");
