# 🪺 Nest - Task Runner for CLI Commands

**⚠️ MVP Version** - This is a Minimum Viable Product. I actively use this tool in my daily work and will continue to maintain and improve it. This project also serves as my learning journey in Rust programming.

## 📋 About

Nest is a declarative task runner that replaces brittle `Makefile`s and scattered shell scripts with a unified, readable, and maintainable task orchestration system. It's especially suited for polyglot, self-hosted, or automation-heavy projects.

## 🚀 Quick Start

### Installation

> **Note:** The install scripts are configured for the `quonaro/nest` repository. If you're using a fork, update the `REPO` variable in `install.sh` and `install.ps1`.

The install scripts will:
- Detect your OS and architecture automatically
- Download the latest release binary
- Install it to `~/.local/bin` (Unix) or `%USERPROFILE%\.local\bin` (Windows)
- Provide instructions if the install directory is not in your PATH

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/quonaro/nest/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/quonaro/nest/main/install.ps1 | iex
```

**Manual Installation:**

1. Download the latest release for your platform from [Releases](https://github.com/quonaro/nest/releases)
2. Extract the binary
3. Add it to your PATH

**From Source:**
```bash
git clone https://github.com/quonaro/nest.git
cd nest
cargo build --release
sudo cp target/release/nest /usr/local/bin/
```

**Note for older Linux distributions (e.g., Debian 12, Ubuntu 20.04):**

If you encounter a GLIBC version error (e.g., `GLIBC_2.39 not found`), the pre-built binaries may be incompatible with your system. In this case, compile from source:

1. Install Rust (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. Install build dependencies:
   ```bash
   # Debian/Ubuntu
   sudo apt-get update
   sudo apt-get install -y build-essential pkg-config libssl-dev
   ```

3. Clone and build:
   ```bash
   git clone https://github.com/quonaro/nest.git
   cd nest
   cargo build --release
   cp target/release/nest ~/.local/bin/
   ```

### Usage

1. Create a `Nestfile` in your project root (see `nestfile.example` for reference)

2. Run commands:
```bash
nest <command>
```

### Hidden Commands

Nest has several hidden commands that don't appear in `nest --help` but are available for advanced usage:

#### `--version` / `-V`
Print version information:
```bash
nest --version
# or
nest -V
```

#### `--show <format>`
Display commands in different formats. Requires a Nestfile to be present:
```bash
nest --show json    # Output commands as JSON
nest --show ast     # Output commands as Abstract Syntax Tree
```

#### `--example`
Download the example `nestfile.example` from GitHub and save it as `nestfile` in the current directory:
```bash
nest --example
```

This command:
- Downloads `nestfile.example` from the official repository
- Saves it as `nestfile` in the current directory
- Works without requiring an existing Nestfile
- Requires `curl` or `wget` to be available on your system

#### `update`
Update Nest CLI to the latest version:
```bash
nest update
```

This command:
- Automatically detects your OS and architecture
- Downloads the latest release from GitHub
- Replaces the binary in `~/.local/bin` (Unix) or `%USERPROFILE%\.local\bin` (Windows)
- Works without requiring an existing Nestfile
- Requires `curl` or `wget` to be available on your system

**Note:** If you get a "Text file busy" error, it means the binary is currently in use. Close the terminal session and run the update command again, or manually replace the binary using the instructions provided in the error message.

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
  - System variable with fallback: `> env: NODE_ENV=${NODE_ENV:-development}`
  - System variable: `> env: DATABASE_URL=${DATABASE_URL}`
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

### Environment Variables in Scripts

Environment variables set via `> env:` directives are available in scripts using standard shell syntax:

```nest
build():
    > env: NODE_ENV=production
    > env: PORT=3000
    > script: |
        echo "Building in $NODE_ENV mode"
        echo "Port: $PORT"
        npm run build
```

**System Environment Variables with Fallback:**

You can use system environment variables with fallback values:

```nest
build():
    > env: NODE_ENV=${NODE_ENV:-development}  # Uses system NODE_ENV or defaults to "development"
    > env: BUILD_NUMBER=${CI_BUILD_NUMBER:-local}  # Uses CI build number or "local"
    > script: |
        echo "Building in $NODE_ENV mode"
        echo "Build number: $BUILD_NUMBER"
        npm run build
```

**Syntax:**
- `${VAR:-default}` - Use system variable `VAR` if exists, otherwise use `default`
- `${VAR}` - Use system variable `VAR` if exists, otherwise empty string

**Priority:**
1. System environment variables (if set)
2. Variables from `.env` files
3. Direct assignments in `> env:` directives
4. Fallback values (for `${VAR:-default}` syntax)

### Variables and Constants

Define variables and constants at the top level (global) or inside commands (local):

```nest
# Global variables and constants
@var APP_NAME = "myapp"
@var VERSION = "1.0.0"
@const COMPANY_NAME = "My Company"

