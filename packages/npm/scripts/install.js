#!/usr/bin/env node

import { createHash } from "node:crypto";
import {
  chmodSync,
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { get } from "node:https";
import { tmpdir } from "node:os";
import { basename, dirname, join } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const packageRoot = join(dirname(fileURLToPath(import.meta.url)), "..");

export function releaseTarget(platform = process.platform, arch = process.arch) {
  const targets = new Map([
    ["linux:x64", ["x86_64-unknown-linux-gnu", "tar.gz"]],
    ["linux:arm64", ["aarch64-unknown-linux-gnu", "tar.gz"]],
    ["darwin:x64", ["x86_64-apple-darwin", "tar.gz"]],
    ["darwin:arm64", ["aarch64-apple-darwin", "tar.gz"]],
    ["win32:x64", ["x86_64-pc-windows-msvc", "zip"]],
  ]);
  const target = targets.get(`${platform}:${arch}`);
  if (!target) {
    throw new Error(`unsupported platform: ${platform}/${arch}`);
  }
  return target;
}

export function expectedChecksum(sums, asset) {
  for (const line of sums.split(/\r?\n/u)) {
    const match = line.match(/^([0-9a-fA-F]{64})\s+\*?(.+)$/u);
    if (match && match[2] === asset) {
      return match[1].toLowerCase();
    }
  }
  throw new Error(`SHA256SUMS does not contain ${asset}`);
}

function download(url, redirects = 0) {
  return new Promise((resolve, reject) => {
    get(url, { headers: { "User-Agent": "ayame-spell-npm" } }, (response) => {
      if (
        response.statusCode &&
        response.statusCode >= 300 &&
        response.statusCode < 400 &&
        response.headers.location
      ) {
        response.resume();
        if (redirects >= 5) {
          reject(new Error(`too many redirects while downloading ${url}`));
        } else {
          resolve(download(new URL(response.headers.location, url), redirects + 1));
        }
        return;
      }
      if (response.statusCode !== 200) {
        response.resume();
        reject(new Error(`download failed (${response.statusCode}): ${url}`));
        return;
      }
      const chunks = [];
      response.on("data", (chunk) => chunks.push(chunk));
      response.on("end", () => resolve(Buffer.concat(chunks)));
      response.on("error", reject);
    }).on("error", reject);
  });
}

async function install() {
  if (process.env.AYAME_SPELL_SKIP_DOWNLOAD === "1") {
    return;
  }
  const packageJson = JSON.parse(
    readFileSync(join(packageRoot, "package.json"), "utf8"),
  );
  const tag = `v${packageJson.version}`;
  const [target, extension] = releaseTarget();
  const asset = `ayame-spell-${tag}-${target}.${extension}`;
  const base = `https://github.com/ayame-editor/ayame-spell/releases/download/${tag}`;
  const temporary = mkdtempSync(join(tmpdir(), "ayame-spell-npm-"));
  try {
    const [archive, sums] = await Promise.all([
      download(`${base}/${asset}`),
      download(`${base}/SHA256SUMS`),
    ]);
    const expected = expectedChecksum(sums.toString("utf8"), asset);
    const actual = createHash("sha256").update(archive).digest("hex");
    if (actual !== expected) {
      throw new Error(`checksum mismatch for ${asset}`);
    }

    const archivePath = join(temporary, basename(asset));
    writeFileSync(archivePath, archive);
    const extracted = spawnSync("tar", ["-xf", archivePath, "-C", temporary], {
      stdio: "inherit",
    });
    if (extracted.status !== 0) {
      throw new Error("could not extract the ayame-spell release archive");
    }

    const executable = process.platform === "win32" ? "ayame-spell.exe" : "ayame-spell";
    const source = join(temporary, `ayame-spell-${tag}-${target}`, executable);
    const vendor = join(packageRoot, "vendor");
    mkdirSync(vendor, { recursive: true });
    const destination = join(vendor, executable);
    copyFileSync(source, destination);
    if (process.platform !== "win32") {
      chmodSync(destination, 0o755);
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  install().catch((error) => {
    process.stderr.write(`ayame-spell install failed: ${error.message}\n`);
    process.exit(1);
  });
}
