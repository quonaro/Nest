<template>
  <Layout>
    <div class="guides">
      <h1>{{ $t('guides.title') }}</h1>
      
      <section id="writing-nestfile">
        <h2>{{ $t('guides.writingNestfile.title') }}</h2>
        
        <h3>{{ $t('guides.writingNestfile.basicStructure') }}</h3>
        <p>{{ $t('guides.writingNestfile.desc') }}</p>
        <pre v-pre><code>command_name(param: type = default):
    > desc: Description of the command
    > cwd: ./working/directory
    > env: VARIABLE_NAME=value
    > env: .env.local
    > script: |
        #!/bin/sh
        echo "Running command..."
        ./script.sh {{param}}</code></pre>
      </section>

      <section id="parameters">
        <h2>{{ $t('guides.parameters.title') }}</h2>
        <p>{{ $t('guides.parameters.desc') }}</p>
        <pre v-pre><code>build(target: str = "x86_64", release: bool = false):
    > desc: Build the project
    > script: cargo build --target {{target}} ${release:+--release}</code></pre>

        <h3>{{ $t('guides.parameters.types') }}</h3>
        <ul>
          <li><code>str</code> - {{ $t('guides.parameters.str') }}</li>
          <li><code>bool</code> - {{ $t('guides.parameters.bool') }}</li>
          <li><code>num</code> - {{ $t('guides.parameters.num') }}</li>
          <li><code>arr</code> - {{ $t('guides.parameters.arr') }}</li>
        </ul>

        <h3>{{ $t('guides.parameters.features') }}</h3>
        <ul>
          <li><strong>{{ $t('guides.parameters.positional') }}</strong></li>
          <li><strong>{{ $t('guides.parameters.named') }}</strong></li>
          <li><strong>{{ $t('guides.parameters.required') }}</strong></li>
          <li><strong>{{ $t('guides.parameters.optional') }}</strong></li>
          <li><strong>{{ $t('guides.parameters.shortForms') }}</strong></li>
        </ul>

        <h3>{{ $t('guides.parameters.usageExamples') }}</h3>
        
        <h4>{{ $t('guides.parameters.positionalExample') }}</h4>
        <pre v-pre><code>greet(name: str, message: str):
    > desc: Greet someone with a message
    > script: echo "Hello {{name}}, {{message}}"</code></pre>
        <pre v-pre><code>$ nest greet "Alice" "welcome!"
# Output: Hello Alice, welcome!</code></pre>

        <h4>{{ $t('guides.parameters.namedExample') }}</h4>
        <pre v-pre><code>deploy(version: str, !env|e: str = "production", !force|f: bool = false):
    > desc: Deploy application
    > script: |
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

        <h4>{{ $t('guides.parameters.mixedExample') }}</h4>
        <pre v-pre><code>copy(source: str, !destination|d: str, !overwrite|o: bool = false):
    > desc: Copy file with optional overwrite
    > script: |
        #!/bin/sh
        if [ "{{overwrite}}" = "true" ]; then
            cp -f "{{source}}" "{{destination}}"
        else
            cp "{{source}}" "{{destination}}"
        fi</code></pre>
        <pre v-pre><code>$ nest copy "file.txt" --destination "backup.txt"
$ nest copy "file.txt" -d "backup.txt" -o true</code></pre>

        <h4>{{ $t('guides.parameters.shortForms') }}</h4>
        <p>{{ $t('guides.parameters.shortFormsDesc') }}</p>
        <pre v-pre><code>command_name(!parameter|short: type = default)</code></pre>
        <p>{{ $t('guides.parameters.shortFormsNote') }}</p>
        <pre v-pre><code>build(!target|t: str = "x86_64", !release|r: bool = false):
    > desc: Build the project
    > script: |
        echo "Building for {{target}}"
        if [ "{{release}}" = "true" ]; then
            echo "Release mode enabled"
        fi</code></pre>
        <pre v-pre><code>$ nest build --target aarch64 --release true
