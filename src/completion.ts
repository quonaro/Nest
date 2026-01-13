import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";
import { validateNestfileDocument, NestfileCommand } from "./validator";

// Valid directives for autocomplete
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
  { name: "if", description: "Conditional execution" },
  { name: "elif", description: "Else-if conditional execution" },
  { name: "else", description: "Else branch for conditional execution" },
  { name: "logs:json", description: "Log command execution as JSON" },
  { name: "logs:txt", description: "Log command execution as text" },
  { name: "privileged", description: "Require privileged access (root/admin)" },
  { name: "require_confirm", description: "Require user confirmation before execution" },
  { name: "watch", description: "Watch files for changes and auto-reload (glob patterns)" },
];

// Meta commands
const META_COMMANDS = [
  { name: "@var", description: "Define a global variable" },
  { name: "@const", description: "Define a global constant" },
  { name: "@function", description: "Define a reusable function" },
  { name: "@include", description: "Include another Nestfile configuration" },
];

// Parameter types for autocomplete
const PARAMETER_TYPES = ["str", "bool", "num", "arr"];

/**
 * Extract all command names from the document AST
 */
function extractCommandNames(commands: NestfileCommand[]): string[] {
  const names: string[] = [];

  function traverse(cmd: NestfileCommand, prefix: string = "") {
    const fullName = prefix ? `${prefix} ${cmd.name}` : cmd.name;
    names.push(fullName);

    for (const child of cmd.children) {
      traverse(child, fullName);
    }
  }

  for (const cmd of commands) {
    traverse(cmd);
  }

  return names;
}

/**
 * Extract all variables (@var and @const) from document text
 */
function extractVariables(text: string): string[] {
  const variables: string[] = [];
  const lines = text.split(/\r?\n/);

  for (const line of lines) {
    const trimmed = line.trim();

    // Match @var NAME = ... or @const NAME = ...
    const varMatch = trimmed.match(/^@(?:var|const)\s+([A-Za-z0-9_]+)\s*=/);
    if (varMatch) {
      variables.push(varMatch[1]);
    }
  }

  return variables;
}

/**
 * Find the command that contains the given line
 */
function findCommandAtLine(
  document: vscode.TextDocument,
  position: vscode.Position,
  commands: NestfileCommand[]
): NestfileCommand | null {
  const targetLine = position.line;
  const lines = document.getText().split(/\r?\n/);

  function findInCommand(cmd: NestfileCommand): NestfileCommand | null {
    // Check if this command's line is after the target line
    if (cmd.line > targetLine) {
      return null;
    }

    // First check nested commands (they have higher priority)
    for (const child of cmd.children) {
      if (child.line <= targetLine) {
        const found = findInCommand(child);
        if (found) {
          return found;
        }
      }
    }

    // Check if target line is within this command's scope
    // Look for the next command at the same or lower indent level
    const cmdIndent = (lines[cmd.line].match(/^(\s*)/)?.[1]?.length || 0);
    let commandEndLine = lines.length - 1;

    // Find the end of this command by looking for next command at same/lower indent
    for (let i = cmd.line + 1; i < lines.length; i++) {
      const currentLine = lines[i];
      if (!currentLine.trim() || currentLine.trim().startsWith("#")) {
        continue;
      }

      const indent = (currentLine.match(/^(\s*)/)?.[1]?.length || 0);

      // If we find a non-directive line at same or lower indent, this command ends before it
      if (indent <= cmdIndent && !currentLine.trim().startsWith(">")) {
        commandEndLine = i - 1;
        break;
      }
    }

    // Check if target line is within this command's range
    if (targetLine >= cmd.line && targetLine <= commandEndLine) {
      return cmd;
    }

    return null;
  }

  // Search through all top-level commands
  for (const cmd of commands) {
    const found = findInCommand(cmd);
    if (found) {
      return found;
    }
  }

  return null;
}

/**
 * Get parameter names from the current command context
 */
