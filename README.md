# 🪺 Nest - Task Runner for CLI Commands

I actively use this tool in my daily work and will continue to maintain and improve it. This project also serves as my learning journey in Rust programming.

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

**Interactive Installation (Recommended):**

```bash
curl -fsSL https://raw.githubusercontent.com/quonaro/nest/main/install.sh | bash
```
The script will interactively ask for your preferences (e.g., glibc vs musl on Linux) if running in a terminal.

**Non-Interactive / CI Installation:**

You can pass arguments to the script to bypass prompts:

```bash
# Install specific version and target musl (static)
curl -fsSL https://raw.githubusercontent.com/quonaro/nest/main/install.sh | bash -s -- -T musl -V 0.15.9

# Install latest version, default libc (glibc on Linux)
curl -fsSL https://raw.githubusercontent.com/quonaro/nest/main/install.sh | bash -s -- -V latest
```

**Options:**
- `-T, --target <flavor>`: Target libc flavor (`glibc` or `musl`). Default: `glibc`.
- `-V, --version <ver>`: Target version (e.g., `0.15.9` or `latest`). Default: `latest`.

**Windows (PowerShell):**

```powershell
# Latest release
irm https://raw.githubusercontent.com/quonaro/nest/main/install.ps1 | iex

# Specific version (e.g. 0.1.0)
irm https://raw.githubusercontent.com/quonaro/nest/main/install.ps1 | iex; install.ps1 -Version 0.1.0
```

**Manual Installation:**

