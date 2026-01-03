import * as vscode from "vscode";
import { validateNestfileDocument } from "./validator";
import { NestfileCompletionProvider } from "./completion";
import { NestfileDefinitionProvider } from "./definition";
import { NestfileHoverProvider } from "./hover";
import { NestfileDocumentSymbolProvider } from "./symbols";
import { NestfileReferenceProvider } from "./references";
import { NestfileCodeActionProvider } from "./codeActions";
import { NestfileFormatter } from "./formatting";
import { NestfileCodeLensProvider } from "./codeLens";
import { NestfileInlayHintsProvider } from "./inlayHints";

export function activate(context: vscode.ExtensionContext) {
  // Create diagnostic collection for nestfile validation errors
  const diagnostics = vscode.languages.createDiagnosticCollection("nestfile");
  context.subscriptions.push(diagnostics);

  // Debounce timer for validation to avoid excessive validation on every keystroke
  const validationTimers = new Map<string, NodeJS.Timeout>();

  const validateActiveDocument = (doc: vscode.TextDocument | undefined) => {
    if (!doc || doc.languageId !== "nestfile") {
      return;
    }

    const uri = doc.uri.toString();

    // Clear existing timer for this document
    const existingTimer = validationTimers.get(uri);
    if (existingTimer) {
      clearTimeout(existingTimer);
    }

    // Debounce validation: wait 300ms after last change
    const timer = setTimeout(() => {
      try {
        const result = validateNestfileDocument(doc.getText(), {}, doc.uri);
        diagnostics.set(doc.uri, result);
      } catch (error) {
        // If validation fails, show error in output
        console.error("Validation error:", error);
        diagnostics.set(doc.uri, []);
      } finally {
        validationTimers.delete(uri);
      }
    }, 300);

    validationTimers.set(uri, timer);
  };

  // Validate on open and change
  context.subscriptions.push(
    vscode.workspace.onDidOpenTextDocument((doc) => {
      // Immediate validation on open (no debounce needed)
      if (doc.languageId === "nestfile") {
        try {
          const result = validateNestfileDocument(doc.getText(), {}, doc.uri);
          diagnostics.set(doc.uri, result);
        } catch (error) {
          console.error("Validation error:", error);
          diagnostics.set(doc.uri, []);
        }
      }
    }),
    vscode.workspace.onDidChangeTextDocument((e) => {
      // Debounced validation on change
      validateActiveDocument(e.document);
    }),
    vscode.workspace.onDidCloseTextDocument((doc) => {
      if (doc.languageId === "nestfile") {
        // Clear timer if document is closed
        const uri = doc.uri.toString();
        const timer = validationTimers.get(uri);
        if (timer) {
          clearTimeout(timer);
          validationTimers.delete(uri);
        }
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
      try {
        const result = validateNestfileDocument(editor.document.getText(), {}, editor.document.uri);
        diagnostics.set(editor.document.uri, result);
      } catch (error) {
        console.error("Validation error:", error);
        diagnostics.set(editor.document.uri, []);
      }
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
      const ast = validateNestfileDocument(text, { returnAst: true }, editor.document.uri);
      const output = vscode.window.createOutputChannel("Nestfile AST");
      output.clear();
      output.appendLine(JSON.stringify(ast, null, 2));
      output.show(true);
    }
  );

  const runCommand = vscode.commands.registerCommand(
    "nestfile.runCommand",
    (commandName: string) => {
      const terminal = vscode.window.terminals.find(t => t.name === "Nest") || vscode.window.createTerminal("Nest");
      terminal.show();
      terminal.sendText(`nest ${commandName}`);
    }
  );

  context.subscriptions.push(validateCommand, showAstCommand, runCommand);

  // Register completion provider
  const completionProvider = vscode.languages.registerCompletionItemProvider(
    "nestfile",
    new NestfileCompletionProvider(),
    ">", // Trigger on > (for directives)
    "@", // Trigger on @ (for meta commands)
    ":", // Trigger on : (for directive values and parameter types)
    "{", // Trigger on { (for template variables {{...}})
    "["  // Trigger on [ (for directive modifiers like [hide])
  );
  context.subscriptions.push(completionProvider);

  // Register definition provider (for @include and other definitions)
  const definitionProvider = vscode.languages.registerDefinitionProvider(
    "nestfile",
    new NestfileDefinitionProvider()
  );
  context.subscriptions.push(definitionProvider);

  // Register hover provider
  const hoverProvider = vscode.languages.registerHoverProvider(
    "nestfile",
    new NestfileHoverProvider()
  );
  context.subscriptions.push(hoverProvider);

  // Register document symbols provider (Outline)
  const documentSymbolProvider = vscode.languages.registerDocumentSymbolProvider(
    "nestfile",
    new NestfileDocumentSymbolProvider()
  );
  context.subscriptions.push(documentSymbolProvider);

  // Register reference provider (Find References)
  const referenceProvider = vscode.languages.registerReferenceProvider(
    "nestfile",
    new NestfileReferenceProvider()
  );
  context.subscriptions.push(referenceProvider);

  // Register code actions provider (Quick Fixes)
  const codeActionProvider = vscode.languages.registerCodeActionsProvider(
    "nestfile",
    new NestfileCodeActionProvider(),
    {
      providedCodeActionKinds: [vscode.CodeActionKind.QuickFix]
    }
  );
  context.subscriptions.push(codeActionProvider);

  // Register document formatter - DISABLED to preserve original formatting
  // const formatter = vscode.languages.registerDocumentFormattingEditProvider(
  //   "nestfile",
  //   new NestfileFormatter()
  // );
  // context.subscriptions.push(formatter);

  // Register code lens provider (shows reference counts)
  const codeLensProvider = vscode.languages.registerCodeLensProvider(
    "nestfile",
    new NestfileCodeLensProvider()
  );
  context.subscriptions.push(codeLensProvider);

  // Register inlay hints provider
  if (vscode.languages.registerInlayHintsProvider) {
    const inlayHintsProvider = vscode.languages.registerInlayHintsProvider(
      "nestfile",
      new NestfileInlayHintsProvider()
    );
    context.subscriptions.push(inlayHintsProvider);
  }

  // Initial validation for already-open document
  validateActiveDocument(vscode.window.activeTextEditor?.document);
}

export function deactivate() {
  // Nothing to clean up explicitly
}


