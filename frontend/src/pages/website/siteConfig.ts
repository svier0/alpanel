import { Plugin } from '@/utils/plugin'
import { parseDomains } from '@/utils/domain'
import DirSelect from '@/components/DirSelect.vue'

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
                onLoad: (_ctx, state) => state.loadSite(),
                render(h, state) {
                    return h('div', [
                        h('div', [
                            h('textarea', {
                                class: 'domain-input',
                                rows: 5,
                                placeholder: '如需填写多个域名，请换行填写，每行一个域名，默认为80端口\nIP地址格式：192.168.1.199\n泛解析添加方法 *.domain.com\n如另加端口格式为 www.domain.com:88\nipv6格式：[2001:db8:85a3::8a2e:370:7334]:88',
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
                onLoad: (_ctx, state) => state.loadSite(),
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
                onLoad: (_ctx, state) => state.loadFiles(),
                render(h, state) {
                    return h('div', [
                        h(state.Editor, {
                            modelValue: state.rewriteContent.value,
                            'onUpdate:modelValue': (v: string) => { state.rewriteContent.value = v },
                            language: 'nginx',
                            height: 480,
                        }),
                        h('div', { class: 'row' }, [
                            h('button', { class: 'btn', onClick: () => state.saveRewrite() }, '保存'),
                        ]),
                    ])
                },
            },
            config: {
                onLoad: (_ctx, state) => state.loadFiles(),
                render(h, state) {
                    return h('div', [
                        h(state.Editor, {
                            modelValue: state.configContent.value,
                            'onUpdate:modelValue': (v: string) => { state.configContent.value = v },
                            language: 'nginx',
                            height: 480,
                        }),
                        h('div', { class: 'row' }, [
                            h('button', { class: 'btn', onClick: () => state.saveConfig() }, '保存'),
                        ]),
                    ])
                },
            },
            ssl: { render: emptyRender },
            fastcgi: {
                onLoad: (_ctx, state) => state.loadSite(),
                render(h, state) {
                    const options = state.phpVersions.value.map((v: string) =>
                        h('option', { value: `php${v}` }, `PHP${v}`))
                    return h('div', [
                        h('div', { class: 'fcgi-row' }, [
                            h('span', { class: 'fcgi-label' }, 'PHP版本'),
                            h('select', {
                                class: 'fcgi-select',
                                value: state.fcgiVersion.value,
                                onInput: (e: any) => { state.fcgiVersion.value = e.target.value },
                            }, [
                                h('option', { value: '0' }, '纯静态'),
                                ...options,
                            ]),
                            h('button', { class: 'btn', onClick: () => state.saveFcgi() }, '切换'),
                        ]),
                    ])
                },
            },
            proxy: { render: emptyRender },
            log: {
                onLoad: (_ctx, state) => state.loadLog(state.logActive.value),
                render(h, state) {
                    const tabs = [
                        { key: 'access', label: '响应日志' },
                        { key: 'error', label: '错误日志' },
                    ]
                    return h('div', [
                        h('div', { class: 'sub-tabs' },
                            tabs.map(t => h('span', {
                                class: 'sub-tab' + (state.logActive.value === t.key ? ' active' : ''),
                                onClick: () => { state.switchLog(t.key) },
                            }, t.label))),
                        h(state.Editor, {
                            modelValue: state.logContent.value,
                            'onUpdate:modelValue': (v: string) => { state.logContent.value = v },
                            language: 'log',
                            readonly: true,
                            height: 520,
                        }),
                    ])
                },
            },
            other: { render: emptyRender },
        },
        setup(ctx) {
            const { ref, toast, Editor } = ctx
            const domainText = ref('')
            const domains = ref<DomainItem[]>([])
            const siteRoot = ref('')
            const runDir = ref('/')
            const rewriteContent = ref('')
            const configContent = ref('')
            const logActive = ref<'access' | 'error'>('access')
            const logContent = ref('')
            const fcgiVersion = ref('0')
            const phpVersions = ref<string[]>([])

            const api = (url: string, opts: any = {}) => ctx.api(url, { method: 'GET', ...opts })

            async function loadSite() {
                try {
                    if (phpVersions.value.length === 0) {
                        phpVersions.value = (await api('/api/system/php-versions')) || []
                    }
                    const s = await api(`/api/sites/${site.id}`)
                    siteRoot.value = s.path || ''
                    runDir.value = s.run_dir || '/'
                    domains.value = (s.domains || []).map((d: any) => ({ id: d.id, name: d.name, port: d.port }))
                    const pv = (s.phpversion || '').replace('.', '')
                    fcgiVersion.value = pv ? `php${pv}` : '0'
                } catch (e: any) {
                    toast(e?.message || '加载失败', 'err')
                }
            }

            async function loadFiles() {
                try {
                    const d = await api(`/api/sites/${site.id}/files`)
                    rewriteContent.value = d.rewrite || ''
                    configContent.value = d.config || ''
                } catch (e: any) {
                    toast(e?.message || '加载失败', 'err')
                }
            }

            async function loadLog(type: string) {
                try {
                    const d = await api(`/api/sites/${site.id}/logs?type=${type}`)
                    logContent.value = d.content || ''
                } catch (e: any) {
                    logContent.value = ''
                    toast(e?.message || '加载失败', 'err')
                }
            }

            function switchLog(key: string) {
                logActive.value = key as any
                loadLog(key)
            }

            function addDomains() {
                const lines = domainText.value.split('\n').map(s => s.trim()).filter(Boolean)
                if (lines.length === 0) return
                const list = parseDomains(domainText.value)
                for (let i = 0; i < list.length; i++) {
                    domains.value.push({ id: -(Date.now() + i), name: list[i].name, port: list[i].port ?? 80 })
                }
                domainText.value = ''
            }

            function removeDomain(id: number) {
                const idx = domains.value.findIndex(d => d.id === id)
                if (idx !== -1) domains.value.splice(idx, 1)
            }

            function onRunPicked(path: string) {
                runDir.value = path.slice(siteRoot.value.length) || '/'
            }

            function saveSiteDir() {
                ctx.api(`/api/sites/${site.id}`, {
                    method: 'PUT',
                    body: JSON.stringify({ path: siteRoot.value }),
                }).then(() => {
                    toast('已保存')
                }).catch((e: any) => {
                    toast(e?.message || '保存失败', 'err')
                })
            }

            function saveRunDir() {
                ctx.api(`/api/sites/${site.id}`, {
                    method: 'PUT',
                    body: JSON.stringify({ php_run_path: runDir.value }),
                }).then(() => {
                    toast('已保存')
                }).catch((e: any) => {
                    toast(e?.message || '保存失败', 'err')
                })
            }

            function saveRewrite() {
                toast('已保存')
            }

            function saveConfig() {
                toast('已保存')
            }

            function saveFcgi() {
                const pv = fcgiVersion.value === '0' ? '' : fcgiVersion.value.replace('php', '').split('').join('.')
                ctx.api(`/api/sites/${site.id}`, {
                    method: 'PUT',
                    body: JSON.stringify({ phpversion: pv }),
                }).then(() => {
                    toast('切换成功')
                }).catch((e: any) => {
                    toast(e?.message || '切换失败', 'err')
                })
            }

            return {
                Editor,
                domainText, domains, addDomains, removeDomain,
                siteRoot, runDir, onRunPicked, saveSiteDir, saveRunDir,
                rewriteContent, configContent, saveRewrite, saveConfig,
                logActive, logContent, switchLog, fcgiVersion, phpVersions, saveFcgi,
                loadSite, loadFiles, loadLog,
            }
        },
        style() {
            return `
                .domain-input{width:100%;padding:8px 10px;border:1px solid #555;background:#1a1a1a;color:#ccc;border-radius:3px;font-size:13px;resize:vertical;outline:none;box-sizing:border-box}
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
                .fcgi-row{display:flex;align-items:center;gap:10px}
                .fcgi-label{color:#aaa;font-size:13px;flex:0 0 auto}
                .fcgi-select{padding:5px 10px;border:1px solid #555;background:#1a1a1a;color:#ccc;border-radius:3px;font-size:13px;outline:none;cursor:pointer}
                .fcgi-select:focus{border-color:#409eff}
            `
        },
    }).show()
}

interface DomainItem {
    id: number
    name: string
    port: number
}

function buildUrl(name: string): string {
    return /^[a-zA-Z]+:\/\//.test(name) ? name : `http://${name}`
}

function emptyRender() {
    return null
}