function getCommandParameters(
  document: vscode.TextDocument,
  position: vscode.Position,
  commands: NestfileCommand[]
): string[] {
  const command = findCommandAtLine(document, position, commands);
  if (command) {
    return command.parameters.map(p => p.name.startsWith("!") ? p.name.substring(1) : p.name);
  }
  return [];
}

/**
 * Find all .env files in the workspace
 */
async function findEnvFiles(document: vscode.TextDocument): Promise<vscode.CompletionItem[]> {
  const envFiles: vscode.CompletionItem[] = [];

  try {
    // Find all .env files in workspace (including .env.local, .env.production, etc.)
    // and *.env files (config.env, production.env, etc.)
    const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri) ||
      vscode.workspace.workspaceFolders?.[0];

    const patterns = workspaceFolder
      ? [
        new vscode.RelativePattern(workspaceFolder, "**/.env*"),
        new vscode.RelativePattern(workspaceFolder, "**/*.env")
      ]
      : ["**/.env*", "**/*.env"];

    // Find files matching both patterns
    const filePromises = patterns.map(pattern =>
      vscode.workspace.findFiles(pattern, "**/node_modules/**", 50)
    );
    const fileArrays = await Promise.all(filePromises);
    const allFiles = fileArrays.flat();

    // Remove duplicates
    const uniqueFiles = Array.from(new Map(allFiles.map(f => [f.fsPath, f])).values());

    for (const file of uniqueFiles) {
      const fileName = file.path.split("/").pop() || file.path.split("\\").pop() || "";
      // Include files that start with .env or end with .env (but not directories)
      if ((fileName.startsWith(".env") || fileName.endsWith(".env")) && !fileName.endsWith("/")) {
        let relativePath: string;

        try {
          // Use VS Code API to get relative path
          relativePath = vscode.workspace.asRelativePath(file, false);
        } catch {
          // Fallback: manual calculation
          if (workspaceFolder) {
            const workspacePath = workspaceFolder.uri.fsPath;
            const filePath = file.fsPath;
            if (filePath.startsWith(workspacePath)) {
              relativePath = filePath.substring(workspacePath.length + 1).replace(/\\/g, "/");
            } else {
              relativePath = fileName;
            }
          } else {
            // Fallback to relative path from document
            const docDir = vscode.Uri.joinPath(document.uri, "..");
            const docDirPath = docDir.fsPath;
            const filePath = file.fsPath;
            if (filePath.startsWith(docDirPath)) {
              relativePath = "./" + filePath.substring(docDirPath.length + 1).replace(/\\/g, "/");
            } else {
              relativePath = fileName;
            }
          }
        }

        const item = new vscode.CompletionItem(fileName, vscode.CompletionItemKind.File);
        item.documentation = `Environment file: ${relativePath}`;
        item.insertText = relativePath;
        item.detail = relativePath;
        item.sortText = "0" + fileName; // Sort .env files first
        envFiles.push(item);
      }
    }
  } catch (error) {
    // Silently fail if we can't find files
    console.error("Error finding .env files:", error);
  }

  return envFiles;
}

/**
 * Check if cursor is inside a template variable {{...}}
 */
function isInsideTemplate(text: string, offset: number): { inside: boolean; start?: number; end?: number } {
  // Find the nearest {{ before cursor
  let lastOpen = -1;
  let lastClose = -1;

  // Search backwards from cursor position
  for (let i = offset - 1; i >= 1; i--) {
    // Check for closing }}
    if (text[i] === "}" && text[i - 1] === "}") {
      if (lastClose === -1) {
        lastClose = i - 1;
      }
      i--; // Skip the previous }
      continue;
    }
    // Check for opening {{
    if (text[i] === "{" && text[i - 1] === "{") {
      lastOpen = i - 1;
      break;
    }
  }

  // If we found an opening {{ and either:
  // 1. No closing }} found yet (we're inside an unclosed template)
  // 2. The cursor is before the closing }} (we're inside a closed template)
  if (lastOpen >= 0) {
    if (lastClose === -1 || offset <= lastClose + 2) {
      return {
        inside: true,
        start: lastOpen + 2,
        end: lastClose >= 0 ? lastClose : text.length
      };
    }
  }

  return { inside: false };
}

