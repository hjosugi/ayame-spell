import { readdir, readFile } from "node:fs/promises";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const site = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repo = path.resolve(site, "..");
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

function headingLevels(source) {
  const levels = [];
  let fence = null;
  for (const line of source.split(/\r?\n/)) {
    const fenceMatch = line.match(/^\s*(`{3,}|~{3,})/);
    if (fenceMatch) {
      const marker = fenceMatch[1][0];
      fence = fence === marker ? null : fence ?? marker;
      continue;
    }
    if (fence) continue;
    const heading = line.match(/^\s{0,3}(#{1,6})\s+\S/);
    if (heading) levels.push(heading[1].length);
  }
  return levels;
}

async function requireHeadingParity(label, englishFile, japaneseFile) {
  const englishLevels = headingLevels(await readFile(englishFile, "utf8"));
  const japaneseLevels = headingLevels(await readFile(japaneseFile, "utf8"));
  if (JSON.stringify(englishLevels) !== JSON.stringify(japaneseLevels)) {
    console.error(`${label} heading structure differs between EN and JA.`);
    console.error("EN:", englishLevels.join(", "));
    console.error("JA:", japaneseLevels.join(", "));
    process.exitCode = 1;
  }
}

await requireHeadingParity(
  "README",
  path.join(repo, "README.md"),
  path.join(repo, "README.ja.md"),
);
await requireHeadingParity(
  "DESIGN",
  path.join(repo, "DESIGN.md"),
  path.join(repo, "DESIGN.ja.md"),
);
for (const page of english) {
  await requireHeadingParity(
    `Docs page ${page}`,
    path.join(docs, page),
    path.join(ja, page),
  );
}

const requiredConfigKeys = [
  "[check]",
  "mode",
  "locale",
  "profile",
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
  "flag-compatibility",
  "kanji-consistency",
  "number-consistency",
  "punctuation-consistency",
  "[japanese.variants]",
  "[[overrides]]",
  "paths",
  "profile",
  "japanese",
];
const requiredIssueCodes = [
  "typo",
  "unknown-word",
  "en-variant",
  "ja-variant",
  "fullwidth-alnum",
  "halfwidth-kana",
  "fullwidth-space",
  "ja-compatibility",
  "ja-number-style",
  "ja-punctuation",
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

function git(...args) {
  return execFileSync("git", args, {
    cwd: repo,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "ignore"],
  }).trim();
}

function i18nSkipReason() {
  const candidates = [process.env.I18N_SKIP];
  if (process.env.GITHUB_EVENT_PATH) {
    try {
      const event = JSON.parse(
        readFileSync(process.env.GITHUB_EVENT_PATH, "utf8"),
      );
      candidates.push(
        event.pull_request?.body,
        event.head_commit?.message,
        ...(event.commits ?? []).map((commit) => commit.message),
      );
    } catch {
      // A malformed or unavailable event file must not disable parity checks.
    }
  }
  for (const candidate of candidates) {
    const match = candidate?.match(/\bi18n-skip:\s*(\S.*)/i);
    if (match) return match[1].trim();
  }
  return null;
}

function changedFiles(base) {
  const files = new Set();
  const add = (output) => {
    for (const file of output.split(/\r?\n/)) {
      if (file) files.add(file.split(path.sep).join("/"));
    }
  };

  if (base && !/^0+$/.test(base)) {
    try {
      add(git("diff", "--name-only", `${base}...HEAD`));
    } catch {
      console.error(`Unable to compare documentation changes with ${base}.`);
      process.exitCode = 1;
    }
  }
  add(git("diff", "--name-only"));
  add(git("ls-files", "--others", "--exclude-standard"));
  return files;
}

const base = process.env.I18N_BASE_SHA;
if (base) {
  const changed = changedFiles(base);
  const pairs = [
    ["README.md", "README.ja.md"],
    ["DESIGN.md", "DESIGN.ja.md"],
    ["CONTRIBUTING.md", "CONTRIBUTING.ja.md"],
    ...english.map((page) => [
      `site/src/content/docs/${page}`,
      `site/src/content/docs/ja/${page}`,
    ]),
  ];
  const unpaired = pairs.filter(
    ([englishFile, japaneseFile]) =>
      changed.has(englishFile) !== changed.has(japaneseFile),
  );
  const skip = i18nSkipReason();
  if (unpaired.length && !skip) {
    console.error("Documentation changed in only one language:");
    for (const [englishFile, japaneseFile] of unpaired) {
      console.error(`  ${englishFile} <-> ${japaneseFile}`);
    }
    console.error(
      "Update both files, or add `i18n-skip: <reason>` to the PR description.",
    );
    process.exitCode = 1;
  } else if (unpaired.length) {
    console.log(`i18n pairing skipped explicitly: ${skip}`);
  }
}

if (!process.exitCode) {
  console.log(
    `i18n parity OK: ${english.length} EN/JA page pairs, matching README/DESIGN/docs headings, ${englishAnchors.length} shared landing anchors, and complete config/rule references`,
  );
}
