<template>
  <Layout>
    <div class="guides">
      <h1>Guides</h1>
      
      <section id="writing-nestfile">
        <h2>Writing Nestfile</h2>
        
        <h3>Basic Structure</h3>
        <p>A Nestfile is a declarative configuration file that defines commands, their parameters, and execution behavior.</p>
        <pre v-pre><code>command_name(param: type = default):
    desc: Description of the command
    cwd: ./working/directory
    env: VARIABLE_NAME=value
    env: .env.local
    script: |
        #!/bin/sh
        echo "Running command..."
        ./script.sh {{param}}</code></pre>
      </section>

      <section id="parameters">
        <h2>Parameters</h2>
        <p>Parameters allow commands to accept input values with type checking and default values.</p>
        <pre v-pre><code>build(target: str = "x86_64", release: bool = false):
    desc: Build the project
    script: cargo build --target {{target}} ${release:+--release}</code></pre>

        <h3>Parameter Types</h3>
        <ul>
          <li><code>str</code> - String values</li>
          <li><code>bool</code> - Boolean values (true/false)</li>
          <li><code>num</code> - Numeric values</li>
          <li><code>arr</code> - Array of strings</li>
        </ul>

        <h3>Parameter Features</h3>
        <ul>
          <li><strong>Positional arguments</strong></li>
          <li><strong>Named arguments</strong></li>
          <li><strong>Required parameters</strong></li>
          <li><strong>Optional parameters with defaults</strong></li>
          <li><strong>Short form aliases</strong></li>
        </ul>

        <h3>Usage Examples</h3>
        
        <h4>Positional Arguments Example</h4>
        <pre v-pre><code>greet(name: str, message: str):
    desc: Greet someone with a message
    script: echo "Hello {{name}}, {{message}}"</code></pre>
        <pre v-pre><code>$ nest greet "Alice" "welcome!"
# Output: Hello Alice, welcome!</code></pre>

        <h4>Named Arguments Example</h4>
        <pre v-pre><code>deploy(version: str, !env|e: str = "production", !force|f: bool = false):
    desc: Deploy application
    script: |
        #!/bin/sh
        echo "Deploying {{version}} to {{env}}"
        if [ "{{force}}" = "true" ]; then
            ./deploy.sh --force --env {{env}} {{version}}
        else
            ./deploy.sh --env {{env}} {{version}}
        fi</code></pre>
        <pre v-pre><code>$ nest deploy "v1.2.3" --env staging
$ nest deploy "v1.2.3" -e staging --force true
$ nest deploy "v1.2.3"  # env defaults to "production"</code></pre>

        <h4>Mixed Arguments Example</h4>
        <pre v-pre><code>copy(source: str, !destination|d: str, !overwrite|o: bool = false):
    desc: Copy file with optional overwrite
    script: |
        #!/bin/sh
        if [ "{{overwrite}}" = "true" ]; then
            cp -f "{{source}}" "{{destination}}"
        else
            cp "{{source}}" "{{destination}}"
        fi</code></pre>
        <pre v-pre><code>$ nest copy "file.txt" --destination "backup.txt"
$ nest copy "file.txt" -d "backup.txt" -o true</code></pre>

        <h4>Short Forms</h4>
        <p>Named parameters can have short aliases for convenience.</p>
        <pre v-pre><code>command_name(!parameter|short: type = default)</code></pre>
        <p>Short forms can be mixed with full names.</p>
        <pre v-pre><code>build(!target|t: str = "x86_64", !release|r: bool = false):
    desc: Build the project
    script: |
        echo "Building for {{target}}"
        if [ "{{release}}" = "true" ]; then
            echo "Release mode enabled"
        fi</code></pre>
        <pre v-pre><code>$ nest build --target aarch64 --release true
$ nest build -t aarch64 -r true
$ nest build --target aarch64 -r true</code></pre>
      </section>

      <section id="aliases">
        <h2>Aliases</h2>
        <p>Aliases allow you to pass all arguments to a command using the wildcard parameter (*).</p>

        <h3>Syntax</h3>
        <pre v-pre><code>alias-name(*):
    desc: Description
    script: |
        long-command-with-many-options {{*}}</code></pre>
        <p>Use (*) as the parameter to accept all arguments.</p>

        <h3>Example 1: Docker Build</h3>
        <pre v-pre><code>docker-build(*):
    desc: Build the project with Docker
    privileged
    env: DOCKER_PROXY=https://example.com
    script: |
        docker compose -f compose.yml up -d --build {{*}}</code></pre>
        <p>Usage:</p>
        <pre v-pre><code>$ nest docker-build
