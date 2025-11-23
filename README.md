Certainly! Here's the English version of your `DSL.md` documentation:

---

# 📜 Nestfile DSL Specification (v0.1)

A `Nestfile` is a declarative configuration file for defining hierarchical, parameterized, and dependency-aware CLI commands with full support for hooks, templating, environment variables, and reusable functions.

## 🌲 Example Command Tree

```text
deploy <version> [--push] [--force]
build
test
migrate
├── up
└── down
nest
└── cli
    ├── run <name> [--dry]
    └── build <name>
```

## 🧩 Nestfile Structure

### 1. Functions (`@func`)

Reusable script blocks with parameters.

```nest
@func build_project(target):
    @desc "Builds the project for the specified target platform"
    @before echo "Building project for {{target}}..."
    script:
        cargo build --release --target {{target}}
    @after echo "Build finished for {{target}}"
    @fallback echo "Build failed for {{target}}"
```

Features:
- Named parameters (`target`)
- Jinja-like templating via `{{var}}`
- Lifecycle hooks: `@before`, `@after`, `@fallback`
- Invokable via `@call func_name(args)`

---

### 2. Top-Level Commands

Standalone commands supporting arguments, flags, dependencies, and environment control.

```nest
deploy(version, push=false):
    @desc "Deploys the application"
    @args
        -p, --push
        version
    @flags
        --force
    @env-file .env
    @env
        APP_ENV=prod
    @defaults
        push=false
    @cwd ./project
    @depends build test
    @before
        echo "Preparing deployment..."
        @call notify("Deploy started")
    @after
        @call notify("Deploy finished")
        echo "Deployment completed."
    @fallback
        echo "Deploy failed! Rolling back..."
        ./rollback.sh
    script:
        @call build_project("x86_64-unknown-linux-gnu")
        ./deploy.sh {{version}}
        if [ "{{push}}" = "true" ]; then
            git push origin main
        fi
```

Features:
- Required and optional arguments
- Boolean flags (`--flag`)
- Default values (`@defaults`)
- Command dependencies (`@depends` — run *before* the main command)
- Working directory (`@cwd`)
- Environment control (`@env`, `@env-file`)
- Embedded scripts and function calls

---

### 3. Nested Commands

Group related subcommands under a namespace using the `cli` block.

```nest
nest:
    cli:
        @desc "Example of a nested CLI command"

        run(name):
            @desc "Runs a named task"
            @args
                name
            @flags
                --dry
            @env
                DEBUG=true
            script:
                echo "Running {{name}}..."
                if [ "{{dry}}" = "true" ]; then
                    echo "(dry run)"
                fi

        build(name):
            @desc "Builds using the nested CLI"
            script:
                @call build_project({{name}})
```

Usage:
```sh
nest nest cli run my-service --dry
nest nest cli build aarch64-apple-darwin
```

> Note: The double `nest` in the command reflects the top-level command `nest` and its subcommand `cli`. You may simplify this in your CLI design (e.g., `nest cli run …`).

---

### 4. Helper Functions

```nest
@func notify(msg, user="{{NOTIFY_USER}}"):
    @desc "Sends a notification to a user"
    script:
        echo "Notify {{user}}: {{msg}}"
```

Can be used in `@before`, `@after`, `@fallback`, or `script` blocks.

---

## 🔁 Lifecycle Hooks

- `@before` — runs **before** the main script
- `@after` — runs **after** successful completion
- `@fallback` — runs on **failure** (non-zero exit code)

All hooks share the same variable context as the main `script`.

---

## 🔄 Dependencies

```nest
@depends build test
```

Ensures `build` and `test` commands complete successfully before the current command starts.

---

## 🌐 Variables and Templating

- Command/function parameters: `{{param}}`
- Environment variables: `{{ENV_VAR}}`
- Default/fallback values (e.g., `{{NOTIFY_USER}}`) can be set via `.env` or `@env`

---

## 📁 File Convention

- Filename: `Nestfile` (no extension)
- Location: Project root

---

## 🚀 Usage

Assuming a CLI tool named `nest`:

```sh
nest deploy v1.2.3 --push
nest migrate up
nest nest cli run worker --dry
```

(You may adjust the CLI routing to avoid repetition, e.g., `nest cli run …` directly.)

---

> 💡 **Goal of this DSL**: Replace brittle `Makefile`s and scattered shell scripts with a unified, readable, composable, and maintainable task orchestration system—especially suited for polyglot, self-hosted, or automation-heavy projects.

---
