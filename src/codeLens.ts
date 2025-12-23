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
    
    function addLensForCommand(cmd: NestfileCommand) {
      const count = countCommandReferences(text, cmd.name);
      if (count > 0) {
        const line = document.lineAt(cmd.line);
        const range = new vscode.Range(cmd.line, 0, cmd.line, line.text.length);
        const codeLens = new vscode.CodeLens(range);
        codeLens.command = {
          title: `${count} reference${count !== 1 ? "s" : ""}`,
          command: "",
        };
        codeLenses.push(codeLens);
      }

      // Recursively add for children
      for (const child of cmd.children) {
        addLensForCommand(child);
      }
    }

    for (const cmd of ast.commands) {
      addLensForCommand(cmd);
    }

    // Add code lens for variables
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const trimmed = line.trim();

      // Match @var NAME = ... or @const NAME = ...
      const varMatch = trimmed.match(/^@(var|const)\s+([A-Za-z0-9_]+)\s*=/);
      if (varMatch) {
        const varName = varMatch[2];
        const count = countVariableReferences(text, varName);
        if (count > 0) {
          const range = new vscode.Range(i, 0, i, line.length);
          const codeLens = new vscode.CodeLens(range);
          codeLens.command = {
            title: `${count} reference${count !== 1 ? "s" : ""}`,
            command: "",
          };
          codeLenses.push(codeLens);
        }
      }
    }

    return codeLenses;
  }
}

