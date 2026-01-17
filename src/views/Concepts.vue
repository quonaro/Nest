<template>
  <Layout>
    <div class="concepts">
      <h1>Concepts</h1>
      
      <section id="projects">
        <h2>Projects</h2>
        <p>
          Nest manages project commands through a Nestfile located in your project root. This file defines all available commands, their parameters, and execution behavior.
        </p>
        
        <h3>File Convention</h3>
        <ul>
          <li><strong>Filename</strong>: <code>Nestfile</code> (Nestfile (no extension))</li>
          <li><strong>Location</strong>: Project root directory</li>
        </ul>
      </section>

      <section id="commands">
        <h2>Commands</h2>
        <p>
          Commands are the building blocks of Nest. Each command can have:
        </p>
        <ul>
          <li><strong>Parameters - Input values with types and defaults</strong></li>
          <li><strong>Directives - Configuration for execution (script, env, cwd, etc.)</strong></li>
          <li><strong>Subcommands - Nested commands for organization</strong></li>
          <li><strong>Default subcommand - Executed when no subcommand is specified</strong></li>
        </ul>

        <h3>Command Structure</h3>
        <pre v-pre><code>command_name(param1: str, !param2|p: bool = false):
    desc: Command description
    cwd: ./path
    env: VAR=value
    script: |
        #!/bin/sh
        echo "{{param1}}"</code></pre>
      </section>

      <section id="templates">
        <h2>Templates</h2>
        <p>Nest uses template variables to inject values into scripts. Variables are replaced with their actual values before script execution.</p>

        <h3>Parameter Substitution</h3>
        <p>Use [param_name] to reference command parameters:</p>
        <pre v-pre><code>greet(name: str):
    script: echo "Hello {{name}}!"</code></pre>

        <h3>Special Variables</h3>
        <ul>
          <li><code v-pre>{{now}}</code> - [now] - Current UTC time in RFC3339 format</li>
          <li><code v-pre>{{user}}</code> - [user] - Current user from $USER environment variable</li>
        </ul>

        <h3>Example</h3>
        <pre v-pre><code>deploy(version: str):
    env: DEPLOYER={{user}}
    env: BUILD_TIME={{now}}
    script: |
        #!/bin/sh
        echo "Deploying {{version}} by {{user}} at {{now}}"</code></pre>
      </section>

      <section id="execution">
        <h2>Execution</h2>
        <p>
          When a command is executed, Nest:
        </p>
        <ol>
          <li>Parses the command and its parameters from the CLI</li>
          <li>Loads environment variables from directives</li>
          <li>Processes template variables in the script</li>
          <li>Sets the working directory (if specified)</li>
          <li>Executes the script with the configured environment</li>
        </ol>

        <h3>Environment Variables</h3>
        <p>Environment variables can be set in multiple ways:</p>
        <ul>
          <li>Direct assignment: env: NODE_ENV=production</li>
          <li>Load from file: env: .env.local</li>
          <li>Multiple directives: Each env: directive adds to the environment</li>
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
