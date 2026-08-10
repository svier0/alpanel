import { createRouter, createWebHistory } from 'vue-router'
import DefaultLayout from '@/layouts/DefaultLayout.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: () => import('@/pages/login/index.vue'),
    },
    {
      path: '/',
      component: DefaultLayout,
      children: [
        { path: '', name: 'home', component: () => import('@/pages/home/index.vue'), meta: { title: 'menu.home' } },
        { path: 'website', name: 'website', component: () => import('@/pages/website/index.vue'), meta: { title: 'menu.website' } },
        { path: 'file', name: 'file', component: () => import('@/pages/file/index.vue'), meta: { title: 'menu.file' } },
        { path: 'database', name: 'database', component: () => import('@/pages/database/index.vue'), meta: { title: 'menu.database' } },
        { path: 'cron', name: 'cron', component: () => import('@/pages/cron/index.vue'), meta: { title: 'menu.cron' } },
        { path: 'plugins', name: 'plugins', component: () => import('@/pages/plugin-market/index.vue'), meta: { title: 'menu.plugins' } },
        { path: 'settings', name: 'settings', component: () => import('@/pages/settings/index.vue'), meta: { title: 'menu.settings' } },
        { path: 'logout', name: 'logout', component: () => import('@/pages/logout/index.vue'), meta: { title: 'menu.logout' } },
      ]
    }
  ]
})

router.beforeEach((to, _from) => {
  const token = localStorage.getItem('token')
  if (!token && to.name !== 'login') {
    return { name: 'login' }
  }
  if (token && to.name === 'login') {
    return { name: 'home' }
  }
})

export default router
