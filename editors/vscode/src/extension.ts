import { execFile } from "child_process";
import * as fs from "fs";
import * as vscode from "vscode";
import {
  ExecuteCommandRequest,
  LanguageClient,
  LanguageClientOptions,
  RevealOutputChannelOn,
  ServerOptions,
} from "vscode-languageclient/node";
import {
  bundledServerPath,
  supportedTarget,
  versionsCompatible,
} from "./platform";

const CONFIG_SECTION = "ayame-spell";
const REGISTRY_URL =
  "https://ayame-editor.github.io/ayame-spell/registry/index.json";
const CONFIG_TEMPLATE = `# ayame-spell configuration
# Reference: https://github.com/ayame-editor/ayame-spell#configuration

[check]
mode = "corrections"

[words]
project = "ayame-words.txt"
ignore = []
dictionaries = []

[japanese]
enabled = true
katakana-style = "consistency"
`;

type Mode = "inherit" | "corrections" | "dictionary" | "off";

let extensionContext: vscode.ExtensionContext;
let client: LanguageClient | undefined;
let clientWatcher: vscode.FileSystemWatcher | undefined;
let statusItem: vscode.StatusBarItem;
let output: vscode.LogOutputChannel;
let restartQueue = Promise.resolve();
let serverVersion: string | undefined;
let startError: string | undefined;

function configuration(): vscode.WorkspaceConfiguration {
  const resource =
    vscode.window.activeTextEditor?.document.uri ??
    vscode.workspace.workspaceFolders?.[0]?.uri;
  return vscode.workspace.getConfiguration(CONFIG_SECTION, resource ?? null);
}

function configuredMode(): Mode {
  return configuration().get<Mode>("mode", "inherit");
}

function effectiveModeLabel(): string {
  const mode = configuredMode();
  return mode === "inherit" ? "config" : mode;
}

function resolveServer(): string {
  const configured = configuration().get<string>("serverPath", "").trim();
  if (configured) {
    return configured;
  }
  const environment = process.env.AYAME_SPELL_SERVER_PATH?.trim();
  if (environment) {
    return environment;
  }
  const bundled = bundledServerPath(extensionContext.extensionPath);
  if (bundled && fs.existsSync(bundled)) {
    return bundled;
  }
  return process.platform === "win32" ? "ayame-spell.exe" : "ayame-spell";
}

function initializationOptions(): Record<string, unknown> {
  const mode = configuredMode();
  const japanese = configuration().inspect<boolean>("japanese.enabled");
  const japaneseOverride =
    japanese?.workspaceFolderValue ??
    japanese?.workspaceValue ??
    japanese?.globalValue;
  return {
    mode: mode === "inherit" ? undefined : mode,
    japaneseEnabled: japaneseOverride,
    diagnosticSeverity: configuration().get<string>(
      "diagnosticSeverity",
      "warning",
    ),
    debounceMs: configuration().get<number>("debounceMs", 150),
    locale: vscode.env.language,
  };
}

async function startClient(): Promise<void> {
  startError = undefined;
  serverVersion = undefined;
  if (!configuration().get<boolean>("enable", true)) {
    output.appendLine("Language server is disabled by ayame-spell.enable.");
    updateStatus();
    return;
  }

  const serverPath = resolveServer();
  const target = supportedTarget();
  output.appendLine(
    `Starting ${serverPath} lsp (${target ?? `${process.platform}-${process.arch}`})`,
  );
  const serverOptions: ServerOptions = {
    command: serverPath,
    args: ["lsp"],
  };
  clientWatcher = vscode.workspace.createFileSystemWatcher(
    "**/{ayame-spell.toml,.ayame-spell.toml,ayame-words.txt}",
  );
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file" }, { scheme: "untitled" }],
    initializationOptions: initializationOptions(),
    outputChannel: output,
    revealOutputChannelOn: RevealOutputChannelOn.Never,
    synchronize: { fileEvents: clientWatcher },
  };
  const nextClient = new LanguageClient(
    "ayame-spell",
    "ayame-spell",
    serverOptions,
    clientOptions,
  );
  client = nextClient;

  try {
    await nextClient.start();
    serverVersion = nextClient.initializeResult?.serverInfo?.version;
    output.appendLine(
      `Server ready${serverVersion ? ` (version ${serverVersion})` : ""}.`,
    );
    const extensionVersion = String(
      extensionContext.extension.packageJSON.version,
    );
    if (serverVersion && !versionsCompatible(extensionVersion, serverVersion)) {
      const message =
        `ayame-spell extension ${extensionVersion} is using server ${serverVersion}. ` +
        "Matching major/minor versions are recommended.";
      output.appendLine(`Warning: ${message}`);
      void vscode.window
        .showWarningMessage(message, "Show output")
        .then((action) => {
          if (action) {
            output.show(true);
          }
        });
    }
  } catch (error) {
    if (client === nextClient) {
      client = undefined;
    }
    startError = error instanceof Error ? error.message : String(error);
    clientWatcher?.dispose();
    clientWatcher = undefined;
    output.appendLine(`Start failed: ${startError}`);
    void vscode.window
      .showErrorMessage(
        `ayame-spell could not start "${serverPath}". The platform package may be missing, or ayame-spell.serverPath is invalid.`,
        "Open settings",
        "Install help",
        "Show output",
      )
      .then(async (choice) => {
        if (choice === "Open settings") {
          await vscode.commands.executeCommand(
            "workbench.action.openSettings",
            "ayame-spell.serverPath",
          );
        } else if (choice === "Install help") {
          await vscode.env.openExternal(
            vscode.Uri.parse(
              "https://github.com/ayame-editor/ayame-spell/tree/main/editors/vscode#installation",
            ),
          );
        } else if (choice === "Show output") {
          output.show(true);
        }
      });
  } finally {
    updateStatus();
  }
}

