<template>
  <div class="flex h-screen bg-gray-100">
    <aside class="w-60 bg-white border-r border-gray-200 flex flex-col shrink-0">
      <div class="h-14 flex items-center px-4 border-b border-gray-200 font-semibold text-lg">
        {{ settings.title }}
      </div>
      <nav class="flex-1 p-2 space-y-1">
        <router-link
          v-for="item in menuItems"
          :key="item.path"
          :to="item.path"
          class="flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors"
          active-class="bg-gray-100 text-gray-900 font-medium"
          inactive-class="text-gray-600 hover:bg-gray-50 hover:text-gray-900"
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
