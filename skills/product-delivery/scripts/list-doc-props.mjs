#!/usr/bin/env node
/**
 * List Markdown docs by YAML frontmatter (or leading "- key: value" props).
 * No deps. Used by product-delivery instead of hand-maintained README indexes.
 *
 * Usage:
 *   node list-doc-props.mjs <dir> [dir...] [--status VALUE] [--glob GLOB] [--json]
 */
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const KEY_ALIASES = {
  状态: "status",
  摘要: "summary",
  说明: "summary",
};

function usage(code = 0) {
  const msg = `Usage: list-doc-props.mjs <dir> [dir...] [--status VALUE] [--glob PATTERN] [--json]
  --status   filter by status (case-insensitive; also matches 状态)
  --glob     only basenames/paths matching glob (* and **; default **/*.md)
  --json     machine-readable output
  Skips README.md and INDEX.md (convention docs, not catalog entries).`;
  console[code ? "error" : "log"](msg);
  process.exit(code);
}

function parseArgs(argv) {
  const dirs = [];
  let status = null;
  let glob = "**/*.md";
  let json = false;
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--help" || a === "-h") usage(0);
    if (a === "--json") {
      json = true;
      continue;
    }
    if (a === "--status") {
      status = argv[++i];
      if (!status) usage(1);
      continue;
    }
    if (a === "--glob") {
      glob = argv[++i];
      if (!glob) usage(1);
      continue;
    }
    if (a.startsWith("-")) usage(1);
    dirs.push(a);
  }
  if (dirs.length === 0) usage(1);
  return { dirs, status, glob, json };
}

function globToRegExp(pattern) {
  const escaped = pattern
    .replace(/[.+^${}()|[\]\\]/g, "\\$&")
    .replace(/\*\*/g, "\0")
    .replace(/\*/g, "[^/]*")
    .replace(/\0/g, ".*");
  return new RegExp(`^${escaped}$`);
}

function walk(dir, out = []) {
  let entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return out;
  }
  for (const ent of entries) {
    const full = path.join(dir, ent.name);
    if (ent.isDirectory()) {
      if (ent.name === "node_modules" || ent.name === ".git") continue;
      walk(full, out);
    } else if (ent.isFile() && ent.name.endsWith(".md")) {
      out.push(full);
    }
  }
  return out;
}

function normalizeKey(key) {
  const k = key.trim();
  return KEY_ALIASES[k] ?? k;
}

function parseFrontmatter(text) {
  if (!text.startsWith("---")) return null;
  const end = text.indexOf("\n---", 3);
  if (end === -1) return null;
  const block = text.slice(3, end).replace(/^\r?\n/, "");
  const props = {};
  for (const line of block.split(/\r?\n/)) {
    const m = line.match(/^([A-Za-z0-9_\u4e00-\u9fff-]+)\s*:\s*(.*)$/);
    if (!m) continue;
    let v = m[2].trim();
    if (
      (v.startsWith('"') && v.endsWith('"')) ||
      (v.startsWith("'") && v.endsWith("'"))
    ) {
      v = v.slice(1, -1);
    }
    props[normalizeKey(m[1])] = v;
  }
  return { props, body: text.slice(end + 4).replace(/^\r?\n/, "") };
}

/** Fallback: leading "- key: value" / "- **key**：value" until blank or heading. */
function parseBulletProps(text) {
  const props = {};
  const lines = text.split(/\r?\n/);
  let i = 0;
  if (lines[0]?.startsWith("#")) i = 1;
  for (; i < lines.length; i++) {
    const line = lines[i];
    if (line.trim() === "") {
      if (Object.keys(props).length) break;
      continue;
    }
    if (line.startsWith("#")) break;
    const m = line.match(
      /^-\s+\*{0,2}([A-Za-z0-9_\u4e00-\u9fff-]+)\*{0,2}\s*[:：]\s*(.+)$/,
    );
    if (!m) {
      if (Object.keys(props).length) break;
      continue;
    }
    let v = m[2].trim().replace(/^\*{0,2}/, "").replace(/\*{0,2}$/, "");
    props[normalizeKey(m[1])] = v;
  }
  return props;
}

function firstHeading(text) {
  const m = text.match(/^#\s+(.+)$/m);
  return m ? m[1].trim() : "";
}

function rel(from, file) {
  return path.relative(from, file).split(path.sep).join("/");
}

/**
 * Leading NNNN from basename (issues/ADR) or first path segment like `0001-slug/` (changes).
 * Missing → +Infinity (unnumbered after numbered).
 */
function leadingNum(filePath) {
  const posix = filePath.split(path.sep).join("/");
  const base = path.basename(posix);
  const fromBase = base.match(/^(\d+)/);
  if (fromBase) return Number(fromBase[1]);
  const fromSeg = posix.match(/(?:^|\/)(\d+)-[^/]+\//);
  if (fromSeg) return Number(fromSeg[1]);
  return Number.POSITIVE_INFINITY;
}

function main() {
  const { dirs, status, glob, json } = parseArgs(process.argv.slice(2));
  const globRe = globToRegExp(glob.replace(/\\/g, "/"));
  const cwd = process.cwd();
  const rows = [];

  for (const dirArg of dirs) {
    const root = path.resolve(cwd, dirArg);
    if (!fs.existsSync(root)) {
      console.error(`not found: ${dirArg}`);
      process.exit(1);
    }
    const files = walk(root);
    for (const file of files) {
      const base = path.basename(file);
      if (base === "README.md" || base === "INDEX.md") continue;
      const relPosix = rel(cwd, file);
      const fromRoot = rel(root, file);
      if (
        !globRe.test(relPosix) &&
        !globRe.test(fromRoot) &&
        !globRe.test(base)
      ) {
        continue;
      }
      const text = fs.readFileSync(file, "utf8");
      const fm = parseFrontmatter(text);
      const props = fm?.props ?? parseBulletProps(text);
      const body = fm?.body ?? text;
      const summary =
        props.summary || firstHeading(body) || firstHeading(text);
      const row = {
        ...props,
        path: relPosix,
        status: props.status ?? "",
        summary,
      };
      if (status && row.status.toLowerCase() !== status.toLowerCase()) continue;
      rows.push(row);
    }
  }

  rows.sort((a, b) => {
    const na = leadingNum(a.path);
    const nb = leadingNum(b.path);
    if (na !== nb) return na - nb;
    return a.path.localeCompare(b.path);
  });

  if (json) {
    console.log(JSON.stringify(rows, null, 2));
    return;
  }
  if (rows.length === 0) {
    console.log("(no matches)");
    return;
  }
  const wPath = Math.max(4, ...rows.map((r) => r.path.length));
  const wStatus = Math.max(6, ...rows.map((r) => (r.status || "-").length));
  for (const r of rows) {
    const st = (r.status || "-").padEnd(wStatus);
    console.log(`${r.path.padEnd(wPath)}  ${st}  ${r.summary}`);
  }
}

main();
