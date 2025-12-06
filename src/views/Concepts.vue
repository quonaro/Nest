<template>
  <Layout>
    <div class="concepts">
      <h1>{{ $t('concepts.title') }}</h1>
      
      <section id="projects">
        <h2>{{ $t('concepts.projects.title') }}</h2>
        <p>
          {{ $t('concepts.projects.desc') }}
        </p>
        
        <h3>{{ $t('concepts.projects.fileConvention') }}</h3>
        <ul>
          <li><strong>{{ $t('concepts.projects.filename') }}</strong>: <code>Nestfile</code> ({{ $t('concepts.projects.filenameValue') }})</li>
          <li><strong>{{ $t('concepts.projects.location') }}</strong>: {{ $t('concepts.projects.locationValue') }}</li>
        </ul>
      </section>

      <section id="commands">
        <h2>{{ $t('concepts.commands.title') }}</h2>
        <p>
          {{ $t('concepts.commands.desc') }}
        </p>
        <ul>
          <li><strong>{{ $t('concepts.commands.parameters') }}</strong></li>
          <li><strong>{{ $t('concepts.commands.directives') }}</strong></li>
          <li><strong>{{ $t('concepts.commands.subcommands') }}</strong></li>
          <li><strong>{{ $t('concepts.commands.defaultSubcommand') }}</strong></li>
        </ul>

        <h3>{{ $t('concepts.commands.structure') }}</h3>
        <pre v-pre><code>command_name(param1: str, !param2|p: bool = false):
    > desc: Command description
    > cwd: ./path
    > env: VAR=value
    > script: |
        #!/bin/sh
        echo "{{param1}}"</code></pre>
      </section>

      <section id="templates">
        <h2>{{ $t('concepts.templates.title') }}</h2>
        <p>
          {{ $t('concepts.templates.desc') }}
        </p>

        <h3>{{ $t('concepts.templates.parameterSubstitution') }}</h3>
        <p>{{ $t('concepts.templates.useParamName') }}</p>
        <pre v-pre><code>greet(name: str):
    > script: echo "Hello {{name}}!"</code></pre>

        <h3>{{ $t('concepts.templates.specialVariables') }}</h3>
        <ul>
          <li><code v-pre>{{now}}</code> - {{ $t('concepts.templates.now') }}</li>
          <li><code v-pre>{{user}}</code> - {{ $t('concepts.templates.user') }}</li>
        </ul>

        <h3>{{ $t('concepts.templates.example') }}</h3>
        <pre v-pre><code>deploy(version: str):
    > env: DEPLOYER={{user}}
    > env: BUILD_TIME={{now}}
    > script: |
        #!/bin/sh
        echo "Deploying {{version}} by {{user}} at {{now}}"</code></pre>
      </section>

      <section id="execution">
        <h2>{{ $t('concepts.execution.title') }}</h2>
        <p>
          {{ $t('concepts.execution.desc') }}
        </p>
        <ol>
          <li>{{ $t('concepts.execution.step1') }}</li>
          <li>{{ $t('concepts.execution.step2') }}</li>
          <li>{{ $t('concepts.execution.step3') }}</li>
          <li>{{ $t('concepts.execution.step4') }}</li>
          <li>{{ $t('concepts.execution.step5') }}</li>
        </ol>

        <h3>{{ $t('concepts.execution.envVars') }}</h3>
        <p>{{ $t('concepts.execution.envVarsDesc') }}</p>
        <ul>
          <li>{{ $t('concepts.execution.directAssignment') }}</li>
          <li>{{ $t('concepts.execution.loadFromFile') }}</li>
          <li>{{ $t('concepts.execution.multipleDirectives') }}</li>
        </ul>
      </section>
    </div>
  </Layout>
</template>

<script setup lang="ts">
import Layout from '../components/Layout.vue'
</script>

<style scoped>
.concepts {
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

p {
  margin-bottom: 1rem;
  line-height: 1.8;
  color: var(--color-text-light);
}

ul, ol {
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
