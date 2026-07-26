import * as assert from "assert";
import * as path from "path";
import {
  bundledServerPath,
  executableName,
  supportedTarget,
  versionsCompatible,
} from "../../src/platform";

suite("platform packaging", () => {
  test("maps every released platform to its VSIX target", () => {
    assert.strictEqual(supportedTarget("linux", "x64"), "linux-x64");
    assert.strictEqual(supportedTarget("linux", "arm64"), "linux-arm64");
    assert.strictEqual(supportedTarget("darwin", "x64"), "darwin-x64");
    assert.strictEqual(supportedTarget("darwin", "arm64"), "darwin-arm64");
    assert.strictEqual(supportedTarget("win32", "x64"), "win32-x64");
    assert.strictEqual(supportedTarget("win32", "arm64"), undefined);
  });

  test("selects the executable inside the platform directory", () => {
    assert.strictEqual(executableName("win32"), "ayame-spell.exe");
    assert.strictEqual(executableName("linux"), "ayame-spell");
    assert.strictEqual(
      bundledServerPath("/extension", "darwin", "arm64"),
      path.join("/extension", "server", "darwin-arm64", "ayame-spell"),
    );
  });

  test("requires matching major/minor extension and server versions", () => {
    assert.ok(versionsCompatible("0.2.0", "0.2.7"));
    assert.ok(versionsCompatible("v0.2.0", "0.2.0+build"));
    assert.ok(!versionsCompatible("0.2.0", "0.3.0"));
  });
});
