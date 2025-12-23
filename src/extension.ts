import * as vscode from "vscode";
import { validateNestfileDocument } from "./validator";

export function activate(context: vscode.ExtensionContext) {
  const diagnostics = vscode.languages.createDiagnosticCollection("nestfile");
  context.subscriptions.push(diagnostics);

  const validateActiveDocument = (doc: vscode.TextDocument | undefined) => {
    if (!doc || doc.languageId !== "nestfile") {
      return;
    }
    const result = validateNestfileDocument(doc.getText());
    diagnostics.set(doc.uri, result);
  };

  // Validate on open and change
  context.subscriptions.push(
    vscode.workspace.onDidOpenTextDocument((doc) => validateActiveDocument(doc)),
    vscode.workspace.onDidChangeTextDocument((e) =>
      validateActiveDocument(e.document)
    ),
    vscode.workspace.onDidCloseTextDocument((doc) => {
      if (doc.languageId === "nestfile") {
        diagnostics.delete(doc.uri);
      }
    })
  );

  // Validate current document on command
  const validateCommand = vscode.commands.registerCommand(
    "nestfile.validate",
    () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        return;
      }
      validateActiveDocument(editor.document);
    }
  );

  const showAstCommand = vscode.commands.registerCommand(
    "nestfile.showAst",
    () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== "nestfile") {
        return;
      }
      const text = editor.document.getText();
      const ast = validateNestfileDocument(text, { returnAst: true });
      const output = vscode.window.createOutputChannel("Nestfile AST");
      output.clear();
      output.appendLine(JSON.stringify(ast, null, 2));
      output.show(true);
    }
  );

  context.subscriptions.push(validateCommand, showAstCommand);

  // Initial validation for already-open document
  validateActiveDocument(vscode.window.activeTextEditor?.document);
}

export function deactivate() {
  // Nothing to clean up explicitly
}


