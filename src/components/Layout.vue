<template>
  <div class="layout">
    <header class="header">
      <div class="container">
        <div class="header-content">
          <router-link to="/" class="logo">
            <img :src="logoPath" alt="Nest" class="logo-icon" />
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
              <router-link to="/getting-started" class="nav-item" :class="{ 'nav-item-active': isGettingStartedPage && !hash }">{{ $t('nav.sidebar.installation') }}</router-link>
              <a href="/getting-started#first-steps" class="nav-item" :class="{ 'nav-item-active': isGettingStartedPage && currentHash === 'first-steps' }">{{ $t('nav.sidebar.firstSteps') }}</a>
              <a href="/getting-started#features" class="nav-item" :class="{ 'nav-item-active': isGettingStartedPage && currentHash === 'features' }">{{ $t('nav.sidebar.features') }}</a>
            </div>
            <div class="nav-section">
              <h3 class="nav-section-title">{{ $t('nav.sidebar.guides') }}</h3>
              <router-link to="/guides" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && !hash }">{{ $t('nav.sidebar.writingNestfile') }}</router-link>
              <a href="/guides#parameters" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'parameters' }">{{ $t('nav.sidebar.parameters') }}</a>
              <a href="/guides#aliases" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'aliases' }">{{ $t('nav.sidebar.aliases') }}</a>
              <a href="/guides#directives" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'directives' }">{{ $t('nav.sidebar.directives') }}</a>
              <a href="/guides#nested-commands" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'nested-commands' }">{{ $t('nav.sidebar.nestedCommands') }}</a>
              <a href="/guides#templates" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'templates' }">{{ $t('nav.sidebar.templates') }}</a>
              <a href="/guides#wildcard" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'wildcard' }">{{ $t('nav.sidebar.wildcard') }}</a>
              <a href="/guides#privileged" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'privileged' }">{{ $t('nav.sidebar.privileged') }}</a>
              <a href="/guides#multiline" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'multiline' }">{{ $t('nav.sidebar.multiline') }}</a>
            </div>
            <div class="nav-section">
              <h3 class="nav-section-title">{{ $t('nav.sidebar.concepts') }}</h3>
              <router-link to="/concepts" class="nav-item" :class="{ 'nav-item-active': isConceptsPage && !hash }">{{ $t('nav.sidebar.projects') }}</router-link>
              <a href="/concepts#commands" class="nav-item" :class="{ 'nav-item-active': isConceptsPage && currentHash === 'commands' }">{{ $t('nav.sidebar.commands') }}</a>
              <a href="/concepts#templates" class="nav-item" :class="{ 'nav-item-active': isConceptsPage && currentHash === 'templates' }">{{ $t('nav.sidebar.templates') }}</a>
            </div>
            <div class="nav-section">
              <h3 class="nav-section-title">{{ $t('nav.sidebar.reference') }}</h3>
              <router-link to="/reference" class="nav-item" :class="{ 'nav-item-active': isReferencePage && !hash }">{{ $t('nav.sidebar.cliReference') }}</router-link>
              <a href="/reference#configuration" class="nav-item" :class="{ 'nav-item-active': isReferencePage && currentHash === 'configuration' }">{{ $t('nav.sidebar.configuration') }}</a>
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
import { onMounted, ref, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import { useTheme } from '../composables/useTheme'

const { locale } = useI18n()
const { theme, toggleTheme } = useTheme()
const route = useRoute()
const router = useRouter()

const logoPath = `${import.meta.env.BASE_URL}Nest.png`
const currentHash = ref('')
const hash = ref('')

// Check current page and hash
const isGuidesPage = ref(false)
const isGettingStartedPage = ref(false)
const isConceptsPage = ref(false)
const isReferencePage = ref(false)

const updateActiveState = () => {
  const path = route.path
  hash.value = window.location.hash.slice(1)
  currentHash.value = hash.value
  
  isGuidesPage.value = path === '/guides'
  isGettingStartedPage.value = path === '/getting-started'
  isConceptsPage.value = path === '/concepts'
  isReferencePage.value = path === '/reference'
}

const handleHashChange = () => {
  updateActiveState()
}

// Initialize locale from localStorage on mount
onMounted(() => {
  const savedLocale = localStorage.getItem('locale')
  if (savedLocale && (savedLocale === 'en' || savedLocale === 'ru')) {
    locale.value = savedLocale
  }
  updateActiveState()
  window.addEventListener('hashchange', handleHashChange)
  // Watch route changes
  router.afterEach(() => {
    setTimeout(updateActiveState, 0)
  })
})

onUnmounted(() => {
  window.removeEventListener('hashchange', handleHashChange)
})

const changeLocale = () => {
  const newLocale = locale.value as string
  localStorage.setItem('locale', newLocale)
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

.nav-item:hover {
  color: var(--color-primary);
}

.nav-item.router-link-active,
.nav-item-active {
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

