# 🪺 Nest - Task Runner for CLI Commands

**⚠️ MVP Version** - This is a Minimum Viable Product. I actively use this tool in my daily work and will continue to maintain and improve it. This project also serves as my learning journey in Rust programming.

## 📋 About

Nest is a declarative task runner that replaces brittle `Makefile`s and scattered shell scripts with a unified, readable, and maintainable task orchestration system. It's especially suited for polyglot, self-hosted, or automation-heavy projects.

## 🚀 Quick Start

1. Build the project:
```bash
cargo build --release
```

2. Create a `Nestfile` in your project root (see `nestfile.example` for reference)

3. Run commands:
```bash
./target/release/nest <command>
```

## 📝 Writing Nestfile

### Basic Command Structure

A Nestfile consists of commands with parameters, directives, and nested subcommands:

```nest
command_name(param: type = default):
    > desc: Description of the command
    > cwd: ./working/directory
    > env: VARIABLE_NAME=value
    > env: .env.local
    > script: |
        #!/bin/sh
        echo "Running command..."
        ./script.sh {{param}}
```

### Command Parameters

Parameters are defined in the function signature:

```nest
build(target: str = "x86_64", release: bool = false):
    > desc: Build the project
    > script: cargo build --target {{target}} ${release:+--release}
```

**Parameter Types:**
- `str` - String value
- `bool` - Boolean flag (true/false)
- `num` - Numeric value
- `arr` - Array of strings

**Parameter Features:**
- **Positional arguments**: `name: str` - passed without `--` prefix
- **Named arguments**: `!name|n: str` - use `!` prefix to make it named (uses `--name` or `-n`)
- Required parameters: `name: str` (no default value)
- Optional parameters: `name: str = "default"` (with default value)
- Aliases: `force|f: bool = false` or `!force|f: bool = false` (use `--force` or `-f`)

**Usage Examples:**

**Positional Arguments:**
```nest
greet(name: str, message: str):
    > desc: Greet someone with a message
    > script: echo "Hello {{name}}, {{message}}"
```
```bash
nest greet "Alice" "welcome!"
# Output: Hello Alice, welcome!
```

**Named Arguments (with `!` prefix):**
```nest
deploy(version: str, !env|e: str = "production", !force|f: bool = false):
    > desc: Deploy application
    > script: |
        #!/bin/sh
        echo "Deploying {{version}} to {{env}}"
        if [ "{{force}}" = "true" ]; then
            ./deploy.sh --force --env {{env}} {{version}}
        else
            ./deploy.sh --env {{env}} {{version}}
        fi
```
```bash
nest deploy "v1.2.3" --env staging
nest deploy "v1.2.3" -e staging --force true
nest deploy "v1.2.3"  # env defaults to "production"
```

**Mixed: Positional + Named:**
```nest
copy(source: str, !destination|d: str, !overwrite|o: bool = false):
    > desc: Copy file with optional overwrite
    > script: |
        #!/bin/sh
        if [ "{{overwrite}}" = "true" ]; then
            cp -f "{{source}}" "{{destination}}"
        else
            cp "{{source}}" "{{destination}}"
        fi
```
```bash
nest copy "file.txt" --destination "backup.txt"
nest copy "file.txt" -d "backup.txt" -o true
```

**Boolean Flags:**
```bash
nest build --target aarch64-apple-darwin --release true
nest build --target x86_64  # release defaults to false
```

### Directives

Directives control command behavior:

- **`> desc:`** - Command description (shown in help)
- **`> cwd:`** - Working directory for script execution
- **`> env:`** - Environment variables:
  - Direct assignment: `> env: NODE_ENV=production`
  - Load from file: `> env: .env.local`
- **`> script:`** - Script to execute:
  - Single line: `> script: echo "Hello"`
  - Multiline: `> script: |` (followed by indented script block)

### Nested Commands

Group related commands under a namespace:

```nest
dev:
    > desc: Development tools

    default(!hot|h: bool = false):
        > desc: Start dev server
        > env: NODE_ENV=development
        > script: |
            #!/bin/sh
            if [ "{{hot}}" = "true" ]; then
                nodemon src/index.js
            else
                node src/index.js
            fi

    lint(!fix|f: bool = false):
        > desc: Lint code
        > script: eslint src/ ${fix:+--fix}
```

**Usage:**
```bash
nest dev                    # Runs default subcommand
nest dev --hot true         # Pass named argument to default
nest dev -h true            # Use short alias
nest dev lint               # Run lint subcommand
nest dev lint --fix true    # Run lint with fix flag
nest dev lint -f true       # Use short alias
```

### Template Variables

Use `{{variable}}` syntax in scripts:

- **Parameters**: `{{param}}` - Replaced with parameter value
- **Special variables**:
  - `{{now}}` - Current UTC time in RFC3339 format
  - `{{user}}` - Current user (from `$USER` environment variable)

**Example:**
```nest
deploy(version: str):
    > desc: Deploy application
    > env: DEPLOYER={{user}}
    > env: BUILD_TIME={{now}}
    > script: |
        #!/bin/sh
        echo "Deploying {{version}} by {{user}} at {{now}}"
        ./deploy.sh {{version}}
```

### Complete Example

See `nestfile.example` for a complete working example with:
- Multiple command types
- Nested command groups
- Parameter types (str, bool, num, arr)
- **Positional and named arguments**
- Environment variable management
- Multiline scripts

## ✨ Supported Features

### Currently Implemented

✅ **Command Structure**
- Top-level commands
- Nested subcommands
- Default subcommands for groups
- Command parameters with types (str, bool, num, arr)
- **Positional arguments** (without `--` prefix)
- **Named arguments** (with `!` prefix, uses `--name` or `-n`)
- Parameter aliases
- Default parameter values

✅ **Directives**
- `> desc:` - Command descriptions
- `> cwd:` - Working directory
- `> env:` - Environment variables (direct assignment and .env files)
- `> script:` - Single-line and multiline scripts

✅ **Template Processing**
- Parameter substitution: `{{param}}`
- Special variables: `{{now}}`, `{{user}}`
- Template processing in scripts

✅ **CLI Features**
- Dynamic CLI generation from Nestfile
- Help system
- JSON output (`--show json`)
- AST output (`--show ast`)
- Version info (`--version`)

✅ **Execution**
- Script execution with environment variables
- Working directory support
- Environment variable loading from .env files

### Not Yet Implemented (Future Plans)

❌ Functions (`@func`) - Reusable script blocks
❌ Lifecycle hooks (`@before`, `@after`, `@fallback`)
❌ Command dependencies (`@depends`)
❌ Function calls (`@call`)
❌ Advanced templating (environment variable fallbacks)

## 📁 File Convention

- **Filename**: `Nestfile` (no extension)
- **Location**: Project root directory
- **Example**: See `nestfile.example` in this repository

## 🛠️ Development Status

This is an **MVP (Minimum Viable Product)** version. I actively use this tool in my projects and will continue to maintain and improve it. This project also serves as my learning journey in Rust programming.

**Current Focus:**
- Stability and bug fixes
- Learning Rust best practices
- Adding features as needed for my use cases

## 📄 License

This project is licensed under the **Creative Commons Attribution-NonCommercial 4.0 International License (CC BY-NC 4.0)**.

This means:
- ✅ You can use, modify, and distribute this software
- ✅ You must give appropriate credit
- ❌ **You cannot use this software for commercial purposes** (selling, commercial products, etc.)

For full license details, see the [LICENSE](LICENSE) file.

---

> 💡 **Goal**: Replace brittle `Makefile`s and scattered shell scripts with a unified, readable, composable, and maintainable task orchestration system.