async function stopClient(): Promise<void> {
  const previous = client;
  client = undefined;
  serverVersion = undefined;
  if (previous) {
    try {
      await previous.stop();
    } catch (error) {
      output.appendLine(`Stop warning: ${String(error)}`);
    }
  }
  clientWatcher?.dispose();
  clientWatcher = undefined;
}

function restartClient(): Promise<void> {
  restartQueue = restartQueue
    .then(stopClient)
    .then(startClient)
    .catch((error) => {
      output.appendLine(`Restart failed: ${String(error)}`);
    });
  return restartQueue;
}

export async function activate(
  context: vscode.ExtensionContext,
): Promise<void> {
  extensionContext = context;
  output = vscode.window.createOutputChannel("ayame-spell", { log: true });
  statusItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    90,
  );
  statusItem.name = "ayame-spell";
  statusItem.command = "ayame-spell.toggleMode";

  context.subscriptions.push(
    output,
    statusItem,
    vscode.languages.onDidChangeDiagnostics(updateStatus),
    vscode.window.onDidChangeActiveTextEditor(updateStatus),
    vscode.workspace.onWillSaveTextDocument((event) => {
      if (
        configuration().get<boolean>("fixOnSave", false) &&
        client &&
        ourDiagnostics(event.document.uri).length > 0
      ) {
        event.waitUntil(fixAllForUri(event.document.uri).then(() => []));
      }
    }),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration(CONFIG_SECTION)) {
        updateStatus();
      }
      if (
        [
          "enable",
          "serverPath",
          "mode",
          "japanese.enabled",
          "diagnosticSeverity",
          "debounceMs",
          "trace.server",
        ].some((setting) =>
          event.affectsConfiguration(`${CONFIG_SECTION}.${setting}`),
        )
      ) {
        void restartClient();
      }
    }),
    vscode.commands.registerCommand("ayame-spell.fixAll", fixAll),
    vscode.commands.registerCommand("ayame-spell.reviewWords", reviewWords),
    vscode.commands.registerCommand("ayame-spell.restart", restartClient),
    vscode.commands.registerCommand("ayame-spell.toggleMode", toggleMode),
    vscode.commands.registerCommand("ayame-spell.addSelectionProject", () =>
      addSelection("project"),
    ),
    vscode.commands.registerCommand("ayame-spell.addSelectionGlobal", () =>
      addSelection("global"),
    ),
    vscode.commands.registerCommand("ayame-spell.openConfig", openConfig),
    vscode.commands.registerCommand(
      "ayame-spell.installDictionary",
      installDictionary,
    ),
  );

  updateStatus();
  await startClient();
}

function ourDiagnostics(uri: vscode.Uri): vscode.Diagnostic[] {
  return vscode.languages
    .getDiagnostics(uri)
    .filter((diagnostic) => diagnostic.source === "ayame-spell");
}