$ nest docker-build --pull
$ nest docker-build --pull --build-arg KEY=value</code></pre>
        <p><em>All arguments passed to docker-build are forwarded to docker compose.</em></p>

        <h3>Example 2: Git Commit</h3>
        <pre v-pre><code>git-commit(*):
    desc: Git commit with custom message
    script: |
        git add .
        git commit {{*}}</code></pre>
        <p>Usage:</p>
        <pre v-pre><code>$ nest git-commit -m "Fix bug"
$ nest git-commit -m "Update docs" --no-verify
$ nest git-commit -am "Quick fix"</code></pre>

        <h3>Best Practices</h3>
        <ul>
          <li>Use aliases for wrapper commands that forward arguments</li>
          <li>Document what the alias does in the description</li>
          <li>Consider using named parameters when you need specific argument validation</li>
        </ul>
      </section>

      <section id="directives">
        <h2>Directives</h2>
        <p>Directives control how commands are executed, including scripts, environment variables, working directory, and more.</p>

        <h3>Description (desc)</h3>
        <p>Provide a description for the command shown in help output.</p>
        <pre v-pre><code>build:
    desc: Build the project for production
    script: npm run build</code></pre>

        <h3>Working Directory (cwd)</h3>
        <p>Set the working directory for command execution.</p>
        <pre v-pre><code>test:
    desc: Run tests
    cwd: ./tests
    script: npm test</code></pre>
        <p><em>Paths can be relative to the project root or absolute.</em></p>

        <h3>Environment Variables (env)</h3>
        <p>Set environment variables for command execution. Multiple env directives can be used.</p>
        
        <h4>Direct Assignment</h4>
        <p>Assign environment variables directly:</p>
        <pre v-pre><code>run-prod:
    desc: Run in production mode
    env: NODE_ENV=production
    env: PORT=3000
    script: node app.js</code></pre>

        <h4>From File</h4>
        <p>Load environment variables from a file:</p>
        <pre v-pre><code>run-dev:
    desc: Run in development mode
    env: .env.local
    env: NODE_ENV=development
    script: node dev-server.js</code></pre>

        <h4>System Variables</h4>
        <p>Use system environment variables with defaults:</p>
        <pre v-pre><code>build():
    desc: Build with system environment variables
    env: NODE_ENV=${NODE_ENV:-development}
    env: BUILD_NUMBER=${CI_BUILD_NUMBER:-local}
    script: |
        echo "Building in $NODE_ENV mode"
        echo "Build number: $BUILD_NUMBER"
        npm run build</code></pre>
        <p><em>The syntax ${VAR:-default} uses VAR if set, otherwise uses default.</em></p>
        <p><em>Multiple env directives can be combined.</em></p>

        <h3>Script (script)</h3>
        <p>Define the script or command to execute.</p>
        
        <h4>Single Line</h4>
        <p>For simple commands, use a single line:</p>
        <pre v-pre><code>hello:
    desc: Print hello
    script: echo "Hello, World!"</code></pre>

        <h4>Multi-line</h4>
        <p>For complex scripts, use multi-line format:</p>
        <pre v-pre><code>setup:
    desc: Setup project
    script: |
        #!/bin/sh
        set -e
        npm install
        cp .env.example .env
        npm run build</code></pre>

        <h3>Privileged Access</h3>
        <p>Require elevated permissions (sudo) for command execution:</p>
        <pre v-pre><code>install-system:
    desc: Install system packages
    privileged: true
    script: |
        apt-get update
        apt-get install -y curl wget</code></pre>
        <p>Short form:</p>
        <pre v-pre><code>install-system:
    desc: Install system packages
    privileged
    script: |
        apt-get update
        apt-get install -y curl wget</code></pre>
        <p><em>The privileged directive prompts for sudo password when needed.</em></p>

        <h3>Before Hook (before)</h3>
        <p>Execute code before the main script:</p>
        <pre v-pre><code>deploy():
    desc: Deploy with before script
    before: |
        echo "Preparing deployment..."
        ./pre-deploy.sh
    script: |
        echo "Deploying..."
        ./deploy.sh</code></pre>

        <h3>After Hook (after)</h3>
        <p>Execute code after the main script succeeds:</p>
        <pre v-pre><code>deploy():
    desc: Deploy with after script
    script: |
        echo "Deploying..."
        ./deploy.sh
    after: |
        echo "Deployment completed!"
        ./post-deploy.sh</code></pre>

        <h3>Fallback (fallback)</h3>
        <p>Execute code if the main script fails (replaces error output):</p>
        <pre v-pre><code>deploy():
    desc: Deploy with error handling
    script: |
        echo "Deploying..."
        ./deploy.sh
    fallback: |
        echo "Deployment failed, rolling back..."
        ./rollback.sh</code></pre>

        <h3>Dependencies (depends)</h3>
        <p>Specify commands that must execute before this command:</p>
        <pre v-pre><code>build():
    desc: Build project
    depends: clean
    script: npm run build

