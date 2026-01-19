import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";

export interface ValidateOptions {
  returnAst?: boolean;
}

export interface NestfileCommand {
  name: string;
  line: number;
  parameters: NestfileParameter[];
  directives: NestfileDirective[];
  children: NestfileCommand[];
}

export interface NestfileParameter {
  name: string;
  type: string;
  alias?: string;
  line: number;
}

export interface NestfileDirective {
  name: string;
  raw: string;
  line: number;
}

const VALID_PARAM_TYPES = new Set(["str", "bool", "num", "arr"]);

const VALID_DIRECTIVES = new Set([
  "desc",
  "cwd",
  "env",
  "script",
  "before",
  "after",
  "fallback",
  "finally",
  "depends",
  "validate",

  "logs",
  "privileged",
  "require_confirm",
]);

const RESERVED_SHORT_OPTIONS = new Set(["h", "V", "v", "n", "c"]);

export function validateNestfileDocument(
  text: string,
  options: ValidateOptions & { returnAst: true },
  documentUri?: vscode.Uri
): { commands: NestfileCommand[] };
export function validateNestfileDocument(
  text: string,
  options?: ValidateOptions,
  documentUri?: vscode.Uri
): vscode.Diagnostic[];
export function validateNestfileDocument(
  text: string,
  options: ValidateOptions = {},
  documentUri?: vscode.Uri
): vscode.Diagnostic[] | { commands: NestfileCommand[] } {
  const lines = text.split(/\r?\n/);
  const commands: NestfileCommand[] = [];
  const diagnostics: vscode.Diagnostic[] = [];

  interface StackItem {
    indent: number;
    command: NestfileCommand;
  }

  const stack: StackItem[] = [];

  const getIndent = (line: string): number => {
    let spaces = 0;
    for (const ch of line) {
      if (ch === " ") {
        spaces++;
      } else {
        break;
      }
    }
    return Math.floor(spaces / 4);
  };

  const currentParent = (indent: number): NestfileCommand | null => {
    for (let i = stack.length - 1; i >= 0; i--) {
      if (stack[i].indent < indent) {
        return stack[i].command;
      }
    }
    return null;
  };

  for (let i = 0; i < lines.length; i++) {
    const rawLine = lines[i];
    const lineNumber = i;
    const trimmed = rawLine.trim();

    if (!trimmed || trimmed.startsWith("#")) {
      continue;
    }

    const indent = getIndent(rawLine);

    // Check for directives (must be checked before commands if they conflict, but directives are reserved)
    // We check if the line starts with a known directive name
    const directiveNameMatch = trimmed.match(/^([a-zA-Z0-9_]+(?:\.[a-zA-Z0-9_]+)?)/);
    let isDirective = false;

    if (directiveNameMatch) {
      let name = directiveNameMatch[1];
      // Split by dot to check base name
      const dotIndex = name.indexOf(".");
      const directiveBase = dotIndex !== -1 ? name.substring(0, dotIndex) : name;

      if (VALID_DIRECTIVES.has(directiveBase)) {
        isDirective = true;
      }
    }

    if (isDirective) {
      const colonIndex = trimmed.indexOf(":");
      let name = "";
      let value = "";

      if (colonIndex !== -1) {
        name = trimmed.substring(0, colonIndex).trim();
        value = trimmed.substring(colonIndex + 1).trim();
      } else {
        // Should not happen if regex matched but good for safety
        name = trimmed.trim();
      }

      const parent = currentParent(indent);
      if (!parent) {
        diagnostics.push(
          createDiagnostic(
            lineNumber,
            0,
            rawLine.length,
            "Directive is not inside a command",
            vscode.DiagnosticSeverity.Warning
          )
        );
      } else {
        parent.directives.push({
          name,
          raw: trimmed,
          line: lineNumber,
        });
      }

      // env format check
      const keyParts = name.split(".");
      const directiveBase = keyParts[0];
      const modifiers = keyParts.slice(1);

      if (directiveBase === "env" && value) {
        const trimmedValue = value.trim();

        // Check if it's in KEY=VALUE format (with optional ${VAR} substitutions)
        // Examples: NODE_ENV=production, KEY=${VAR}, KEY=${VAR:-default}
        const keyValuePattern = /^[A-Za-z_][A-Za-z0-9_]*=.*$/;

        // Check if it looks like a .env file path
        // Examples: .env, .env.local, config.env, ./path/.env, path/to/file.env
        const envFilePattern = /^(\.env|\.\/.*\.env|.*\/\.env|.*\.env)$/;

        // Empty value is also invalid
        if (!trimmedValue) {
          const valueStart = rawLine.indexOf(":");
          const valueEnd = rawLine.length;

          diagnostics.push(
            createDiagnostic(
              lineNumber,
              valueStart >= 0 ? valueStart + 1 : 0,
              valueEnd,
              'Invalid env directive. Expected format: "env: KEY=VALUE" or "env: .env" or "env: path/to/file.env".',
              vscode.DiagnosticSeverity.Error
            )
          );
        } else if (!keyValuePattern.test(trimmedValue) && !envFilePattern.test(trimmedValue)) {
          // Find the position of the value after "env:"
          const colonIndex = rawLine.indexOf(":");
          const valueStart = colonIndex >= 0 ? colonIndex + 1 : rawLine.indexOf(trimmedValue);
          const valueEnd = valueStart >= 0 ? valueStart + trimmedValue.length : rawLine.length;

          diagnostics.push(
            createDiagnostic(
              lineNumber,
              Math.max(0, valueStart),
              valueEnd,
              'Invalid env directive. Expected format: "env: KEY=VALUE" or "env: .env" or "env: path/to/file.env".',
              vscode.DiagnosticSeverity.Error
            )
          );
        }
      }

      // logs format check
      if (directiveBase === "logs") {
        if (name !== "logs.json" && name !== "logs.txt") {
          diagnostics.push(
            createDiagnostic(
              lineNumber,
              0,
              rawLine.length,
              'Invalid logs directive. Expected "logs.json <path>" or "logs.txt <path>".',
              vscode.DiagnosticSeverity.Error
            )
          );
        } else if (!value) {
          diagnostics.push(
            createDiagnostic(
              lineNumber,
              0,
              rawLine.length,
              'Invalid logs directive. Path is required.',
              vscode.DiagnosticSeverity.Error
            )
          );
        }
      }

      // validate format check
      if (directiveBase === "validate") {
        if (modifiers.length === 0) {
          // Check for "target matches regex" or "target in [...]"
          if (!value.includes(" matches ") && !value.includes(" in ")) {
            diagnostics.push(
              createDiagnostic(
                lineNumber,
                0,
                rawLine.length,
                'Invalid validate syntax. Expected "validate: target matches regex" or "validate: target in [...]" or "validate.PARAM: regex".',
                vscode.DiagnosticSeverity.Error
              )
            );
          }
        }
      }

      // Basic detection of unclosed $(...) substitutions in value
      if (value.includes("$(") && !hasBalancedSubstitutions(value)) {
        diagnostics.push(
          createDiagnostic(
            lineNumber,
            0,
            rawLine.length,
            "Unclosed command substitution \"$(...)\" in directive value.",
            vscode.DiagnosticSeverity.Warning
          )
        );
      }

      // Check for multiline script directives (script, before, after, fallback, finally)
      const multilineDirectives = new Set(["script", "before", "after", "fallback", "finally"]);
      if (multilineDirectives.has(directiveBase)) {
        const baseIndentSpaces = rawLine.match(/^(\s*)/)?.[1]?.length || 0;

        if (value === "|") {
          // Check if the multiline block is empty
          let hasContent = false;
          const expectedIndentSpaces = baseIndentSpaces + 4; // One level deeper (4 spaces)

          // Look ahead to find content in the multiline block
          for (let j = i + 1; j < lines.length; j++) {
            const nextLine = lines[j];
            const nextTrimmed = nextLine.trim();
            const nextIndentSpaces = nextLine.match(/^(\s*)/)?.[1]?.length || 0;

            // If we hit a line with indent <= base indent and it's not empty, block ended
            if (nextIndentSpaces <= baseIndentSpaces && nextTrimmed && !nextTrimmed.startsWith("#")) {
              break;
            }

            // If we hit an empty line at base indent, block ended
            if (nextIndentSpaces === baseIndentSpaces && !nextTrimmed) {
              break;
            }

            // If we have a line with proper indent and content, block has content
            if (nextIndentSpaces >= expectedIndentSpaces && nextTrimmed && !nextTrimmed.startsWith("#")) {
              hasContent = true;
              break;
            }
          }

          if (!hasContent) {
            const pipeIndex = rawLine.indexOf("|");
            const pipeEnd = pipeIndex >= 0 ? pipeIndex + 1 : rawLine.length;

            diagnostics.push(
              createDiagnostic(
                lineNumber,
                pipeIndex >= 0 ? pipeIndex : rawLine.length - 1,
                pipeEnd,
                "Multiline script block is empty. Add script content after '|' or use single-line format without '|'.",
                vscode.DiagnosticSeverity.Error
              )
            );
          }
        } else {
          // Single line format - check if there are indented lines after (missing | for multiline)
          if (i + 1 < lines.length) {
            const nextLine = lines[i + 1];
            const nextTrimmed = nextLine.trim();
            const nextIndentSpaces = nextLine.match(/^(\s*)/)?.[1]?.length || 0;
            const expectedIndentSpaces = baseIndentSpaces + 4; // One level deeper (4 spaces)

            // If next line has greater indent and is not empty/comment/directive, it looks like multiline without |
            if (nextIndentSpaces >= expectedIndentSpaces &&
              nextTrimmed &&
              !nextTrimmed.startsWith("#")) {

              // Simple check if it's not another directive or command
              const isOtherDirective = VALID_DIRECTIVES.has(nextTrimmed.split(':')[0].split('.')[0]);
              if (!isOtherDirective) {
                const colonIndex = rawLine.indexOf(":");
                const directiveEnd = colonIndex >= 0 ? colonIndex + 1 : rawLine.length;

                diagnostics.push(
                  createDiagnostic(
                    lineNumber,
                    colonIndex >= 0 ? colonIndex : rawLine.length - 1,
                    directiveEnd,
                    `Multiline script detected but missing '|' after '${directiveBase}:'. Add '|' for multiline scripts or put script content on the same line.`,
                    vscode.DiagnosticSeverity.Error
                  )
                );
              }
            }
          }
        }
      }

      continue;
    }

    // Command or group definition
    const commandMatch = trimmed.match(
      /^([A-Za-z0-9_]+)\s*(\((.*)\))?\s*:\s*$/
    );
    if (commandMatch) {
      const name = commandMatch[1];
      const paramsStr = commandMatch[3] ?? "";

      const parameters: NestfileParameter[] = [];
      if (paramsStr.trim().length > 0) {
        const paramParts = splitTopLevel(paramsStr, ",");
        for (const part of paramParts) {
          const parsed = parseParameter(part.trim(), lineNumber, rawLine);
          if (parsed) {
            parameters.push(parsed);
          } else {
            diagnostics.push(
              createDiagnostic(
                lineNumber,
                0,
                rawLine.length,
                `Invalid parameter syntax "${part.trim()}"`,
                vscode.DiagnosticSeverity.Error
              )
            );
          }
        }
      }

      const cmd: NestfileCommand = {
        name,
        line: lineNumber,
        parameters,
        directives: [],
        children: [],
      };

      // Attach to parent based on indent
      const parent = currentParent(indent);
      if (parent) {
        parent.children.push(cmd);
      } else {
        commands.push(cmd);
      }

      // Pop stack items with indent >= current
      while (stack.length && stack[stack.length - 1].indent >= indent) {
        stack.pop();
      }
      stack.push({ indent, command: cmd });

      // Parameter-level validation
      validateParameters(cmd, diagnostics, lineNumber, rawLine);
      continue;
    }

    // var / const / function / import syntax sanity checks
    if (trimmed.startsWith("var ")) {
      if (!trimmed.includes("=")) {
        diagnostics.push(
          createDiagnostic(
            lineNumber,
            0,
            rawLine.length,
            "Invalid var definition. Expected format: var NAME = value.",
            vscode.DiagnosticSeverity.Error
          )
        );
      }
    } else if (trimmed.startsWith("const ")) {
      if (!trimmed.includes("=")) {
        diagnostics.push(
          createDiagnostic(
            lineNumber,
            0,
            rawLine.length,
            "Invalid const definition. Expected format: const NAME = value.",
            vscode.DiagnosticSeverity.Error
          )
        );
      }
    } else if (trimmed.startsWith("function ")) {
      if (!trimmed.includes(":")) {
        diagnostics.push(
          createDiagnostic(
            lineNumber,
            0,
            rawLine.length,
            "Invalid function definition. Expected format: function name(params):",
            vscode.DiagnosticSeverity.Error
          )
        );
      }
    } else if (trimmed.startsWith("import ")) {
      const parts = trimmed.split(/\s+/);
      // Regex for "import SYMBOL from FILE [into GROUP]"
      const fromMatch = trimmed.match(/^import\s+(\S+)\s+from\s+(\S+)(?:\s+into\s+(\S+))?$/);

      if (fromMatch) {
        const symbol = fromMatch[1];
        const importPath = fromMatch[2];

        if (documentUri && importPath) {
          const dir = path.dirname(documentUri.fsPath);
          const absolutePath = path.resolve(dir, importPath);

          if (!fs.existsSync(absolutePath)) {
            diagnostics.push(
              createDiagnostic(
                lineNumber,
                trimmed.indexOf(importPath),
                trimmed.indexOf(importPath) + importPath.length,
                `Imported file not found: ${importPath}`,
                vscode.DiagnosticSeverity.Error
              )
            );
          } else if (fs.statSync(absolutePath).isFile()) {
            // File exists, check for symbol if it's not a wildcard
            if (symbol !== "*") {
              try {
                const content = fs.readFileSync(absolutePath, "utf-8");
                const importedLines = content.split(/\r?\n/);
                let symbolFound = false;

                // Simple scan for top-level commands: ^NAME(...)?:
                const escapedSymbol = symbol.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
                const cmdRegex = new RegExp(`^${escapedSymbol}\\s*(\\(.*\\))?\\s*:\\s*$`);

                for (const l of importedLines) {
                  if (cmdRegex.test(l)) {
                    symbolFound = true;
                    break;
                  }
                }

                if (!symbolFound) {
                  diagnostics.push(
                    createDiagnostic(
                      lineNumber,
                      trimmed.indexOf(symbol),
                      trimmed.indexOf(symbol) + symbol.length,
                      `Symbol "${symbol}" not found in ${importPath}`,
                      vscode.DiagnosticSeverity.Error
                    )
                  );
                }
              } catch (e) {
                // Ignore read errors
              }
            }
          }
        }
      } else {
        // Fallback for "import FILE" syntax
        // import FILE
        if (parts.length === 2 && !trimmed.includes(" from ")) {
          const importPath = parts[1];
          if (documentUri && importPath) {
            const dir = path.dirname(documentUri.fsPath);
            const absolutePath = path.resolve(dir, importPath);
            if (!fs.existsSync(absolutePath)) {
              diagnostics.push(
                createDiagnostic(
                  lineNumber,
                  trimmed.indexOf(importPath),
                  trimmed.indexOf(importPath) + importPath.length,
                  `Imported file not found: ${importPath}`,
                  vscode.DiagnosticSeverity.Error
                )
              );
            }
          }
        } else {
          // Invalid format
          diagnostics.push(
            createDiagnostic(
              lineNumber,
              0,
              rawLine.length,
              "Invalid import. Expected format: import <file> or import <symbol> from <file> [into <group>]",
              vscode.DiagnosticSeverity.Warning
            )
          );
        }
      }
    }
  }


  // Final pass to check for undefined variables in the entire document
  validateUndefinedVariables(text, diagnostics, lines);

  // Command-level checks after full pass
  for (const cmd of commands) {
    validateCommandTree(cmd, diagnostics, lines);
  }

  if (options.returnAst) {
    return { commands };
  }

  return diagnostics;
}