$ nest build -t aarch64 -r true
$ nest build --target aarch64 -r true</code></pre>
      </section>

      <section id="aliases">
        <h2>{{ $t('guides.aliases.title') }}</h2>
        <p>{{ $t('guides.aliases.desc') }}</p>

        <h3>{{ $t('guides.aliases.syntax') }}</h3>
        <pre v-pre><code>alias-name(*):
    > desc: Description
    > script: |
        long-command-with-many-options {{*}}</code></pre>
        <p>{{ $t('guides.aliases.syntaxDesc') }}</p>

        <h3>{{ $t('guides.aliases.example1') }}</h3>
        <pre v-pre><code>docker-build(*):
    > desc: Build the project with Docker
    > privileged
    > env: DOCKER_PROXY=https://example.com
    > script: |
        docker compose -f compose.yml up -d --build {{*}}</code></pre>
        <p>{{ $t('guides.aliases.example1Usage') }}</p>
        <pre v-pre><code>$ nest docker-build
$ nest docker-build --pull
$ nest docker-build --pull --build-arg KEY=value</code></pre>
        <p><em>{{ $t('guides.aliases.example1Note') }}</em></p>

        <h3>{{ $t('guides.aliases.example2') }}</h3>
        <pre v-pre><code>git-commit(*):
    > desc: Git commit with custom message
    > script: |
        git add .
        git commit {{*}}</code></pre>
        <p>{{ $t('guides.aliases.example2Usage') }}</p>
        <pre v-pre><code>$ nest git-commit -m "Fix bug"
