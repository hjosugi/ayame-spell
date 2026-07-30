#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const executable = process.platform === "win32" ? "ayame-spell.exe" : "ayame-spell";
const binary = join(packageRoot, "vendor", executable);

if (!existsSync(binary)) {
  process.stderr.write(
    "ayame-spell binary is missing; reinstall the package with install scripts enabled\n",
  );
  process.exit(1);
}

const result = spawnSync(binary, process.argv.slice(2), {
  stdio: "inherit",
  windowsHide: false,
});
if (result.error) {
  throw result.error;
}
process.exit(result.status ?? 1);