# Variables can be redefined (last definition wins)
@var APP_NAME = "production-app"  # Overrides previous definition

# Command with local variables and constants
build():
    # Local variable overrides global
    @var APP_NAME = "local-app"
    # Local constant overrides global
    @const COMPANY_NAME = "Local Company"
    # New local variable (only in this command)
    @var BUILD_DIR = "./build"
    > script: |
        echo "Building {{APP_NAME}} v{{VERSION}}"
        echo "Company: {{COMPANY_NAME}}"
        echo "Build dir: {{BUILD_DIR}}"
        npm run build
```

**Usage in scripts:**
```nest
build():
    > script: |
        echo "Building {{APP_NAME}} v{{VERSION}}"
        echo "Company: {{COMPANY_NAME}}"
        npm run build
```

**Priority order:**
1. Parameters (from command arguments) - highest priority
2. Local variables (from command) - override global variables
3. Local constants (from command) - override global constants
4. Global variables (can be redefined, last definition wins)
5. Global constants (cannot be redefined)
6. Special variables ({{now}}, {{user}}) - lowest priority

**Scope:**
- Global variables/constants: Available in all commands
- Local variables/constants: Only available in the command where they're defined
- Local variables/constants override global ones for that specific command

**Example: Overriding Global Variables Inside Commands**

```nest
# Global variables
@var APP_NAME = "global-app"
@var NODE_ENV = "development"
@const COMPANY = "Global Company"

# Command 1: Uses global variables
show_global():
    > script: |
        echo "APP_NAME: {{APP_NAME}}"      # Output: "global-app"
        echo "NODE_ENV: {{NODE_ENV}}"      # Output: "development"
        echo "COMPANY: {{COMPANY}}"        # Output: "Global Company"

# Command 2: Overrides global variables with local ones
show_local():
    # Local variables override global for this command only
    @var APP_NAME = "local-app"
    @var NODE_ENV = "production"
    @const COMPANY = "Local Company"
    > script: |
        echo "APP_NAME: {{APP_NAME}}"      # Output: "local-app" (local overrides global)
        echo "NODE_ENV: {{NODE_ENV}}"      # Output: "production" (local overrides global)
        echo "COMPANY: {{COMPANY}}"        # Output: "Local Company" (local overrides global)

# Command 3: Still uses global variables (no local overrides)
show_global_again():
    > script: |
        echo "APP_NAME: {{APP_NAME}}"      # Output: "global-app" (no local override)
        echo "NODE_ENV: {{NODE_ENV}}"      # Output: "development" (no local override)
```

**Key Points:**
- Local variables/constants defined inside a command override global ones **only for that command**
- Other commands without local definitions still use global variables/constants
- Local variables can be redefined multiple times (last definition wins)
- Local constants cannot be redefined within the same command

### Template Variables

Use `{{variable}}` syntax in scripts:

- **Parameters**: `{{param}}` - Replaced with parameter value
- **Variables**: `{{VAR}}` - Replaced with variable value (can be redefined)
- **Constants**: `{{CONST}}` - Replaced with constant value (cannot be redefined)
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

**Using Environment Variables in Scripts:**

Environment variables are available in scripts using `$VAR` syntax:

```nest
run_app():
    > env: NODE_ENV=production
    > env: PORT=3000
    > script: |
        echo "Starting app in $NODE_ENV mode on port $PORT"
        node server.js --port $PORT
```

**Combining Templates and Environment Variables:**

You can combine template variables and environment variables:

```nest
deploy(version: str):
    > env: NODE_ENV=${NODE_ENV:-production}
    > env: APP_VERSION={{version}}
    > script: |
        echo "Deploying version {{version}} to $NODE_ENV"
        echo "App version: $APP_VERSION"
        ./deploy.sh --version {{version}} --env $NODE_ENV
```

### Command Dependencies

You can specify dependencies between commands using the `> depends:` directive:

```nest
clean():
    > desc: Clean build artifacts
    > script: rm -rf build/

build():
    > desc: Build the project
    > depends: clean
    > script: npm run build

test():
    > desc: Run tests
    > depends: build
    > script: npm test

deploy():
    > desc: Deploy application
    > depends: build, test
    > script: npm run deploy
