import * as path from "path";
import * as vscode from "vscode";
import {
  LanguageClient,
  type LanguageClientOptions,
  type ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | null = null;

export async function activate(cx: vscode.ExtensionContext) {
  if (client !== null) {
    return;
  }
  const config = vscode.workspace.getConfiguration("calcagebra");
  if (!config.get("server.enable")) {
    return;
  }
  const ext = process.platform === "win32" ? ".exe" : "";
  const configPath = config.get("server.path");
  const serverOpts: ServerOptions = {
    command:
      typeof configPath === "string" && configPath.length !== 0
        ? configPath
        : cx.asAbsolutePath(path.join("out", "calcagebra-ls" + ext)),
  };
  const clientOpts: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "calcagebra", pattern: "**/*.{cal}" }],
    initializationOptions: {
      token_hover: config.get("server.hover.token.enable"),
      fs_watcher: config.get("server.fileSystemWatcher.enable"),
      diagnostics: {
        on_change: config.get("server.diagnostics.onChange.enable"),
        more_info_hint: config.get("server.diagnostics.moreInfoHint.enable"),
        ignore: config.get("server.diagnostics.ignore"),
      },
    },
  };
  client = new LanguageClient("calcagebra", serverOpts, clientOpts);
  await client.start();
}

export async function deactivate() {
  if (client === null) {
    return;
  }
  await client.stop();
  client = null;
}
