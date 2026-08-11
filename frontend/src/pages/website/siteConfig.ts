import { Plugin } from '@/utils/plugin'

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
                        h('div', { class: 'domain-add' }, [
                            h('textarea', {
                                class: 'domain-input',
                                placeholder: '请输入域名，每行一个\n如：example.com',
                                value: state.domainText.value,
                                onInput: (e: any) => { state.domainText.value = e.target.value },
                            }),
                            h('button', { class: 'btn add', onClick: () => state.addDomains() }, '添加'),
                        ]),
                        h('table', { class: 'table' }, [
                            h('thead', [h('tr', [h('th', '域名'), h('th', '端口'), h('th', '操作')])]),
                            h('tbody', state.domains.value.length === 0
                                ? [h('tr', [h('td', { attrs: { colspan: 3, class: 'empty' } }, '暂无域名')])]
                                : state.domains.value.map((d: DomainItem) => h('tr', [
                                    h('td', h('a', { class: 'dlink', href: buildUrl(d.name), target: '_blank' }, d.name)),
                                    h('td', String(d.port)),
                                    h('td', h('button', { class: 'btn danger', onClick: () => state.removeDomain(d.id) }, '删除')),
                                ]))),
                        ]),
                    ])
                },
            },
            directory: { render: emptyRender },
            rewrite: { render: emptyRender },
            config: { render: emptyRender },
            ssl: { render: emptyRender },
            fastcgi: { render: emptyRender },
            proxy: { render: emptyRender },
            log: { render: emptyRender },
            other: { render: emptyRender },
        },
        setup(ctx) {
            const { ref } = ctx
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

            return { domainText, domains, addDomains, removeDomain }
        },
        style() {
            return `
                .domain-add{display:flex;gap:10px;align-items:stretch;margin-bottom:12px}
                .domain-input{flex:1;height:110px;padding:8px 10px;border:1px solid #555;background:#1a1a1a;color:#ccc;border-radius:3px;font-size:13px;resize:vertical;outline:none;box-sizing:border-box}
                .domain-input:focus{border-color:#409eff}
                .btn.add{flex:0 0 70px}
                .btn.danger{color:#f56c6c;border-color:#7a3b3b}
                .btn.danger:hover{color:#fff;border-color:#f56c6c}
                .dlink{color:#409eff;cursor:pointer;text-decoration:none}
                .dlink:hover{text-decoration:underline}
                .empty{text-align:center;color:#666;padding:20px 0}
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