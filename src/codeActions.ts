import * as vscode from "vscode";
import { validateNestfileDocument, NestfileCommand } from "./validator";

const VALID_PARAM_TYPES = ["str", "bool", "num", "arr"];

/**
 * Get command at a specific line
 */
function getCommandAtLine(
  document: vscode.TextDocument,
  line: number,
  commands: NestfileCommand[]
): NestfileCommand | null {
  const lines = document.getText().split(/\r?\n/);

  function findInCommand(cmd: NestfileCommand): NestfileCommand | null {
    if (cmd.line > line) {
      return null;
    }

    for (const child of cmd.children) {
      if (child.line <= line) {
        const found = findInCommand(child);
        if (found) {
          return found;
        }
      }
    }

    const cmdIndent = (lines[cmd.line].match(/^(\s*)/)?.[1]?.length || 0);
    let commandEndLine = lines.length - 1;

    for (let i = cmd.line + 1; i < lines.length; i++) {
      const currentLine = lines[i];
      if (!currentLine.trim() || currentLine.trim().startsWith("#")) {
        continue;
      }

      const indent = (currentLine.match(/^(\s*)/)?.[1]?.length || 0);

      if (indent <= cmdIndent && !currentLine.trim().startsWith(">")) {
        commandEndLine = i - 1;
        break;
      }
    }

    if (line >= cmd.line && line <= commandEndLine) {
      return cmd;
    }

    return null;
  }

  for (const cmd of commands) {
    const found = findInCommand(cmd);
    if (found) {
      return found;
    }
  }

  return null;
}

export class NestfileCodeActionProvider implements vscode.CodeActionProvider {
  provideCodeActions(
    document: vscode.TextDocument,
    range: vscode.Range | vscode.Selection,
    context: vscode.CodeActionContext,
    token: vscode.CancellationToken
  ): vscode.ProviderResult<vscode.CodeAction[]> {
    const actions: vscode.CodeAction[] = [];
    const diagnostics = context.diagnostics;

    for (const diagnostic of diagnostics) {
      const message = diagnostic.message;

      // Fix invalid parameter type
      if (message.includes("Invalid parameter type")) {
        const typeMatch = message.match(/Invalid parameter type "([^"]+)"/);
        if (typeMatch) {
          const invalidType = typeMatch[1];
          
          // Suggest valid types
          for (const validType of VALID_PARAM_TYPES) {
            const action = new vscode.CodeAction(
              `Change type to '${validType}'`,
              vscode.CodeActionKind.QuickFix
            );
            action.diagnostics = [diagnostic];
            action.isPreferred = validType === "str"; // Prefer 'str' as default
            
            const edit = new vscode.WorkspaceEdit();
            const line = document.lineAt(diagnostic.range.start.line);
            const lineText = line.text;
            
            // Find the invalid type and replace it
            const typeRegex = new RegExp(`\\b${invalidType.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\b`);
            const typeMatch = lineText.match(typeRegex);
            if (typeMatch && typeMatch.index !== undefined) {
              const startPos = new vscode.Position(diagnostic.range.start.line, typeMatch.index);
              const endPos = new vscode.Position(diagnostic.range.start.line, typeMatch.index + invalidType.length);
              edit.replace(document.uri, new vscode.Range(startPos, endPos), validType);
              action.edit = edit;
              actions.push(action);
            }
          }
        }
      }

      // Add missing script directive
      if (message.includes("has no script directive")) {
        const cmdMatch = message.match(/Command "([^"]+)"/);
        if (cmdMatch) {
          const cmdName = cmdMatch[1];
          const ast = validateNestfileDocument(document.getText(), { returnAst: true }, document.uri);
          const cmd = getCommandAtLine(document, diagnostic.range.start.line, ast.commands);
          
          if (cmd) {
            const action = new vscode.CodeAction(
              `Add script directive to '${cmdName}'`,
              vscode.CodeActionKind.QuickFix
            );
            action.diagnostics = [diagnostic];
            
            // Find the end of the command (before first child or at end of command block)
            const lines = document.getText().split(/\r?\n/);
            const cmdIndent = (lines[cmd.line].match(/^(\s*)/)?.[1]?.length || 0);
            let insertLine = cmd.line + 1;
            
            // Find the right place to insert (after last directive or before first child)
            for (let i = cmd.line + 1; i < lines.length; i++) {
              const currentLine = lines[i];
              if (!currentLine.trim() || currentLine.trim().startsWith("#")) {
                insertLine = i + 1;
                continue;
              }
              
              const indent = (currentLine.match(/^(\s*)/)?.[1]?.length || 0);
              
              if (currentLine.trim().startsWith(">")) {
                insertLine = i + 1;
              } else if (indent <= cmdIndent) {
                break;
              }
            }
            
            const indentSpaces = " ".repeat(cmdIndent + 1);
            const edit = new vscode.WorkspaceEdit();
            const insertPos = new vscode.Position(insertLine, 0);
            edit.insert(document.uri, insertPos, `${indentSpaces}> script: |\n${indentSpaces}    ${"$0: echo \"Hello\""}\n`);
            action.edit = edit;
            actions.push(action);
          }
        }
      }

      // Fix unknown directive (suggest correct directive name)
      if (message.includes("Unknown directive")) {
        const directiveMatch = message.match(/Unknown directive "([^"]+)"/);
        if (directiveMatch) {
          const unknownDir = directiveMatch[1].toLowerCase();
          
          // Find similar directives
          const validDirectives = [
            "desc", "cwd", "env", "script", "before", "after", "fallback", "finaly",
            "depends", "validate", "if", "elif", "else", "logs", "privileged", "require_confirm"
          ];
          
          // Simple similarity check
          for (const validDir of validDirectives) {
            if (validDir.startsWith(unknownDir) || unknownDir.startsWith(validDir) || 
                validDir.includes(unknownDir) || unknownDir.includes(validDir)) {
              const action = new vscode.CodeAction(
                `Change to '${validDir}'`,
                vscode.CodeActionKind.QuickFix
              );
              action.diagnostics = [diagnostic];
              
              const edit = new vscode.WorkspaceEdit();
              const line = document.lineAt(diagnostic.range.start.line);
              const lineText = line.text;
              
              // Find and replace the directive name
              const dirMatch = lineText.match(/^(\s*>)\s*([a-zA-Z_]+(\[[^\]]+\])?)/);
              if (dirMatch) {
                const dirStart = dirMatch.index! + dirMatch[1].length + 1;
                const dirEnd = dirStart + dirMatch[2].length;
                edit.replace(
                  document.uri,
                  new vscode.Range(diagnostic.range.start.line, dirStart, diagnostic.range.start.line, dirEnd),
                  validDir
                );
                action.edit = edit;
                actions.push(action);
                break; // Only suggest the first match
              }
            }
          }
        }
      }
    }

    return actions;
  }
}