$ nest git-commit -m "Update docs" --no-verify
$ nest git-commit -am "Quick fix"</code></pre>

        <h3>{{ $t('guides.aliases.bestPractices') }}</h3>
        <ul>
          <li>{{ $t('guides.aliases.practice1') }}</li>
          <li>{{ $t('guides.aliases.practice2') }}</li>
          <li>{{ $t('guides.aliases.practice3') }}</li>
        </ul>
      </section>

      <section id="directives">
        <h2>{{ $t('guides.directives.title') }}</h2>
        <p>{{ $t('guides.directives.desc') }}</p>

        <h3>{{ $t('guides.directives.descTitle') }}</h3>
        <p>{{ $t('guides.directives.descDirective') }}</p>
        <pre v-pre><code>build:
    > desc: Build the project for production
    > script: npm run build</code></pre>

        <h3>{{ $t('guides.directives.cwdTitle') }}</h3>
        <p>{{ $t('guides.directives.cwdDirective') }}</p>
        <pre v-pre><code>test:
    > desc: Run tests
    > cwd: ./tests
    > script: npm test</code></pre>
        <p><em>{{ $t('guides.directives.cwdNote') }}</em></p>

        <h3>{{ $t('guides.directives.envTitle') }}</h3>
        <p>{{ $t('guides.directives.envDirective') }}</p>
        
        <h4>{{ $t('guides.directives.envDirectTitle') }}</h4>
        <p>{{ $t('guides.directives.envDirect') }}</p>
        <pre v-pre><code>run-prod:
    > desc: Run in production mode
    > env: NODE_ENV=production
    > env: PORT=3000
    > script: node app.js</code></pre>

        <h4>{{ $t('guides.directives.envFileTitle') }}</h4>
        <p>{{ $t('guides.directives.envFile') }}</p>
        <pre v-pre><code>run-dev:
    > desc: Run in development mode
    > env: .env.local
    > env: NODE_ENV=development
    > script: node dev-server.js</code></pre>

        <h4>{{ $t('guides.directives.envSystemTitle') }}</h4>
        <p>{{ $t('guides.directives.envSystem') }}</p>
        <pre v-pre><code>build():
    > desc: Build with system environment variables
    > env: NODE_ENV=${NODE_ENV:-development}
    > env: BUILD_NUMBER=${CI_BUILD_NUMBER:-local}
    > script: |
        echo "Building in $NODE_ENV mode"
        echo "Build number: $BUILD_NUMBER"
        npm run build</code></pre>
        <p><em>{{ $t('guides.directives.envSystemExample') }}</em></p>
        <p><em>{{ $t('guides.directives.envMultiple') }}</em></p>

        <h3>{{ $t('guides.directives.scriptTitle') }}</h3>
        <p>{{ $t('guides.directives.scriptDirective') }}</p>
        
        <h4>{{ $t('guides.directives.scriptSingleTitle') }}</h4>
        <p>{{ $t('guides.directives.scriptSingle') }}</p>
        <pre v-pre><code>hello:
    > desc: Print hello
    > script: echo "Hello, World!"</code></pre>

        <h4>{{ $t('guides.directives.scriptMultiTitle') }}</h4>
        <p>{{ $t('guides.directives.scriptMulti') }}</p>
        <pre v-pre><code>setup:
    > desc: Setup project
    > script: |
        #!/bin/sh
        set -e
        npm install
        cp .env.example .env
        npm run build</code></pre>

        <h3>{{ $t('guides.directives.privilegedTitle') }}</h3>
        <p>{{ $t('guides.directives.privilegedDirective') }}</p>
        <pre v-pre><code>install-system:
    > desc: Install system packages
    > privileged: true
    > script: |
        apt-get update
        apt-get install -y curl wget</code></pre>
        <p>{{ $t('guides.directives.privilegedShortForm') }}</p>
        <pre v-pre><code>install-system:
    > desc: Install system packages
    > privileged
    > script: |
        apt-get update
        apt-get install -y curl wget</code></pre>
        <p><em>{{ $t('guides.directives.privilegedNote') }}</em></p>

        <h3>{{ $t('guides.directives.beforeTitle') }}</h3>
        <p>{{ $t('guides.directives.beforeDirective') }}</p>
        <pre v-pre><code>deploy():
    > desc: Deploy with before script
    > before: |
        echo "Preparing deployment..."
        ./pre-deploy.sh
    > script: |
        echo "Deploying..."
        ./deploy.sh</code></pre>

        <h3>{{ $t('guides.directives.afterTitle') }}</h3>
        <p>{{ $t('guides.directives.afterDirective') }}</p>
        <pre v-pre><code>deploy():
    > desc: Deploy with after script
    > script: |
        echo "Deploying..."
        ./deploy.sh
    > after: |
        echo "Deployment completed!"
        ./post-deploy.sh</code></pre>

        <h3>{{ $t('guides.directives.fallbackTitle') }}</h3>
        <p>{{ $t('guides.directives.fallbackDirective') }}</p>
        <pre v-pre><code>deploy():
    > desc: Deploy with error handling
    > script: |
        echo "Deploying..."
        ./deploy.sh
    > fallback: |
        echo "Deployment failed, rolling back..."
        ./rollback.sh</code></pre>

        <h3>{{ $t('guides.directives.dependsTitle') }}</h3>
        <p>{{ $t('guides.directives.dependsDirective') }}</p>
        <pre v-pre><code>build():
    > desc: Build project
    > depends: clean
    > script: npm run build

deploy():
    > desc: Deploy application
    > depends: build, test
    > script: npm run deploy</code></pre>

        <h3>{{ $t('guides.directives.validateTitle') }}</h3>
        <p>{{ $t('guides.directives.validateDirective') }}</p>
        <pre v-pre><code>deploy(version: str):
    > desc: Deploy with validation
    > validate: version matches /^v?\d+\.\d+\.\d+$/
    > script: |
        echo "Deploying {{version}}"</code></pre>

        <h3>{{ $t('guides.directives.ifTitle') }}</h3>
        <p>{{ $t('guides.directives.ifDirective') }}</p>
        <pre v-pre><code>deploy(env: str):
    > desc: Deploy to different environments
    > if: env == "production"
    > script: |
        echo "Deploying to PRODUCTION..."
    > elif: env == "staging"
    > script: |
        echo "Deploying to STAGING..."
    > else:
    > script: |
        echo "Deploying to development..."</code></pre>

        <h3>{{ $t('guides.directives.logsTitle') }}</h3>
        <p>{{ $t('guides.directives.logsDirective') }}</p>
        <pre v-pre><code>deploy(version: str):
    > desc: Deploy with JSON logging
    > logs:json ./logs/deploy-{{version}}.json
    > script: |
        echo "Deploying {{version}}"

