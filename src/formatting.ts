import * as vscode from "vscode";

export class NestfileFormatter implements vscode.DocumentFormattingEditProvider {
  provideDocumentFormattingEdits(
    document: vscode.TextDocument,
    options: vscode.FormattingOptions,
    token: vscode.CancellationToken
  ): vscode.ProviderResult<vscode.TextEdit[]> {
    const edits: vscode.TextEdit[] = [];
    const lines = document.getText().split(/\r?\n/);
    const formattedLines: string[] = [];

    const indentSize = options.tabSize || 4;
    const useSpaces = options.insertSpaces !== false;
    const indentStack: number[] = [0]; // Track indent levels

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const trimmed = line.trim();

      // Preserve empty lines and comments as-is
      if (!trimmed || trimmed.startsWith("#")) {
        formattedLines.push(trimmed ? line : "");
        continue;
      }

      // Calculate current indent
      const currentIndent = (line.match(/^(\s*)/)?.[1]?.length || 0);
      
      // Determine expected indent
      let expectedIndent = 0;

      // Check if it's a command or group definition (ends with :)
      const isCommandDefinition = /^[A-Za-z0-9_]+\s*(\([^)]*\))?\s*:\s*$/.test(trimmed);
      
      // Check if it's a meta directive (@var, @const, @function, @include)
      const isMetaDirective = trimmed.startsWith("@var ") || 
                             trimmed.startsWith("@const ") ||
                             trimmed.startsWith("@function ") ||
                             trimmed.startsWith("@include ");

      // Check if it's a directive (starts with >)
      const isDirective = trimmed.startsWith(">");

      if (isMetaDirective) {
        // Meta directives are at root level
        expectedIndent = 0;
        indentStack.length = 1; // Reset stack
        indentStack[0] = 0;
      } else if (isCommandDefinition) {
        // Command definition: find parent indent
        // Look backwards for the last command at a lower indent level
        let parentIndent = 0;
        for (let j = i - 1; j >= 0; j--) {
          const prevLine = lines[j];
          const prevTrimmed = prevLine.trim();
          if (prevTrimmed && !prevTrimmed.startsWith("#")) {
            const prevIsCommand = /^[A-Za-z0-9_]+\s*(\([^)]*\))?\s*:\s*$/.test(prevTrimmed);
            if (prevIsCommand) {
              parentIndent = (prevLine.match(/^(\s*)/)?.[1]?.length || 0);
              break;
            }
          }
        }
        
        expectedIndent = parentIndent;
        
        // Update indent stack - commands create new indent level for their directives
        while (indentStack.length > 0 && indentStack[indentStack.length - 1] >= expectedIndent) {
          indentStack.pop();
        }
        indentStack.push(expectedIndent);
      } else if (isDirective) {
        // Directives are indented relative to their command
        expectedIndent = indentStack.length > 0 
          ? indentStack[indentStack.length - 1] + indentSize
          : indentSize;
      } else {
        // Keep original indent for other content
        expectedIndent = currentIndent;
      }

      // Create indent string
      const indentStr = useSpaces 
        ? " ".repeat(expectedIndent)
        : "\t".repeat(Math.floor(expectedIndent / indentSize));

      // Format the line with proper indentation
      const formattedLine = indentStr + trimmed;
      formattedLines.push(formattedLine);
    }

    // Create edit replacing the entire document
    const fullRange = new vscode.Range(
      document.positionAt(0),
      document.positionAt(document.getText().length)
    );

    const formattedText = formattedLines.join("\n");
    
    // Only create edit if something changed
    if (formattedText !== document.getText()) {
      edits.push(new vscode.TextEdit(fullRange, formattedText));
    }

    return edits;
  }
}