```

**Dependency Resolution:**
- Dependencies are executed **before** the main command
- Multiple dependencies can be specified (comma-separated)
- Dependencies are executed in the order specified
- Dependencies can have their own dependencies (recursive)
- Circular dependencies are detected and will cause an error

**Dependency Paths:**
- **Relative**: `clean` - relative to current command's parent
- **Absolute**: `dev:build` - absolute path from root (use `:` separator)

**Example:**
```bash
nest deploy
# Executes: clean -> build -> test -> deploy
```

**Circular Dependency Detection:**
```nest
a():
    > depends: b
    > script: echo "A"

b():
    > depends: a  # ERROR: Circular dependency detected
    > script: echo "B"
```

### Before, After, and Fallback Scripts

You can define scripts that run before, after, or as a fallback for the main script:

```nest
deploy():
    > desc: Deploy with before/after hooks and error handling
    > before: |
        echo "Setting up deployment environment..."
        # Pre-deployment checks
    > script: |
        echo "Deploying application..."
        # Main deployment logic
        # If this fails, fallback will execute
    > after: |
        echo "Deployment completed successfully"
        # Post-deployment tasks (only if main script succeeds)
    > fallback: |
        echo "Deployment failed, rolling back..."
        # Error handling (only if main script fails)
        # This replaces the error output
```

**Execution Order:**
1. `> before:` - Executed first (always)
2. `> script:` - Main script execution
3. If successful: `> after:` - Executed after success
4. If failed: `> fallback:` - Executed instead of error output

**Key Points:**
- All script directives (`before`, `after`, `fallback`, `script`) support multiline syntax with `|`
- `before` always executes, even if main script fails
- `after` only executes if main script succeeds
- `fallback` only executes if main script fails, and replaces the error output
- All scripts share the same environment variables and working directory

**Example with Error Handling:**
```nest
build():
    > before: echo "Starting build..."
    > script: |
        npm run build
        if [ $? -ne 0 ]; then
            exit 1
        fi
    > after: echo "Build completed successfully!"
    > fallback: |
        echo "Build failed, cleaning up..."
        rm -rf dist/
        echo "Cleanup complete"
```

### Complete Example

See `nestfile.example` for a complete working example with:
- Multiple command types
- Nested command groups
- Parameter types (str, bool, num, arr)
- **Positional and named arguments**
- Environment variable management
- Multiline scripts
- Command dependencies
- Before/after/fallback scripts
- Parameter validation with regex

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
- `> before:` - Script executed before the main script (single-line or multiline)
- `> after:` - Script executed after the main script succeeds (single-line or multiline)
- `> fallback:` - Script executed if the main script fails (replaces error output, single-line or multiline)
- `> depends:` - Command dependencies (executed before the command)
- `> validate:` - Parameter validation rules (regex patterns)

✅ **Variables and Constants**
- Global variables (`@var`) - Can be redefined (last definition wins)
- Global constants (`@const`) - Cannot be redefined
- Local variables (`@var` inside commands) - Override global for that command
- Local constants (`@const` inside commands) - Override global for that command
- Usage in templates: `{{VAR}}` or `{{CONST}}`

✅ **Template Processing**
- Parameter substitution: `{{param}}`
- Variable substitution: `{{VAR}}`
- Constant substitution: `{{CONST}}`
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

## 🔧 CI/CD Setup

### GitHub Actions Configuration

This project uses GitHub Actions for automated releases. To enable automatic builds and releases:

1. **Create a Personal Access Token (PAT):**
   - Go to [GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)](https://github.com/settings/tokens)
   - Click "Generate new token (classic)"
   - Give it a descriptive name (e.g., "Nest Release Token")
   - Select scope: `repo` (full control of private repositories)
   - Click "Generate token"
   - **Copy the token immediately** (you won't be able to see it again)

2. **Add Token to Repository Secrets:**
   - Go to your repository → Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Name: `NEST_TOKEN`
   - Value: Paste your personal access token
   - Click "Add secret"

3. **Workflow Triggers:**
   - The workflow automatically runs on push to `main` or `master` branch
   - It builds binaries for all platforms (Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64)
   - Creates a release with version from `Cargo.toml`
   - Uploads all binaries and SHA256 checksums

**Note:** The workflow uses `NEST_TOKEN` secret instead of the default `GITHUB_TOKEN` to have full control over releases. Make sure the token has `repo` scope enabled.

## 📄 License

This project is licensed under the **Creative Commons Attribution-NonCommercial 4.0 International License (CC BY-NC 4.0)**.

This means:
- ✅ You can use, modify, and distribute this software
- ✅ You must give appropriate credit
- ❌ **You cannot use this software for commercial purposes** (selling, commercial products, etc.)

For full license details, see the [LICENSE](LICENSE) file.

---

> 💡 **Goal**: Replace brittle `Makefile`s and scattered shell scripts with a unified, readable, composable, and maintainable task orchestration system.