build():
    > desc: Build with text logging
    > logs:txt ./logs/build.log
    > script: npm run build</code></pre>
      </section>

      <section id="nested-commands">
        <h2>{{ $t('guides.nestedCommands.title') }}</h2>
        <p>{{ $t('guides.nestedCommands.desc') }}</p>
        <pre v-pre><code>dev:
    > desc: Development tools

    default(!hot|h: bool = false):
        > desc: Start dev server
        > env: NODE_ENV=development
        > script: |
            #!/bin/sh
            if [ "$hot" = "true" ]; then
                nodemon src/index.js
            else
                node src/index.js
            fi

    lint(!fix|f: bool = false):
        > desc: Lint code
        > script: eslint src/ ${fix:+--fix}</code></pre>

        <h3>{{ $t('guides.nestedCommands.usage') }}</h3>
        <pre v-pre><code>$ nest dev                    # Runs default subcommand
$ nest dev --hot true         # Pass named argument to default
$ nest dev -h true            # Use short alias
$ nest dev lint               # Run lint subcommand
$ nest dev lint --fix true    # Run lint with fix flag
$ nest dev lint -f true       # Use short alias</code></pre>
      </section>

      <section id="templates">
        <h2>{{ $t('guides.templates.title') }}</h2>
        <p>{{ $t('guides.templates.desc') }}</p>
        <ul>
          <li><strong>{{ $t('guides.templates.parameters') }}</strong></li>
          <li><strong>{{ $t('guides.templates.special') }}</strong>:
            <ul>
              <li><code v-pre>{{now}}</code> - {{ $t('guides.templates.now') }}</li>
              <li><code v-pre>{{user}}</code> - {{ $t('guides.templates.user') }}</li>
            </ul>
          </li>
        </ul>

        <h3>{{ $t('guides.templates.example') }}</h3>
        <pre v-pre><code>deploy(version: str):
    > desc: Deploy application
    > env: DEPLOYER={{user}}
    > env: BUILD_TIME={{now}}
    > script: |
        #!/bin/sh
        echo "Deploying {{version}} by {{user}} at {{now}}"
        ./deploy.sh {{version}}</code></pre>
        <p><em>{{ $t('guides.templates.note') }}</em></p>
      </section>

      <section id="wildcard">
        <h2>{{ $t('guides.wildcard.title') }}</h2>
        <p>{{ $t('guides.wildcard.desc') }}</p>

        <h3>{{ $t('guides.wildcard.syntax') }}</h3>
        <pre v-pre><code>command_name(*):</code></pre>

        <h3>{{ $t('guides.wildcard.example') }}</h3>
        <pre v-pre><code>docker-build(*):
    > desc: Build the project with Docker, passing all arguments through
    > privileged
    > script: |
        docker compose -f compose.yml up -d --build {{*}}</code></pre>

        <h3>{{ $t('guides.wildcard.usage') }}</h3>
        <pre v-pre><code>$ nest docker-build
