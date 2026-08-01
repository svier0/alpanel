import { h, createApp, ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import ElementPlus from 'element-plus'
import { ElDialog } from 'element-plus'
import { apiFetch, authHeaders } from './api'
import { EditorView, keymap } from '@codemirror/view'
import { EditorState } from '@codemirror/state'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { oneDark } from '@codemirror/theme-one-dark'
import { nginx } from '@codemirror/legacy-modes/mode/nginx'
import { shell } from '@codemirror/legacy-modes/mode/shell'
import { json } from '@codemirror/lang-json'
import { StreamLanguage } from '@codemirror/language'

export interface PluginContext {
  api(action: string, opts?: RequestInit): Promise<any>
  plugin_name: string
  ref: typeof ref
  reactive: typeof reactive
  computed: typeof computed
  onMounted: (fn: () => void) => void
  onUnmounted: (fn: () => void) => void
  createEditor: (el: HTMLElement, options: EditorOptions) => EditorHandle
}

export interface EditorOptions {
  value: string
  language?: string
  readonly?: boolean
  onChange?: (value: string) => void
}

export interface EditorHandle {
  destroy: () => void
  setValue: (value: string) => void
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

  const scopeClass = 'plugin-dlg'

  function scopeCSS(css: string, prefix: string): string {
    return css.split('}').map(block => {
      const idx = block.lastIndexOf('{')
      if (idx === -1) return block
      const selector = block.slice(0, idx).trim()
      if (!selector || selector.startsWith('@')) return block
      return selector.split(',').map(s => `${prefix} ${s.trim()}`).join(', ') + '{' + block.slice(idx + 1)
    }).join('}')
  }

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

  const mountFns: (() => void)[] = []
  const unmountFns: (() => void)[] = []

  const ctx: PluginContext = {
    api(action: string, opts: RequestInit = {}) {
      const url = action.startsWith('/')
        ? action
        : `/api/plugins/action/${plugin_name}/${action}`
      return apiFetch(url, { method: 'POST', ...opts })
    },
    plugin_name,
    ref,
    reactive,
    computed,
    onMounted(fn: () => void) { mountFns.push(fn) },
    onUnmounted(fn: () => void) { unmountFns.push(fn) },
    createEditor(el: HTMLElement, options: EditorOptions): EditorHandle {
      const extensions: any[] = [
        history(),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        oneDark,
        EditorView.lineWrapping,
      ]
      if (options.readonly) {
        extensions.push(EditorState.readOnly.of(true))
        extensions.push(EditorView.editable.of(false))
      }
      const lang = options.language || ''
      if (lang === 'nginx') {
        extensions.push(StreamLanguage.define(nginx))
      } else if (lang === 'shell') {
        extensions.push(StreamLanguage.define(shell))
      } else if (lang === 'json') {
        extensions.push(json())
      }
      if (options.onChange) {
        extensions.push(EditorView.updateListener.of((update: any) => {
          if (update.docChanged) {
            options.onChange!(update.state.doc.toString())
          }
        }))
      }
      const view = new EditorView({
        state: EditorState.create({ doc: options.value, extensions }),
        parent: el,
      })
      return {
        destroy() { view.destroy() },
        setValue(value: string) {
          view.dispatch({
            changes: { from: 0, to: view.state.doc.length, insert: value }
          })
        },
      }
    },
  }

  const styleCSS = style ? scopeCSS(style(), `.${scopeClass}`) : ''
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
          onMounted(() => { mountFns.forEach(fn => fn()) })
          onUnmounted(() => { unmountFns.forEach(fn => fn()) })
          return () => {
            const children: any[] = []
            children.push(h('style', {}, `.el-dialog{height:${dialogHeight};display:flex;flex-direction:column}.el-dialog__body{flex:1;overflow:auto}`))
            if (styleCSS) {
              children.push(h('style', {}, styleCSS))
            }
            if (render) {
              children.push(render(h, state))
            }
            return h(ElDialog, {
              modelValue: visible.value,
              'onUpdate:modelValue': (v: boolean) => { if (!v) close() },
              class: scopeClass,
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
