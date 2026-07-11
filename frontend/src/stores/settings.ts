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
  if (theme === 'dark') {
    document.documentElement.classList.add('dark')
  } else {
    document.documentElement.classList.remove('dark')
  }
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
    }
  } catch {}
}

export function saveThemeLocally(theme: string) {
  settings.theme = theme as 'light' | 'dark' | 'auto'
  applyTheme(theme)
}

export function saveTitleLocally(title: string) {
  settings.title = title
}
