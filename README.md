## Nestfile Language Support (VS Code)

Language support for Nest task runner configuration files (`Nestfile`, `nestfile`, `nest`, `*.nest`).

**[Install from VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=quonaro.vscode-nestfile-support)**

Features:

- Syntax highlighting based on the real Nestfile grammar from the CLI
- Inline validation (diagnostics) for common mistakes
- Autocompletion for directives, commands, variables, parameters, and template variables
- Hover information for directives, variables, and commands
- Go to Definition (F12) for variables, commands, and @include files
- Find References (Shift+F12) for variables and commands
- Quick Fixes (Code Actions) for common errors
- Document formatting
- Document Symbols (Outline navigation - Ctrl+Shift+O)
- Code Lens showing reference counts
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
- Invalid `> env:` format (expects `KEY=VALUE` or `.env` file path)
- Missing `@include` files (warning if included file doesn't exist)
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

### Language Features

#### Autocompletion

- **Directives**: Type `>` to get suggestions for all available directives
- **Meta commands**: Type `@` to get suggestions for `@var`, `@const`, `@function`, `@include`
- **Template variables**: Type `{{` to get suggestions for variables and command parameters
- **Command names**: Get suggestions when typing in `depends:` directives
- **Parameter types**: Get suggestions for `str`, `bool`, `num`, `arr` types
- **Environment files**: Get `.env` file suggestions when typing `> env:`

#### Hover Information

- Hover over directives to see their descriptions
- Hover over template variables `{{VAR}}` to see their values
- Hover over command names in `depends:` to see command descriptions

#### Navigation

- **Go to Definition (F12)**: 
  - Jump to variable definition from `{{VAR}}`
  - Jump to command definition from `depends: command`
  - Open included files from `@include path.nest`
- **Find References (Shift+F12)**: Find all usages of variables and commands
- **Outline (Ctrl+Shift+O)**: Navigate document structure - see all commands, variables, and functions

#### Quick Fixes

- Fix invalid parameter types (suggest correct type)
- Add missing `> script:` directive
- Fix typos in directive names (suggest correct directive)

#### Formatting

- Format entire document (Shift+Alt+F)
- Consistent indentation (4 spaces)
- Proper alignment of directives

#### Code Lens

- See reference counts above variable and command definitions

### File Display and Icons Configuration

#### Local Configuration via settings.json

To configure nest file display locally in your project, create or update `.vscode/settings.json`:

```json
{
  // Associate nest files with nestfile language for syntax highlighting
  "files.associations": {
    "Nestfile": "nestfile",
    "nestfile": "nestfile",
    "nest": "nestfile",
    "*.nest": "nestfile"
  },
  
  // Select Nestfile Icons theme for custom file icons (optional)
  "workbench.iconTheme": "nest-icons",
  
  // Editor settings for nest files
  "[nestfile]": {
    "editor.insertSpaces": true,
    "editor.tabSize": 4,
    "editor.detectIndentation": false,
    "files.eol": "\n"
  }
}
```

#### Global Configuration via VS Code Settings UI

Alternatively, you can configure this globally:

1. Open VS Code Settings (`Ctrl+,` / `Cmd+,`)
2. Search for "File Icon Theme"
3. Select **"Nestfile Icons"** from the dropdown

This will apply custom icons to:
- Files with `.nest` extension
- Files named `Nestfile`, `nestfile`, or `nest`

**Note**: The `files.associations` setting ensures proper language recognition and syntax highlighting. The icon theme is optional but provides enhanced visual appearance.

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


