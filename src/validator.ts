import * as vscode from "vscode";

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
  "finaly",
  "depends",
  "validate",
  "if",
  "elif",
  "else",
  "logs",
  "privileged",
  "require_confirm",
]);

const RESERVED_SHORT_OPTIONS = new Set(["h", "V", "v", "n", "c"]);

export function validateNestfileDocument(
  text: string
): vscode.Diagnostic[];
export function validateNestfileDocument(
  text: string,
  options: ValidateOptions & { returnAst: true }
): { commands: NestfileCommand[] };
export function validateNestfileDocument(
  text: string,
  options: ValidateOptions = {}
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

    // Directives
    if (trimmed.startsWith(">")) {
      const directiveLine = trimmed.substring(1).trim();
      const colonIndex = directiveLine.indexOf(":");

      let name = directiveLine;
      let value = "";

      if (colonIndex !== -1) {
        name = directiveLine.substring(0, colonIndex).trim();
        value = directiveLine.substring(colonIndex + 1).trim();
      }

      // Strip modifiers, e.g. script[hide]
      const modifierIndex = name.indexOf("[");
      if (modifierIndex !== -1) {
        name = name.substring(0, modifierIndex).trim();
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
          raw: directiveLine,
          line: lineNumber,
        });
      }

      // Unknown directive
      const directiveBase = name.startsWith("logs")
        ? "logs"
        : name === "else"
        ? "else"
        : name;
      if (!VALID_DIRECTIVES.has(directiveBase)) {
        // Find the position of the directive name after ">"
        const directiveMatch = trimmed.match(/^>\s*([a-zA-Z_]+(?:\[[^\]]+\])?)/);
        const directiveStart = directiveMatch 
          ? rawLine.indexOf(directiveMatch[1])
          : rawLine.indexOf(name);
        const directiveEnd = directiveStart >= 0 ? directiveStart + name.length : rawLine.length;
        
        diagnostics.push(
          createDiagnostic(
            lineNumber,
            Math.max(0, directiveStart),
            directiveEnd,
            `Unknown directive "${name}"`,
            vscode.DiagnosticSeverity.Error
          )
        );
      }

      // env format check
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
        const parts = value.split(/\s+/);
        if (parts.length < 2 || (parts[0] !== "json" && parts[0] !== "txt")) {
          diagnostics.push(
            createDiagnostic(
              lineNumber,
              0,
              rawLine.length,
              'Invalid logs directive. Expected "logs:json <path>" or "logs:txt <path>".',
              vscode.DiagnosticSeverity.Warning
            )
          );
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

      continue;
    }

    // @var / @const / @function / @include syntax sanity checks
    if (trimmed.startsWith("@var ")) {
      if (!trimmed.includes("=")) {
        diagnostics.push(
          createDiagnostic(
            lineNumber,
            0,
            rawLine.length,
            "Invalid @var definition. Expected format: @var NAME = value.",
            vscode.DiagnosticSeverity.Error
          )
        );
      }
    } else if (trimmed.startsWith("@const ")) {
      if (!trimmed.includes("=")) {
        diagnostics.push(
          createDiagnostic(
            lineNumber,
            0,
            rawLine.length,
            "Invalid @const definition. Expected format: @const NAME = value.",
            vscode.DiagnosticSeverity.Error
          )
        );
      }
    } else if (trimmed.startsWith("@function ")) {
      if (!trimmed.endsWith(":")) {
        diagnostics.push(
          createDiagnostic(
            lineNumber,
            0,
            rawLine.length,
            "Invalid @function definition. Expected trailing ':'.",
            vscode.DiagnosticSeverity.Error
          )
        );
      }
    } else if (trimmed.startsWith("@include ")) {
      const pathPart = trimmed.substring(9).trim();
      if (!pathPart) {
        diagnostics.push(
          createDiagnostic(
            lineNumber,
            0,
            rawLine.length,
            "Empty @include path.",
            vscode.DiagnosticSeverity.Error
          )
        );
      }
    }
  }

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
    (d) => d.name === "script" || d.name === "script[hide]"
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


