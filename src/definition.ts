import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";
import { validateNestfileDocument, NestfileCommand } from "./validator";

/**
 * Extract variables from document
 */
function extractVariables(text: string): Map<string, { line: number; char: number }> {
  const variables = new Map<string, { line: number; char: number }>();
  const lines = text.split(/\r?\n/);

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();

    // Match @var NAME = ... or @const NAME = ...
    const varMatch = line.match(/^\s*@(var|const)\s+([A-Za-z0-9_]+)\s*=/);
    if (varMatch) {
      const name = varMatch[2];
      const char = line.indexOf(name);
      variables.set(name, { line: i, char });
    }
  }

  return variables;
}

/**
 * Find command by name in the AST
 */
function findCommand(
  commands: NestfileCommand[],
  name: string
): NestfileCommand | null {
  for (const cmd of commands) {
    if (cmd.name === name) {
      return cmd;
    }
    const found = findCommand(cmd.children, name);
    if (found) {
      return found;
    }
  }
  return null;
}

/**
 * Check if cursor is inside a template variable {{...}}
 */
function isInsideTemplate(text: string, offset: number): { inside: boolean; varName?: string } {
  let lastOpen = -1;
  let lastClose = -1;

  for (let i = offset - 1; i >= 1; i--) {
    if (text[i] === "}" && text[i - 1] === "}") {
      if (lastClose === -1) {
        lastClose = i - 1;
      }
      i--;
      continue;
    }
    if (text[i] === "{" && text[i - 1] === "{") {
      lastOpen = i - 1;
      break;
    }
  }

  if (lastOpen >= 0) {
    if (lastClose === -1 || offset <= lastClose + 2) {
      const varStart = lastOpen + 2;
      const varEnd = lastClose >= 0 ? lastClose : text.length;
      const varName = text.substring(varStart, varEnd).trim();
      return { inside: true, varName };
    }
  }

  return { inside: false };
}

export class NestfileDefinitionProvider implements vscode.DefinitionProvider {
  provideDefinition(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken
  ): vscode.ProviderResult<vscode.Definition | vscode.LocationLink[]> {
    const line = document.lineAt(position);
    const lineText = line.text;
    const trimmed = lineText.trim();
    const fullText = document.getText();
    const offset = document.offsetAt(position);

    // Check if we're inside a template variable {{VAR}}
    const templateInfo = isInsideTemplate(fullText, offset);
    if (templateInfo.inside && templateInfo.varName) {
      const variables = extractVariables(fullText);
      if (variables.has(templateInfo.varName)) {
        const varInfo = variables.get(templateInfo.varName)!;
        return new vscode.Location(
          document.uri,
          new vscode.Position(varInfo.line, varInfo.char)
        );
      }
    }

    // Check if we're on a command name in depends:
    const dependsMatch = lineText.match(/^(\s*>\s*depends:\s*)(.*)$/);
    if (dependsMatch) {
      const dependsValue = dependsMatch[2];
      const cursorInDepends = position.character >= lineText.indexOf(dependsValue);

      if (cursorInDepends) {
        const ast = validateNestfileDocument(fullText, { returnAst: true }, document.uri);
        const lines = fullText.split(/\r?\n/);

        // Try to find which command name we're hovering over
        const words = dependsValue.split(/[,\s]+/);
        for (const word of words) {
          const cleanWord = word.trim().replace(/\(.*$/, ""); // Remove arguments
          if (cleanWord && lineText.includes(cleanWord)) {
            const wordIndex = lineText.indexOf(cleanWord);
            if (position.character >= wordIndex && position.character <= wordIndex + cleanWord.length) {
              const cmd = findCommand(ast.commands, cleanWord);
              if (cmd) {
                const cmdLine = lines[cmd.line];
                const nameMatch = cmdLine.match(/^(\s*)([A-Za-z0-9_]+)/);
                const nameStart = nameMatch ? nameMatch[1].length : 0;
                return new vscode.Location(
                  document.uri,
                  new vscode.Position(cmd.line, nameStart)
                );
              }
            }
          }
        }
      }
    }

    // Check if we're on an @include line
    if (trimmed.startsWith("@include ")) {
      const includePart = trimmed.substring(9).trim();
      if (!includePart) {
        return null;
      }

      // Find the position of the path in the line
      const pathStart = lineText.indexOf(includePart);
      if (pathStart === -1 || position.character < pathStart || position.character > pathStart + includePart.length) {
        return null;
      }

      // Resolve the include path
      const baseDir = path.dirname(document.uri.fsPath);
      let includePath: string;

      if (path.isAbsolute(includePart)) {
        includePath = includePart;
      } else {
        includePath = path.resolve(baseDir, includePart);
      }

      // Handle wildcards and directories - for now, just check if the directory exists
      if (includePart.includes("*") || includePart.endsWith("/") || includePart.endsWith("\\")) {
        // For wildcards and directories, we can't point to a single file
        // But we can point to the directory
        const dirPath = includePart.includes("*")
          ? path.dirname(includePath)
          : includePath.endsWith("/") || includePath.endsWith("\\")
            ? includePath.slice(0, -1)
            : includePath;

        if (fs.existsSync(dirPath) && fs.statSync(dirPath).isDirectory()) {
          return new vscode.Location(
            vscode.Uri.file(dirPath),
            new vscode.Position(0, 0)
          );
        }
        return null;
      }

      // Check if file exists
      if (fs.existsSync(includePath) && fs.statSync(includePath).isFile()) {
        return new vscode.Location(
          vscode.Uri.file(includePath),
          new vscode.Position(0, 0)
        );
      }

      // If file doesn't exist but has no extension, try common config file names
      if (!path.extname(includePath)) {
        for (const configName of ["nestfile", "Nestfile", "nest", "Nest"]) {
          const configPath = path.isAbsolute(includePart)
            ? path.join(includePath, configName)
            : path.resolve(baseDir, includePart, configName);

          if (fs.existsSync(configPath) && fs.statSync(configPath).isFile()) {
            return new vscode.Location(
              vscode.Uri.file(configPath),
              new vscode.Position(0, 0)
            );
          }
        }
      }
    }

    return null;
  }
}

