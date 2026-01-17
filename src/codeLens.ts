import * as vscode from "vscode";
import { validateNestfileDocument, NestfileCommand } from "./validator";

/**
 * Count how many times a command is referenced
 */
function countCommandReferences(text: string, cmdName: string): number {
  let count = 0;
  const lines = text.split(/\r?\n/);

  for (const line of lines) {
    const trimmed = line.trim();

    // Check depends: directives
    if (trimmed.startsWith(">") && trimmed.includes("depends:")) {
      const dependsMatch = trimmed.match(/depends:\s*(.+)/);
      if (dependsMatch) {
        const dependsValue = dependsMatch[1];
        const regex = new RegExp(`\\b${cmdName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\b`, "g");
        const matches = dependsValue.match(regex);
        if (matches) {
          count += matches.length;
        }
      }
    }
  }

  return count;
}

/**
 * Count how many times a variable is referenced
 */
function countVariableReferences(text: string, varName: string): number {
  const regex = new RegExp(`\\{\\{\\s*${varName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\s*\\}\\}`, "g");
  const matches = text.match(regex);
  return matches ? matches.length : 0;
}

export class NestfileCodeLensProvider implements vscode.CodeLensProvider {
  provideCodeLenses(
    document: vscode.TextDocument,
    token: vscode.CancellationToken
  ): vscode.ProviderResult<vscode.CodeLens[]> {
    const codeLenses: vscode.CodeLens[] = [];
    const text = document.getText();
    const lines = text.split(/\r?\n/);

    // Add code lens for commands
    const ast = validateNestfileDocument(text, { returnAst: true }, document.uri);

    const addLensForCommand = (cmd: NestfileCommand) => {
      const line = document.lineAt(cmd.line);
      const range = new vscode.Range(cmd.line, 0, cmd.line, line.text.length);

      // 1. Add "Run" Lens
      const runLens = new vscode.CodeLens(range);
      runLens.command = {
        title: "▶ Run Command",
        command: "nestfile.runCommand",
        arguments: [cmd.name, document.uri.fsPath]
      };
      codeLenses.push(runLens);

      // Recursively add for children
      for (const child of cmd.children) {
        addLensForCommand(child);
      }
    };

    for (const cmd of ast.commands) {
      addLensForCommand(cmd);
    }
    return codeLenses;
  }
}


