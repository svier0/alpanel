import { Plugin } from '@/utils/plugin'
import DirSelect from '@/components/DirSelect.vue'

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
                        h('div', { class: 'dir-field' }, [
                            h('span', { class: 'dir-field-label' }, '网站目录'),
                            h(DirSelect, {
                                modelValue: state.siteRoot.value,
                                'onUpdate:modelValue': (v: string) => { state.siteRoot.value = v },
                            }),
                            h('button', { class: 'btn btn-save', onClick: () => state.saveSiteDir() }, '保存'),
                        ]),
                        h('div', { class: 'dir-field' }, [
                            h('span', { class: 'dir-field-label' }, '运行目录'),
                            h(DirSelect, {
                                modelValue: state.runDir.value,
                                basePath: state.siteRoot.value,
                                'onUpdate:modelValue': (v: string) => { state.onRunPicked(v) },
                            }),
                            h('button', { class: 'btn btn-save', onClick: () => state.saveRunDir() }, '保存'),
                        ]),
                    ])
                },
            },
            rewrite: {
                render(h, state) {
                    return h('div', [
                        h(state.Editor, {
                            modelValue: state.rewriteContent.value,
                            'onUpdate:modelValue': (v: string) => { state.rewriteContent.value = v },
                            language: 'nginx',
                        }),
                        h('div', { class: 'row' }, [
                            h('button', { class: 'btn', onClick: () => state.saveRewrite() }, '保存'),
                        ]),
                    ])
                },
            },
            config: {
                render(h, state) {
                    return h('div', [
                        h(state.Editor, {
                            modelValue: state.configContent.value,
                            'onUpdate:modelValue': (v: string) => { state.configContent.value = v },
                            language: 'nginx',
                        }),
                        h('div', { class: 'row' }, [
                            h('button', { class: 'btn', onClick: () => state.saveConfig() }, '保存'),
                        ]),
                    ])
                },
            },
            ssl: { render: emptyRender },
            fastcgi: { render: emptyRender },
            proxy: { render: emptyRender },
            log: {
                render(h, state) {
                    const tabs = [
                        { key: 'access', label: '响应日志' },
                        { key: 'error', label: '错误日志' },
                    ]
                    return h('div', [
                        h('div', { class: 'sub-tabs' },
                            tabs.map(t => h('span', {
                                class: 'sub-tab' + (state.logActive.value === t.key ? ' active' : ''),
                                onClick: () => { state.logActive.value = t.key },
                            }, t.label))),
                        h(state.Editor, {
                            modelValue: state.logContent.value,
                            'onUpdate:modelValue': (v: string) => { state.logContent.value = v },
                            language: 'log',
                            readonly: true,
                        }),
                    ])
                },
            },
            other: { render: emptyRender },
        },
        setup(ctx) {
            const { ref, toast, Editor } = ctx
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
            const rewriteContent = ref('# 伪静态规则\n\nlocation / {\n    try_files $uri $uri/ /index.php?$query_string;\n}\n')
            const configContent = ref('# 站点配置文件\n')
            const logActive = ref<'access' | 'error'>('access')
            const logContent = ref('')

            function onRunPicked(path: string) {
                runDir.value = path.slice(siteRoot.value.length) || '/'
            }

            function saveSiteDir() {
                toast('已保存')
            }

            function saveRunDir() {
                toast('已保存')
            }

            function saveRewrite() {
                toast('已保存')
            }

            function saveConfig() {
                toast('已保存')
            }

            return {
                Editor,
                domainText, domains, addDomains, removeDomain,
                siteRoot, runDir, onRunPicked, saveSiteDir, saveRunDir,
                rewriteContent, configContent, saveRewrite, saveConfig,
                logActive, logContent,
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
                .dir-field{display:flex;align-items:center;gap:10px;margin-bottom:12px}
                .dir-field-label{color:#aaa;font-size:13px;white-space:nowrap;flex:0 0 auto}
                .btn-save{padding:5px 10px;flex:0 0 auto}
                .sub-tabs{display:flex;gap:4px;margin-bottom:10px;border-bottom:1px solid #2a2a2a}
                .sub-tab{padding:6px 16px;font-size:13px;color:#888;cursor:pointer;border-bottom:2px solid transparent}
                .sub-tab:hover{color:#ccc}
                .sub-tab.active{color:#fff;border-bottom-color:#409eff}
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

function emptyRender() {
    return null
}