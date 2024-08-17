import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'empty',
      component: () => import('../components/Empty.vue')
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('../components/Settings.vue')
    },
    {
      path: '/system',
      name: 'system',
      component: () => import('../components/System.vue')
    },
    {
      path: '/dashboard',
      name: 'dashboard',
      component: () => import('../components/Dashboard.vue')
    },
    {
      path: '/monitors',
      name: 'monitors',
      component: () => import('../components/Monitors.vue')
    },
    {
      path: '/about',
      name: 'about',
      component: () => import('../components/About.vue')
    }
  ]
})

export default router