$ nest docker-build --pull
$ nest docker-build --pull --build-arg KEY=value</code></pre>
        <p><em>{{ $t('guides.wildcard.note') }}</em></p>
      </section>

      <section id="privileged">
        <h2>{{ $t('guides.privileged.title') }}</h2>
        <p>{{ $t('guides.privileged.desc') }}</p>

        <h3>{{ $t('guides.privileged.syntax') }}</h3>
        <pre v-pre><code>install-system:
    > desc: Install system packages (requires sudo)
    > privileged: true
    > script: |
        apt-get update
        apt-get install -y curl wget git</code></pre>
        <p>Или короткая форма:</p>
        <pre v-pre><code>install-system:
    > desc: Install system packages (requires sudo)
    > privileged
    > script: |
        apt-get update
        apt-get install -y curl wget git</code></pre>

        <p><em>{{ $t('guides.privileged.note') }}</em></p>
      </section>

      <section id="multiline">
        <h2>{{ $t('guides.multiline.title') }}</h2>
        <p>{{ $t('guides.multiline.desc') }}</p>

        <h3>{{ $t('guides.multiline.scriptMultiline') }}</h3>
        <pre v-pre><code>setup-project:
    > desc: Setup new project with multiple steps
    > script: |
        #!/bin/sh
        set -e
        
        echo "Setting up project..."
        npm install
        mkdir -p logs data cache
        cp .env.example .env.local</code></pre>

        <h3>{{ $t('guides.multiline.paramMultiline') }}</h3>
        <pre v-pre><code>complex-command(
    input: str,
    !output|o: str,
    !format|f: str = "json",
    !compress|c: bool = false
):
    > desc: Complex command with multiline parameters
    > script: |
        ./process.sh {{input}} {{output}}</code></pre>
      </section>

      <section id="variables">
        <h2>{{ $t('guides.variables.title') }}</h2>
        <p>{{ $t('guides.variables.desc') }}</p>

        <h3>{{ $t('guides.variables.globalTitle') }}</h3>
        <p>{{ $t('guides.variables.globalDesc') }}</p>
        <pre v-pre><code># Global variables and constants
@var APP_NAME = "myapp"
@var VERSION = "1.0.0"
@const COMPANY_NAME = "My Company"

# Variables can be redefined (last definition wins)
@var APP_NAME = "production-app"  # Overrides previous definition</code></pre>

        <h4>{{ $t('guides.variables.varSyntax') }}</h4>
        <p>{{ $t('guides.variables.varDesc') }}</p>
        <pre v-pre><code>@var APP_NAME = "myapp"
@var NODE_ENV = "development"
@var APP_NAME = "production-app"  # OK: Variables can be redefined</code></pre>

        <h4>{{ $t('guides.variables.constSyntax') }}</h4>
        <p>{{ $t('guides.variables.constDesc') }}</p>
        <pre v-pre><code>@const COMPANY_NAME = "My Company"
@const API_URL = "https://api.example.com"
# @const COMPANY_NAME = "Other"  # ERROR: Constants cannot be redefined</code></pre>

        <h3>{{ $t('guides.variables.localTitle') }}</h3>
        <p>{{ $t('guides.variables.localDesc') }}</p>
        <pre v-pre><code># Global variables
@var APP_NAME = "global-app"
@var NODE_ENV = "development"

