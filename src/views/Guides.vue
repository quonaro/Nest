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
          <li><strong>{{ $t('guides.parameters.aliases') }}</strong></li>
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
      </section>

      <section id="directives">
        <h2>{{ $t('guides.directives.title') }}</h2>
        <p>{{ $t('guides.directives.desc') }}</p>
        <ul>
          <li><strong><code>> desc:</code></strong> - {{ $t('guides.directives.descDirective') }}</li>
          <li><strong><code>> cwd:</code></strong> - {{ $t('guides.directives.cwdDirective') }}</li>
          <li><strong><code>> env:</code></strong> - {{ $t('guides.directives.envDirective') }}
            <ul>
              <li>{{ $t('guides.directives.envDirect') }}</li>
              <li>{{ $t('guides.directives.envFile') }}</li>
            </ul>
          </li>
          <li><strong><code>> script:</code></strong> - {{ $t('guides.directives.scriptDirective') }}
            <ul>
              <li>{{ $t('guides.directives.scriptSingle') }}</li>
              <li>{{ $t('guides.directives.scriptMulti') }}</li>
            </ul>
          </li>
        </ul>
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
        <p>{{ templatesDesc }}</p>
        <ul>
          <li><strong>{{ templatesParameters }}</strong></li>
          <li><strong>{{ $t('guides.templates.special') }}</strong>:
            <ul>
              <li><code v-pre>{{now}}</code> - {{ templatesNow }}</li>
              <li><code v-pre>{{user}}</code> - {{ templatesUser }}</li>
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
      </section>
    </div>
  </Layout>
</template>

<script setup lang="ts">
import Layout from '../components/Layout.vue'
import { useTemplateText } from '../composables/useTemplateText'

const templatesDesc = useTemplateText('guides.templates.desc')
const templatesParameters = useTemplateText('guides.templates.parameters')
const templatesNow = useTemplateText('guides.templates.now')
const templatesUser = useTemplateText('guides.templates.user')
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
