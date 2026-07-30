import assert from "node:assert/strict";
import test from "node:test";

import { expectedChecksum, releaseTarget } from "./install.js";

test("maps every released Node platform", () => {
  assert.deepEqual(releaseTarget("linux", "x64"), [
    "x86_64-unknown-linux-gnu",
    "tar.gz",
  ]);
  assert.deepEqual(releaseTarget("linux", "arm64"), [
    "aarch64-unknown-linux-gnu",
    "tar.gz",
  ]);
  assert.deepEqual(releaseTarget("darwin", "arm64"), [
    "aarch64-apple-darwin",
    "tar.gz",
  ]);
  assert.deepEqual(releaseTarget("win32", "x64"), [
    "x86_64-pc-windows-msvc",
    "zip",
  ]);
  assert.throws(() => releaseTarget("win32", "arm64"), /unsupported platform/u);
});

test("selects the checksum for the exact asset", () => {
  const hash = "a".repeat(64);
  assert.equal(
    expectedChecksum(`${"b".repeat(64)}  other.zip\n${hash} *wanted.zip\n`, "wanted.zip"),
    hash,
  );
  assert.throws(() => expectedChecksum(`${hash}  other.zip\n`, "wanted.zip"));
});
