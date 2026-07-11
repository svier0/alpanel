<template>
  <div class="max-w-lg mx-auto">
    <h1 class="text-2xl font-semibold mb-6">{{ t('page.settings') }}</h1>
    <form @submit.prevent="handleSave" class="space-y-4">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('settings.port') }}</label>
        <input
          v-model.number="form.port"
          type="number"
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('settings.user') }}</label>
        <input
          v-model="form.user"
          type="text"
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('settings.password') }}</label>
        <input
          v-model="form.password"
          type="password"
          placeholder="••••••••"
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('settings.title') }}</label>
        <input
          v-model="form.title"
          type="text"
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('settings.theme') }}</label>
        <select
          v-model="form.theme"
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white"
        >
          <option value="auto">{{ t('settings.themeAuto') }}</option>
          <option value="light">{{ t('settings.themeLight') }}</option>
          <option value="dark">{{ t('settings.themeDark') }}</option>
        </select>
      </div>
      <p v-if="message" :class="messageType === 'success' ? 'text-green-600' : 'text-red-600'" class="text-sm">{{ message }}</p>
      <button
        type="submit"
        :disabled="saving"
        class="w-full py-2 px-4 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 transition-colors"
      >
        {{ saving ? t('settings.saving') : t('settings.save') }}
      </button>
    </form>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { saveThemeLocally, saveTitleLocally } from '@/stores/settings'

const { t } = useI18n()

const form = reactive({
  port: 0,
  user: '',
  password: '',
  title: '',
  theme: 'auto',
})

const saving = ref(false)
const message = ref('')
const messageType = ref<'success' | 'error'>('success')

onMounted(async () => {
  const token = localStorage.getItem('token')
  if (!token) return
  try {
    const res = await fetch('/api/settings', {
      headers: { Authorization: `Bearer ${token}` }
    })
    if (res.ok) {
      const data = await res.json()
      form.port = data.port
      form.user = data.user
      form.title = data.title
      form.theme = data.theme
    }
  } catch {}
})

async function handleSave() {
  saving.value = true
  message.value = ''
  const token = localStorage.getItem('token')
  if (!token) return
  try {
    const res = await fetch('/api/settings', {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({
        port: form.port,
        user: form.user || undefined,
        password: form.password || undefined,
        title: form.title || undefined,
        theme: form.theme,
      }),
    })
    if (res.ok) {
      const data = await res.json()
      saveTitleLocally(data.title)
      saveThemeLocally(data.theme)
      message.value = t('settings.saved')
      messageType.value = 'success'
      form.password = ''
    } else {
      message.value = t('settings.failed')
      messageType.value = 'error'
    }
  } catch {
    message.value = t('settings.failed')
    messageType.value = 'error'
  } finally {
    saving.value = false
  }
}
</script>
