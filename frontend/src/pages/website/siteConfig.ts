import { Plugin } from '@/utils/plugin'
import DirPicker from '@/components/DirPicker.vue'

const DEFAULT_ROOT = '/www/wwwroot/test.w.j7yx.com'

export function openSiteConfig(site: { id: number; name: string }) {
    Plugin({
        plugin_name: 'site_config',
        title: `站点修改[${site.name}]`,
        width: 800,
        height: 620,
        layout: 'tabpages',
        tabs: {
            domain: '域名管理',
            directory: '网站目录',
            rewrite: '伪静态',
            config: '配置文件',
            ssl: 'SSL证书',
            fastcgi: 'FastCgi',
            proxy: '反向代理',
            log: '网站日志',
            other: '其它设置',
        },
        pages: {
            domain: {
                render(h, state) {
                    return h('div', [
                        h('div', [
                            h('textarea', {
                                class: 'domain-input',
                                placeholder: '请输入域名，每行一个\n如：example.com',
                                value: state.domainText.value,
                                onInput: (e: any) => { state.domainText.value = e.target.value },
                            }),
                            h('div', { class: 'row' }, [
                                h('button', { class: 'btn', onClick: () => state.addDomains() }, '添加'),
                            ]),
                        ]),
                        h('table', { class: 'table' }, [
                            h('thead', [h('tr', [h('th', '域名'), h('th', '端口'), h('th', '操作')])]),
                            h('tbody', state.domains.value.length === 0
                                ? [h('tr', [h('td', { attrs: { colspan: 3, class: 'empty' } }, '暂无域名')])]
                                : state.domains.value.map((d: DomainItem) => h('tr', [
                                    h('td', h('a', { class: 'dlink', href: buildUrl(d.name), target: '_blank' }, d.name)),
                                    h('td', String(d.port)),
                                    h('td', h('a', { class: 'dlink danger-link', onClick: () => state.removeDomain(d.id) }, '删除')),
                                ]))),
                        ]),
                    ])
                },
            },
            directory: {
                render(h, state) {
                    return h('div', [
                        h('div', { class: 'dir-row' }, [
                            h('label', '网站目录'),
                            h('div', { class: 'dir-input-wrap' }, [
                                h('input', {
                                    class: 'dir-input',
                                    value: state.siteRoot.value,
                                    onInput: (e: any) => { state.siteRoot.value = e.target.value },
                                }),
                                h('button', { class: 'btn', onClick: () => state.openRootPicker() }, '浏览'),
                            ]),
                        ]),
                        h('div', { class: 'dir-row' }, [
                            h('label', '运行目录'),
                            h('div', { class: 'dir-input-wrap' }, [
                                h('input', {
                                    class: 'dir-input',
                                    value: state.runDir.value,
                                    onInput: (e: any) => { state.runDir.value = e.target.value },
                                }),
                                h('button', { class: 'btn', onClick: () => state.openRunPicker() }, '浏览'),
                            ]),
                        ]),
                        h('p', { class: 'tip' }, '运行目录只能是网站目录下的子目录，显示相对路径；与网站目录拼接后为 Nginx root'),
                        h('div', { class: 'row' }, [
                            h('button', { class: 'btn', onClick: () => state.saveDirs() }, '保存'),
                        ]),
                        h(DirPicker, {
                            modelValue: state.rootPickerVisible.value,
                            'onUpdate:modelValue': (v: boolean) => { state.rootPickerVisible.value = v },
                            initialPath: state.siteRoot.value,
                            onConfirm: (path: string) => state.onRootPicked(path),
                        }),
                        h(DirPicker, {
                            modelValue: state.runPickerVisible.value,
                            'onUpdate:modelValue': (v: boolean) => { state.runPickerVisible.value = v },
                            initialPath: state.siteRoot.value,
                            basePath: state.siteRoot.value,
                            showUp: true,
                            onConfirm: (path: string) => state.onRunPicked(path),
                        }),
                    ])
                },
            },
            rewrite: { render: emptyRender },
            config: { render: emptyRender },
            ssl: { render: emptyRender },
            fastcgi: { render: emptyRender },
            proxy: { render: emptyRender },
            log: { render: emptyRender },
            other: { render: emptyRender },
        },
        setup(ctx) {
            const { ref, toast } = ctx
            const domainText = ref('')
            let domainId = 3
            const domains = ref([
                { id: 1, name: 'example.com', port: 80 },
                { id: 2, name: 'www.example.com', port: 80 },
            ])

            function addDomains() {
                const lines = domainText.value.split('\n').map(s => s.trim()).filter(Boolean)
                if (lines.length === 0) return
                for (const line of lines) {
                    domains.value.push(parseDomain(++domainId, line))
                }
                domainText.value = ''
            }

            function removeDomain(id: number) {
                const idx = domains.value.findIndex(d => d.id === id)
                if (idx !== -1) domains.value.splice(idx, 1)
            }

            const siteRoot = ref(DEFAULT_ROOT)
            const runDir = ref('/')
            const rootPickerVisible = ref(false)
            const runPickerVisible = ref(false)

            function openRootPicker() {
                rootPickerVisible.value = true
            }

            function onRootPicked(path: string) {
                siteRoot.value = path
            }

            function openRunPicker() {
                runPickerVisible.value = true
            }

            function onRunPicked(path: string) {
                const root = normalizePath(siteRoot.value)
                const rel = path.startsWith(root) ? path.slice(root.length) : path
                runDir.value = rel || '/'
            }

            function saveDirs() {
                toast('目录已保存')
            }

            return {
                domainText, domains, addDomains, removeDomain,
                siteRoot, runDir, rootPickerVisible, runPickerVisible,
                openRootPicker, onRootPicked, openRunPicker, onRunPicked, saveDirs,
            }
        },
        style() {
            return `
                .domain-input{width:100%;height:110px;padding:8px 10px;border:1px solid #555;background:#1a1a1a;color:#ccc;border-radius:3px;font-size:13px;resize:vertical;outline:none;box-sizing:border-box}
                .domain-input:focus{border-color:#409eff}
                .dlink{color:#409eff;cursor:pointer;text-decoration:none}
                .dlink:hover{text-decoration:underline}
                .danger-link{color:#f56c6c}
                .danger-link:hover{text-decoration:underline}
                .empty{text-align:center;color:#666;padding:20px 0}
                .dir-row{display:flex;align-items:center;margin-bottom:10px}
                .dir-row label{width:70px;color:#aaa;font-size:13px;flex:0 0 auto}
                .dir-input-wrap{flex:1;display:flex;gap:8px;align-items:center}
                .dir-input{flex:1;padding:5px 10px;border:1px solid #555;background:#1a1a1a;color:#ccc;border-radius:3px;font-size:13px;outline:none;box-sizing:border-box}
                .dir-input:focus{border-color:#409eff}
            `
        },
    }).show()
}

interface DomainItem {
    id: number
    name: string
    port: number
}

function parseDomain(id: number, line: string): DomainItem {
    const m = line.match(/^(.+?)(?::(\d+))?$/)
    const name = m ? m[1].trim() : line
    const port = m && m[2] ? parseInt(m[2], 10) : 80
    return { id, name, port }
}

function buildUrl(name: string): string {
    return /^[a-zA-Z]+:\/\//.test(name) ? name : `http://${name}`
}

function normalizePath(p: string): string {
    let s = p.trim()
    if (!s.endsWith('/')) s += '/'
    return s
}

function emptyRender() {
    return null
}