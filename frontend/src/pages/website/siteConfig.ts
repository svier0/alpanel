import { Plugin } from '@/utils/plugin'

export function openSiteConfig(site: { id: number; name: string }) {
    Plugin({
        plugin_name: `站点修改 - ${site.name}`,
        width: 700,
        height: 620,
        setup() {
            return {}
        },
        render() {
            return null
        },
    }).show()
}
