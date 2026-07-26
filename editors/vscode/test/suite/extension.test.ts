import * as assert from "assert";
import * as vscode from "vscode";

const extensionId = "hjosugi.ayame-spell";
let extensionRoot: vscode.Uri;

async function waitUntil(
  predicate: () => boolean,
  message: string,
  timeout = 15000,
): Promise<void> {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    if (predicate()) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  assert.fail(message);
}

suite("ayame-spell extension", () => {
  suiteSetup(async () => {
    assert.ok(
      process.env.AYAME_SPELL_SERVER_PATH,
      "AYAME_SPELL_SERVER_PATH must point at the test server",
    );
    const extension = vscode.extensions.getExtension(extensionId);
    assert.ok(extension, `${extensionId} should be installed in the test host`);
    extensionRoot = vscode.Uri.file(extension.extensionPath);
    await extension.activate();
  });

  test("registers the complete command surface", async () => {
    const commands = await vscode.commands.getCommands(true);
    for (const command of [
      "ayame-spell.fixAll",
      "ayame-spell.reviewWords",
      "ayame-spell.addSelectionProject",
      "ayame-spell.addSelectionGlobal",
      "ayame-spell.openConfig",
      "ayame-spell.installDictionary",
      "ayame-spell.toggleMode",
      "ayame-spell.restart",
    ]) {
      assert.ok(commands.includes(command), `${command} should be registered`);
    }
    await vscode.commands.executeCommand("ayame-spell.reviewWords");
  });

  test("publishes diagnostics and applies safe fixes", async () => {
    const uri = vscode.Uri.joinPath(
      extensionRoot,
      "test-fixtures",
      "english.md",
    );
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document);
    await waitUntil(
      () =>
        vscode.languages
          .getDiagnostics(uri)
          .some((diagnostic) => diagnostic.source === "ayame-spell"),
      "expected an ayame-spell diagnostic for the English fixture",
    );

    await vscode.commands.executeCommand("ayame-spell.fixAll");
    await waitUntil(
      () => document.getText().includes("recommend the reliable checker"),
      "expected Fix All to apply the safe correction",
    );
  });

  test("reports Japanese notation inconsistency", async () => {
    const uri = vscode.Uri.joinPath(
      extensionRoot,
      "test-fixtures",
      "japanese.md",
    );
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document);
    await waitUntil(
      () =>
        vscode.languages
          .getDiagnostics(uri)
          .some(
            (diagnostic) =>
              diagnostic.source === "ayame-spell" &&
              String(
                typeof diagnostic.code === "object"
                  ? diagnostic.code.value
                  : diagnostic.code,
              ) === "ja-variant",
          ),
      "expected a Japanese variant diagnostic",
    );
  });
});