function splitTopLevel(text: string, separator: string): string[] {
  const result: string[] = [];
  let current = "";
  let depth = 0;
  let inQuotes = false;
  let quoteChar = "";

  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (inQuotes) {
      current += ch;
      if (ch === quoteChar) {
        inQuotes = false;
      }
      continue;
    }

    if (ch === "'" || ch === '"') {
      inQuotes = true;
      quoteChar = ch;
      current += ch;
      continue;
    }

    if (ch === "(") {
      depth++;
      current += ch;
      continue;
    }
    if (ch === ")") {
      depth = Math.max(0, depth - 1);
      current += ch;
      continue;
    }

    if (ch === separator && depth === 0) {
      result.push(current);
      current = "";
      continue;
    }

    current += ch;
  }

  if (current.trim().length > 0) {
    result.push(current);
  }

  return result;
}

function parseParameter(
  text: string,
  line: number,
  rawLine: string
): NestfileParameter | null {
  if (!text) {
    return null;
  }

  // Wildcards: we treat them as type arr and skip strict syntax checks here
  if (text.startsWith("*")) {
    return {
      name: text,
      type: "arr",
      line,
    };
  }

  // Format: [!]name|alias: type = default
  const typeSep = text.indexOf(":");
  if (typeSep === -1) {
    return null;
  }

  let namePart = text.substring(0, typeSep).trim();
  const typeAndDefault = text.substring(typeSep + 1).trim();

  let isNamed = false;
  if (namePart.startsWith("!")) {
    isNamed = true;
    namePart = namePart.substring(1).trim();
  }

  let name = namePart;
  let alias: string | undefined;
  const aliasIndex = namePart.indexOf("|");
  if (aliasIndex !== -1) {
    name = namePart.substring(0, aliasIndex).trim();
    alias = namePart.substring(aliasIndex + 1).trim();
  }

  let paramType = typeAndDefault;
  const eqIndex = typeAndDefault.indexOf("=");
  if (eqIndex !== -1) {
    paramType = typeAndDefault.substring(0, eqIndex).trim();
  }

  if (!name) {
    return null;
  }

  return {
    name: isNamed ? `!${name}` : name,
    type: paramType,
    alias,
    line,
  };
}

