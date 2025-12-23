<template>
  <Layout>
    <div class="reference">
      <h1>{{ $t('reference.title') }}</h1>
      
      <section id="cli-reference">
        <h2>{{ $t('reference.cliReference.title') }}</h2>
        
        <h3>{{ $t('reference.cliReference.basicUsage') }}</h3>
        <pre v-pre><code>nest [command] [subcommand] [arguments] [options]</code></pre>

        <h3>{{ $t('reference.cliReference.globalFlags') }}</h3>
        <ul>
          <li><code>--version</code>, <code>-V</code> - {{ $t('reference.cliReference.version') }}</li>
          <li><code>--help</code>, <code>-h</code> - {{ $t('reference.cliReference.help') }}</li>
          <li><code>--show &lt;format&gt;</code> - {{ $t('reference.cliReference.show') }}
            <ul>
              <li><code>json</code> - {{ $t('reference.cliReference.json') }}</li>
              <li><code>ast</code> - {{ $t('reference.cliReference.ast') }}</li>
            </ul>
          </li>
          <li><code>--dry-run</code>, <code>-n</code> - {{ $t('reference.cliReference.dryRun') }}</li>
          <li><code>--verbose</code>, <code>-v</code> - {{ $t('reference.cliReference.verbose') }}</li>
          <li><code>--example</code> - {{ $t('reference.cliReference.example') }}</li>
          <li><code>--config</code>, <code>-c &lt;path&gt;</code> - {{ $t('reference.cliReference.config') }}</li>
          <li><code>update</code> - {{ $t('reference.cliReference.update') }}</li>
        </ul>

        <h3>{{ $t('reference.cliReference.commandExecution') }}</h3>
        <p>{{ $t('reference.cliReference.executionDesc') }}</p>
        <pre v-pre><code>$ nest &lt;command&gt;
$ nest &lt;command&gt; &lt;subcommand&gt;
$ nest &lt;command&gt; --param value
$ nest &lt;command&gt; positional_arg --named-arg value</code></pre>

        <h3>{{ $t('reference.cliReference.helpSystem') }}</h3>
        <p>{{ $t('reference.cliReference.helpDesc') }}</p>
        <pre v-pre><code>$ nest --help
