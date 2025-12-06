import { createRouter, createWebHistory } from 'vue-router'
import Home from '../views/Home.vue'
import GettingStarted from '../views/GettingStarted.vue'
import Guides from '../views/Guides.vue'
import Concepts from '../views/Concepts.vue'
import Reference from '../views/Reference.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: Home
    },
    {
      path: '/getting-started',
      name: 'getting-started',
      component: GettingStarted
    },
    {
      path: '/guides',
      name: 'guides',
      component: Guides
    },
    {
      path: '/concepts',
      name: 'concepts',
      component: Concepts
    },
    {
      path: '/reference',
      name: 'reference',
      component: Reference
    }
  ]
})

// Handle GitHub Pages 404 redirect
// When 404.html redirects to index.html?/path, we need to navigate to that path
// Check for the special query parameter format before Vue Router processes it
if (typeof window !== 'undefined') {
  var l = window.location
  var base = '/Nest/'
  
  // Check if we have the special query parameter format from 404.html
  if (l.search && l.search[1] === '/') {
    // Decode the path from query string
    var decoded = l.search.slice(1).split('&').map(function(s) {
      return s.replace(/~and~/g, '&')
    }).join('?')
    
    // Remove the base path if present
    if (decoded.startsWith(base)) {
      decoded = decoded.slice(base.length - 1)
    }
    
    // Update the URL without reloading
    var newPath = base.slice(0, -1) + decoded + l.hash
    window.history.replaceState(null, '', newPath)
  }
}

router.beforeEach((to, from, next) => {
  // Normal navigation
  next()
})

export default router