# Command with local variables
build():
    # Local variable overrides global for this command only
    @var APP_NAME = "local-app"
    @var BUILD_DIR = "./build"
    > script: |
        echo "Building {{APP_NAME}} in {{BUILD_DIR}}"
        # APP_NAME = "local-app" (not "global-app")</code></pre>

        <h3>{{ $t('guides.variables.priority') }}</h3>
        <p>{{ $t('guides.variables.priorityDesc') }}</p>
        <ol>
          <li>{{ $t('guides.variables.priority1') }}</li>
          <li>{{ $t('guides.variables.priority2') }}</li>
          <li>{{ $t('guides.variables.priority3') }}</li>
          <li>{{ $t('guides.variables.priority4') }}</li>
          <li>{{ $t('guides.variables.priority5') }}</li>
          <li>{{ $t('guides.variables.priority6') }}</li>
        </ol>

        <h3>{{ $t('guides.variables.scope') }}</h3>
        <ul>
          <li>{{ $t('guides.variables.scopeGlobal') }}</li>
          <li>{{ $t('guides.variables.scopeLocal') }}</li>
          <li>{{ $t('guides.variables.scopeLocalOverride') }}</li>
        </ul>
      </section>

      <section id="functions">
        <h2>{{ $t('guides.functions.title') }}</h2>
        <p>{{ $t('guides.functions.desc') }}</p>

        <h3>{{ $t('guides.functions.syntax') }}</h3>
        <p>{{ $t('guides.functions.syntaxDesc') }}</p>
        <pre v-pre><code>@function function_name(param1: str, param2: bool):
    @var LOCAL_VAR = "value"
    echo "Function body"
    # Can call commands, other functions, use variables, etc.</code></pre>

        <h3>{{ $t('guides.functions.features') }}</h3>
        <ul>
          <li>{{ $t('guides.functions.feature1') }}</li>
          <li>{{ $t('guides.functions.feature2') }}</li>
          <li>{{ $t('guides.functions.feature3') }}</li>
          <li>{{ $t('guides.functions.feature4') }}</li>
          <li>{{ $t('guides.functions.feature5') }}</li>
        </ul>

        <h3>{{ $t('guides.functions.example') }}</h3>
        <pre v-pre><code># Global variables
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
        build_app(target="x86_64")</code></pre>

        <h3>{{ $t('guides.functions.calling') }}</h3>
        <p>{{ $t('guides.functions.callingDesc') }}</p>
        <pre v-pre><code>deploy():
    > script: |
        deploy(version="1.0.0", force="true")
        # Or call without arguments if function has defaults
        deploy(version="1.0.0")</code></pre>

        <h3>{{ $t('guides.functions.vsCommands') }}</h3>
        <ul>
          <li>{{ $t('guides.functions.vs1') }}</li>
          <li>{{ $t('guides.functions.vs2') }}</li>
          <li>{{ $t('guides.functions.vs3') }}</li>
        </ul>
      </section>

      <section id="dependencies">
        <h2>{{ $t('guides.dependencies.title') }}</h2>
        <p>{{ $t('guides.dependencies.desc') }}</p>

        <h3>{{ $t('guides.dependencies.syntax') }}</h3>
        <p>{{ $t('guides.dependencies.syntaxDesc') }}</p>
        <pre v-pre><code>clean():
    > desc: Clean build artifacts
    > script: rm -rf build/

build():
    > desc: Build the project
    > depends: clean
    > script: npm run build</code></pre>

        <h3>{{ $t('guides.dependencies.multiple') }}</h3>
        <p>{{ $t('guides.dependencies.multipleDesc') }}</p>
        <pre v-pre><code>deploy():
    > desc: Deploy application
    > depends: build, test
    > script: npm run deploy</code></pre>

        <h3>{{ $t('guides.dependencies.executionOrder') }}</h3>
        <p>{{ $t('guides.dependencies.orderDesc') }}</p>
        <pre v-pre><code>$ nest deploy
# Executes: clean -> build -> test -> deploy</code></pre>

        <h3>{{ $t('guides.dependencies.recursive') }}</h3>
        <p>{{ $t('guides.dependencies.recursiveDesc') }}</p>
        <pre v-pre><code>test():
    > depends: build  # test depends on build
    > script: npm test

deploy():
    > depends: test   # deploy depends on test (which depends on build)
    > script: npm run deploy</code></pre>

        <h3>{{ $t('guides.dependencies.paths') }}</h3>
        <ul>
          <li><strong>{{ $t('guides.dependencies.relative') }}</strong></li>
          <li><strong>{{ $t('guides.dependencies.absolute') }}</strong></li>
        </ul>

        <h3>{{ $t('guides.dependencies.withArgs') }}</h3>
        <p>{{ $t('guides.dependencies.withArgsDesc') }}</p>
        <pre v-pre><code>build_custom(!target|t: str = "x86_64", !release|r: bool = false):
    > desc: Build with target and release options
    > script: echo "Building for {{target}} (release={{release}})..."

