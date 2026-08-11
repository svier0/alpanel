import { h, createApp, ref, reactive, computed, onMounted, onUnmounted, watch } from 'vue'
import ElementPlus from 'element-plus'
import { ElDialog } from 'element-plus'
import { apiFetch, authHeaders } from './api'
import { EditorView, keymap, lineNumbers } from '@codemirror/view'
import { EditorState } from '@codemirror/state'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { oneDark } from '@codemirror/theme-one-dark'
import { nginx } from '@codemirror/legacy-modes/mode/nginx'
import { shell } from '@codemirror/legacy-modes/mode/shell'
import { properties } from '@codemirror/legacy-modes/mode/properties'
import { json } from '@codemirror/lang-json'
import { StreamLanguage } from '@codemirror/language'
import { indentationMarkers } from '@replit/codemirror-indentation-markers'

function langExtension(lang: string) {
    if (lang === 'nginx') return StreamLanguage.define(nginx)
    if (lang === 'shell') return StreamLanguage.define(shell)
    if (lang === 'json') return json()
    if (lang === 'ini') return StreamLanguage.define(properties)
    return []
}

const PluginEditor = {
    props: { modelValue: String, language: String, readonly: Boolean },
    emits: ['update:modelValue'],
    setup(props: any, { emit }: any) {
        const el = ref<HTMLElement>()
        let view: EditorView | null = null

        function create(viewEl: HTMLElement) {
            const exts: any[] = [lineNumbers(), history(), keymap.of([...defaultKeymap, ...historyKeymap]), oneDark, EditorView.lineWrapping, indentationMarkers()]
            if (props.readonly) { exts.push(EditorState.readOnly.of(true), EditorView.editable.of(false)) }
            const langExt = langExtension(props.language || '')
            if (langExt) exts.push(...(Array.isArray(langExt) ? langExt : [langExt]))
            exts.push(EditorView.updateListener.of((u: any) => { if (u.docChanged) emit('update:modelValue', u.state.doc.toString()) }))
            view = new EditorView({ state: EditorState.create({ doc: props.modelValue || '', extensions: exts }), parent: viewEl })
        }

        onMounted(() => { if (el.value) create(el.value) })
        onUnmounted(() => { view?.destroy(); view = null })
        watch(() => props.modelValue, (v: string) => {
            if (view && v !== view.state.doc.toString()) {
                view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: v } })
            }
        })

        return () => h('div', { ref: el, class: 'plugin-editor' })
    },
}

export interface PluginContext {
    api(action: string, opts?: RequestInit): Promise<any>
    plugin_name: string
    plugin_title: { value: string }
    ref: typeof ref
    reactive: typeof reactive
    computed: typeof computed
    toast(msg: string, type?: string): void
    onMounted: (fn: () => void) => void
    onUnmounted: (fn: () => void) => void
    Editor: typeof PluginEditor
}

export interface PluginPage {
    onLoad?(ctx: PluginContext, state: any): void | Promise<any>
    render?(h: any, state: any): any
}

export interface PluginConfig {
    plugin_name: string
    title?: string
    layout?: 'none' | 'tabpages' | string
    width?: string | number
    height?: string | number
    tabs?: Record<string, string>
    pages?: Record<string, PluginPage>
    setup?(ctx: PluginContext): Record<string, any>
    style?(): string
    render?(h: any, state: any): any
    onClose?: () => void
}