function validateParameters(
  cmd: NestfileCommand,
  diagnostics: vscode.Diagnostic[],
  lineNumber: number,
  rawLine: string
) {
  const seenNames = new Set<string>();

  for (const param of cmd.parameters) {
    const cleanName = param.name.startsWith("!")
      ? param.name.substring(1)
      : param.name;

    if (seenNames.has(cleanName)) {
      diagnostics.push(
        createDiagnostic(
          param.line,
          0,
          rawLine.length,
          `Duplicate parameter name "${cleanName}" in command "${cmd.name}".`,
          vscode.DiagnosticSeverity.Error
        )
      );
    } else {
      seenNames.add(cleanName);
    }

    if (!VALID_PARAM_TYPES.has(param.type)) {
      // Find the position of the type in the parameter string
      const typeMatch = rawLine.match(new RegExp(`\\b${param.type.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\b`));
      const typeStart = typeMatch ? typeMatch.index || 0 : 0;
      const typeEnd = typeStart + param.type.length;

      diagnostics.push(
        createDiagnostic(
          param.line,
          typeStart,
          typeEnd,
          `Invalid parameter type "${param.type}" for parameter "${cleanName}". Expected one of: str, bool, num, arr.`,
          vscode.DiagnosticSeverity.Error
        )
      );
    }

    if (param.alias !== undefined) {
      if (!param.alias) {
        diagnostics.push(
          createDiagnostic(
            param.line,
            0,
            rawLine.length,
            `Empty alias for parameter "${cleanName}".`,
            vscode.DiagnosticSeverity.Warning
          )
        );
      } else if (param.alias.length !== 1) {
        diagnostics.push(
          createDiagnostic(
            param.line,
            0,
            rawLine.length,
            `Alias "${param.alias}" for parameter "${cleanName}" must be a single character.`,
            vscode.DiagnosticSeverity.Warning
          )
        );
      } else if (RESERVED_SHORT_OPTIONS.has(param.alias)) {
        diagnostics.push(
          createDiagnostic(
            param.line,
            0,
            rawLine.length,
            `Alias "${param.alias}" for parameter "${cleanName}" conflicts with a reserved short option.`,
            vscode.DiagnosticSeverity.Warning
          )
        );
      }
    }
  }
}

