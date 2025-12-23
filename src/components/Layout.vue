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
              <router-link to="/getting-started" class="nav-link">Getting started</router-link>
              <router-link to="/guides" class="nav-link">Guides</router-link>
              <router-link to="/concepts" class="nav-link">Concepts</router-link>
              <router-link to="/reference" class="nav-link">Reference</router-link>
            </nav>
            <div class="controls">
              <button @click="toggleTheme" class="control-btn" :title="theme === 'light' ? 'Switch to dark theme' : 'Switch to light theme'">
                <span v-if="theme === 'light'">🌙</span>
                <span v-else>☀️</span>
              </button>
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
              <h3 class="nav-section-title">Introduction</h3>
              <router-link to="/" class="nav-item">Overview</router-link>
            </div>
            <div class="nav-section">
              <h3 class="nav-section-title">Getting started</h3>
              <router-link to="/getting-started" class="nav-item" :class="{ 'nav-item-active': isGettingStartedPage && (!hash && !currentHash) }">Installation</router-link>
              <a href="#first-steps" @click.prevent="scrollToSection('first-steps')" class="nav-item" :class="{ 'nav-item-active': isGettingStartedPage && currentHash === 'first-steps' }">First steps</a>
              <a href="#features" @click.prevent="scrollToSection('features')" class="nav-item" :class="{ 'nav-item-active': isGettingStartedPage && currentHash === 'features' }">Features</a>
            </div>
            <div class="nav-section">
              <h3 class="nav-section-title">Guides</h3>
              <a href="#writing-nestfile" @click.prevent="scrollToSection('writing-nestfile')" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && (currentHash === 'writing-nestfile' || (!currentHash && !hash && route.path === '/guides')) }">Writing Nestfile</a>
              <a href="#parameters" @click.prevent="scrollToSection('parameters')" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'parameters' }">Parameters</a>
              <a href="#aliases" @click.prevent="scrollToSection('aliases')" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'aliases' }">Aliases</a>
              <a href="#directives" @click.prevent="scrollToSection('directives')" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'directives' }">Directives</a>
              <a href="#nested-commands" @click.prevent="scrollToSection('nested-commands')" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'nested-commands' }">Nested Commands</a>
              <a href="#templates" @click.prevent="scrollToSection('templates')" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'templates' }">Templates</a>
              <a href="#variables" @click.prevent="scrollToSection('variables')" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'variables' }">Variables</a>
              <a href="#functions" @click.prevent="scrollToSection('functions')" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'functions' }">Functions</a>
              <a href="#dependencies" @click.prevent="scrollToSection('dependencies')" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'dependencies' }">Dependencies</a>
              <a href="#before-after-fallback" @click.prevent="scrollToSection('before-after-fallback')" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'before-after-fallback' }">Before, After, and Fallback</a>
              <a href="#include" @click.prevent="scrollToSection('include')" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'include' }">Include</a>
              <a href="#conditional" @click.prevent="scrollToSection('conditional')" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'conditional' }">Conditional</a>
              <a href="#validation" @click.prevent="scrollToSection('validation')" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'validation' }">Validation</a>
              <a href="#logging" @click.prevent="scrollToSection('logging')" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'logging' }">Logging</a>
              <a href="#wildcard" @click.prevent="scrollToSection('wildcard')" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'wildcard' }">Wildcard Parameters</a>
              <a href="#privileged" @click.prevent="scrollToSection('privileged')" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'privileged' }">Privileged Access</a>
              <a href="#multiline" @click.prevent="scrollToSection('multiline')" class="nav-item" :class="{ 'nav-item-active': isGuidesPage && currentHash === 'multiline' }">Multiline Scripts</a>
            </div>
            <div class="nav-section">
              <h3 class="nav-section-title">Concepts</h3>
              <router-link to="/concepts" class="nav-item" :class="{ 'nav-item-active': isConceptsPage && (!hash && !currentHash) }">Projects</router-link>
              <a href="#commands" @click.prevent="scrollToSection('commands')" class="nav-item" :class="{ 'nav-item-active': isConceptsPage && currentHash === 'commands' }">Commands</a>
              <a href="#templates" @click.prevent="scrollToSection('templates')" class="nav-item" :class="{ 'nav-item-active': isConceptsPage && currentHash === 'templates' }">Templates</a>
            </div>
            <div class="nav-section">
              <h3 class="nav-section-title">Reference</h3>
              <router-link to="/reference" class="nav-item" :class="{ 'nav-item-active': isReferencePage && (!hash && !currentHash) }">CLI Reference</router-link>
              <a href="#configuration" @click.prevent="scrollToSection('configuration')" class="nav-item" :class="{ 'nav-item-active': isReferencePage && currentHash === 'configuration' }">Configuration</a>
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
import { onMounted, ref, onUnmounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useTheme } from '../composables/useTheme'

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
  // Get hash from route.hash (without #) or window.location.hash
  const routeHash = route.hash ? route.hash.slice(1) : ''
  const windowHash = window.location.hash.slice(1)
  hash.value = routeHash || windowHash
  currentHash.value = hash.value
  
  isGuidesPage.value = path === '/guides'
  isGettingStartedPage.value = path === '/getting-started'
  isConceptsPage.value = path === '/concepts'
  isReferencePage.value = path === '/reference'
}

