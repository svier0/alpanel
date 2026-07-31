import { h, createApp, ref, reactive, computed } from 'vue'
import ElementPlus from 'element-plus'
import { apiFetch, authHeaders } from './api'

export interface PluginContext {
  fetch(action: string, opts?: RequestInit): Promise<any>
  plugin_name: string
  ref: typeof ref
  reactive: typeof reactive
  computed: typeof computed
}

export interface PluginConfig {
  plugin_name: string
  setup?(ctx: PluginContext): Record<string, any>
  style?(): string
  render?(h: any, state: any): any
}

export function Plugin(config: PluginConfig) {
  const { plugin_name, setup, style, render } = config

  const ctx: PluginContext = {
    fetch(action: string, opts: RequestInit = {}) {
      const url = action.startsWith('/')
        ? action
        : `/api/plugins/action/${plugin_name}/${action}`
      return apiFetch(url, { method: 'POST', ...opts })
    },
    plugin_name,
    ref,
    reactive,
    computed,
  }

  const styleCSS = style ? style() : ''
  const state = setup ? setup(ctx) : {}

  return {
    show() {
      const container = document.createElement('div')
      document.body.appendChild(container)

      const visible = ref(true)

      function close() {
        visible.value = false
        setTimeout(() => {
          app.unmount()
          container.remove()
        }, 300)
      }

      const App = {
        setup() {
          return () => {
            const children: any[] = []
            if (styleCSS) {
              children.push(h('style', {}, styleCSS))
            }
            if (render) {
              children.push(render(h, state))
            }
            return h('el-dialog', {
              modelValue: visible.value,
              'onUpdate:modelValue': (v: boolean) => { if (!v) close() },
              title: plugin_name,
              width: '800px',
              top: '5vh',
              appendToBody: false,
              destroyOnClose: true,
            }, { default: () => children })
          }
        },
      }

      const app = createApp(App)
      app.use(ElementPlus)
      app.mount(container)
    },
  }
}

export async function loadAndShow(name: string) {
  const url = `/iframe/${name}/index.js?_=${Date.now()}`
  const res = await fetch(url, { headers: authHeaders() })
  if (!res.ok) throw new Error(`加载插件失败: ${name}`)
  const code = await res.text()
  new Function('Plugin', code)(Plugin)
}
