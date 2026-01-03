import * as vscode from "vscode";
import { validateNestfileDocument } from "./validator";

export class NestfileInlayHintsProvider implements vscode.InlayHintsProvider {
    provideInlayHints(
        document: vscode.TextDocument,
        range: vscode.Range,
        token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.InlayHint[]> {
        const inlayHints: vscode.InlayHint[] = [];
        const text = document.getText();
        const variables = this.extractVariables(text);

        for (let i = range.start.line; i <= range.end.line; i++) {
            const line = document.lineAt(i);
            const lineText = line.text;

            // Find all {{var}} in the line
            const regex = /\{\{\s*([A-Za-z0-9_]+)\s*\}\}/g;
            let match;
            while ((match = regex.exec(lineText)) !== null) {
                const varName = match[1];
                if (variables.has(varName)) {
                    const value = variables.get(varName)!;
                    const position = new vscode.Position(i, match.index + match[0].length);
                    const hint = new vscode.InlayHint(
                        position,
                        ` (${value})`
                    );
                    hint.tooltip = `Current value of ${varName}: ${value}`;
                    inlayHints.push(hint);
                }
            }
        }

        return inlayHints;
    }

    private extractVariables(text: string): Map<string, string> {
        const variables = new Map<string, string>();
        const lines = text.split(/\r?\n/);
        for (const line of lines) {
            const varMatch = line.match(/^\s*@(var|const)\s+([A-Za-z0-9_]+)\s*=\s*(.+)$/);
            if (varMatch) {
                variables.set(varMatch[2], varMatch[3].trim());
            }
        }
        return variables;
    }
}
