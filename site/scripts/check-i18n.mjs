import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const site = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const docs = path.join(site, "src", "content", "docs");
const ja = path.join(docs, "ja");

async function pages(root, relative = "") {
  const found = [];
  for (const entry of await readdir(path.join(root, relative), {
    withFileTypes: true,
  })) {
    const child = path.join(relative, entry.name);
    if (entry.isDirectory()) {
      found.push(...(await pages(root, child)));
    } else if (entry.isFile() && /\.(md|mdx)$/.test(entry.name)) {
      found.push(child);
    }
  }
  return found.sort();
}

const english = (await pages(docs)).filter(
  (page) => page !== "ja" && !page.startsWith(`ja${path.sep}`),
);
const japanese = await pages(ja);
const missingInJapanese = english.filter((page) => !japanese.includes(page));
const missingInEnglish = japanese.filter((page) => !english.includes(page));

if (missingInJapanese.length || missingInEnglish.length) {
  if (missingInJapanese.length) {
    console.error("Missing Japanese pages:", missingInJapanese.join(", "));
  }
  if (missingInEnglish.length) {
    console.error("Missing English pages:", missingInEnglish.join(", "));
  }
  process.exitCode = 1;
}

const idPattern = /\sid=["']([^"']+)["']/g;
const anchors = (source) =>
  [...source.matchAll(idPattern)].map((match) => match[1]).sort();
const englishHome = await readFile(path.join(docs, "index.mdx"), "utf8");
const japaneseHome = await readFile(path.join(ja, "index.mdx"), "utf8");
const englishAnchors = anchors(englishHome);
const japaneseAnchors = anchors(japaneseHome);

if (JSON.stringify(englishAnchors) !== JSON.stringify(japaneseAnchors)) {
  console.error("Landing-page section anchors differ between EN and JA.");
  console.error("EN:", englishAnchors);
  console.error("JA:", japaneseAnchors);
  process.exitCode = 1;
}

const requiredConfigKeys = [
  "[check]",
  "mode",
  "min-word-len",
  "max-token-len",
  "[files]",
  "exclude",
  "include-hidden",
  "max-file-size",
  "[words]",
  "project",
  "ignore",
  "dictionaries",
  "[corrections]",
  "builtin",
  "extra",
  "[corrections.words]",
  "[japanese]",
  "enabled",
  "katakana-style",
  "variant-files",
  "flag-fullwidth-alnum",
  "flag-halfwidth-kana",
  "fullwidth-space",
  "[japanese.variants]",
  "[[overrides]]",
  "paths",
  "japanese",
];
const requiredIssueCodes = [
  "typo",
  "unknown-word",
  "ja-variant",
  "fullwidth-alnum",
  "halfwidth-kana",
  "fullwidth-space",
];

async function requireTokens(relative, tokens) {
  for (const localeRoot of [docs, ja]) {
    const file = path.join(localeRoot, relative);
    const source = await readFile(file, "utf8");
    const missing = tokens.filter((token) => !source.includes(`\`${token}\``));
    if (missing.length) {
      console.error(
        `${path.relative(site, file)} is missing reference tokens: ${missing.join(", ")}`,
      );
      process.exitCode = 1;
    }
  }
}

await requireTokens(
  path.join("reference", "configuration.md"),
  requiredConfigKeys,
);
await requireTokens(path.join("reference", "rules.md"), requiredIssueCodes);

if (!process.exitCode) {
  console.log(
    `i18n parity OK: ${english.length} EN/JA page pairs, ${englishAnchors.length} shared landing anchors, and complete config/rule references`,
  );
}