deploy_with_args():
    > desc: Deploy with specific build configuration
    > depends: build_custom(target="arm64", release=true), test_custom(coverage=true)
    > script: |
        echo "Deploying with custom build configuration..."</code></pre>

        <h3>{{ $t('guides.dependencies.circular') }}</h3>
        <p>{{ $t('guides.dependencies.circularDesc') }}</p>
        <pre v-pre><code>a():
    > depends: b
    > script: echo "A"

b():
    > depends: a  # ERROR: Circular dependency detected
    > script: echo "B"</code></pre>
      </section>

      <section id="before-after-fallback">
        <h2>{{ $t('guides.beforeAfterFallback.title') }}</h2>
        <p>{{ $t('guides.beforeAfterFallback.desc') }}</p>

        <h3>{{ $t('guides.beforeAfterFallback.executionOrder') }}</h3>
        <ol>
          <li>{{ $t('guides.beforeAfterFallback.order1') }}</li>
          <li>{{ $t('guides.beforeAfterFallback.order2') }}</li>
          <li>{{ $t('guides.beforeAfterFallback.order3') }}</li>
          <li>{{ $t('guides.beforeAfterFallback.order4') }}</li>
        </ol>

        <h3>{{ $t('guides.beforeAfterFallback.keyPoints') }}</h3>
        <ul>
          <li>{{ $t('guides.beforeAfterFallback.point1') }}</li>
          <li>{{ $t('guides.beforeAfterFallback.point2') }}</li>
          <li>{{ $t('guides.beforeAfterFallback.point3') }}</li>
          <li>{{ $t('guides.beforeAfterFallback.point4') }}</li>
          <li>{{ $t('guides.beforeAfterFallback.point5') }}</li>
        </ul>

        <h3>{{ $t('guides.beforeAfterFallback.example') }}</h3>
        <pre v-pre><code>deploy():
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
        # This replaces the error output</code></pre>
      </section>

      <section id="include">
        <h2>{{ $t('guides.include.title') }}</h2>
        <p>{{ $t('guides.include.desc') }}</p>

        <h3>{{ $t('guides.include.syntax') }}</h3>
        <ul>
          <li>{{ $t('guides.include.type1') }}</li>
          <li>{{ $t('guides.include.type2') }}</li>
          <li>{{ $t('guides.include.type3') }}</li>
        </ul>

        <h3>{{ $t('guides.include.keyPoints') }}</h3>
        <ul>
          <li>{{ $t('guides.include.point1') }}</li>
          <li>{{ $t('guides.include.point2') }}</li>
          <li>{{ $t('guides.include.point3') }}</li>
          <li>{{ $t('guides.include.point4') }}</li>
          <li>{{ $t('guides.include.point5') }}</li>
        </ul>

        <h3>{{ $t('guides.include.example') }}</h3>
        <pre v-pre><code># Main nestfile
@var APP_NAME = "myapp"

@include docker.nest
@include database.nest

