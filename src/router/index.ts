import { createRouter, createWebHistory } from 'vue-router'
import Home from '../views/Home.vue'
import GettingStarted from '../views/GettingStarted.vue'
import Guides from '../views/Guides.vue'
import Concepts from '../views/Concepts.vue'
import Reference from '../views/Reference.vue'
import VSCodeExtension from '../views/VSCodeExtension.vue'

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
    },
    {
      path: '/vscode-extension',
      name: 'vscode-extension',
      component: VSCodeExtension
    }
  ],
  scrollBehavior(to, from, savedPosition) {
    if (savedPosition) {
      return savedPosition
    } else if (to.hash) {
      return {
        el: to.hash,
        behavior: 'smooth',
        top: 80 // Offset for sticky header
      }
    } else {
      return { top: 0 }
    }
  }
})

export default router

