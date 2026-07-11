import { reactive } from 'vue'

export interface Settings {
  title: string
  theme: 'light' | 'dark' | 'auto'
}

export const settings = reactive<Settings>({
  title: 'Alpanel',
  theme: 'auto',
})

function applyTheme(theme: string) {
  const isDark = theme === 'dark' || (theme === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches)
  document.documentElement.classList.toggle('dark', isDark)
}
let mq: MediaQueryList | null = null
export function listenTheme() {
  mq?.removeEventListener('change', onThemeChange)
  mq = window.matchMedia('(prefers-color-scheme: dark)')
  mq.addEventListener('change', onThemeChange)
}
function onThemeChange() {
  if (settings.theme === 'auto') applyTheme('auto')
}

export async function fetchSettings() {
  const token = localStorage.getItem('token')
  if (!token) return
  try {
    const res = await fetch('/api/settings', {
      headers: { Authorization: `Bearer ${token}` }
    })
    if (res.ok) {
      const data = await res.json()
      settings.title = data.title
      settings.theme = data.theme
      applyTheme(data.theme)
      if (data.theme === 'auto') listenTheme()
    }
  } catch {}
}

export function saveThemeLocally(theme: string) {
  settings.theme = theme as 'light' | 'dark' | 'auto'
  applyTheme(theme)
  if (theme === 'auto') listenTheme()
}

export function saveTitleLocally(title: string) {
  settings.title = title
}

applyTheme(settings.theme)
listenTheme()