function validateCommandTree(
  cmd: NestfileCommand,
  diagnostics: vscode.Diagnostic[],
  lines: string[]
) {
  const hasScript = cmd.directives.some(
    (d) => d.name === "script" || d.name.startsWith("script.") || d.name === "script[hide]"
  );

  if (!hasScript && cmd.children.length === 0) {
    diagnostics.push(
      createDiagnostic(
        cmd.line,
        0,
        lines[cmd.line]?.length ?? 0,
        `Command "${cmd.name}" has no script directive.`,
        vscode.DiagnosticSeverity.Warning
      )
    );
  }

  if (hasScript && cmd.children.length > 0) {
    diagnostics.push(
      createDiagnostic(
        cmd.line,
        0,
        lines[cmd.line]?.length ?? 0,
        `Group command "${cmd.name}" has a script directive. Group commands typically do not need scripts.`,
        vscode.DiagnosticSeverity.Information
      )
    );
  }

  for (const child of cmd.children) {
    validateCommandTree(child, diagnostics, lines);
  }
}

function hasBalancedSubstitutions(value: string): boolean {
  let depth = 0;
  let i = 0;
  while (i < value.length) {
    if (value[i] === "$" && i + 1 < value.length && value[i + 1] === "(") {
      depth++;
      i += 2;
      continue;
    }
    if (value[i] === ")" && depth > 0) {
      depth--;
    }
    i++;
  }
  return depth === 0;
}

