<template>
  <Layout>
    <div class="home">
      <div class="hero">
        <h1 class="hero-title">
          <img :src="logoPath" alt="Nest" class="hero-icon" />
          Nest
        </h1>
        <p class="hero-subtitle">
          A declarative task runner that replaces brittle Makefiles and scattered shell scripts with a unified, readable, and maintainable task orchestration system.
        </p>
        <div class="hero-actions">
          <router-link to="/getting-started" class="btn btn-primary">Get Started</router-link>
          <a href="https://github.com/quonaro/nest" target="_blank" class="btn btn-secondary">GitHub</a>
        </div>
      </div>

      <section class="highlights">
        <h2 class="section-title">Highlights</h2>
        <div class="highlights-grid">
          <div class="highlight-card">
            <div class="highlight-icon">🚀</div>
            <h3 class="highlight-title">Single Tool</h3>
            <p class="highlight-text">Replace Makefiles and shell scripts with one unified system</p>
          </div>
          <div class="highlight-card">
            <div class="highlight-icon">📝</div>
            <h3 class="highlight-title">Declarative</h3>
            <p class="highlight-text">Define commands in a simple, readable Nestfile format</p>
          </div>
          <div class="highlight-card">
            <div class="highlight-icon">🔧</div>
            <h3 class="highlight-title">Flexible</h3>
            <p class="highlight-text">Support for parameters, nested commands, and environment variables</p>
          </div>
          <div class="highlight-card">
            <div class="highlight-icon">⚡</div>
            <h3 class="highlight-title">Fast</h3>
            <p class="highlight-text">Written in Rust for maximum performance</p>
          </div>
        </div>
      </section>

      <section class="installation">
        <h2 class="section-title">Installation</h2>
        <p class="section-text">Installation:</p>
        
        <div class="code-block">
          <div class="code-tabs">
            <button 
              class="code-tab" 
              :class="{ active: activeTab === 'unix' }"
              @click="activeTab = 'unix'"
            >
              macOS and Linux
            </button>
            <button 
              class="code-tab" 
              :class="{ active: activeTab === 'windows' }"
              @click="activeTab = 'windows'"
            >
              Windows
            </button>
          </div>
          <pre><code>{{ activeTab === 'unix' ? unixInstall : windowsInstall }}</code></pre>
        </div>
      </section>

      <section class="quick-start">
        <h2 class="section-title">Quick Start</h2>
        <p class="section-text">Create a Nestfile in your project root:</p>
        
        <pre v-pre><code>greet(name: str, message: str):
    desc: Greet someone with a message
    script: echo "Hello {{name}}, {{message}}!"</code></pre>

        <p class="section-text">Then run your command:</p>
        
        <pre v-pre><code>$ nest greet "Alice" "welcome!"
Hello Alice, welcome!</code></pre>
      </section>

      <section class="features">
        <h2 class="section-title">Features</h2>
        <ul class="features-list">
          <li>✅ <strong>Command Structure - Top-level and nested subcommands</strong></li>
          <li>✅ <strong>Parameters - Positional and named arguments with types (str, bool, num, arr)</strong></li>
          <li>✅ <strong>Directives - Control working directory, environment variables, and scripts</strong></li>
          <li>✅ <strong>Template Processing - Variable substitution with <code v-pre>{{param}}</code> syntax</strong></li>
          <li>✅ <strong>Dynamic CLI - Automatically generated from your Nestfile</strong></li>
          <li>✅ <strong>Environment Management - Load from .env files or direct assignment</strong></li>
        </ul>
      </section>
    </div>
  </Layout>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import Layout from '../components/Layout.vue'
const activeTab = ref<'unix' | 'windows'>('unix')
const logoPath = `${import.meta.env.BASE_URL}Nest.png`

const unixInstall = '$ curl -fsSL https://raw.githubusercontent.com/quonaro/nest/main/install.sh | bash'

const windowsInstall = 'PS> irm https://raw.githubusercontent.com/quonaro/nest/main/install.ps1 | iex'
</script>

<style scoped>
.home {
  padding: 2rem 0;
}

.hero {
  text-align: center;
  padding: 4rem 0;
  border-bottom: 1px solid var(--color-border);
  margin-bottom: 4rem;
}

.hero-title {
  font-size: 4rem;
  font-weight: 700;
  margin-bottom: 1rem;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1rem;
}

.hero-icon {
  width: 5rem;
  height: 5rem;
  object-fit: contain;
  display: block;
}

.hero-subtitle {
  font-size: 1.25rem;
  color: var(--color-text-light);
  max-width: 700px;
  margin: 0 auto 2rem;
  line-height: 1.8;
}

.hero-subtitle code {
  background-color: var(--color-code-bg);
  padding: 0.2em 0.4em;
  border-radius: 3px;
}

.hero-actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
}

.btn {
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  font-weight: 500;
  transition: all 0.2s;
  display: inline-block;
}

.btn-primary {
  background-color: var(--color-primary);
  color: white;
}

.btn-primary:hover {
  background-color: var(--color-primary-dark);
  color: white;
}

.btn-secondary {
  background-color: var(--color-bg-secondary);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

.btn-secondary:hover {
  background-color: var(--color-border);
}

.section-title {
  font-size: 2rem;
  font-weight: 700;
  margin-bottom: 1rem;
}

.section-text {
  color: var(--color-text-light);
  margin-bottom: 1rem;
  line-height: 1.8;
}

.highlights {
  margin-bottom: 4rem;
}

.highlights-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 2rem;
  margin-top: 2rem;
}

.highlight-card {
  padding: 1.5rem;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background-color: var(--color-bg-secondary);
}

.highlight-icon {
  font-size: 2.5rem;
  margin-bottom: 1rem;
}

.highlight-title {
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
}

.highlight-text {
  color: var(--color-text-light);
  line-height: 1.6;
}

.installation,
.quick-start,
.features {
  margin-bottom: 4rem;
}

.code-block {
  margin: 1.5rem 0;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  overflow: hidden;
}

.code-tabs {
  display: flex;
  background-color: var(--color-bg-secondary);
  border-bottom: 1px solid var(--color-border);
}

.code-tab {
  padding: 0.75rem 1.5rem;
  background: none;
  border: none;
  cursor: pointer;
  font-weight: 500;
  color: var(--color-text-light);
  transition: all 0.2s;
}

.code-tab:hover {
  color: var(--color-text);
}

.code-tab.active {
  color: var(--color-primary);
  border-bottom: 2px solid var(--color-primary);
}

.code-block pre {
  margin: 0;
  border-radius: 0;
}

.features-list {
  list-style: none;
  padding: 0;
}

.features-list li {
  padding: 0.75rem 0;
  font-size: 1.1rem;
  line-height: 1.8;
}

.features-list strong {
  color: var(--color-text);
}
</style>