function updateStatus(): void {
  if (!statusItem) {
    return;
  }
  const enabled = configuration().get<boolean>("enable", true);
  const editor = vscode.window.activeTextEditor;
  const count = editor ? ourDiagnostics(editor.document.uri).length : 0;
  const mode = effectiveModeLabel();

  if (!enabled) {
    statusItem.text = "$(circle-slash) ayame off";
    statusItem.tooltip =
      "ayame-spell is disabled. Click to change checking mode.";
  } else if (startError) {
    statusItem.text = "$(error) ayame";
    statusItem.tooltip = `ayame-spell failed to start: ${startError}`;
  } else if (!client) {
    statusItem.text = "$(loading~spin) ayame";
    statusItem.tooltip = "ayame-spell is starting.";
  } else {
    statusItem.text =
      count > 0 ? `$(book) ayame ${mode} ${count}` : `$(book) ayame ${mode}`;
    statusItem.tooltip = [
      `Mode: ${mode}`,
      count > 0
        ? `${count} finding(s) in this file`
        : "No findings in this file",
      serverVersion ? `Server: ${serverVersion}` : undefined,
      "Click to cycle corrections → dictionary → off.",
    ]
      .filter(Boolean)
      .join("\n");
  }
  statusItem.show();
}

async function toggleMode(): Promise<void> {
  const current = configuredMode();
  const next: Mode =
    current === "dictionary"
      ? "off"
      : current === "off"
        ? "corrections"
        : "dictionary";
  await configuration().update(
    "mode",
    next,
    vscode.ConfigurationTarget.Workspace,
  );
  void vscode.window.showInformationMessage(`ayame-spell mode: ${next}`);
}

async function fixAll(): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (editor) {
    await fixAllForUri(editor.document.uri);
  }
}

async function fixAllForUri(uri: vscode.Uri): Promise<void> {
  if (!client) {
    return;
  }
  await client.sendRequest(ExecuteCommandRequest.type, {
    command: "ayame-spell.server.fixAll",
    arguments: [{ uri: uri.toString() }],
  });
}

function diagnosticCode(diagnostic: vscode.Diagnostic): string {
  if (typeof diagnostic.code === "string") {
    return diagnostic.code;
  }
  if (
    diagnostic.code &&
    typeof diagnostic.code === "object" &&
    "value" in diagnostic.code
  ) {
    return String(diagnostic.code.value);
  }
  return "";
}

