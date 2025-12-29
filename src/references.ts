import * as vscode from "vscode";
import { validateNestfileDocument, NestfileCommand } from "./validator";

/**
 * Extract variable definitions from document
 */
function extractVariableDefinitions(
  text: string
): Map<string, { line: number; char: number; charEnd: number }> {
  const variables = new Map<string, { line: number; char: number; charEnd: number }>();
  const lines = text.split(/\r?\n/);

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();

    // Match @var NAME = ... or @const NAME = ...
    const varMatch = trimmed.match(/^@(var|const)\s+([A-Za-z0-9_]+)\s*=/);
    if (varMatch) {
      const name = varMatch[2];
      const char = line.indexOf(name);
      variables.set(name, { line: i, char, charEnd: char + name.length });
    }
  }

  return variables;
}

/**
 * Find command definition by name
 */
function findCommandDefinition(
  commands: NestfileCommand[],
  name: string
): NestfileCommand | null {
  for (const cmd of commands) {
    if (cmd.name === name) {
      return cmd;
    }
    const found = findCommandDefinition(cmd.children, name);
    if (found) {
      return found;
    }
  }
  return null;
}

/**
 * Find all references to a variable in template syntax {{VAR}}
 */
function findVariableReferences(
  text: string,
  varName: string,
  document: vscode.TextDocument
): vscode.Location[] {
  const locations: vscode.Location[] = [];
  const lines = text.split(/\r?\n/);
  const regex = new RegExp(`\\{\\{\\s*${varName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\s*\\}\\}`, "g");

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    let match: RegExpExecArray | null;

    while ((match = regex.exec(line)) !== null) {
      const startPos = new vscode.Position(i, match.index + 2); // +2 for {{
      const endPos = new vscode.Position(i, match.index + 2 + varName.length);
      locations.push(new vscode.Location(document.uri, new vscode.Range(startPos, endPos)));
    }
  }

  return locations;
}

/**
 * Find all references to a command (in depends: and other places)
 */
function findCommandReferences(
  text: string,
  cmdName: string,
  document: vscode.TextDocument
): vscode.Location[] {
  const locations: vscode.Location[] = [];
  const lines = text.split(/\r?\n/);

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();

    // Check depends: directives
    if (trimmed.startsWith(">") && trimmed.includes("depends:")) {
      const dependsMatch = trimmed.match(/depends:\s*(.+)/);
      if (dependsMatch) {
        const dependsValue = dependsMatch[1];
        const regex = new RegExp(`\\b${cmdName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\b`, "g");
        let match: RegExpExecArray | null;
        const dependsStart = line.indexOf(dependsValue);

        while ((match = regex.exec(dependsValue)) !== null) {
          const startPos = new vscode.Position(i, dependsStart + match.index);
          const endPos = new vscode.Position(i, dependsStart + match.index + cmdName.length);
          locations.push(new vscode.Location(document.uri, new vscode.Range(startPos, endPos)));
        }
      }
    }
  }

  return locations;
}

export class NestfileReferenceProvider implements vscode.ReferenceProvider {
  provideReferences(
    document: vscode.TextDocument,
    position: vscode.Position,
    context: vscode.ReferenceContext,
    token: vscode.CancellationToken
  ): vscode.ProviderResult<vscode.Location[]> {
    const line = document.lineAt(position);
    const lineText = line.text;
    const fullText = document.getText();
    const offset = document.offsetAt(position);

    // Get the word at the cursor position
    const wordRange = document.getWordRangeAtPosition(position, /\b[A-Za-z0-9_]+\b/);
    if (!wordRange) {
      return [];
    }

    const word = document.getText(wordRange);

    // Check if it's a variable definition (@var VAR or @const VAR)
    const variables = extractVariableDefinitions(fullText);
    if (variables.has(word)) {
      const varDef = variables.get(word)!;
      const defPos = new vscode.Position(varDef.line, varDef.char);
      
      // Check if we're at the definition
      if (position.line === varDef.line && position.character >= varDef.char && position.character <= varDef.charEnd) {
        // Find all references
        const references = findVariableReferences(fullText, word, document);
        return context.includeDeclaration ? references : references.filter(ref => 
          !(ref.range.start.line === varDef.line && ref.range.start.character === varDef.char)
        );
      }
    }

    // Check if it's a command definition
    const ast = validateNestfileDocument(fullText, { returnAst: true }, document.uri);
    const cmd = findCommandDefinition(ast.commands, word);
    if (cmd) {
      const lines = fullText.split(/\r?\n/);
      const cmdLine = lines[cmd.line];
      const nameMatch = cmdLine.match(/^(\s*)([A-Za-z0-9_]+)/);
      
      if (nameMatch) {
        const nameStart = nameMatch[1].length;
        const nameEnd = nameStart + word.length;
        
        // Check if we're at the command definition
        if (position.line === cmd.line && position.character >= nameStart && position.character <= nameEnd) {
          const references = findCommandReferences(fullText, word, document);
          const defLocation = new vscode.Location(
            document.uri,
            new vscode.Range(cmd.line, nameStart, cmd.line, nameEnd)
          );
          
          if (context.includeDeclaration) {
            return [defLocation, ...references];
          } else {
            return references;
          }
        }
      }
    }

    // Check if we're on a variable reference (in template {{VAR}})
    const templateMatch = lineText.substring(Math.max(0, position.character - word.length - 2), position.character + 2);
    if (templateMatch.includes(`{{${word}}}`) || templateMatch.includes(`{{ ${word} }}`)) {
      if (variables.has(word)) {
        const varDef = variables.get(word)!;
        const defLocation = new vscode.Location(
          document.uri,
          new vscode.Range(varDef.line, varDef.char, varDef.line, varDef.charEnd)
        );
        const references = findVariableReferences(fullText, word, document);
        
        if (context.includeDeclaration) {
          return [defLocation, ...references];
        } else {
          return references.filter(ref => 
            !(ref.range.start.line === varDef.line && ref.range.start.character === varDef.char)
          );
        }
      }
    }

    // Check if we're on a command reference (in depends:)
    const trimmed = lineText.trim();
    if (trimmed.startsWith(">") && trimmed.includes("depends:")) {
      const dependsMatch = trimmed.match(/depends:\s*(.+)/);
      if (dependsMatch && dependsMatch[1].includes(word)) {
        const cmd = findCommandDefinition(ast.commands, word);
        if (cmd) {
          const lines = fullText.split(/\r?\n/);
          const cmdLine = lines[cmd.line];
          const nameMatch = cmdLine.match(/^(\s*)([A-Za-z0-9_]+)/);
          
          if (nameMatch) {
            const nameStart = nameMatch[1].length;
            const nameEnd = nameStart + word.length;
            const defLocation = new vscode.Location(
              document.uri,
              new vscode.Range(cmd.line, nameStart, cmd.line, nameEnd)
            );
            const references = findCommandReferences(fullText, word, document);
            
            if (context.includeDeclaration) {
              return [defLocation, ...references];
            } else {
              return references.filter(ref => 
                !(ref.range.start.line === cmd.line && ref.range.start.character === nameStart)
              );
            }
          }
        }
      }
    }

    return [];
  }
}


