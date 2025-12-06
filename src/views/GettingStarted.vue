<template>
  <Layout>
    <div class="getting-started">
      <h1>Getting Started</h1>
      
      <section id="installation">
        <h2>Installation</h2>
        <p>
          Nest can be installed using our official installers or manually from releases.
        </p>
        
        <h3>Using Installer Scripts</h3>
        <p>The install scripts will:</p>
        <ul>
          <li>Detect your OS and architecture automatically</li>
          <li>Download the latest release binary</li>
          <li>Install it to <code>~/.local/bin</code> (Unix) or <code>%USERPROFILE%\.local\bin</code> (Windows)</li>
          <li>Provide instructions if the install directory is not in your PATH</li>
        </ul>

        <h4>Linux/macOS:</h4>
        <pre v-pre><code>curl -fsSL https://raw.githubusercontent.com/quonaro/nest/main/install.sh | bash</code></pre>

        <h4>Windows (PowerShell):</h4>
        <pre v-pre><code>irm https://raw.githubusercontent.com/quonaro/nest/main/install.ps1 | iex</code></pre>

        <h3>Manual Installation</h3>
        <ol>
          <li>Download the latest release for your platform from <a href="https://github.com/quonaro/nest/releases" target="_blank">Releases</a></li>
          <li>Extract the binary</li>
          <li>Add it to your PATH</li>
        </ol>

        <h3>From Source</h3>
        <pre v-pre><code>git clone https://github.com/quonaro/nest.git
cd nest
cargo build --release
sudo cp target/release/nest /usr/local/bin/</code></pre>
      </section>

      <section id="first-steps">
        <h2>First Steps</h2>
        
        <h3>1. Create a Nestfile</h3>
        <p>Create a <code>Nestfile</code> in your project root:</p>
        <pre v-pre><code>greet(name: str, message: str):
    > desc: Greet someone with a message
    > script: echo "Hello {{name}}, {{message}}!"</code></pre>

        <h3>2. Run Your Command</h3>
        <pre v-pre><code>$ nest greet "Alice" "welcome!"
Hello Alice, welcome!</code></pre>

        <h3>3. View Help</h3>
        <pre v-pre><code>$ nest --help
$ nest greet --help</code></pre>
      </section>

      <section id="features">
        <h2>Features</h2>
        
        <h3>Currently Implemented</h3>
        <ul>
          <li>✅ <strong>Command Structure</strong> - Top-level commands, nested subcommands, default subcommands</li>
          <li>✅ <strong>Parameters</strong> - Types (str, bool, num, arr), positional and named arguments, aliases, defaults</li>
          <li>✅ <strong>Directives</strong> - Descriptions, working directory, environment variables, scripts</li>
          <li>✅ <strong>Template Processing</strong> - Parameter substitution, special variables (<code v-pre>{{now}}</code>, <code v-pre>{{user}}</code>)</li>
          <li>✅ <strong>CLI Features</strong> - Dynamic CLI generation, help system, JSON/AST output</li>
          <li>✅ <strong>Execution</strong> - Script execution with environment variables and working directory support</li>
        </ul>

        <h3>Future Plans</h3>
        <ul>
          <li>❌ Functions (<code>@func</code>) - Reusable script blocks</li>
          <li>❌ Lifecycle hooks (<code>@before</code>, <code>@after</code>, <code>@fallback</code>)</li>
          <li>❌ Command dependencies (<code>@depends</code>)</li>
          <li>❌ Function calls (<code>@call</code>)</li>
        </ul>
      </section>
    </div>
  </Layout>
</template>

<script setup lang="ts">
import Layout from '../components/Layout.vue'
</script>

<style scoped>
.getting-started {
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

a {
  color: var(--color-primary);
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}
</style>