# Commands from included files are now available
# nest docker build
# nest database migrate</code></pre>
      </section>

      <section id="conditional">
        <h2>{{ $t('guides.conditional.title') }}</h2>
        <p>{{ $t('guides.conditional.desc') }}</p>

        <h3>{{ $t('guides.conditional.syntax') }}</h3>
        <pre v-pre><code>deploy(env: str):
    > desc: Deploy to different environments
    > if: env == "production"
    > script: |
        echo "Deploying to PRODUCTION..."
    > elif: env == "staging"
    > script: |
        echo "Deploying to STAGING..."
    > else:
    > script: |
        echo "Deploying to development..."</code></pre>

        <h3>{{ $t('guides.conditional.operators') }}</h3>
        <ul>
          <li><strong>{{ $t('guides.conditional.comparison') }}</strong></li>
          <li><strong>{{ $t('guides.conditional.logical') }}</strong></li>
        </ul>

        <h3>{{ $t('guides.conditional.conditionTypes') }}</h3>
        <ul>
          <li>{{ $t('guides.conditional.string') }}</li>
          <li>{{ $t('guides.conditional.numeric') }}</li>
          <li>{{ $t('guides.conditional.boolean') }}</li>
          <li>{{ $t('guides.conditional.complex') }}</li>
        </ul>

        <h3>{{ $t('guides.conditional.example') }}</h3>
        <pre v-pre><code>build(!target|t: str = "x86_64", !release|r: bool = false):
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
        cargo build</code></pre>

        <h3>{{ $t('guides.conditional.keyPoints') }}</h3>
        <ul>
          <li>{{ $t('guides.conditional.point1') }}</li>
          <li>{{ $t('guides.conditional.point2') }}</li>
          <li>{{ $t('guides.conditional.point3') }}</li>
          <li>{{ $t('guides.conditional.point4') }}</li>
        </ul>
      </section>

      <section id="validation">
        <h2>{{ $t('guides.validation.title') }}</h2>
        <p>{{ $t('guides.validation.desc') }}</p>

        <h3>{{ $t('guides.validation.syntax') }}</h3>
        <pre v-pre><code>deploy(version: str):
    > desc: Deploy with version validation
    > validate: version matches /^v?\d+\.\d+\.\d+$/
    > script: |
        echo "Deploying {{version}}"

register(email: str, username: str):
    > desc: Register user with validation
    > validate: email matches /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/
    > validate: username matches /^[a-zA-Z0-9_]{3,20}$/
    > script: |
        echo "Registering {{username}} with {{email}}"</code></pre>

        <h3>{{ $t('guides.validation.features') }}</h3>
        <ul>
          <li>{{ $t('guides.validation.feature1') }}</li>
          <li>{{ $t('guides.validation.feature2') }}</li>
          <li>{{ $t('guides.validation.feature3') }}</li>
          <li>{{ $t('guides.validation.feature4') }}</li>
        </ul>
      </section>

      <section id="logging">
        <h2>{{ $t('guides.logging.title') }}</h2>
        <p>{{ $t('guides.logging.desc') }}</p>

        <h3>{{ $t('guides.logging.formats') }}</h3>
        <ul>
          <li><strong>{{ $t('guides.logging.json') }}</strong></li>
          <li><strong>{{ $t('guides.logging.txt') }}</strong></li>
        </ul>

        <h3>{{ $t('guides.logging.contents') }}</h3>
        <ul>
          <li>{{ $t('guides.logging.content1') }}</li>
          <li>{{ $t('guides.logging.content2') }}</li>
          <li>{{ $t('guides.logging.content3') }}</li>
          <li>{{ $t('guides.logging.content4') }}</li>
          <li>{{ $t('guides.logging.content5') }}</li>
        </ul>

        <h3>{{ $t('guides.logging.templates') }}</h3>
        <p>{{ $t('guides.logging.templatesDesc') }}</p>
        <pre v-pre><code>deploy_logged(env: str):
    > desc: Deploy with logging using template variables
    > logs:json ./logs/{{env}}/deploy-{{now}}.json
    > script: |
        echo "Deploying to {{env}}"</code></pre>

        <h3>{{ $t('guides.logging.keyPoints') }}</h3>
        <ul>
          <li>{{ $t('guides.logging.point1') }}</li>
          <li>{{ $t('guides.logging.point2') }}</li>
          <li>{{ $t('guides.logging.point3') }}</li>
          <li>{{ $t('guides.logging.point4') }}</li>
        </ul>

        <h3>{{ $t('guides.logging.example') }}</h3>
        <pre v-pre><code>deploy(version: str):
    > desc: Deploy with JSON logging
    > logs:json ./logs/deploy-{{version}}.json
    > script: |
        echo "Deploying {{version}}"

build():
    > desc: Build with text logging
    > logs:txt ./logs/build.log
    > script: npm run build</code></pre>
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
