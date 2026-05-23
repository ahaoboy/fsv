import fs from "fs";

const icon = fs.readFileSync("./dist/icon.ico").toString("base64");

const iconTag =
  `<link rel="icon" href="data:image/x-icon;base64,${icon}" />`;

let html = fs.readFileSync("./dist/index.html", "utf8");

html = html.replace(
  /<link[^>]*rel=["']icon["'][^>]*>/i,
  iconTag
);

fs.writeFileSync("./dist/index.html", html);

console.log("favicon inlined");