deploy():
    desc: Deploy application
    depends: build, test
    script: npm run deploy</code></pre>

        <h3>Validation (validate)</h3>
        <p>Validate parameter values using regex patterns:</p>
        <pre v-pre><code>deploy(version: str):
    desc: Deploy with validation
    validate: version matches /^v?\d+\.\d+\.\d+$/
    script: |
        echo "Deploying {{version}}"</code></pre>



        <h3>Logging (logs)</h3>
        <p>Log command execution to files:</p>
        <pre v-pre><code>deploy(version: str):
    desc: Deploy with JSON logging
    logs:json ./logs/deploy-{{version}}.json
    script: |
        echo "Deploying {{version}}"

build():
    desc: Build with text logging
    logs:txt ./logs/build.log
    script: npm run build</code></pre>
      </section>

      <section id="nested-commands">
        <h2>Nested Commands</h2>
        <p>Commands can be organized into groups with nested subcommands for better organization.</p>
        <pre v-pre><code>dev:
    desc: Development tools

    default(!hot|h: bool = false):
        desc: Start dev server
        env: NODE_ENV=development
        script: |
            #!/bin/sh
            if [ "$hot" = "true" ]; then
                nodemon src/index.js
            else
                node src/index.js
            fi

    lint(!fix|f: bool = false):
        desc: Lint code
        script: eslint src/ ${fix:+--fix}</code></pre>

        <h3>Usage</h3>
        <pre v-pre><code>$ nest dev                    # Runs default subcommand
$ nest dev --hot true         # Pass named argument to default
$ nest dev -h true            # Use short alias
$ nest dev lint               # Run lint subcommand
$ nest dev lint --fix true    # Run lint with fix flag
$ nest dev lint -f true       # Use short alias</code></pre>
      </section>

      <section id="templates">
        <h2>Templates</h2>
        <p>Templates allow you to substitute parameters, variables, and special values in your scripts.</p>
        <ul>
          <li><strong>Parameters</strong></li>
          <li><strong>Special</strong>:
            <ul>
              <li><code v-pre>{{now}}</code> - Current UTC time in RFC3339 format</li>
              <li><code v-pre>{{user}}</code> - Current user from $USER environment variable</li>
            </ul>
          </li>
        </ul>

        <h3>Example</h3>
        <pre v-pre><code>deploy(version: str):
    desc: Deploy application
    env: DEPLOYER={{user}}
    env: BUILD_TIME={{now}}
    script: |
        #!/bin/sh
        echo "Deploying {{version}} by {{user}} at {{now}}"
        ./deploy.sh {{version}}</code></pre>
        <p><em>Note: Template variables are processed before script execution.</em></p>
      </section>

      <section id="wildcard">
        <h2>Wildcard Parameters</h2>
        <p>Wildcard parameters allow commands to accept any number of arguments.</p>

        <h3>Syntax</h3>
        <pre v-pre><code>command_name(*):</code></pre>

        <h3>Example</h3>
        <pre v-pre><code>docker-build(*):
    desc: Build the project with Docker, passing all arguments through
    privileged
    script: |
        docker compose -f compose.yml up -d --build {{*}}</code></pre>

        <h3>Usage</h3>
        <pre v-pre><code>$ nest docker-build
