<template>
  <Layout>
    <div class="vscode-extension">
      <h1>VS Code Extension</h1>
      
      <section id="overview">
        <h2>Overview</h2>
        <p>The VS Code extension provides comprehensive language support for Nestfile, including syntax highlighting, validation, autocompletion, and more.</p>
      </section>

      <section id="features">
        <h2>Features</h2>
        <ul>
          <li><strong>Syntax Highlighting</strong> - Colorized syntax for better readability</li>
          <li><strong>Validation</strong> - Real-time error checking and validation</li>
          <li><strong>Autocompletion</strong> - Intelligent code completion for commands, directives, and parameters</li>
          <li><strong>Hover Information</strong> - Detailed information on hover</li>
          <li><strong>Go to Definition</strong> - Navigate to command and function definitions</li>
          <li><strong>Find References</strong> - Find all references to commands and functions</li>
          <li><strong>Code Actions</strong> - Quick fixes and refactoring options</li>
          <li><strong>Formatting</strong> - Code formatting support</li>
          <li><strong>Snippets</strong> - Code snippets for common patterns</li>
          <li><strong>Code Lens</strong> - Additional context and actions</li>
        </ul>
      </section>

      <section id="installation">
        <h2>Installation</h2>
        
        <h3>From Marketplace</h3>
        <ol>
          <li>Open VS Code</li>
          <li>Go to Extensions view (Ctrl+Shift+X / Cmd+Shift+X)</li>
          <li>Search for "Nestfile Language Support"</li>
        </ol>

        <h3>From VSIX</h3>
        <ol>
          <li>Download the .vsix file from the releases page</li>
          <li>Open VS Code</li>
          <li>Go to Extensions view</li>
          <li>Click the "..." menu and select "Install from VSIX..."</li>
        </ol>
      </section>

      <section id="usage">
        <h2>Usage</h2>
        
        <h3>Syntax Highlighting</h3>
        <p>The extension automatically highlights Nestfile syntax when you open .nest files or files named "Nestfile".</p>

        <h3>Autocompletion</h3>
        <p>Autocompletion is available for:</p>
        <ul>
          <li>Directives (desc, cwd, env, script, etc.)</li>
          <li>Meta commands (@var, @const, @function, @include)</li>
          <li>Templates and variables</li>
          <li>Commands and parameters</li>
        </ul>

        <h3>Validation</h3>
        <p>The extension validates your Nestfile and shows errors for:</p>
        <ul>
          <li>Duplicate command definitions</li>
          <li>Invalid parameter types</li>
          <li>Invalid directive usage</li>
          <li>Missing include files</li>
        </ul>

        <h3>Navigation</h3>
        <p>Use the following shortcuts to navigate your code:</p>
        <ul>
          <li><strong>F12</strong> - Go to definition</li>
          <li><strong>Shift+F12</strong> - Find references</li>
          <li><strong>Ctrl+Shift+O</strong> / <strong>Cmd+Shift+O</strong> - Show symbol outline</li>
        </ul>

        <h3>Commands</h3>
        <p>The extension provides the following commands:</p>
        <ul>
          <li><strong>Nestfile: Validate Current File</strong> - Validate the current Nestfile</li>
          <li><strong>Nestfile: Show Parsed Commands (Debug)</strong> - Show the AST representation of commands</li>
        </ul>
      </section>

      <section id="supported-syntax">
        <h2>Supported Syntax</h2>
        <p>The extension supports all Nestfile syntax features.</p>
        
        <h3>Commands</h3>
        <pre v-pre><code>command_name(param: type = default):
    > desc: Description
    > script: echo "Hello"</code></pre>

        <h3>Directives</h3>
        <p>All directives are supported:</p>
        <ul>
          <li><code>> desc:</code>, <code>> cwd:</code>, <code>> env:</code>, <code>> script:</code></li>
          <li><code>> before:</code>, <code>> after:</code>, <code>> fallback:</code>, <code>> finaly:</code></li>
          <li><code>> depends:</code>, <code>> validate:</code>, <code>> if:</code>, <code>> elif:</code>, <code>> else:</code></li>
          <li><code>> logs:json</code>, <code>> logs:txt</code>, <code>> privileged</code>, <code>> require_confirm:</code></li>
          <li><code>> script[hide]:</code>, <code>> before[hide]:</code>, <code>> after[hide]:</code>, <code>> fallback[hide]:</code>, <code>> finaly[hide]:</code></li>
        </ul>

        <h3>Meta Commands</h3>
        <ul>
          <li><code>@var</code> - Variable declarations</li>
          <li><code>@const</code> - Constant declarations</li>
          <li><code>@function</code> - Function definitions</li>
          <li><code>@include</code> - Include other files</li>
        </ul>

        <h3>Templates</h3>
        <p>Templates are supported for variable substitution:</p>
        <ul>
          <li><code>&#123;&#123;variable&#125;&#125;</code> - Variable substitution</li>
          <li><code>&#123;&#123;param&#125;&#125;</code> - Parameter substitution</li>
          <li><code>&#123;&#123;now&#125;&#125;</code>, <code>&#123;&#123;user&#125;&#125;</code> - Special variables</li>
        </ul>
      </section>

      <section id="more-info">
        <h2>More Information</h2>
        <p>For more information, visit the extension repository:</p>
        <p>
          <a href="https://github.com/quonaro/Nest/tree/main/vscode-nestfile-support" target="_blank">
            VS Code Extension Repository
          </a>
        </p>
      </section>
    </div>
  </Layout>
</template>

<script setup lang="ts">
import Layout from '../components/Layout.vue'
</script>

<style scoped>
.vscode-extension {
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

a {
  color: var(--color-primary);
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}
</style>
