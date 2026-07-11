import { createRouter, createWebHistory } from 'vue-router'
import DefaultLayout from '@/layouts/DefaultLayout.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      component: DefaultLayout,
      children: [
        { path: '', name: 'home', component: () => import('@/pages/Home.vue'), meta: { title: 'menu.home' } },
        { path: 'website', name: 'website', component: () => import('@/pages/Website.vue'), meta: { title: 'menu.website' } },
        { path: 'file', name: 'file', component: () => import('@/pages/File.vue'), meta: { title: 'menu.file' } },
        { path: 'database', name: 'database', component: () => import('@/pages/Database.vue'), meta: { title: 'menu.database' } },
        { path: 'cron', name: 'cron', component: () => import('@/pages/Cron.vue'), meta: { title: 'menu.cron' } },
        { path: 'settings', name: 'settings', component: () => import('@/pages/Settings.vue'), meta: { title: 'menu.settings' } },
        { path: 'logout', name: 'logout', component: () => import('@/pages/Logout.vue'), meta: { title: 'menu.logout' } },
      ]
    }
  ]
})

export default router