$ nest &lt;command&gt; --help
$ nest &lt;command&gt; &lt;subcommand&gt; --help</code></pre>

        <p><em>{{ $t('reference.cliReference.configNote') }}</em></p>
      </section>

      <section id="configuration">
        <h2>{{ $t('reference.configuration.title') }}</h2>
        
        <h3>{{ $t('reference.configuration.nestfileFormat') }}</h3>
        <p>{{ $t('reference.configuration.formatDesc') }}</p>
        
        <h4>{{ $t('reference.configuration.commandDefinition') }}</h4>
        <pre v-pre><code>command_name(param1: type, !param2|alias: type = default):
    > directive: value</code></pre>

        <h4>{{ $t('reference.configuration.parameterTypes') }}</h4>
        <ul>
          <li><code>str</code> - {{ $t('guides.parameters.str') }}</li>
          <li><code>bool</code> - {{ $t('guides.parameters.bool') }}</li>
          <li><code>num</code> - {{ $t('guides.parameters.num') }}</li>
          <li><code>arr</code> - {{ $t('guides.parameters.arr') }}</li>
        </ul>

        <h4>{{ $t('reference.configuration.directives') }}</h4>
        <ul>
          <li><code>> desc: &lt;description&gt;</code> - {{ $t('reference.configuration.descDirective') }}</li>
          <li><code>> cwd: &lt;path&gt;</code> - {{ $t('reference.configuration.cwdDirective') }}</li>
          <li><code>> env: &lt;VAR=value&gt;</code> - {{ $t('reference.configuration.envDirective') }}</li>
          <li><code>> env: &lt;.env.file&gt;</code> - {{ $t('reference.configuration.envFileDirective') }}</li>
          <li><code>> env: ${VAR:-default}</code> - {{ $t('reference.configuration.envSystemDirective') }}</li>
          <li><code>> script: &lt;command&gt;</code> - {{ $t('reference.configuration.scriptDirective') }}</li>
          <li><code>> script: |</code> - {{ $t('reference.configuration.scriptMultiDirective') }}</li>
          <li><code>> script[hide]:</code> - {{ $t('reference.configuration.scriptHideDirective') }}</li>
          <li><code>> before:</code> - {{ $t('reference.configuration.beforeDirective') }}</li>
          <li><code>> before[hide]:</code> - {{ $t('reference.configuration.beforeHideDirective') }}</li>
          <li><code>> after:</code> - {{ $t('reference.configuration.afterDirective') }}</li>
          <li><code>> after[hide]:</code> - {{ $t('reference.configuration.afterHideDirective') }}</li>
          <li><code>> fallback:</code> - {{ $t('reference.configuration.fallbackDirective') }}</li>
          <li><code>> fallback[hide]:</code> - {{ $t('reference.configuration.fallbackHideDirective') }}</li>
          <li><code>> finaly:</code> - {{ $t('reference.configuration.finalyDirective') }}</li>
          <li><code>> finaly[hide]:</code> - {{ $t('reference.configuration.finalyHideDirective') }}</li>
          <li><code>> require_confirm: &lt;message&gt;</code> - {{ $t('reference.configuration.requireConfirmDirective') }}</li>
          <li><code>> depends:</code> - {{ $t('reference.configuration.dependsDirective') }}</li>
          <li><code>> validate:</code> - {{ $t('reference.configuration.validateDirective') }}</li>
          <li><code>> if: / > elif: / > else:</code> - {{ $t('reference.configuration.ifDirective') }}</li>
          <li><code>> logs:json &lt;path&gt; / > logs:txt &lt;path&gt;</code> - {{ $t('reference.configuration.logsDirective') }}</li>
          <li><code>> privileged: true</code> {{ $t('reference.configuration.privilegedOr') }} <code>> privileged</code> - {{ $t('reference.configuration.privilegedDirective') }}</li>
        </ul>

        <h3>{{ $t('reference.configuration.fileLocation') }}</h3>
        <p>{{ $t('reference.configuration.fileLocationDesc') }}</p>

        <h3>{{ $t('reference.examples.withDependencies') }}</h3>
        <pre v-pre><code>clean():
    > desc: Clean build artifacts
    > script: rm -rf build/

build():
    > desc: Build the project
    > depends: clean
    > script: npm run build

deploy():
    > desc: Deploy application
    > depends: build, test
    > script: npm run deploy</code></pre>

        <h3>{{ $t('reference.examples.withConditional') }}</h3>
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

        <h3>{{ $t('reference.examples.withValidation') }}</h3>
        <pre v-pre><code>deploy(version: str):
    > desc: Deploy with version validation
    > validate: version matches /^v?\d+\.\d+\.\d+$/
    > script: |
        echo "Deploying {{version}}"</code></pre>

        <h3>{{ $t('reference.examples.withLogging') }}</h3>
        <pre v-pre><code>deploy(version: str):
    > desc: Deploy with JSON logging
    > logs:json ./logs/deploy-{{version}}.json
    > script: |
        echo "Deploying {{version}}"</code></pre>
      </section>

      <section id="examples">
        <h2>{{ $t('reference.examples.title') }}</h2>
        
        <h3>{{ $t('reference.examples.simpleCommand') }}</h3>
        <pre v-pre><code>hello():
    > desc: Print hello world
    > script: echo "Hello, World!"</code></pre>

        <h3>{{ $t('reference.examples.commandWithParams') }}</h3>
        <pre v-pre><code>greet(name: str, message: str):
    > desc: Greet someone
    > script: echo "Hello {{name}}, {{message}}!"</code></pre>

        <h3>{{ $t('reference.examples.commandWithNamed') }}</h3>
        <pre v-pre><code>build(!target|t: str = "x86_64", !release|r: bool = false):
    > desc: Build project
    > script: cargo build --target {{target}} ${release:+--release}</code></pre>

        <h3>{{ $t('reference.examples.nestedCommands') }}</h3>
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