function selectedWords(): string[] {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    return [];
  }
  const words = new Set<string>();
  for (const selection of editor.selections) {
    let text: string;
    if (selection.isEmpty) {
      const range = editor.document.getWordRangeAtPosition(selection.active);
      if (!range) {
        continue;
      }
      text = editor.document.getText(range);
    } else {
      text = editor.document.getText(selection);
    }
    for (const word of text.match(/[\p{L}\p{N}_'-]+/gu) ?? []) {
      words.add(word);
    }
  }
  return [...words];
}

function requireWorkspaceTrust(): boolean {
  if (vscode.workspace.isTrusted) {
    return true;
  }
  void vscode.window.showWarningMessage(
    "ayame-spell: trust this workspace before changing configuration or dictionaries.",
  );
  return false;
}

async function addSelection(scope: "project" | "global"): Promise<void> {
  if (!client || !requireWorkspaceTrust()) {
    return;
  }
  const words = selectedWords();
  if (words.length === 0) {
    void vscode.window.showInformationMessage(
      "ayame-spell: select one or more words first.",
    );
    return;
  }
  await sendWordCommand("ayame-spell.addWords", words, scope);
}

async function sendWordCommand(
  command: string,
  words: string[],
  scope?: string,
): Promise<void> {
  if (!client || !requireWorkspaceTrust()) {
    return;
  }
  await client.sendRequest(ExecuteCommandRequest.type, {
    command,
    arguments: [{ words, scope }],
  });
  void vscode.window.showInformationMessage(
    `ayame-spell: updated ${words.length} word(s).`,
  );
}

async function reviewWords(): Promise<void> {
  if (!client || !requireWorkspaceTrust()) {
    return;
  }
  const counts = new Map<string, { count: number; kind: string }>();
  for (const [, diagnostics] of vscode.languages.getDiagnostics()) {
    for (const diagnostic of diagnostics) {
      if (diagnostic.source !== "ayame-spell") {
        continue;
      }
      const kind = diagnosticCode(diagnostic);
      if (!["typo", "unknown-word", "ja-variant"].includes(kind)) {
        continue;
      }
      const match = /`([^`]+)`/.exec(diagnostic.message);
      if (!match) {
        continue;
      }
      const entry = counts.get(match[1]) ?? { count: 0, kind };
      entry.count += 1;
      counts.set(match[1], entry);
    }
  }
  if (counts.size === 0) {
    void vscode.window.showInformationMessage(
      "ayame-spell: no flagged words in open files.",
    );
    return;
  }

  const items = [...counts.entries()]
    .sort((a, b) => b[1].count - a[1].count)
    .map(([word, entry]) => ({
      label: word,
      description: `${entry.count}× ${entry.kind}`,
    }));
  const picked = await vscode.window.showQuickPick(items, {
    canPickMany: true,
    title: "ayame-spell: select words to handle in bulk",
    placeHolder: "Space to select, Enter to confirm",
  });
  if (!picked?.length) {
    return;
  }

  type Action = vscode.QuickPickItem & { command: string; scope?: string };
  const action = await vscode.window.showQuickPick<Action>(
    [
      {
        label: "$(add) Add to project words",
        description: "ayame-words.txt (committed and shared)",
        command: "ayame-spell.addWords",
        scope: "project",
      },
      {
        label: "$(globe) Add to global words",
        description: "Personal words for all projects",
        command: "ayame-spell.addWords",
        scope: "global",
      },
      {
        label: "$(mute) Ignore in this project",
        description: "[words].ignore in ayame-spell.toml",
        command: "ayame-spell.ignoreWords",
      },
    ],
    { title: `Apply to ${picked.length} word(s)` },
  );
  if (action) {
    await sendWordCommand(
      action.command,
      picked.map((item) => item.label),
      action.scope,
    );
  }
}

function workspaceRoot(): vscode.Uri | undefined {
  return vscode.workspace.workspaceFolders?.[0]?.uri;
}

async function openConfig(): Promise<void> {
  if (!requireWorkspaceTrust()) {
    return;
  }
  const root = workspaceRoot();
  if (!root) {
    void vscode.window.showErrorMessage(
      "ayame-spell: open a workspace folder first.",
    );
    return;
  }
  const candidates = ["ayame-spell.toml", ".ayame-spell.toml"].map((name) =>
    vscode.Uri.joinPath(root, name),
  );
  let target = candidates[0];
  for (const candidate of candidates) {
    try {
      await vscode.workspace.fs.stat(candidate);
      target = candidate;
      break;
    } catch {
      // Continue to the next supported filename.
    }
  }
  try {
    await vscode.workspace.fs.stat(target);
  } catch {
    await vscode.workspace.fs.writeFile(target, Buffer.from(CONFIG_TEMPLATE));
    output.appendLine(`Created ${target.fsPath}`);
  }
  const document = await vscode.workspace.openTextDocument(target);
  await vscode.window.showTextDocument(document);
}

interface RegistryEntry extends vscode.QuickPickItem {
  name: string;
}

async function installDictionary(): Promise<void> {
  if (!requireWorkspaceTrust()) {
    return;
  }
  const root = workspaceRoot();
  if (!root) {
    void vscode.window.showErrorMessage(
      "ayame-spell: open a workspace folder first.",
    );
    return;
  }
  try {
    const response = await fetch(REGISTRY_URL);
    if (!response.ok) {
      throw new Error(`${response.status} ${response.statusText}`);
    }
    const registry = (await response.json()) as {
      dictionaries: Array<{
        name: string;
        language: string;
        kind: string;
        description: string;
        entries: number;
      }>;
    };
    const picked = await vscode.window.showQuickPick<RegistryEntry>(
      registry.dictionaries.map((entry) => ({
        name: entry.name,
        label: entry.name,
        description: `${entry.language} · ${entry.kind} · ${entry.entries.toLocaleString()} entries`,
        detail: entry.description,
      })),
      {
        canPickMany: true,
        title: "ayame-spell: install shared dictionaries",
        placeHolder: "Select dictionaries to download and add to this project",
      },
    );
    if (!picked?.length) {
      return;
    }
    await runCli(
      ["dict", "add", ...picked.map((entry) => entry.name)],
      root.fsPath,
    );
    void vscode.window.showInformationMessage(
      `ayame-spell: installed ${picked.map((entry) => entry.name).join(", ")}.`,
    );
    await restartClient();
  } catch (error) {
    output.appendLine(`Dictionary installation failed: ${String(error)}`);
    const choice = await vscode.window.showErrorMessage(
      `ayame-spell: dictionary installation failed. ${String(error)}`,
      "Show output",
    );
    if (choice) {
      output.show(true);
    }
  }
}

function runCli(args: string[], cwd: string): Promise<void> {
  const executable = resolveServer();
  output.appendLine(`Running ${executable} ${args.join(" ")}`);
  return new Promise((resolve, reject) => {
    execFile(executable, args, { cwd }, (error, stdout, stderr) => {
      if (stdout) {
        output.append(stdout);
      }
      if (stderr) {
        output.append(stderr);
      }
      if (error) {
        reject(error);
      } else {
        resolve();
      }
    });
  });
}

export async function deactivate(): Promise<void> {
  await stopClient();
}
