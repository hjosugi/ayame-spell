import * as fs from "fs";
import * as path from "path";
import * as vscode from "vscode";
import {
  ExecuteCommandRequest,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;
let statusItem: vscode.StatusBarItem;

function findServer(context: vscode.ExtensionContext): string {
  const configured = vscode.workspace
    .getConfiguration("ayame-spell")
    .get<string>("serverPath");
  if (configured) {
    return configured;
  }
  const exe = process.platform === "win32" ? "ayame-spell.exe" : "ayame-spell";
  const bundled = context.asAbsolutePath(
    path.join("server", `${process.platform}-${process.arch}`, exe),
  );
  if (fs.existsSync(bundled)) {
    return bundled;
  }
  return "ayame-spell"; // rely on $PATH
}

export async function activate(context: vscode.ExtensionContext) {
  if (
    !vscode.workspace.getConfiguration("ayame-spell").get<boolean>("enable", true)
  ) {
    return;
  }

  const serverPath = findServer(context);
  const serverOptions: ServerOptions = {
    command: serverPath,
    args: ["lsp"],
    transport: TransportKind.stdio,
  };
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file" }, { scheme: "untitled" }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher(
        "**/{ayame-spell.toml,.ayame-spell.toml,ayame-words.txt}",
      ),
    },
  };
  client = new LanguageClient(
    "ayame-spell",
    "ayame-spell",
    serverOptions,
    clientOptions,
  );
  try {
    await client.start();
  } catch {
    void vscode.window.showErrorMessage(
      `ayame-spell: could not start "${serverPath}". ` +
        "Install the CLI (cargo install ayame-spell) or set ayame-spell.serverPath.",
    );
    client = undefined;
    return;
  }

  statusItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    90,
  );
  statusItem.name = "ayame-spell";
  statusItem.command = "workbench.actions.view.problems";
  context.subscriptions.push(
    statusItem,
    vscode.languages.onDidChangeDiagnostics(updateStatus),
    vscode.window.onDidChangeActiveTextEditor(updateStatus),
    vscode.commands.registerCommand("ayame-spell.fixAll", fixAll),
    vscode.commands.registerCommand("ayame-spell.reviewWords", reviewWords),
    vscode.commands.registerCommand("ayame-spell.restart", async () => {
      await client?.restart();
    }),
  );
  updateStatus();
}

function ourDiagnostics(uri: vscode.Uri): vscode.Diagnostic[] {
  return vscode.languages
    .getDiagnostics(uri)
    .filter((d) => d.source === "ayame-spell");
}

function updateStatus() {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    statusItem.hide();
    return;
  }
  const count = ourDiagnostics(editor.document.uri).length;
  statusItem.text = count > 0 ? `$(book) ayame ${count}` : "$(book) ayame";
  statusItem.tooltip =
    count > 0
      ? `ayame-spell: ${count} finding(s) in this file`
      : "ayame-spell: no findings in this file";
  statusItem.show();
}

async function fixAll() {
  const editor = vscode.window.activeTextEditor;
  if (!editor || !client) {
    return;
  }
  await client.sendRequest(ExecuteCommandRequest.type, {
    command: "ayame-spell.fixAll",
    arguments: [{ uri: editor.document.uri.toString() }],
  });
}

function diagnosticCode(d: vscode.Diagnostic): string {
  if (typeof d.code === "string") {
    return d.code;
  }
  if (d.code && typeof d.code === "object" && "value" in d.code) {
    return String(d.code.value);
  }
  return "";
}

/** Bulk triage of every flagged word in open documents: multi-select, then
 * add all selected words to the project/global dictionary or the ignore
 * list in one action. */
async function reviewWords() {
  if (!client) {
    return;
  }
  const counts = new Map<string, { count: number; kind: string }>();
  for (const [, diags] of vscode.languages.getDiagnostics()) {
    for (const d of diags) {
      if (d.source !== "ayame-spell") {
        continue;
      }
      const kind = diagnosticCode(d);
      if (!["typo", "unknown-word", "ja-variant"].includes(kind)) {
        continue;
      }
      const match = /`([^`]+)`/.exec(d.message);
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
    .map(([word, e]) => ({
      label: word,
      description: `${e.count}× ${e.kind}`,
    }));
  const picked = await vscode.window.showQuickPick(items, {
    canPickMany: true,
    title: "ayame-spell: select words to handle in bulk",
    placeHolder: "Space to select, Enter to confirm",
  });
  if (!picked || picked.length === 0) {
    return;
  }

  type Action = vscode.QuickPickItem & { cmd: string; scope?: string };
  const actions: Action[] = [
    {
      label: "$(add) Add to project words",
      description: "ayame-words.txt (committed, shared with the team)",
      cmd: "ayame-spell.addWords",
      scope: "project",
    },
    {
      label: "$(globe) Add to global words",
      description: "~/.config/ayame-spell/words.txt (all your projects)",
      cmd: "ayame-spell.addWords",
      scope: "global",
    },
    {
      label: "$(mute) Ignore in this project",
      description: "[words].ignore in ayame-spell.toml",
      cmd: "ayame-spell.ignoreWords",
    },
  ];
  const action = await vscode.window.showQuickPick(actions, {
    title: `Apply to ${picked.length} word(s)`,
  });
  if (!action) {
    return;
  }
  const words = picked.map((p) => p.label);
  await client.sendRequest(ExecuteCommandRequest.type, {
    command: action.cmd,
    arguments: [{ words, scope: action.scope }],
  });
  void vscode.window.showInformationMessage(
    `ayame-spell: applied "${action.label.replace(/\$\([a-z-]+\) /, "")}" to ${words.length} word(s).`,
  );
}

export async function deactivate() {
  await client?.stop();
}
