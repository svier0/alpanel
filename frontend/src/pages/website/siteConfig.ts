import { Plugin } from '@/utils/plugin'

export function openSiteConfig(site: { id: number; name: string }) {
    Plugin({
        plugin_name: 'site_config',
        title: `站点修改[${site.name}]`,
        width: 700,
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
            domain: { render: emptyRender },
            directory: { render: emptyRender },
            rewrite: { render: emptyRender },
            config: { render: emptyRender },
            ssl: { render: emptyRender },
            fastcgi: { render: emptyRender },
            proxy: { render: emptyRender },
            log: { render: emptyRender },
            other: { render: emptyRender },
        },
        setup() {
            return {}
        },
    }).show()
}

function emptyRender() {
    return null
}