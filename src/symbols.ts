import * as vscode from "vscode";
import { validateNestfileDocument, NestfileCommand } from "./validator";

/**
 * Extract variables and constants from document text
 */
function extractVariableSymbols(
  text: string,
  document: vscode.TextDocument
): vscode.DocumentSymbol[] {
  const symbols: vscode.DocumentSymbol[] = [];
  const lines = text.split(/\r?\n/);

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();

    // Match @var NAME = ... or @const NAME = ...
    const varMatch = line.match(/^\s*@(var|const)\s+([A-Za-z0-9_]+)\s*=\s*(.+)$/);
    if (varMatch) {
      const isConst = varMatch[1] === "const";
      const name = varMatch[2];
      const value = varMatch[3].trim();

      const startPos = new vscode.Position(i, line.indexOf(name));
      const endPos = new vscode.Position(i, startPos.character + name.length);

      symbols.push(
        new vscode.DocumentSymbol(
          name,
          `= ${value}`,
          isConst ? vscode.SymbolKind.Constant : vscode.SymbolKind.Variable,
          new vscode.Range(startPos, endPos),
          new vscode.Range(startPos, endPos)
        )
      );
    }
  }

  return symbols;
}

/**
 * Extract functions from document text
 */
function extractFunctionSymbols(
  text: string,
  document: vscode.TextDocument
): vscode.DocumentSymbol[] {
  const symbols: vscode.DocumentSymbol[] = [];
  const lines = text.split(/\r?\n/);

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();

    // Match @function name(...):
    const funcMatch = trimmed.match(/^@function\s+([A-Za-z0-9_]+)\s*(\([^)]*\))?\s*:\s*$/);
    if (funcMatch) {
      const name = funcMatch[1];
      const params = funcMatch[2] || "()";

      const startPos = new vscode.Position(i, line.indexOf(name));
      const endPos = new vscode.Position(i, startPos.character + name.length);

      symbols.push(
        new vscode.DocumentSymbol(
          name,
          `@function${params}`,
          vscode.SymbolKind.Function,
          new vscode.Range(startPos, endPos),
          new vscode.Range(startPos, endPos)
        )
      );
    }
  }

  return symbols;
}

/**
 * Convert command to document symbol
 */
function commandToSymbol(
  cmd: NestfileCommand,
  lines: string[],
  document: vscode.TextDocument
): vscode.DocumentSymbol {
  const line = lines[cmd.line];
  const trimmed = line.trim();

  // Find the command name position
  const nameMatch = trimmed.match(/^([A-Za-z0-9_]+)/);
  const nameStart = nameMatch ? line.indexOf(nameMatch[1]) : 0;
  const nameEnd = nameStart + cmd.name.length;

  const startPos = new vscode.Position(cmd.line, nameStart);
  const endPos = new vscode.Position(cmd.line, nameEnd);

  // Get description from directives
  const descDirective = cmd.directives.find(d => d.name === "desc");
  const paramsPreview = cmd.parameters.length > 0
    ? `(${cmd.parameters.map(p => `${p.name}:${p.type}`).join(", ")})`
    : "";

  const detail = descDirective
    ? descDirective.raw.match(/desc:\s*(.+)/)?.[1]?.trim() || paramsPreview
    : paramsPreview;

  const symbol = new vscode.DocumentSymbol(
    cmd.name,
    detail,
    cmd.children.length > 0 ? vscode.SymbolKind.Namespace : vscode.SymbolKind.Function,
    new vscode.Range(startPos, endPos),
    new vscode.Range(startPos, endPos)
  );

  // Add child commands
  if (cmd.children.length > 0) {
    symbol.children = cmd.children.map(child =>
      commandToSymbol(child, lines, document)
    );
  }

  return symbol;
}

export class NestfileDocumentSymbolProvider implements vscode.DocumentSymbolProvider {
  provideDocumentSymbols(
    document: vscode.TextDocument,
    token: vscode.CancellationToken
  ): vscode.ProviderResult<vscode.DocumentSymbol[]> {
    const text = document.getText();
    const lines = text.split(/\r?\n/);
    const symbols: vscode.DocumentSymbol[] = [];

    // Add variables and constants
    const variableSymbols = extractVariableSymbols(text, document);
    symbols.push(...variableSymbols);

    // Add functions
    const functionSymbols = extractFunctionSymbols(text, document);
    symbols.push(...functionSymbols);

    // Add commands
    const ast = validateNestfileDocument(text, { returnAst: true }, document.uri);
    const commandSymbols = ast.commands.map(cmd => commandToSymbol(cmd, lines, document));
    symbols.push(...commandSymbols);

    return symbols;
  }
}