$ nest docker-build --pull
$ nest docker-build --pull --build-arg KEY=value</code></pre>
        <p><em>Note: All arguments passed to the command are forwarded using &#123;&#123;*&#125;&#125;.</em></p>
      </section>

      <section id="privileged">
        <h2>Privileged Access</h2>
        <p>Some commands require elevated permissions to execute.</p>

        <h3>Syntax</h3>
        <pre v-pre><code>install-system:
    desc: Install system packages (requires sudo)
    privileged: true
    script: |
        apt-get update
        apt-get install -y curl wget git</code></pre>
        <p>Or short form:</p>
        <pre v-pre><code>install-system:
    desc: Install system packages (requires sudo)
    privileged
    script: |
        apt-get update
        apt-get install -y curl wget git</code></pre>

        <p><em>Note: The privileged directive prompts for sudo password when needed.</em></p>
      </section>

      <section id="multiline">
        <h2>Multiline Scripts</h2>
        <p>Nest supports multiline scripts and parameter definitions for better readability.</p>

        <h3>Script Multiline</h3>
        <pre v-pre><code>setup-project:
    desc: Setup new project with multiple steps
    script: |
        #!/bin/sh
        set -e
        
        echo "Setting up project..."
        npm install
        mkdir -p logs data cache
        cp .env.example .env.local</code></pre>

        <h3>Param Multiline</h3>
        <pre v-pre><code>complex-command(
    input: str,
    !output|o: str,
    !format|f: str = "json",
    !compress|c: bool = false
):
    desc: Complex command with multiline parameters
    script: |
        ./process.sh {{input}} {{output}}</code></pre>
      </section>

      <section id="variables">
        <h2>Variables</h2>
        <p>Variables and constants allow you to define reusable values throughout your Nestfile.</p>

        <h3>Global Variables and Constants</h3>
        <p>Global variables and constants are defined at the top level of the Nestfile and are available throughout the file.</p>
        <pre v-pre><code># Global variables and constants
var APP_NAME = "myapp"
var VERSION = "1.0.0"
const COMPANY_NAME = "My Company"

# Variables can be redefined (last definition wins)
var APP_NAME = "production-app"  # Overrides previous definition</code></pre>

        <h4>Variable Syntax (var)</h4>
        <p>Variables can be redefined - the last definition wins:</p>
        <pre v-pre><code>var APP_NAME = "myapp"
var NODE_ENV = "development"
var APP_NAME = "production-app"  # OK: Variables can be redefined</code></pre>

        <h4>Constant Syntax (const)</h4>
        <p>Constants cannot be redefined once set:</p>
        <pre v-pre><code>const COMPANY_NAME = "My Company"
const API_URL = "https://api.example.com"
# const COMPANY_NAME = "Other"  # ERROR: Constants cannot be redefined</code></pre>

        <h3>Local Variables</h3>
        <p>Local variables are defined within commands and override global variables for that command only:</p>
        <pre v-pre><code># Global variables
var APP_NAME = "global-app"
var NODE_ENV = "development"

# Command with local variables
build():
    # Local variable overrides global for this command only
    var APP_NAME = "local-app"
    var BUILD_DIR = "./build"
    script: |
        echo "Building {{APP_NAME}} in {{BUILD_DIR}}"
        # APP_NAME = "local-app" (not "global-app")</code></pre>

        <h3>Priority</h3>
        <p>Variable resolution follows this priority order:</p>
        <ol>
          <li>Local variables in commands</li>
          <li>Local constants in commands</li>
          <li>Global variables</li>
          <li>Global constants</li>
          <li>Environment variables</li>
          <li>System environment variables</li>
        </ol>

        <h3>Scope</h3>
        <ul>
          <li>Global variables are available throughout the Nestfile</li>
          <li>Local variables are scoped to the command where they are defined</li>
          <li>Local variables override global variables within their scope</li>
        </ul>
      </section>

      <section id="functions">
        <h2>Functions</h2>
        <p>Functions are reusable script blocks that can be called from commands or other functions.</p>

        <h3>Syntax</h3>
        <p>Functions are defined using the function keyword:</p>
        <pre v-pre><code>function function_name(param1: str, param2: bool):
    var LOCAL_VAR = "value"
    echo "Function body"
    # Can call commands, other functions, use variables, etc.</code></pre>

        <h3>Features</h3>
        <ul>
          <li>Functions can accept parameters</li>
          <li>Functions can use local variables</li>
          <li>Functions can call other functions</li>
          <li>Functions can use global variables and constants</li>
          <li>Functions are reusable across commands</li>
        </ul>

        <h3>Example</h3>
        <pre v-pre><code># Global variables
var APP_NAME = "myapp"
var VERSION = "1.0.0"

# Function definition
function setup_env(env_name: str):
    var LOCAL_ENV = "{{env_name}}"
    echo "Setting up environment: {{LOCAL_ENV}}"
    echo "App: {{APP_NAME}} v{{VERSION}}"

# Function that calls another function
function build_app(target: str):
    echo "Building for target: {{target}}"
    setup_env(env_name="{{target}}")
    npm run build --target={{target}}