export function Plugin(config: PluginConfig) {
    const { plugin_name, title, width, height, setup, style, onClose } = config
    const layout = (config.layout || 'none') === 'none' ? 'none' : (config.layout || 'none')
    const plugin_title = ref(title || plugin_name)

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

    const TABPAGES_CSS = [
        '@keyframes spin{to{transform:rotate(360deg)}}',
        '.app{display:flex;height:100%}',
        '.side{width:100px;background:#202020}',
        '.item{padding:10px 16px;color:#666;cursor:pointer;border-left:2px solid transparent;font-size:13px}',
        '.item:hover{color:#aaa}',
        '.item.active{color:#fff;background:#141414;border-left-color:#444}',
        '.content{flex:1;padding:0 20px;background:#141414;color:#ccc;overflow:auto}',
        '.spin-wrap{display:flex;align-items:center;justify-content:center;height:100%;min-height:200px}',
        '.spin{width:28px;height:28px;border:2px solid #333;border-top-color:#666;border-radius:50%;animation:spin .6s linear infinite}',
        '.row{display:flex;gap:10px;margin-top:10px}',
        '.btn{padding:5px 14px;border:1px solid #555;background:#141414;color:#ccc;border-radius:3px;cursor:pointer;font-size:13px}',
        '.btn:hover{color:#fff;border-color:#888}',
        '.btn.loading{opacity:.6;pointer-events:none}',
        '.on{color:#4ade80;font-weight:bold}',
        '.off{color:red;font-weight:bold}',
        '.tip{color:#666;font-size:13px}',
        '.form{display:grid;grid-template-columns:auto 58px 1fr;align-items:center;gap:4px 10px;margin-bottom:12px}',
        '.form label{font-family:monospace;color:#aaa;font-size:13px;white-space:nowrap}',
        '.form .slt{justify-self:start}',
        '.form input{padding:5px 10px;border:1px solid #555;background:#1a1a1a;color:#ccc;border-radius:3px;font-size:13px;width:auto;outline:none;box-sizing:border-box}',
        '.form input:focus{border-color:#409eff}',
        '.form-grid{display:grid;grid-template-columns:max-content 80px 1fr;gap:4px 10px;align-items:center;margin-bottom:4px}',
        '.form-grid label{font-family:monospace;color:#aaa;font-size:13px;white-space:nowrap}',
        '.form-grid input{padding:5px 10px;border:1px solid #555;background:#1a1a1a;color:#ccc;border-radius:3px;font-size:13px;outline:none}',
        '.form-grid input:focus{border-color:#409eff}',
        '.slt{padding:5px 10px;border:1px solid #555;background:#1a1a1a;color:#ccc;border-radius:3px;font-size:13px;outline:none;cursor:pointer;width:80px}',
        '.slt:focus{border-color:#409eff}',
        '.table{width:100%;border-collapse:collapse;margin-top:4px}',
        '.table th{background:#202020;color:#ccc;font-weight:500;white-space:nowrap;text-align:left}',
        '.table th,.table td{padding:8px 14px;border-bottom:1px solid #2a2a2a;font-size:14px}',
        '.table td{color:#aaa}',
        '.plugin-editor{font-size:13px;min-height:200px}',
        '.plugin-editor .cm-editor{outline:none}',
        '.plugin-editor .cm-scroller{font-family:monospace;line-height:1.5}',
        '.toast{position:fixed;top:12px;right:20px;padding:8px 18px;border-radius:4px;font-size:13px;z-index:9999;color:#fff;background:#333}',
        '.toast.ok{background:#16a34a}',
        '.toast.err{background:#dc2626}',
    ].join(' ')

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

    const toastMsg = ref('')
    const toastType = ref('')
    let toastTimer: ReturnType<typeof setTimeout> | null = null

    const ctx: PluginContext = {
        api(action: string, opts: RequestInit = {}) {
            const url = action.startsWith('/')
                ? action
                : `/api/plugins/action/${plugin_name}/${action}`
            return apiFetch(url, { method: 'POST', ...opts })
        },
        plugin_name,
        plugin_title,
        ref,
        reactive,
        computed,
        toast(msg: string, type?: string) {
            toastMsg.value = msg
            toastType.value = type || 'ok'
            if (toastTimer) clearTimeout(toastTimer)
            toastTimer = setTimeout(() => { toastMsg.value = '' }, 3000)
        },
        onMounted(fn: () => void) { mountFns.push(fn) },
        onUnmounted(fn: () => void) { unmountFns.push(fn) },
        Editor: PluginEditor,
    }

    const state = setup ? setup(ctx) : {}

    // layout == 'tabpages'：由 tabs/pages 生成侧面 tab + 内容区外壳，代替原 render
    if (layout === 'tabpages') {
        const tabKeys = config.tabs ? Object.keys(config.tabs) : []
        const activeTab = ref(tabKeys[0] || '')
        const loading = ref(false)

        async function runOnLoad(key: string) {
            const page = config.pages?.[key]
            if (!page?.onLoad) { loading.value = false; return }
            loading.value = true
            try {
                const ret = page.onLoad(ctx, state)
                if (ret && typeof (ret as any).then === 'function') await ret
            } catch (e) {
                console.error(`[plugin:${plugin_name}] page ${key} onLoad error:`, e)
            } finally {
                loading.value = false
            }
        }

        const switchTo = (key: string) => {
            if (key === activeTab.value) return
            activeTab.value = key
            runOnLoad(key)
        }

        mountFns.push(() => { runOnLoad(tabKeys[0] || '') })

        // 覆盖配置里的 render
        ;(config as any).render = (h: any, st: any) => {
            return h('div', { class: 'app' }, [
                h('nav', { class: 'side' },
                    tabKeys.map(k => h('div', {
                        class: 'item' + (activeTab.value === k ? ' active' : ''),
                        onClick: () => switchTo(k),
                    }, config.tabs![k]))),
                h('main', { class: 'content' }, [
                    toastMsg.value ? h('div', { class: 'toast ' + (toastType.value || 'ok') }, toastMsg.value) : null,
                    loading.value
                        ? h('div', { class: 'spin-wrap' }, h('div', { class: 'spin' }))
                        : h('div', { key: activeTab.value },
                                (config.pages?.[activeTab.value]?.render ?? (() => null))(h, st)),
                ]),
            ])
        }
    }

    const render = config.render

    // tabpages 内置样式拼到插件 style 前面，一并 scope
    const builtinCSS = layout === 'tabpages' ? TABPAGES_CSS + ' ' : ''
    const styleCSS = scopeCSS(builtinCSS + (style ? style() : ''), `.${scopeClass}`)

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
                    if (onClose) onClose()
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
                            title: plugin_title.value,
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
