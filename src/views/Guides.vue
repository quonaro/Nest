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