1. Download the latest release for your platform from [Releases](https://github.com/quonaro/nest/releases)
2. Extract the binary:
   - `nest-linux-x86_64.tar.gz` – Linux x86_64 glibc
   - `nest-linux-musl-x86_64.tar.gz` – Linux x86_64 static musl (no GLIBC dependency)
3. Add it to your PATH

**Binary size notes (approximate, may vary by release):**

- Linux x86_64 glibc:
  - Compressed archive: usually around **2–3 MB**
  - Unpacked binary: typically **3–5 MB**
- Linux x86_64 static musl:
  - Compressed archive: usually around **3–5 MB**
  - Unpacked binary: typically **6–9 MB**

Static musl builds are larger but are more portable (no GLIBC version dependency, work better on older/minimal distributions).

**From Source:**

```bash
git clone https://github.com/quonaro/nest.git
cd nest
cargo build --release
sudo cp target/release/nest /usr/local/bin/
```

### Shell Completion

Nest CLI automatically generates and installs shell completion for all supported shells (bash, zsh, fish, PowerShell, elvish). Completion is automatically set up when you run any `nest` command.

**Automatic Installation:**

- Completion scripts are generated automatically when your nestfile changes
- Installation happens automatically in your shell's configuration file
- Works for bash, zsh, fish, PowerShell, and elvish

**Manual Installation:**

```bash
# Generate and install completion for current shell
nest --complete zsh

# View completion script content
nest --complete zsh -V

# Generate for specific shell
nest --complete bash
nest --complete fish
```

**Completion Script Locations:**

- All completion scripts: `~/.cache/nest/completions/`
- Hash file (for change detection): `~/.cache/nest/completions/nestfile.hash`
- Shell configs: Automatically added to `.zshrc`, `.bashrc`, etc.

**Supported Shells:**

- **Bash**: Added to `~/.bashrc` or `~/.bash_profile`
- **Zsh**: Added to `~/.zshrc`
- **Fish**: Copied to `~/.config/fish/completions/nest.fish`
- **PowerShell**: Added to PowerShell profile
- **Elvish**: Added to `~/.elvish/rc.elv`

After installation, reload your shell configuration:

```bash
source ~/.zshrc  # for zsh
source ~/.bashrc  # for bash
# or simply restart your terminal
```

**Note for older Linux distributions (e.g., Debian 12, Ubuntu 20.04):**

Linux x86_64 релизные бинарники теперь собираются как **статически слинкованные (musl)**, поэтому ошибка вроде `GLIBC_2.39 not found` при установке через `install.sh` или из GitHub Releases не должна возникать.
Если вы хотите собрать такой же статический бинарник локально:

1. Install Rust (if not already installed):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. Install build dependencies:

   ```bash
   # Debian/Ubuntu
   sudo apt-get update
   sudo apt-get install -y build-essential pkg-config libssl-dev musl-tools
   ```

3. Add musl target and build static binary:
   ```bash
   git clone https://github.com/quonaro/nest.git
   cd nest
   rustup target add x86_64-unknown-linux-musl
   cargo build --release --target x86_64-unknown-linux-musl
   cp target/x86_64-unknown-linux-musl/release/nest ~/.local/bin/
   ```

### Usage

1. Create a `Nestfile` in your project root (see `examples/` folder for comprehensive examples)

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

#### `--complete <shell>`

Generate and install shell completion. Supports: `bash`, `zsh`, `fish`, `powershell`, `elvish`:

```bash
nest --complete zsh        # Generate and install for zsh
nest --complete zsh -V      # Show completion script content (verbose)
nest --complete bash        # Generate and install for bash
```

This command:

- Automatically detects your current shell
- Generates completion scripts for all supported shells
- Installs completion in your shell's configuration file
- Shows informational message about installation
- Sources completion in current terminal (if possible)

Use `-V` or `--verbose` flag to view the generated completion script content instead of installing it.

#### `--example`

Download the examples folder from GitHub:

```bash
nest --example
```

This command:

- Prompts for confirmation before downloading
- Downloads the entire `examples/` folder from the official repository
- Includes comprehensive examples with `@include` directives, `.env` file, and documentation
- Works without requiring an existing Nestfile
- Requires `git`, `curl`, or `wget` to be available on your system

#### `--config` / `-c <path>`

Specify a custom path to the configuration file:

```bash
nest --config /path/to/nestfile build
nest -c ./custom/nestfile deploy v1.0.0
```

This flag:

- Allows you to use a Nestfile from any location
- Works with any command
- Useful when working with multiple projects or custom file locations
- If the file is not found, shows an error message with a helpful tip

**Note:** When Nest cannot find a configuration file automatically, it will suggest using `--config` to specify the path manually.

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

**Libc selection on Linux x86_64 (glibc vs musl):**

- By default, `nest update` on Linux x86_64 installs the **glibc** build:
  - `nest-linux-x86_64.tar.gz`
- To install the **static musl** build instead:

```bash
NEST_LIBC=musl nest update
```

This matches the behavior of the install scripts (`install.sh` / `install.static.sh`), which also respect `NEST_LIBC` for choosing the appropriate archive.

**Note:** If you get a "Text file busy" error, it means the binary is currently in use. Close the terminal session and run the update command again, or manually replace the binary using the instructions provided in the error message.

### Global Flags

#### `--dry-run` / `-n`

Show what would be executed without actually running the commands:

```bash
nest build --dry-run
nest deploy v1.0.0 -n
```

#### `--verbose` / `-v`

Show detailed output including environment variables and working directory:

```bash
nest build --verbose
nest deploy v1.0.0 -v
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
  - System variable with fallback: `> env: NODE_ENV=${NODE_ENV:-development}`
  - System variable: `> env: DATABASE_URL=${DATABASE_URL}`
- **`> script:`** - Script to execute:
  - Single line: `> script: echo "Hello"`
  - Multiline: `> script: |` (followed by indented script block)
- **`> before:`** - Script executed before the main script (see Before/After/Fallback section)
- **`> after:`** - Script executed after successful completion (see Before/After/Fallback section)
- **`> fallback:`** - Script executed on failure (see Before/After/Fallback section)
- **`> depends:`** - Command dependencies (see Command Dependencies section)
- **`> validate:`** - Parameter validation rules (see Parameter Validation section)
- **`> if:` / `> elif:` / `> else:`** - Conditional execution (see Conditional Execution section)
- **`> logs:json <path>` / `> logs:txt <path>`** - Log command execution (see Logging section)
- **`> privileged`** - Require privileged access (root/admin)

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

**Dependencies with Arguments:**
You can pass arguments to dependency commands:

```nest
build_custom(!target|t: str = "x86_64", !release|r: bool = false):
    > desc: Build with target and release options
    > script: echo "Building for {{target}} (release={{release}})..."

deploy_with_args():
    > desc: Deploy with specific build configuration
    > depends: build_custom(target="arm64", release=true), test_custom(coverage=true)
    > script: |
        echo "Deploying with custom build configuration..."
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

### Functions

Functions allow you to create reusable scripts that can be called from commands or other functions. Functions are defined at the global level and can:

- Execute commands
- Call other functions
- Use variables, constants, and environment variables (from global definitions)
- Use system environment variables
- Have parameters
- Define local variables

**Syntax:**

```nest
@function function_name(param1: str, param2: bool):
    @var LOCAL_VAR = "value"
    echo "Function body"
    # Can call commands, other functions, use variables, etc.
```

**Example:**

```nest
# Global variables
@var APP_NAME = "myapp"
@var VERSION = "1.0.0"

# Function definition
@function setup_env(env_name: str):
    @var LOCAL_ENV = "{{env_name}}"
    echo "Setting up environment: {{LOCAL_ENV}}"
    echo "App: {{APP_NAME}} v{{VERSION}}"

# Function that calls another function
@function build_app(target: str):
    echo "Building for target: {{target}}"
    setup_env(env_name="{{target}}")
    npm run build --target={{target}}

# Command using functions
build():
    > script: |
        setup_env(env_name="production")
        build_app(target="x86_64")
```

**Key Points:**

- Functions are defined at the global level (cannot be defined inside commands)
- Functions can have parameters (same syntax as command parameters)
- Functions can define local variables using `@var` inside the function body
- Functions can call commands, other functions, and use variables/constants
- Functions have access to global variables, constants, and system environment variables
- Functions are called using the same syntax as commands: `function_name(arg="value")`

**Function Parameters:**

```nest
@function deploy(version: str, force: bool):
    echo "Deploying version {{version}}"
    if [ "{{force}}" = "true" ]; then
        echo "Force deployment enabled"
    fi
```

**Calling Functions:**

```nest
deploy():
    > script: |
        deploy(version="1.0.0", force="true")
        # Or call without arguments if function has defaults
        deploy(version="1.0.0")
```

**Functions vs Commands:**

- Functions are reusable scripts that can be called from anywhere
- Commands are CLI entry points that can be executed directly via `nest <command>`
- Functions cannot be executed directly - they must be called from commands or other functions
- Functions are useful for code reuse and modularity

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

### Include Directives

Include directives allow you to split your configuration into multiple files for better organization and code reuse:

```nest
# Include a specific file
@include docker.nest

# Include all files matching a pattern
@include modules/*.nest

# Include all config files from a directory
@include commands/
```

**Types of includes:**

1. **Specific file**: `@include docker.nest` - Includes commands from a specific file
2. **Pattern with wildcard**: `@include modules/*.nest` - Includes all files matching the pattern
- **Directory**: `@include commands/` - Includes all configuration files (nestfile, Nestfile, nest, Nest) from the directory

### Include into Group

You can import commands from a file directly into a specific group using the `into` keyword:

```nest
@include modules/database.nest into db
```

This will wrap all commands from `modules/database.nest` under the `db:` group.

### Command Overriding and Merging

When you define a command with the same name multiple times (e.g., via `@include`), Nest merges them instead of showing an error. This allows you to override or extend commands from included files.

**Merging Rules:**

1. **Parameters**: Replaced completely if the overriding command defines any parameters.
2. **Directives**: Appended (e.g., `> desc:` in the override replaces the original if it's the last one).
3. **Children**: Merged recursively.
4. **Variables/Constants**: Merged (local overrides override previous local definitions).

**Example:**

`base.nest`:
```nest
serve:
    > desc: "Start server"
    > script: echo "Starting..."
```

`nestfile`:
```nest
@include base.nest

serve:
    > desc: "Start dev server" # Overrides description
    # Script directive from base.nest is preserved unless overridden by another > script:
```


**Key Points:**

- Include directives are processed before parsing
- Included commands are merged into the main configuration
- Circular includes are detected and will cause an error
- Included files can use variables, constants, and functions defined in the main file
- Relative paths are resolved relative to the file containing the `@include` directive

**Example:**

```nest
# Main nestfile
@var APP_NAME = "myapp"

@include docker.nest
@include database.nest

# Commands from included files are now available
# nest docker build
# nest database migrate
```

### Conditional Execution

You can execute different scripts based on conditions using `if`, `elif`, and `else` directives:

```nest
deploy(env: str):
    > desc: Deploy to different environments
    > if: env == "production"
    > script: |
        echo "Deploying to PRODUCTION..."
        # Production deployment
    > elif: env == "staging"
    > script: |
        echo "Deploying to STAGING..."
        # Staging deployment
    > else:
    > script: |
        echo "Deploying to development..."
        # Development deployment
```

**Supported Operators:**

- Comparison: `==`, `!=`, `<=`, `>=`, `<`, `>`
- Logical: `&&` (AND), `||` (OR), `!` (NOT)

**Condition Types:**

- String comparisons: `param == "value"`
- Numeric comparisons: `count >= 10`
- Boolean checks: `debug == "true"`
- Complex conditions: `env == "prod" && force == "true"`

**Example with Logical Operators:**

```nest
build(!target|t: str = "x86_64", !release|r: bool = false):
    > desc: Build with conditional logic
    > if: target == "x86_64" && release == "true"
    > script: |
        echo "Building optimized x86_64 release..."
        cargo build --release
    > elif: target == "arm64" || target == "aarch64"
    > script: |
        echo "Building for ARM64..."
        cargo build --target aarch64
    > else:
    > script: |
        echo "Building default..."
        cargo build
```

**Key Points:**

- First matching condition executes
- `if` can be used multiple times (acts as separate conditions)
- `elif` and `else` are evaluated only if previous conditions didn't match
- Conditions are evaluated in order

### Parameter Validation

Validate command parameters using regex patterns:

```nest
deploy(version: str):
    > desc: Deploy with version validation
    > validate: version matches /^v?\d+\.\d+\.\d+$/
    > script: |
        echo "Deploying {{version}}"

register(email: str, username: str):
    > desc: Register user with validation
    > validate: email matches /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/
    > validate: username matches /^[a-zA-Z0-9_]{3,20}$/
    > script: |
        echo "Registering {{username}} with {{email}}"
```

**Validation Features:**

- Multiple `> validate:` directives can be used for different parameters
- Case-insensitive regex: use `/pattern/i` flag
- Validation runs before command execution
- Clear error messages when validation fails

### Logging

Log command execution to files in JSON or text format:

```nest
deploy(version: str):
    > desc: Deploy with JSON logging
    > logs:json ./logs/deploy-{{version}}.json
    > script: |
        echo "Deploying {{version}}"

build():
    > desc: Build with text logging
    > logs:txt ./logs/build.log
    > script: |
        npm run build
```

**Log Formats:**

- **JSON**: `> logs:json <path>` - Structured JSON format with timestamp, command, args, success status, and errors
- **Text**: `> logs:txt <path>` - Human-readable text format

**Log Entry Contents:**

- Timestamp (RFC3339 format)
- Command name and path
- Arguments passed to the command
- Success/failure status
- Error message (if failed)

**Template Variables in Paths:**
You can use template variables in log file paths:

```nest
deploy_logged(env: str):
    > desc: Deploy with logging using template variables
    > logs:json ./logs/{{env}}/deploy-{{now}}.json
    > script: |
        echo "Deploying to {{env}}"
```

**Key Points:**

- Log directories are created automatically if they don't exist
- Logs are appended to existing files
- Template variables are processed in log paths
- Both successful and failed executions are logged

### Complete Example

See `examples/` folder for comprehensive working examples including:

- Multiple command types
- Nested command groups
- Parameter types (str, bool, num, arr)
- **Positional and named arguments**
- Environment variable management
- Multiline scripts
- Command dependencies
- Before/after/fallback scripts
- Parameter validation with regex
- **Include directives** for modular configuration
- **Conditional execution** (if/elif/else)
- **Logging** to files

## ✨ Supported Features

### Currently Implemented

✅ **Functions** - Reusable scripts with parameters and local variables
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
- `> if:` / `> elif:` / `> else:` - Conditional execution based on parameter values
- `> logs:json <path>` / `> logs:txt <path>` - Log command execution to files
- `> privileged` - Require privileged access

✅ **Include Directives**

- `@include <file>` - Include specific file
- `@include <pattern>` - Include files matching wildcard pattern
- `@include <directory>/` - Include all config files from directory

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
- Custom config file path (`--config` / `-c`)
- Dry-run mode (`--dry-run` / `-n`)
- Verbose output (`--verbose` / `-v`)

✅ **Execution**

- Script execution with environment variables
- Working directory support
- Environment variable loading from .env files

### Future Plans

Future features that may be added based on user needs and feedback.

## 📁 File Convention

- **Filename**: `Nestfile` (no extension)
- **Location**: Project root directory
- **Examples**: See `examples/` folder in this repository or run `nest --example` to download examples

## 🛠️ Development Status

I actively use this tool in my projects and will continue to maintain and improve it. This project also serves as my learning journey in Rust programming.

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