# Command using functions
build():
    script: |
        setup_env(env_name="production")
        build_app(target="x86_64")</code></pre>

        <h3>Calling Functions</h3>
        <p>Functions are called from within scripts using the function name with parameters:</p>
        <pre v-pre><code>deploy():
    script: |
        deploy(version="1.0.0", force="true")
        # Or call without arguments if function has defaults
        deploy(version="1.0.0")</code></pre>

        <h3>Functions vs Commands</h3>
        <ul>
          <li>Functions are reusable code blocks, commands are executable tasks</li>
          <li>Functions are called from within scripts, commands are called from CLI</li>
          <li>Functions can be used across multiple commands, commands are standalone</li>
        </ul>
      </section>

      <section id="dependencies">
        <h2>Dependencies</h2>
        <p>Commands can depend on other commands, which will be executed automatically before the main command.</p>

        <h3>Syntax</h3>
        <p>Functions are defined using the function keyword:</p>
        <pre v-pre><code>clean():
    desc: Clean build artifacts
    script: rm -rf build/

build():
    desc: Build the project
    depends: clean
    script: npm run build</code></pre>

        <h3>Multiple</h3>
        <p>Multiple Desc</p>
        <pre v-pre><code>deploy():
    desc: Deploy application
    depends: build, test
    script: npm run deploy</code></pre>

        <h3>Execution Order</h3>
        <p>Order Desc</p>
        <pre v-pre><code>$ nest deploy
# Executes: clean -> build -> test -> deploy</code></pre>

        <h3>Recursive</h3>
        <p>Recursive Desc</p>
        <pre v-pre><code>test():
    depends: build  # test depends on build
    script: npm test

deploy():
    depends: test   # deploy depends on test (which depends on build)
    script: npm run deploy</code></pre>

        <h3>Paths</h3>
        <ul>
          <li><strong>Relative</strong></li>
          <li><strong>Absolute</strong></li>
        </ul>

        <h3>With Args</h3>
        <p>With Args Desc</p>
        <pre v-pre><code>build_custom(!target|t: str = "x86_64", !release|r: bool = false):
    desc: Build with target and release options
    script: echo "Building for {{target}} (release={{release}})..."

deploy_with_args():
    desc: Deploy with specific build configuration
    depends: build_custom(target="arm64", release=true), test_custom(coverage=true)
    script: |
        echo "Deploying with custom build configuration..."</code></pre>

        <h3>Circular</h3>
        <p>Circular Desc</p>
        <pre v-pre><code>a():
    depends: b
    script: echo "A"

