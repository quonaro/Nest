<template>
  <div class="layout">
    <header class="header">
      <div class="container">
        <div class="header-content">
          <router-link to="/" class="logo">
            <img src="/Nest.png" alt="Nest" class="logo-icon" />
            <span class="logo-text">Nest</span>
          </router-link>
          <div class="header-right">
            <nav class="nav">
              <router-link to="/getting-started" class="nav-link">{{ $t('nav.gettingStarted') }}</router-link>
              <router-link to="/guides" class="nav-link">{{ $t('nav.guides') }}</router-link>
              <router-link to="/concepts" class="nav-link">{{ $t('nav.concepts') }}</router-link>
              <router-link to="/reference" class="nav-link">{{ $t('nav.reference') }}</router-link>
            </nav>
            <div class="controls">
              <button @click="toggleTheme" class="control-btn" :title="theme === 'light' ? 'Switch to dark theme' : 'Switch to light theme'">
                <span v-if="theme === 'light'">🌙</span>
                <span v-else>☀️</span>
              </button>
              <select v-model="locale" @change="changeLocale" class="locale-select">
                <option value="en">EN</option>
                <option value="ru">RU</option>
              </select>
            </div>
          </div>
        </div>
      </div>
    </header>
    <main class="main">
      <div class="container">
        <aside class="sidebar">
          <nav class="sidebar-nav">
            <div class="nav-section">
              <h3 class="nav-section-title">{{ $t('nav.sidebar.introduction') }}</h3>
              <router-link to="/" class="nav-item">{{ $t('nav.sidebar.overview') }}</router-link>
            </div>
            <div class="nav-section">
              <h3 class="nav-section-title">{{ $t('nav.sidebar.gettingStarted') }}</h3>
              <router-link to="/getting-started" class="nav-item">{{ $t('nav.sidebar.installation') }}</router-link>
              <router-link to="/getting-started#first-steps" class="nav-item">{{ $t('nav.sidebar.firstSteps') }}</router-link>
              <router-link to="/getting-started#features" class="nav-item">{{ $t('nav.sidebar.features') }}</router-link>
            </div>
            <div class="nav-section">
              <h3 class="nav-section-title">{{ $t('nav.sidebar.guides') }}</h3>
              <router-link to="/guides" class="nav-item">{{ $t('nav.sidebar.writingNestfile') }}</router-link>
              <router-link to="/guides#parameters" class="nav-item">{{ $t('nav.sidebar.parameters') }}</router-link>
              <router-link to="/guides#directives" class="nav-item">{{ $t('nav.sidebar.directives') }}</router-link>
              <router-link to="/guides#nested-commands" class="nav-item">{{ $t('nav.sidebar.nestedCommands') }}</router-link>
            </div>
            <div class="nav-section">
              <h3 class="nav-section-title">{{ $t('nav.sidebar.concepts') }}</h3>
              <router-link to="/concepts" class="nav-item">{{ $t('nav.sidebar.projects') }}</router-link>
              <router-link to="/concepts#commands" class="nav-item">{{ $t('nav.sidebar.commands') }}</router-link>
              <router-link to="/concepts#templates" class="nav-item">{{ $t('nav.sidebar.templates') }}</router-link>
            </div>
            <div class="nav-section">
              <h3 class="nav-section-title">{{ $t('nav.sidebar.reference') }}</h3>
              <router-link to="/reference" class="nav-item">{{ $t('nav.sidebar.cliReference') }}</router-link>
              <router-link to="/reference#configuration" class="nav-item">{{ $t('nav.sidebar.configuration') }}</router-link>
            </div>
          </nav>
        </aside>
        <div class="content">
          <slot />
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useTheme } from '../composables/useTheme'

const { locale } = useI18n()
const { theme, toggleTheme } = useTheme()

const changeLocale = () => {
  localStorage.setItem('locale', locale.value as string)
}
</script>

<style scoped>
.layout {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.header {
  border-bottom: 1px solid var(--color-border);
  position: sticky;
  top: 0;
  z-index: 100;
  backdrop-filter: blur(10px);
  transition: background-color 0.3s ease, border-color 0.3s ease;
}

[data-theme='light'] .header {
  background-color: rgba(255, 255, 255, 0.9);
}

[data-theme='dark'] .header {
  background-color: rgba(17, 24, 39, 0.9);
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 0;
}

.logo {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--color-text);
  text-decoration: none;
}

.logo:hover {
  color: var(--color-primary);
}

.logo-icon {
  width: 2rem;
  height: 2rem;
  object-fit: contain;
  display: block;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 2rem;
}

.nav {
  display: flex;
  gap: 2rem;
}

.controls {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.control-btn {
  background: none;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  padding: 0.5rem;
  cursor: pointer;
  font-size: 1.25rem;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.5rem;
  height: 2.5rem;
}

.control-btn:hover {
  background-color: var(--color-bg-secondary);
  border-color: var(--color-primary);
}

.locale-select {
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background-color: var(--color-bg);
  color: var(--color-text);
  font-size: 0.9rem;
  cursor: pointer;
  transition: all 0.2s;
}

.locale-select:hover {
  border-color: var(--color-primary);
}

.locale-select:focus {
  outline: none;
  border-color: var(--color-primary);
}

.nav-link {
  color: var(--color-text-light);
  font-weight: 500;
  transition: color 0.2s;
}

.nav-link:hover,
.nav-link.router-link-active {
  color: var(--color-primary);
}

.container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 0 2rem;
}

.main {
  flex: 1;
  display: flex;
}

.main .container {
  display: flex;
  gap: 3rem;
  padding-top: 2rem;
  padding-bottom: 4rem;
}

.sidebar {
  width: 250px;
  flex-shrink: 0;
  position: sticky;
  top: 80px;
  height: fit-content;
  max-height: calc(100vh - 100px);
  overflow-y: auto;
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.nav-section-title {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-light);
  margin-bottom: 0.5rem;
}

.nav-item {
  display: block;
  padding: 0.5rem 0;
  color: var(--color-text-light);
  font-size: 0.9rem;
  transition: color 0.2s;
}

.nav-item:hover,
.nav-item.router-link-active {
  color: var(--color-primary);
}

.content {
  flex: 1;
  max-width: 800px;
  min-width: 0;
}

@media (max-width: 1024px) {
  .sidebar {
    display: none;
  }
  
  .main .container {
    flex-direction: column;
  }
}

@media (max-width: 768px) {
  .nav {
    display: none;
  }
  
  .container {
    padding: 0 1rem;
  }
}
</style>

