import router from '@/router'

function handle401() {
  localStorage.removeItem('token')
  router.push('/login')
}

export function authHeaders(): Record<string, string> {
  const token = localStorage.getItem('token')
  return token ? { Authorization: `Bearer ${token}` } : {}
}

export function checkRes401(res: Response) {
  if (res.status === 401) handle401()
}

export async function apiFetch(url: string, init?: RequestInit): Promise<any> {
  const headers: Record<string, string> = { ...authHeaders() }
  if (init?.method && init.method !== 'GET') headers['Content-Type'] = 'application/json'

  const res = await fetch(url, { ...init, headers: { ...headers, ...init?.headers } })

  if (res.status === 401) {
    handle401()
    throw new Error('unauthorized')
  }

  const ct = res.headers.get('content-type') || ''
  if (!res.ok) {
    if (ct.includes('application/json')) {
      const err = await res.json()
      throw new Error(err.error || `Request failed (${res.status})`)
    }
    const text = await res.text()
    throw new Error(text.slice(0, 100) || `Request failed (${res.status})`)
  }
  if (ct.includes('application/json')) return res.json()
  return null
}