b():
    depends: a  # ERROR: Circular dependency detected
    script: echo "B"</code></pre>
      </section>

      <section id="before-after-fallback">
        <h2>Before, After, and Fallback</h2>
        <p>Lifecycle hooks allow you to execute code at different stages of command execution.</p>

        <h3>Execution Order</h3>
        <ol>
          <li>before scripts execute</li>
          <li>main script executes</li>
          <li>if main script succeeds: after scripts execute</li>
          <li>if main script fails: fallback scripts execute</li>
          <li>finally scripts always execute</li>
        </ol>

        <h3>Key Points</h3>
        <ul>
          <li>before executes before the main script</li>
          <li>after executes only if main script succeeds</li>
          <li>fallback executes only if main script fails and replaces error output</li>
          <li>finally always executes regardless of success or failure</li>
          <li>multiple directives of the same type are executed in order</li>
          <li>all hooks have access to the same environment variables</li>
        </ul>

        <h3>Example</h3>
        <pre v-pre><code>deploy():
    desc: Deploy with before/after hooks and error handling
    before: |
        echo "Setting up deployment environment..."
        # Pre-deployment checks
    script: |
        echo "Deploying application..."
        # Main deployment logic
        # If this fails, fallback will execute
    after: |
        echo "Deployment completed successfully"
        # Post-deployment tasks (only if main script succeeds)
    fallback: |
        echo "Deployment failed, rolling back..."
        # Error handling (only if main script fails)
        # This replaces the error output
    finaly: |
        echo "Cleaning up temporary files..."
        # Always executes, regardless of success or failure</code></pre>
      </section>

      <section id="finaly">
        <h2>Finaly Title</h2>
        <p>Finaly Directive</p>

        <h3>Syntax</h3>
        <pre v-pre><code>deploy():
    desc: Deploy with cleanup
    script: |
        echo "Deploying..."
        ./deploy.sh
    finaly: |
        echo "Cleaning up..."
        rm -rf /tmp/deploy-*</code></pre>

        <h3>Execution Order</h3>
        <p>Execution Order Desc</p>
        <ol>
          <li>Step1</li>
          <li>Step2</li>
          <li>Step3</li>
          <li>Step4</li>
          <li>Step5</li>
        </ol>

        <h3>Example</h3>
        <pre v-pre><code>build():
    desc: Build with cleanup
    before: echo "Starting build..."
    script: |
        npm run build
        if [ $? -ne 0 ]; then
            exit 1
        fi
    after: echo "Build completed!"
    fallback: echo "Build failed!"
    finaly: |
        echo "Cleaning up build artifacts..."
        rm -rf .cache/
        echo "Cleanup complete"</code></pre>
      </section>

      <section id="require-confirm">
        <h2>Require Confirm Title</h2>
        <p>Require Confirm Directive</p>

        <h3>Syntax</h3>
        <pre v-pre><code>dangerous():
    desc: Dangerous operation
    require_confirm: Are you sure you want to proceed?
    script: |
        echo "Performing dangerous operation..."
        rm -rf /tmp/important-data</code></pre>

        <h3>Default Message</h3>
        <p>Default Message Desc</p>
        <pre v-pre><code>dangerous():
    desc: Dangerous operation
    require_confirm:
    script: |
        echo "Performing dangerous operation..."</code></pre>

        <h3>Example</h3>
        <pre v-pre><code>delete-all():
    desc: Delete all data (irreversible)
    require_confirm: This will delete ALL data. Are you absolutely sure?
    script: |
        echo "Deleting all data..."
        rm -rf data/</code></pre>
      </section>

      <section id="hide-modifier">
        <h2>Hide Modifier Title</h2>
        <p>Hide Modifier Desc</p>

        <h3>Hide Modifier Examples</h3>
        <ul>
          <li><code>Hide Modifier Script</code></li>
          <li><code>Hide Modifier Before</code></li>
          <li><code>Hide Modifier After</code></li>
          <li><code>Hide Modifier Fallback</code></li>
          <li><code>Hide Modifier Finaly</code></li>
        </ul>

        <h3>Example</h3>
        <pre v-pre><code>build():
    desc: Build with hidden verbose output
    before[hide]: |
        echo "Preparing build environment..."
        # Verbose setup output hidden
    script[hide]: |
        npm run build --verbose
        # Build output hidden
    after[hide]: |
        echo "Post-build tasks..."
        # Post-build output hidden
    script: |
        echo "Build completed successfully!"
        # This output is visible</code></pre>

        <h3>Use Case</h3>
        <p>Use Case Desc</p>
        <pre v-pre><code>deploy():
    desc: Deploy with clean output
    before[hide]: |
        # Verbose pre-deployment checks
        ./check-dependencies.sh
        ./validate-config.sh
    script: |
        echo "Deploying..."
        ./deploy.sh
    after[hide]: |
        # Verbose post-deployment tasks
        ./update-cache.sh
        ./notify-services.sh</code></pre>
      </section>

      <section id="include">
        <h2>Include Files</h2>
        <p>You can split your Nestfile into multiple files and include them using the import directive.</p>

        <h3>Syntax</h3>
        <ul>
          <li>Relative paths: import ./path/to/file.nest</li>
          <li>Absolute paths: import /absolute/path/to/file.nest</li>
          <li>Files in same directory: import filename.nest</li>
        </ul>

        <h3>Key Points</h3>
        <ul>
          <li>Include directives are processed before parsing</li>
          <li>Included commands are merged into the main configuration</li>
          <li>Circular includes are detected and will cause an error</li>
          <li>Included files can use variables, constants, and functions defined in the main file</li>
          <li>Relative paths are resolved relative to the file containing the <code>import</code> directive</li>
        </ul>

        <h3>Include into Group</h3>
        <p>You can import commands from a file directly into a specific group using the <code>into</code> keyword:</p>
        <pre v-pre><code>import modules/database.nest into db</code></pre>
        <p>This will wrap all commands from <code>modules/database.nest</code> under the <code>db:</code> group.</p>

        <h3>Example</h3>
        <pre v-pre><code># Main nestfile
var APP_NAME = "myapp"

import docker.nest
import database.nest

# Commands from included files are now available
# nest docker build
# nest database migrate</code></pre>
      <h3>Filtered Imports</h3>
        <p>You can import specific commands or groups from a file using the <code>from</code> keyword. This allows you to pick components selectively.</p>
        
        <h4>Import Group (Deep Import)</h4>
        <p>Importing a group name includes the group and all its subcommands recursively.</p>
        <pre v-pre><code>import modules/utils.nest from db</code></pre>
        
        <h4>Import Command (Partial Import)</h4>
        <p>Use dot notation to import a specific command from within a group.</p>
        <pre v-pre><code># Imports only 'migrate' from the 'db' group