const getActiveSectionFromScroll = () => {
  if (!isGuidesPage.value && !isGettingStartedPage.value && !isConceptsPage.value && !isReferencePage.value) {
    return
  }

  const sections = document.querySelectorAll('section[id]')
  if (sections.length === 0) return

  const scrollPos = window.scrollY + 150 // Offset for header
  let activeSection = ''

  // Find the section that's currently in view
  for (let i = sections.length - 1; i >= 0; i--) {
    const section = sections[i] as HTMLElement
    const sectionTop = section.offsetTop

    if (scrollPos >= sectionTop - 50) {
      activeSection = section.id
      break
    }
  }

  // If we're near the top of the page, clear active section
  if (scrollPos < 150 && sections.length > 0) {
    activeSection = (sections[0] as HTMLElement).id || ''
  }

  // Update hash only if it changed
  if (activeSection !== currentHash.value) {
    currentHash.value = activeSection
    hash.value = activeSection
    // Update URL without scrolling and without triggering hashchange
    if (window.history.replaceState) {
      const currentPath = route.path
      window.history.replaceState(null, '', activeSection ? `${currentPath}#${activeSection}` : currentPath)
    }
  }
}

const handleHashChange = () => {
  updateActiveState()
}

const handleScroll = () => {
  getActiveSectionFromScroll()
}

// Watch route changes
watch(() => route.path, () => {
  updateActiveState()
  // Scroll to hash after route change
  if (route.hash) {
    setTimeout(() => {
      const element = document.querySelector(route.hash)
      if (element) {
        const headerOffset = 80
        const elementPosition = (element as HTMLElement).offsetTop
        const offsetPosition = elementPosition - headerOffset

        window.scrollTo({
          top: offsetPosition,
          behavior: 'smooth'
        })
      }
      updateActiveState()
    }, 150)
  }
})

watch(() => route.hash, (newHash) => {
  updateActiveState()
  if (newHash) {
    setTimeout(() => {
      const element = document.querySelector(newHash)
      if (element) {
        const headerOffset = 80
        const elementPosition = (element as HTMLElement).offsetTop
        const offsetPosition = elementPosition - headerOffset

        window.scrollTo({
          top: offsetPosition,
          behavior: 'smooth'
        })
      }
      updateActiveState()
    }, 150)
  }
})

onMounted(() => {
  // Initial state update
  updateActiveState()
  
  // Handle initial hash after content loads
  const handleInitialHash = () => {
    const hash = window.location.hash || route.hash
    if (hash) {
      setTimeout(() => {
        const element = document.querySelector(hash)
        if (element) {
          const headerOffset = 80
          const elementPosition = (element as HTMLElement).offsetTop
          const offsetPosition = elementPosition - headerOffset

          window.scrollTo({
            top: offsetPosition,
            behavior: 'smooth'
          })
        }
        updateActiveState()
      }, 200)
    } else {
      // If no hash, set first section as active if near top
      setTimeout(() => {
        const sections = document.querySelectorAll('section[id]')
        if (sections.length > 0 && window.scrollY < 200) {
          const firstSection = (sections[0] as HTMLElement).id
          if (firstSection) {
            currentHash.value = firstSection
            hash.value = firstSection
          }
        } else {
          getActiveSectionFromScroll()
        }
      }, 300)
    }
  }
  
  // Wait for DOM to be ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', handleInitialHash)
  } else {
    handleInitialHash()
  }
  
  window.addEventListener('hashchange', handleHashChange)
  // Use throttled scroll handler
  let scrollTimeout: ReturnType<typeof setTimeout> | null = null
  const throttledScroll = () => {
    if (scrollTimeout) return
    scrollTimeout = setTimeout(() => {
      handleScroll()
      scrollTimeout = null
    }, 50)
  }
  window.addEventListener('scroll', throttledScroll, { passive: true })
  
  // Watch route changes
  router.afterEach((to) => {
    setTimeout(() => {
      updateActiveState()
      if (to.hash) {
        const element = document.querySelector(to.hash)
        if (element) {
          const headerOffset = 80
          const elementPosition = (element as HTMLElement).offsetTop
          const offsetPosition = elementPosition - headerOffset

          window.scrollTo({
            top: offsetPosition,
            behavior: 'smooth'
          })
        }
      } else {
        // Scroll to top and determine active section
        window.scrollTo({ top: 0, behavior: 'smooth' })
        setTimeout(getActiveSectionFromScroll, 500)
      }
    }, 100)
  })
})

