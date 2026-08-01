import { h, createApp, ref, reactive, computed } from 'vue'
import ElementPlus from 'element-plus'
import { ElDialog } from 'element-plus'
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
  width?: string | number
  height?: string | number
  setup?(ctx: PluginContext): Record<string, any>
  style?(): string
  render?(h: any, state: any): any
}

export function Plugin(config: PluginConfig) {
  const { plugin_name, width, height, setup, style, render } = config

  const dialogWidth = typeof width === 'number'
    ? `${width}px`
    : width && typeof width === 'string'
      ? width
      : '620px'

  const dialogHeight = typeof height === 'number'
    ? `${height}px`
    : height && typeof height === 'string'
      ? height
      : '620px'

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
            children.push(h('style', {}, `.el-dialog{height:${dialogHeight};display:flex;flex-direction:column}.el-dialog__body{flex:1;overflow:auto;padding:0}`))
            if (styleCSS) {
              children.push(h('style', {}, styleCSS))
            }
            if (render) {
              children.push(render(h, state))
            }
            return h(ElDialog, {
              modelValue: visible.value,
              'onUpdate:modelValue': (v: boolean) => { if (!v) close() },
              title: plugin_name,
              width: dialogWidth,
              alignCenter: true,
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

export async function openPlugin(name: string) {
  const url = `/iframe/${name}/index.js?_=${Date.now()}`
  const res = await fetch(url, { headers: authHeaders() })
  if (!res.ok) throw new Error(`加载插件失败: ${name}`)
  const code = await res.text()
  try {
    new Function('Plugin', code)(Plugin)
  } catch (e) {
    console.error('[openPlugin] execution error:', e)
  }
}
