<template>
  <div class="flex h-screen bg-gray-100 dark:bg-gray-900">
    <aside class="w-60 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col shrink-0">
      <div class="h-14 flex items-center px-4 border-b border-gray-200 dark:border-gray-700 font-semibold text-lg dark:text-gray-100">
        {{ settings.title }}
      </div>
      <nav class="flex-1 p-2 space-y-1">
        <router-link
          to="/"
          class="flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 hover:text-gray-900 dark:hover:text-white"
          exact-active-class="bg-gray-100 dark:bg-gray-700 font-medium"
        >
          {{ t('menu.home') }}
        </router-link>
        <router-link
          v-for="item in menuItems.slice(1)"
          :key="item.path"
          :to="item.path"
          class="flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 hover:text-gray-900 dark:hover:text-white"
          active-class="bg-gray-100 dark:bg-gray-700 font-medium"
        >
          {{ t(item.label) }}
        </router-link>
      </nav>
    </aside>
    <main class="flex-1 overflow-auto">
      <div class="p-6">
        <router-view />
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { settings, fetchSettings } from '@/stores/settings'

const { t } = useI18n()

const menuItems = [
  { path: '/', label: 'menu.home' },
  { path: '/website', label: 'menu.website' },
  { path: '/file', label: 'menu.file' },
  { path: '/database', label: 'menu.database' },
  { path: '/cron', label: 'menu.cron' },
  { path: '/settings', label: 'menu.settings' },
  { path: '/logout', label: 'menu.logout' },
]

onMounted(fetchSettings)
</script>
