<template>
  <Layout>
    <div class="reference">
      <h1>Reference</h1>
      
      <section id="cli-reference">
        <h2>CLI Reference</h2>
        
        <h3>Basic Usage</h3>
        <pre v-pre><code>nest [command] [subcommand] [arguments] [options]</code></pre>

        <h3>Global Flags</h3>
        <ul>
          <li><code>--version</code>, <code>-V</code> - Show version information</li>
          <li><code>--help</code>, <code>-h</code> - Show help message</li>
          <li><code>--show &lt;format&gt;</code> - Show commands in specified format
            <ul>
              <li><code>json</code> - JSON format</li>
              <li><code>ast</code> - AST format</li>
            </ul>
          </li>
          <li><code>--dry-run</code>, <code>-n</code> - Show what would be executed without running</li>
          <li><code>--verbose</code>, <code>-v</code> - Enable verbose output</li>
          <li><code>--example</code> - Download example Nestfile</li>
          <li><code>--config</code>, <code>-c &lt;path&gt;</code> - Specify custom config file path</li>
          <li><code>update</code> - Update Nest CLI to latest version</li>
        </ul>

        <h3>Command Execution</h3>
        <p>Commands can be executed in various ways:</p>
        <pre v-pre><code>$ nest &lt;command&gt;
$ nest &lt;command&gt; &lt;subcommand&gt;
$ nest &lt;command&gt; --param value
$ nest &lt;command&gt; positional_arg --named-arg value</code></pre>

        <h3>Help System</h3>
        <p>Get help for commands and subcommands:</p>
        <pre v-pre><code>$ nest --help
$ nest &lt;command&gt; --help
$ nest &lt;command&gt; &lt;subcommand&gt; --help</code></pre>

        <p><em>Note: The config file must be named "Nestfile" or "nestfile" and located in the project root, or specified with --config flag.</em></p>
      </section>

      <section id="configuration">
        <h2>Configuration</h2>
        
        <h3>Nestfile Format</h3>
        <p>The Nestfile uses a declarative syntax to define commands, their parameters, and execution behavior.</p>
        
        <h4>Command Definition</h4>
        <pre v-pre><code>command_name(param1: type, !param2|alias: type = default):
    directive: value</code></pre>

        <h4>Parameter Types</h4>
        <ul>
          <li><code>str</code> - String values</li>
          <li><code>bool</code> - Boolean values (true/false)</li>
          <li><code>num</code> - Numeric values</li>
          <li><code>arr</code> - Array of strings</li>
        </ul>

        <h4>Directives</h4>
        <ul>
          <li><code>desc: &lt;description&gt;</code> - Command description shown in help</li>
          <li><code>cwd: &lt;path&gt;</code> - Set working directory for command execution</li>
          <li><code>env: &lt;VAR=value&gt;</code> - Set environment variable</li>
          <li><code>env: &lt;.env.file&gt;</code> - Load environment variables from file</li>
          <li><code>env: ${VAR:-default}</code> - Use system environment variable with default</li>
          <li><code>script: &lt;command&gt;</code> - Single-line script to execute</li>
          <li><code>script: |</code> - Multi-line script to execute</li>
          <li><code>script.hide:</code> - Hide script output from console</li>
          <li><code>before:</code> - Execute before main script</li>
          <li><code>before.hide:</code> - Execute before main script (hidden output)</li>
          <li><code>after:</code> - Execute after main script (on success)</li>
          <li><code>after.hide:</code> - Execute after main script (hidden output, on success)</li>
          <li><code>fallback:</code> - Execute on script failure (replaces error output)</li>
          <li><code>fallback.hide:</code> - Execute on script failure (hidden output)</li>
          <li><code>finally:</code> - Always execute (regardless of success/failure)</li>
          <li><code>finally.hide:</code> - Always execute (hidden output)</li>
          <li><code>require_confirm: &lt;message&gt;</code> - Require user confirmation before execution</li>
          <li><code>depends:</code> - Specify command dependencies</li>
          <li><code>validate:</code> - Validate parameter values</li>
          <li><code>logs.json: &lt;path&gt;</code> / <code>logs.txt: &lt;path&gt;</code> - Log command execution (see Logging section)</li>
          <li><code>privileged</code> - Require elevated permissions</li>
        </ul>

        <h3>File Location</h3>
        <p>The Nestfile must be named "Nestfile" or "nestfile" (case-sensitive) and located in the project root directory. You can also specify a custom path using the --config flag.</p>

        <h3>Commands with Dependencies</h3>
        <pre v-pre><code>clean():
    desc: Clean build artifacts
    script: rm -rf build/

build():
    desc: Build the project
    depends: clean
    script: npm run build

deploy():
    desc: Deploy application
    depends: build, test
    script: npm run deploy</code></pre>



        <h3>Commands with Validation</h3>
        <pre v-pre><code>deploy(version: str):
    desc: Deploy with version validation
    validate: version matches /^v?\d+\.\d+\.\d+$/
    script: |
        echo "Deploying {{version}}"</code></pre>

        <h3>Commands with Logging</h3>
        <pre v-pre><code>deploy(version: str):
    desc: Deploy with JSON logging
    logs.json: ./logs/deploy-{{version}}.json
    script: |
        echo "Deploying {{version}}"</code></pre>
      </section>

      <section id="examples">
        <h2>Examples</h2>
        
        <h3>Simple Command</h3>
        <pre v-pre><code>hello():
    desc: Print hello world
    script: echo "Hello, World!"</code></pre>

        <h3>Command with Parameters</h3>
        <pre v-pre><code>greet(name: str, message: str):
    desc: Greet someone
    script: echo "Hello {{name}}, {{message}}!"</code></pre>

        <h3>Command with Named Parameters</h3>
        <pre v-pre><code>build(!target|t: str = "x86_64", !release|r: bool = false):
    desc: Build project
    script: cargo build --target {{target}} ${release:+--release}</code></pre>

        <h3>Nested Commands</h3>
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
      </section>
    </div>
  </Layout>
</template>

<script setup lang="ts">
import Layout from '../components/Layout.vue'
</script>

<style scoped>
.reference {
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