function createDiagnostic(
  line: number,
  startChar: number,
  endChar: number,
  message: string,
  severity: vscode.DiagnosticSeverity
): vscode.Diagnostic {
  // Ensure valid range (endChar should be >= startChar)
  const validEndChar = Math.max(startChar, endChar);
  const range = new vscode.Range(
    new vscode.Position(line, startChar),
    new vscode.Position(line, validEndChar)
  );
  const diagnostic = new vscode.Diagnostic(range, message, severity);
  diagnostic.source = "nestfile";

  // Add tags for better visibility (if needed)
  // diagnostic.tags = [vscode.DiagnosticTag.Unnecessary]; // Optional: mark as unnecessary

  return diagnostic;
}

function validateUndefinedVariables(
  text: string,
  diagnostics: vscode.Diagnostic[],
  lines: string[]
) {
  // 1. Extract all defined variables
  const definedVars = new Set<string>(["now", "user", "env", "cwd"]); // Built-in variables whitelist

  // From @var and @const (now var and const)
  for (const line of lines) {
    // Modified to match standard var/const without @
    const varMatch = line.match(/^\s*(?:var|const)\s+([A-Za-z0-9_]+)\s*=/);
    if (varMatch) {
      definedVars.add(varMatch[1]);
    }
    const envMatch = line.match(/^\s*env\s+([A-Za-z0-9_]+)\s*=/);
    if (envMatch) {
      definedVars.add(envMatch[1]);
    }
  }

  // From command parameters
  for (const line of lines) {
    const paramMatch = line.match(/\(([^\)]+)\)\s*:\s*$/);
    if (paramMatch) {
      const params = paramMatch[1].split(",");
      for (const p of params) {
        // Match both normal parameters, named params (!), and wildcards (*)
        // Example: !version, !force|f, *args, *[2]
        const trimmed = p.trim();
        if (trimmed.startsWith("!")) {
          // Named param: !force|f: bool -> force
          const namePart = trimmed.substring(1).split(":")[0].trim();
          const name = namePart.split("|")[0].trim();
          definedVars.add(name);
        } else if (trimmed.startsWith("*")) {
          // Wildcard: *args or * or *[2] -> keep * prefix
          // Strip [N] and :type if present (though wildcards shouldn't have types)
          const namePart = trimmed.split(":")[0].trim();
          const name = namePart.split("[")[0].trim();
          definedVars.add(name);
        } else {
          // Positional: version|v: str -> version
          const namePart = trimmed.split(":")[0].trim();
          const name = namePart.split("|")[0].trim();
          definedVars.add(name);
        }
      }
    }
  }

  // 2. Check all {{var}} usages
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    // Allow * prefix for wildcards
    const regex = /\{\{\s*(\*?[A-Za-z0-9_]+)\s*\}\}/g;
    let match;
    while ((match = regex.exec(line)) !== null) {
      const varName = match[1];
      if (!definedVars.has(varName)) {
        diagnostics.push(
          createDiagnostic(
            i,
            match.index + 2,
            match.index + 2 + varName.length,
            `Undefined variable "${varName}". Ensure it is defined with var/const or is a command parameter.`,
            vscode.DiagnosticSeverity.Error
          )
        );
      }
    }
  }
}