export class NestfileCompletionProvider implements vscode.CompletionItemProvider {
  async provideCompletionItems(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken,
    context: vscode.CompletionContext
  ): Promise<vscode.CompletionItem[] | vscode.CompletionList | undefined> {
    const line = document.lineAt(position);
    const lineText = line.text;
    const textBeforeCursor = lineText.substring(0, position.character);
    const fullText = document.getText();
    const offset = document.offsetAt(position);

    const completions: vscode.CompletionItem[] = [];

    // Check if we're inside a template variable {{...}}
    const templateInfo = isInsideTemplate(fullText, offset);
    if (templateInfo.inside) {
      const commands = validateNestfileDocument(fullText, { returnAst: true }, document.uri).commands;
      const variables = extractVariables(fullText);
      const parameters = getCommandParameters(document, position, commands);

      // Add variables
      for (const variable of variables) {
        const item = new vscode.CompletionItem(variable, vscode.CompletionItemKind.Variable);
        item.documentation = `Variable: ${variable}`;
        item.insertText = variable;
        completions.push(item);
      }

      // Add parameters
      for (const param of parameters) {
        const item = new vscode.CompletionItem(param, vscode.CompletionItemKind.Field);
        item.documentation = `Parameter: ${param}`;
        item.insertText = param;
        completions.push(item);
      }

      return completions;
    }

    // Check if we're inside a command (for directive suggestions)
    const commands = validateNestfileDocument(fullText, { returnAst: true }, document.uri).commands;
    const currentCommand = findCommandAtLine(document, position, commands);
    const trimmedBefore = textBeforeCursor.trim();
    const lineIndent = (textBeforeCursor.match(/^(\s*)/)?.[1]?.length || 0);

    // If we're inside a command and on a line with indent (inside command body)
    if (currentCommand && lineIndent > 0) {
      const contentAfterIndent = textBeforeCursor.substring(lineIndent);
      const contentAfterIndentTrimmed = contentAfterIndent.trim();

      // Offer directives when line doesn't start with ">" (handled by directiveMatch below)
      // Always offer on empty line or when user explicitly invokes completion
      if (!contentAfterIndentTrimmed.startsWith(">")) {
        const indentSpaces = " ".repeat(lineIndent);
        const typedText = contentAfterIndentTrimmed.toLowerCase();

        // Always offer directives inside command - VS Code will filter by what user typed
        for (const directive of DIRECTIVES) {

          const item = new vscode.CompletionItem(
            "> " + directive.name,
            vscode.CompletionItemKind.Property
          );
          item.documentation = directive.description;
          item.sortText = "1" + directive.name; // Sort before other completions
          item.filterText = "> " + directive.name; // For better filtering

          // Add appropriate snippet with proper indentation
          if (directive.name === "script") {
            item.insertText = new vscode.SnippetString(indentSpaces + "> script: |\n" + indentSpaces + "    ${1:echo \"Hello\"}");
          } else if (directive.name === "depends") {
            item.insertText = new vscode.SnippetString(indentSpaces + "> depends: ${1:command1}, ${2:command2}");
          } else if (directive.name.startsWith("logs:")) {
            item.insertText = new vscode.SnippetString(
              indentSpaces + "> " + (directive.name === "logs:json"
                ? "logs:json ${1:./logs/{{now}}.json}"
                : "logs:txt ${1:./logs/{{now}}.txt}")
            );
          } else if (directive.name === "env") {
            item.insertText = new vscode.SnippetString(indentSpaces + "> env: ${1:KEY}=${2:value}");
          } else if (directive.name === "validate") {
            item.insertText = new vscode.SnippetString(indentSpaces + "> validate: ${1:param} matches /${2:regex}/");
          } else if (directive.name === "cwd") {
            item.insertText = new vscode.SnippetString(indentSpaces + "> cwd: ${1:./path}");
          } else if (directive.name === "if" || directive.name === "elif") {
            item.insertText = new vscode.SnippetString(indentSpaces + "> " + directive.name + ": ${1:condition}");
          } else if (directive.name === "else" || directive.name === "privileged" || directive.name === "require_confirm") {
            item.insertText = indentSpaces + "> " + directive.name;
          } else {
            item.insertText = new vscode.SnippetString(indentSpaces + "> " + directive.name + ": ${1:value}");
          }

          completions.push(item);
        }

        // Return early if we found directive completions
        if (completions.length > 0) {
          return completions;
        }
      }
    }

    // Check if we're at the start of a line (for meta commands and new commands)
    // Only show completions if line is empty or starts with comment and we're NOT inside a command
    if ((trimmedBefore.length === 0 || trimmedBefore.startsWith("#")) && !currentCommand) {
      for (const meta of META_COMMANDS) {
        const item = new vscode.CompletionItem(meta.name, vscode.CompletionItemKind.Keyword);
        item.documentation = meta.description;

        if (meta.name === "@var") {
          item.insertText = new vscode.SnippetString("@var ${1:NAME} = \"${2:value}\"");
        } else if (meta.name === "@const") {
          item.insertText = new vscode.SnippetString("@const ${1:NAME} = \"${2:value}\"");
        } else if (meta.name === "@function") {
          item.insertText = new vscode.SnippetString("@function ${1:name}(${2:param}: ${3:str}):\n    ${4:// body}");
        } else if (meta.name === "@include") {
          item.insertText = new vscode.SnippetString("@include ${1:path/to/file.nest} from ${2:command}");
        }

        completions.push(item);
      }

      // Also offer command names if we're at the start
      const commandNames = extractCommandNames(commands);

      for (const cmdName of commandNames) {
        const item = new vscode.CompletionItem(cmdName, vscode.CompletionItemKind.Function);
        item.documentation = `Command: ${cmdName}`;
        item.insertText = cmdName;
        completions.push(item);
      }
    }

    // Check if we're typing a directive with modifier in brackets (e.g., "> script[" or "> depends[")
    const directiveWithBracketMatch = textBeforeCursor.match(/^(\s*>\s*)([a-zA-Z_]+)\[([a-zA-Z_]*)$/);
    if (directiveWithBracketMatch) {
      const indentBefore = directiveWithBracketMatch[1];
      const directiveName = directiveWithBracketMatch[2].toLowerCase();
      const modifierPrefix = directiveWithBracketMatch[3].toLowerCase();
      const indentSpaces = indentBefore.replace(/>\s*$/, "").replace(/>$/, "");

      // Script-like directives support [hide] modifier
      const scriptDirectives = ["script", "before", "after", "fallback", "finaly"];

      if (scriptDirectives.includes(directiveName)) {
        const modifiers = ["hide"];

        for (const modifier of modifiers) {
          if (modifier.toLowerCase().startsWith(modifierPrefix)) {
            const item = new vscode.CompletionItem(
              modifier,
              vscode.CompletionItemKind.Property
            );
            item.documentation = `Hide output for ${directiveName} directive`;
            item.filterText = directiveName + "[" + modifier;
            item.insertText = modifier + "]";
            completions.push(item);
          }
        }
      }

      // depends directive supports [parallel] modifier
      if (directiveName === "depends") {
        const modifiers = ["parallel"];

        for (const modifier of modifiers) {
          if (modifier.toLowerCase().startsWith(modifierPrefix)) {
            const item = new vscode.CompletionItem(
              modifier,
              vscode.CompletionItemKind.Property
            );
            item.documentation = "Run dependencies in parallel";
            item.filterText = directiveName + "[" + modifier;
            item.insertText = modifier + "]";
            completions.push(item);
          }
        }
      }

      if (completions.length > 0) {
        return completions;
      }
    }

    // Check if we're typing after @include for "into" or "from"
    const includeMatch = textBeforeCursor.match(/^(\s*)@include\s+[^@]+?(\s.*?)$/);
    if (includeMatch && !currentCommand) {
      const afterPath = includeMatch[2];
      const keywords = [];

      if (!afterPath.includes(" into ")) {
        keywords.push({ name: "into", desc: "Import into a specific group" });
      }
      if (!afterPath.includes(" from ")) {
        keywords.push({ name: "from", desc: "Import specific commands or groups" });
      }

      for (const kw of keywords) {
        const item = new vscode.CompletionItem(kw.name, vscode.CompletionItemKind.Keyword);
        item.documentation = kw.desc;
        item.insertText = kw.name + " ";
        completions.push(item);
      }

      if (completions.length > 0) return completions;
    }

    // Check if we're typing AFTER "from" in @include
    const fromMatch = textBeforeCursor.match(/^(\s*)@include\s+(.+?)(?:\s+into\s+[a-zA-Z0-9_]+)?\s+from\s+(.*)$/);
    if (fromMatch && !currentCommand) {
      let filePathStr = fromMatch[2].trim();

      // Remove quotes if present
      if ((filePathStr.startsWith('"') && filePathStr.endsWith('"')) ||
        (filePathStr.startsWith("'") && filePathStr.endsWith("'"))) {
        filePathStr = filePathStr.substring(1, filePathStr.length - 1);
      }

      // Suggest commands from that file
      try {
        const currentDir = path.dirname(document.uri.fsPath);
        const absolutePath = path.isAbsolute(filePathStr)
          ? filePathStr
          : path.resolve(currentDir, filePathStr);

        if (fs.existsSync(absolutePath)) {
          // Read and parse
          const content = fs.readFileSync(absolutePath, 'utf-8');
          const parsed = validateNestfileDocument(content, { returnAst: true });
          // extractCommandNames gives us "cmd", "group", "group sub"
          const commands = extractCommandNames(parsed.commands);

          for (const cmd of commands) {
            // Convert spaces to dots: "group sub" -> "group.sub"
            const dotNotation = cmd.replace(/\s+/g, ".");

            const item = new vscode.CompletionItem(dotNotation, vscode.CompletionItemKind.Reference);
            item.documentation = `Import from ${filePathStr}`;
            item.sortText = "0" + dotNotation; // Priority

            completions.push(item);
          }

          if (completions.length > 0) return completions;
        }
      } catch (e) {
        console.error("Error reading included file:", e);
      }
    }

    // Check if we're typing a directive (after "> " or ">")
    const directiveMatch = textBeforeCursor.match(/^(\s*>\s*)([a-zA-Z_:-]*)$/);
    if (directiveMatch) {
      const indentBefore = directiveMatch[1];
      const prefix = directiveMatch[2].toLowerCase();
      const indentSpaces = indentBefore.replace(/>\s*$/, "").replace(/>$/, "");

      for (const directive of DIRECTIVES) {
        // Match exact directive name or prefix
        const directiveLower = directive.name.toLowerCase();
        if (prefix.length === 0 || directiveLower.startsWith(prefix) || directiveLower.includes(prefix)) {
          const item = new vscode.CompletionItem(
            directive.name,
            vscode.CompletionItemKind.Property
          );
          item.documentation = directive.description;
          item.filterText = "> " + directive.name;

          // Add appropriate snippet based on directive type with proper indentation
          if (directive.name === "script") {
            item.insertText = new vscode.SnippetString(indentSpaces + "> script: |\n" + indentSpaces + "    ${1:echo \"Hello\"}");
          } else if (directive.name === "depends") {
            item.insertText = new vscode.SnippetString(indentSpaces + "> depends: ${1:command1}, ${2:command2}");
          } else if (directive.name.startsWith("logs:")) {
            item.insertText = new vscode.SnippetString(
              indentSpaces + "> " + (directive.name === "logs:json"
                ? "logs:json ${1:./logs/{{now}}.json}"
                : "logs:txt ${1:./logs/{{now}}.txt}")
            );
          } else if (directive.name === "env") {
            item.insertText = new vscode.SnippetString(indentSpaces + "> env: ${1:KEY}=${2:value}");
          } else if (directive.name === "validate") {
            item.insertText = new vscode.SnippetString(indentSpaces + "> validate: ${1:param} matches /${2:regex}/");
          } else if (directive.name === "cwd") {
            item.insertText = new vscode.SnippetString(indentSpaces + "> cwd: ${1:./path}");
          } else if (directive.name === "if" || directive.name === "elif") {
            item.insertText = new vscode.SnippetString(indentSpaces + "> " + directive.name + ": ${1:condition}");
          } else if (directive.name === "else" || directive.name === "privileged" || directive.name === "require_confirm") {
            item.insertText = indentSpaces + "> " + directive.name;
          } else {
            item.insertText = new vscode.SnippetString(indentSpaces + "> " + directive.name + ": ${1:value}");
          }

          completions.push(item);
        }
      }
    }

    // Check if we're typing after "depends:" - suggest command names
    const dependsMatch = textBeforeCursor.match(/^(\s*>\s*depends:\s*)(.*)$/);
    if (dependsMatch) {
      const commands = validateNestfileDocument(fullText, { returnAst: true }, document.uri).commands;
      const commandNames = extractCommandNames(commands);
      const existing = dependsMatch[2].split(",").map(s => s.trim()).filter(s => s);

      for (const cmdName of commandNames) {
        if (!existing.includes(cmdName)) {
          const item = new vscode.CompletionItem(cmdName, vscode.CompletionItemKind.Function);
          item.documentation = `Command: ${cmdName}`;
          item.insertText = cmdName;
          completions.push(item);
        }
      }
    }

    // Check if we're typing after "env:" - suggest .env files
    const envMatch = textBeforeCursor.match(/^(\s*>\s*env:\s*)(.*)$/);
    if (envMatch) {
      const typedValue = envMatch[2].trim();

      // Only suggest .env files if user hasn't typed "=" (meaning it's not a KEY=VALUE format)
      if (!typedValue.includes("=")) {
        const envFiles = await findEnvFiles(document);

        // Filter by what user typed
        const filterPrefix = typedValue.toLowerCase();
        for (const envFile of envFiles) {
          const fileName = envFile.label as string;
          const detail = envFile.detail as string || fileName;

          if (filterPrefix.length === 0 ||
            fileName.toLowerCase().includes(filterPrefix) ||
            detail.toLowerCase().includes(filterPrefix)) {
            completions.push(envFile);
          }
        }
      }
    }

    // Check if we're typing parameter types (after ": ")
    const paramTypeMatch = textBeforeCursor.match(/:\s*([a-z]*)$/);
    if (paramTypeMatch) {
      const prefix = paramTypeMatch[1].toLowerCase();

      for (const type of PARAMETER_TYPES) {
        if (type.startsWith(prefix)) {
          const item = new vscode.CompletionItem(type, vscode.CompletionItemKind.TypeParameter);
          item.documentation = `Parameter type: ${type}`;
          item.insertText = type;
          completions.push(item);
        }
      }
    }

    // Check if we're typing a template variable {{
    const templateMatch = textBeforeCursor.match(/\{\{([a-zA-Z0-9_]*)$/);
    if (templateMatch) {
      const builtins = [
        { name: "now", desc: "Current timestamp (RFC3339)" },
        { name: "user", desc: "Current user name" },
        { name: "env", desc: "Environment variables map" },
        { name: "cwd", desc: "Current working directory" }
      ];

      for (const builtin of builtins) {
        const item = new vscode.CompletionItem(builtin.name, vscode.CompletionItemKind.Constant);
        item.documentation = builtin.desc;
        item.insertText = builtin.name;
        // Trigger suggestion list
        completions.push(item);
      }
      return completions;
    }

    return completions.length > 0 ? completions : undefined;
  }
}