onUnmounted(() => {
  window.removeEventListener('hashchange', handleHashChange)
  window.removeEventListener('scroll', handleScroll)
})

const scrollToSection = (sectionId: string) => {
  // First, ensure we're on the correct page
  const targetPath = getPathForSection(sectionId)
  if (targetPath && route.path !== targetPath) {
    // Navigate to the correct page first
    router.push(`${targetPath}#${sectionId}`).then(() => {
      // Wait for route to be ready and DOM to update
      setTimeout(() => {
        performScroll(sectionId)
      }, 200)
    })
  } else {
    // We're already on the correct page, just scroll
    performScroll(sectionId)
  }
}

const getPathForSection = (sectionId: string): string | null => {
  // Determine which page this section belongs to
  const guidesSections = ['writing-nestfile', 'parameters', 'aliases', 'directives', 'nested-commands', 'templates', 'variables', 'functions', 'dependencies', 'before-after-fallback', 'include', 'conditional', 'validation', 'logging', 'wildcard', 'privileged', 'multiline']
  const gettingStartedSections = ['first-steps', 'features']
  const conceptsSections = ['commands', 'templates']
  const referenceSections = ['configuration']
  
  if (guidesSections.includes(sectionId)) return '/guides'
  if (gettingStartedSections.includes(sectionId)) return '/getting-started'
  if (conceptsSections.includes(sectionId)) return '/concepts'
  if (referenceSections.includes(sectionId)) return '/reference'
  
  // If section not found, assume current page
  return null
}

const performScroll = (sectionId: string) => {
  // Try multiple times in case DOM is not ready
  let attempts = 0
  const maxAttempts = 10
  
  const tryScroll = () => {
    attempts++
    const element = document.getElementById(sectionId)
    if (element) {
      const headerOffset = 80
      const elementPosition = element.offsetTop
      const offsetPosition = elementPosition - headerOffset

      window.scrollTo({
        top: offsetPosition,
        behavior: 'smooth'
      })
      
      // Update URL hash without triggering hashchange
      const currentPath = route.path
      window.history.pushState(null, '', `${currentPath}#${sectionId}`)
      currentHash.value = sectionId
      hash.value = sectionId
    } else if (attempts < maxAttempts) {
      // Element not found yet, try again
      setTimeout(tryScroll, 50)
    }
  }
  
  tryScroll()
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

.nav-item-active {
  color: var(--color-primary);
  font-weight: 600;
}

/* Disable Vue Router's automatic active class for sidebar items */
.nav-item.router-link-active {
  color: var(--color-text-light);
  font-weight: normal;
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

/* Custom scrollbar styles */
:global(::-webkit-scrollbar) {
  width: 8px;
  height: 8px;
}

:global(::-webkit-scrollbar-track) {
  background: transparent;
}

:global(::-webkit-scrollbar-thumb) {
  background: var(--color-border);
  border-radius: 4px;
  transition: background 0.2s ease;
}

:global(::-webkit-scrollbar-thumb:hover) {
  background: var(--color-text-light);
}

/* Firefox scrollbar */
:global(html) {
  scrollbar-width: thin;
  scrollbar-color: var(--color-border) transparent;
}

[data-theme='dark'] :global(html) {
  scrollbar-color: rgba(255, 255, 255, 0.2) transparent;
}
</style>