import modules/utils.nest from db.migrate</code></pre>

        <h4>Multiple Selection</h4>
        <p>Select multiple items separated by commas.</p>
        <pre v-pre><code>import modules/utils.nest from db.migrate, db.seed, format</code></pre>

        <h4>Combine with Into</h4>
        <p>Filter commands and place them into a new group.</p>
        <pre v-pre><code>import modules/utils.nest into core from db.migrate</code></pre>
        <p>This creates <code>core -> db -> migrate</code>.</p>
      </section>

      <section id="overriding">
        <h2>Command Overriding and Merging</h2>
        <p>When you define a command with the same name multiple times (e.g., via <code>import</code>), Nest merges them instead of showing an error. This allows you to override or extend commands from included files.</p>

        <h3>Merging Rules</h3>
        <ol>
          <li><strong>Parameters</strong>: Replaced completely if the overriding command defines any parameters.</li>
          <li><strong>Directives</strong>: Appended (e.g., <code>&gt; desc:</code> in the override replaces the original if it's the last one).</li>
          <li><strong>Children</strong>: Merged recursively.</li>
          <li><strong>Variables/Constants</strong>: Merged (local overrides override previous local definitions).</li>
        </ol>

        <h3>Example</h3>
        <p><strong>base.nest</strong>:</p>
        <pre v-pre><code>serve:
    desc: "Start server"
    script: echo "Starting..."</code></pre>

        <p><strong>nestfile</strong>:</p>
        <pre v-pre><code>import base.nest

serve:
    desc: "Start dev server" # Overrides description
    # Script directive from base.nest is preserved unless overridden by another > script:</code></pre>
      </section>

      <section id="watch-mode">
        <h2>Watch Mode</h2>
        <p>Watch mode allows you to automatically re-run a command when files change. This is useful for development workflows where you want to rebuild, test, or restart your application on file changes.</p>

        <h3>Using the <code>> watch:</code> Directive</h3>
        <pre v-pre><code>dev():
    desc: Start development server with auto-reload
    watch: src/**/*.js, src/**/*.ts
    script: |
        echo "Starting dev server..."
        npm run dev</code></pre>
        <p>When you run <code>nest dev</code>, the command will execute initially, watch the specified file patterns, and re-run automatically when any watched file changes.</p>

        <h3>Using the <code>--watch</code> Flag</h3>
        <p>You can also enable watch mode from the command line:</p>
        <pre v-pre><code>$ nest build --watch "src/**/*.rs"</code></pre>

        <h3>Multiple Patterns</h3>
        <pre v-pre><code>test():
    desc: Run tests with auto-reload
    watch: src/**/*.js, tests/**/*.test.js, package.json
    script: npm test</code></pre>

        <h3>Common Use Cases</h3>
        <pre v-pre><code># Frontend development
frontend():
    desc: Build frontend with hot reload
    watch: src/**/*.jsx, src/**/*.css
    script: npm run build

# Backend development
backend():
    desc: Restart server on code changes
    watch: src/**/*.go, go.mod
    script: go run main.go

# Tests
test():
    desc: Run tests on file changes
    watch: src/**/*.py, tests/**/*.py
    script: pytest</code></pre>
      </section>

      <section id="parallel-dependencies">
        <h2>Parallel Dependencies</h2>
        <p>By default, dependencies run sequentially. You can run them in parallel using the <code>[parallel]</code> modifier:</p>

        <h3>Syntax</h3>
        <pre v-pre><code>deploy():
    desc: Deploy with parallel dependencies
    depends[parallel]: build-frontend, build-backend, run-tests
    script: ./deploy.sh</code></pre>

        <h3>Sequential vs Parallel</h3>
        <h4>Sequential (default)</h4>
        <pre v-pre><code>deploy():
    depends: clean, build, test
    script: ./deploy.sh
# Executes: clean -> build -> test -> deploy</code></pre>

        <h4>Parallel</h4>
        <pre v-pre><code>deploy():
    depends[parallel]: build-frontend, build-backend, build-api
    script: ./deploy.sh
# Executes: build-frontend, build-backend, build-api (all at once) -> deploy</code></pre>

        <h3>Use Cases</h3>
        <ul>
          <li>Building multiple independent components simultaneously</li>
          <li>Running multiple test suites in parallel</li>
          <li>Any tasks that don't depend on each other</li>
        </ul>
      </section>

        </ul>

        <h3>Condition Types</h3>
        <ul>
          <li>String</li>
          <li>Numeric</li>
          <li>Boolean</li>
          <li>Complex</li>
        </ul>

        <h3>Example</h3>
        <pre v-pre><code>build(!target|t: str = "x86_64", !release|r: bool = false):
    desc: Build with conditional logic
    if: target == "x86_64" && release == "true"
    script: |
        echo "Building optimized x86_64 release..."
        cargo build --release
    elif: target == "arm64" || target == "aarch64"
    script: |
        echo "Building for ARM64..."
        cargo build --target aarch64
    else:
    script: |
        echo "Building default..."
        cargo build</code></pre>

        <h3>Key Points</h3>
        <ul>
          <li>before executes before the main script</li>
          <li>after executes only if main script succeeds</li>
          <li>fallback executes only if main script fails and replaces error output</li>
          <li>finally always executes regardless of success or failure</li>
        </ul>
      </section>

      <section id="validation">
        <h2>Parameter Validation</h2>
        <p>You can validate parameter values using regex patterns before command execution.</p>

        <h3>Syntax</h3>
        <pre v-pre><code>deploy(version: str):
    desc: Deploy with version validation
    validate: version matches /^v?\d+\.\d+\.\d+$/
    script: |
        echo "Deploying {{version}}"

register(email: str, username: str):
    desc: Register user with validation
    validate: email matches /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/
    validate: username matches /^[a-zA-Z0-9_]{3,20}$/
    script: |
        echo "Registering {{username}} with {{email}}"</code></pre>

        <h3>Features</h3>
        <ul>
          <li>Functions can accept parameters</li>
          <li>Functions can use local variables</li>
          <li>Functions can call other functions</li>
          <li>Functions can use global variables and constants</li>
        </ul>
      </section>

      <section id="logging">
        <h2>Command Logging</h2>
        <p>Command execution can be logged to files for debugging and auditing purposes.</p>

        <h3>Formats</h3>
        <ul>
          <li><strong>JSON</strong></li>
          <li><strong>TXT</strong></li>
        </ul>

        <h3>Contents</h3>
        <ul>
          <li>Command execution start and end times</li>
          <li>All script output (stdout and stderr)</li>
          <li>Exit codes and status</li>
          <li>Environment variables used</li>
          <li>Parameter values passed to the command</li>
        </ul>

        <h3>Templates</h3>
        <p>Templates Desc</p>
        <pre v-pre><code>deploy_logged(env: str):
    desc: Deploy with logging using template variables
    logs:json ./logs/{{env}}/deploy-{{now}}.json
    script: |
        echo "Deploying to {{env}}"</code></pre>

        <h3>Key Points</h3>
        <ul>
          <li>before executes before the main script</li>
          <li>after executes only if main script succeeds</li>
          <li>fallback executes only if main script fails and replaces error output</li>
          <li>finally always executes regardless of success or failure</li>
        </ul>

        <h3>Example</h3>
        <pre v-pre><code>deploy(version: str):
    desc: Deploy with JSON logging
    logs:json ./logs/deploy-{{version}}.json
    script: |
        echo "Deploying {{version}}"

build():
    desc: Build with text logging
    logs:txt ./logs/build.log
    script: npm run build</code></pre>
      </section>
    </div>
  </Layout>
</template>

<script setup lang="ts">
import Layout from '../components/Layout.vue'
</script>

<style scoped>
.guides {
  padding: 2rem 0;
}

h1 {
  font-size: 3rem;
  font-weight: 700;
  margin-bottom: 2rem;
}

h2 {
  font-size: 2rem;
  font-weight: 700;
  margin-top: 3rem;
  margin-bottom: 1rem;
  padding-top: 2rem;
  border-top: 1px solid var(--color-border);
}

h2:first-of-type {
  border-top: none;
  padding-top: 0;
  margin-top: 0;
}

h3 {
  font-size: 1.5rem;
  font-weight: 600;
  margin-top: 2rem;
  margin-bottom: 1rem;
}

h4 {
  font-size: 1.25rem;
  font-weight: 600;
  margin-top: 1.5rem;
  margin-bottom: 0.75rem;
}

p {
  margin-bottom: 1rem;
  line-height: 1.8;
  color: var(--color-text-light);
}

ul {
  margin: 1rem 0;
  padding-left: 2rem;
  line-height: 1.8;
}

li {
  margin-bottom: 0.5rem;
}

code {
  font-family: var(--font-mono);
  font-size: 0.9em;
  background-color: var(--color-code-bg);
  padding: 0.2em 0.4em;
  border-radius: 3px;
  color: var(--color-code-text);
}

pre {
  background-color: var(--color-code-bg);
  padding: 1rem;
  border-radius: 6px;
  overflow-x: auto;
  margin: 1rem 0;
}

pre code {
  background: none;
  padding: 0;
  color: var(--color-text);
}
</style>
