## Nestfile Language Support (VS Code)

Language support for Nest task runner configuration files (`Nestfile`, `nestfile`, `nest`, `*.nest`).

Features:

- Syntax highlighting based on the real Nestfile grammar from the CLI
- Basic inline validation (diagnostics) for common mistakes
- Snippets for typical Nestfile patterns (commands, groups, variables, functions, includes, directives)

### Supported syntax

The extension understands the main Nestfile constructs:

- Commands and groups: `command(...)` / `group:` with nested commands
- Parameters and types: `str`, `bool`, `num`, `arr`, positional and named (`!name|n`)
- Directives: `> desc:`, `> cwd:`, `> env:`, `> script:`, `> before:`, `> after:`,
  `> fallback:`, `> finaly:`, `> depends:`, `> validate:`, `> if:`, `> elif:`, `> else:`,
  `> logs:json`, `> logs:txt`, `> privileged`, `> require_confirm:`
- Meta directives: `@var`, `@const`, `@function`, `@include`
- Templates and substitutions: `{{...}}`, `$(...)` in values and scripts

For a full language reference see the main CLI README in the Nest repository.

### Validation in the editor

The extension runs a lightweight text validator for files with language `nestfile`:

- Duplicate parameter names in a single command
- Invalid parameter types (anything except `str`, `bool`, `num`, `arr`)
- Suspicious aliases (empty, length > 1, or conflicting with reserved short options)
- Commands without `> script` and without children (warning)
- Group commands with both children and `> script` (informational)
- Unknown directives (`> something:` that does not exist in the Nest syntax)
- Invalid `> logs:` format (expects `logs:json <path>` or `logs:txt <path>`)
- Obvious syntax issues in `@var`, `@const`, `@function`, `@include`
- Likely unclosed `$(...)` substitutions in directive values

You can also trigger validation manually via the command palette:

- **Nestfile: Validate Current File** (`nestfile.validate`)

For debugging the internal parse tree there is a helper command:

- **Nestfile: Show Parsed Commands (Debug)** (`nestfile.showAst`) – opens an output
  channel with a JSON representation of the parsed command tree.

### Snippets

The extension contributes snippets for:

- Command with parameters and script
- Group with `default` subcommand
- Global `@var` / `@const`
- `@include` and `@function`
- `> depends:`, `> validate:`, `> logs:json`, conditional `if/elif/else` block

Type the snippet prefix (for example `nest-command`, `nest-group`, `nest-var`, `nest-include`)
and accept the suggestion to insert a template.

### How to build and test the extension

1. Install dependencies:

   ```bash
   cd /home/user/git/nest/vscode-nestfile-support
   npm install
   ```

2. Open the extension folder in VS Code:

   - `File` → `Open Folder...` → select `vscode-nestfile-support`

3. Run the extension in a development host:

   - Press `F5` or run the **Run Extension** launch configuration.
   - A new VS Code window will open with the extension loaded.

4. Test on real Nestfile examples:

   - In the extension host window, open files from the CLI repo, for example:
     - `/home/user/git/nest/cli/examples/database.nest`
     - `/home/user/git/nest/cli/examples/docker.nest`
     - `/home/user/git/nest/cli/examples/testing.nest`
     - `/home/user/git/nest/cli/examples/nestfile`
   - Check that:
     - Syntax highlighting works for commands, directives, `@var/@const/@function/@include`,
       templates `{{...}}`, and substitutions `$(...)`.
     - Diagnostics appear for obvious issues (try introducing an invalid type or
       duplicate parameter).

5. Manual validation:

   - Open the command palette (`Ctrl+Shift+P` / `Cmd+Shift+P`).
   - Run **Nestfile: Validate Current File** to force a validation pass.

6. Optional – watch mode during development:

   ```bash
   npm run watch
   ```

   Rebuilds the TypeScript sources on change; reload the extension host window
   to pick up changes.

### Notes

- This extension performs only static text-based validation and does not execute
  any scripts or external commands.
- For full runtime validation, you can still run the Nest CLI (`nest --show ast`,
  `nest --show json`, regular command execution) in a terminal alongside the editor.


