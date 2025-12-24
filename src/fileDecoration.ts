import * as vscode from "vscode";
import * as path from "path";

export class NestfileFileDecorationProvider implements vscode.FileDecorationProvider {
  provideFileDecoration(
    uri: vscode.Uri,
    token: vscode.CancellationToken
  ): vscode.ProviderResult<vscode.FileDecoration> {
    // Check if this is a nest file
    const fileName = path.basename(uri.fsPath);
    const ext = path.extname(uri.fsPath);

    const isNestFile =
      fileName === "Nestfile" ||
      fileName === "nestfile" ||
      fileName === "nest" ||
      ext === ".nest";

    if (isNestFile) {
      return {
        badge: "N",
        tooltip: "Nestfile",
        color: new vscode.ThemeColor("textLink.foreground"),
      };
    }

    return undefined;
  }
}

