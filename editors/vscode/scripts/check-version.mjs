import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import process from "node:process";
import { fileURLToPath, URL } from "node:url";

const repositoryRoot = fileURLToPath(new URL("../../../", import.meta.url));
const extensionPackage = JSON.parse(
  readFileSync(new URL("../package.json", import.meta.url), "utf8"),
);
const cargoMetadata = JSON.parse(
  execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
    cwd: repositoryRoot,
    encoding: "utf8",
  }),
);
const cliPackage = cargoMetadata.packages.find(
  (candidate) => candidate.name === "ayame-spell",
);

if (!cliPackage) {
  throw new Error("cargo metadata did not contain the ayame-spell package");
}

if (extensionPackage.version !== cliPackage.version) {
  throw new Error(
    `Version mismatch: VS Code ${extensionPackage.version}, CLI ${cliPackage.version}`,
  );
}

process.stdout.write(
  `VS Code and CLI versions match (${extensionPackage.version})\n`,
);
