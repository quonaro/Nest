import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import i18n from './i18n'
import './assets/main.css'

// Initialize theme before app mount to prevent flash
const savedTheme = localStorage.getItem('theme') || 
  (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
document.documentElement.setAttribute('data-theme', savedTheme)

const app = createApp(App)

app.use(router)
app.use(i18n)

app.mount('#app')

