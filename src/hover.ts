import * as vscode from "vscode";
import { validateNestfileDocument, NestfileCommand } from "./validator";

// Directives with descriptions for hover
const DIRECTIVES = [
  { name: "desc", description: "Command description (shown in help)" },
  { name: "cwd", description: "Working directory for script execution" },
  { name: "env", description: "Environment variables" },
  { name: "script", description: "Script to execute (single line or multiline with |)" },
  { name: "before", description: "Script executed before the main script" },
  { name: "after", description: "Script executed after successful completion" },
  { name: "fallback", description: "Script executed on failure" },
  { name: "finaly", description: "Script always executed (finally)" },
  { name: "depends", description: "Command dependencies (comma-separated)" },
  { name: "validate", description: "Parameter validation rules" },

  { name: "logs:json", description: "Log command execution as JSON" },
  { name: "logs:txt", description: "Log command execution as text" },
  { name: "privileged", description: "Require privileged access (root/admin)" },
  { name: "require_confirm", description: "Require user confirmation before execution" },
];

/**
 * Extract all variables (@var and @const) from document text
 */
function extractVariables(text: string): Map<string, { value: string; isConst: boolean }> {
  const variables = new Map<string, { value: string; isConst: boolean }>();
  const lines = text.split(/\r?\n/);

  for (const line of lines) {
    const trimmed = line.trim();

    // Match @var NAME = ... or @const NAME = ...
    const varMatch = line.match(/^\s*@(var|const)\s+([A-Za-z0-9_]+)\s*=\s*(.+)$/);
    if (varMatch) {
      const isConst = varMatch[1] === "const";
      const name = varMatch[2];
      const value = varMatch[3].trim();
      variables.set(name, { value, isConst });
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
 * Get command description from directives
 */
function getCommandDescription(cmd: NestfileCommand): string | null {
  const descDirective = cmd.directives.find(d => d.name === "desc");
  if (descDirective) {
    const match = descDirective.raw.match(/desc:\s*(.+)/);
    return match ? match[1].trim() : null;
  }
  return null;
}

/**
 * Find command at a specific line
 */
function findCommandAtLine(
  document: vscode.TextDocument,
  position: vscode.Position,
  commands: NestfileCommand[]
): NestfileCommand | null {
  const targetLine = position.line;
  const lines = document.getText().split(/\r?\n/);

  function findInCommand(cmd: NestfileCommand): NestfileCommand | null {
    if (cmd.line > targetLine) {
      return null;
    }

    // First check nested commands
    for (const child of cmd.children) {
      if (child.line <= targetLine) {
        const found = findInCommand(child);
        if (found) {
          return found;
        }
      }
    }

    // Check if target line is within this command's scope
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

    if (targetLine >= cmd.line && targetLine <= commandEndLine) {
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

/**
 * Get parameter value from command context
 */
function getParameterValue(
  cmd: NestfileCommand,
  paramName: string
): string | null {
  const param = cmd.parameters.find(p => {
    const cleanName = p.name.startsWith("!") ? p.name.substring(1) : p.name;
    return cleanName === paramName;
  });
  return param ? param.type : null;
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

export class NestfileHoverProvider implements vscode.HoverProvider {
  provideHover(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken
  ): vscode.ProviderResult<vscode.Hover> {
    const line = document.lineAt(position);
    const lineText = line.text;
    const textBeforeCursor = lineText.substring(0, position.character);
    const fullText = document.getText();
    const offset = document.offsetAt(position);

    // Check if we're inside a template variable {{VAR}}
    const templateInfo = isInsideTemplate(fullText, offset);
    if (templateInfo.inside && templateInfo.varName) {
      const commands = validateNestfileDocument(fullText, { returnAst: true }, document.uri).commands;
      const variables = extractVariables(fullText);
      const currentCommand = findCommandAtLine(document, position, commands);

      // Check if it's a variable
      if (variables.has(templateInfo.varName)) {
        const varInfo = variables.get(templateInfo.varName)!;
        const varType = varInfo.isConst ? "Constant" : "Variable";
        const markdown = new vscode.MarkdownString();
        markdown.appendMarkdown(`**${varType}:** \`${templateInfo.varName}\`\n\n`);
        markdown.appendMarkdown(`\`\`\`\n${varInfo.value}\n\`\`\``);
        return new vscode.Hover(markdown);
      }

      // Check if it's a parameter of the current command
      if (currentCommand) {
        const paramValue = getParameterValue(currentCommand, templateInfo.varName);
        if (paramValue !== null) {
          const markdown = new vscode.MarkdownString();
          markdown.appendMarkdown(`**Parameter:** \`${templateInfo.varName}\`\n\n`);
          markdown.appendMarkdown(`Type: \`${paramValue}\``);
          return new vscode.Hover(markdown);
        }
      }
    }

    // Check if we're hovering over a directive
    const trimmed = lineText.trim();
    if (trimmed.startsWith(">")) {
      const directiveLine = trimmed.substring(1).trim();
      const colonIndex = directiveLine.indexOf(":");

      if (colonIndex !== -1) {
        let name = directiveLine.substring(0, colonIndex).trim();

        // Strip modifiers, e.g. script[hide]
        const modifierIndex = name.indexOf("[");
        if (modifierIndex !== -1) {
          name = name.substring(0, modifierIndex).trim();
        }

        // Handle logs:json and logs:txt
        const directiveBase = name.startsWith("logs")
          ? "logs"
          : name;

        const directive = DIRECTIVES.find(d =>
          d.name === directiveBase ||
          (directiveBase === "logs" && (name === "logs:json" || name === "logs:txt"))
        );

        if (directive) {
          const displayName = directiveBase === "logs" && (name === "logs:json" || name === "logs:txt")
            ? name
            : directive.name;

          const markdown = new vscode.MarkdownString();
          markdown.appendMarkdown(`**Directive:** \`> ${displayName}:\`\n\n`);
          markdown.appendMarkdown(directive.description);
          return new vscode.Hover(markdown);
        }
      }
    }

    // Check if we're hovering over a command name in depends:
    const dependsMatch = lineText.match(/^(\s*>\s*depends:\s*)(.*)$/);
    if (dependsMatch) {
      const dependsValue = dependsMatch[2];
      const cursorInDepends = position.character >= lineText.indexOf(dependsValue);

      if (cursorInDepends) {
        const commands = validateNestfileDocument(fullText, { returnAst: true }, document.uri).commands;

        // Try to find which command name we're hovering over
        const words = dependsValue.split(/[,\s]+/);
        for (const word of words) {
          const cleanWord = word.trim().replace(/\(.*$/, ""); // Remove arguments
          if (cleanWord && textBeforeCursor.includes(cleanWord)) {
            const cmd = findCommand(commands, cleanWord);
            if (cmd) {
              const desc = getCommandDescription(cmd);
              const markdown = new vscode.MarkdownString();
              markdown.appendMarkdown(`**Command:** \`${cleanWord}\`\n\n`);
              if (desc) {
                markdown.appendMarkdown(`${desc}`);
              } else {
                markdown.appendMarkdown(`Command definition`);
              }
              return new vscode.Hover(markdown);
            }
          }
        }
      }
    }

    return null;
  }
}


