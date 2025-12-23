import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import './assets/main.css'

// Initialize theme before app mount to prevent flash
const savedTheme = localStorage.getItem('theme') || 
  (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
document.documentElement.setAttribute('data-theme', savedTheme)

const app = createApp(App)

app.use(router)

// Handle GitHub Pages 404 redirect after router is set up
// If we have a saved route from 404.html, navigate to it
const savedRoute = sessionStorage.getItem('ghp_redirect')
if (savedRoute) {
  sessionStorage.removeItem('ghp_redirect')
  // The route path should already have leading slash from 404.html
  const routePath = savedRoute.startsWith('/') ? savedRoute : '/' + savedRoute
  const fullPath = routePath + window.location.search + window.location.hash
  
  // Wait for router to be ready, then navigate
  // Use replace instead of push to avoid adding to history
  router.isReady().then(() => {
    router.replace(fullPath).catch((err) => {
      // Ignore navigation errors (e.g., if route doesn't exist)
      console.warn('Navigation error:', err)
    })
  })
}

app.mount('#app')

