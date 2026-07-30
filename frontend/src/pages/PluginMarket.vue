<template>
  <div class="plugin-page">
    <el-card shadow="never">
      <template #header>
        <div class="plugin-header">
          <span class="sect-title">插件市场</span>
          <el-button size="small" :icon="RefreshRight" @click="loadAll"/>
        </div>
      </template>
      <el-tabs v-model="activeTab">
        <el-tab-pane label="全部" name="all" />
        <el-tab-pane label="已安装" name="installed" />
        <el-tab-pane label="可升级" name="upgradable" />
      </el-tabs>
      <el-table :data="filtered" stripe size="small" style="width:100%">
        <el-table-column label="名称" min-width="200">
          <template #default="{ row }">
            <div class="plugin-name-cell">
              <img v-if="row.logo" :src="row.logo" class="plugin-logo" />
              <span>{{ row.title }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="author" label="作者" min-width="100" />
        <el-table-column prop="desc" label="描述" min-width="200" show-overflow-tooltip />
        <el-table-column label="位置" width="60" align="center">
          <template #default="{ row }">
            <el-icon v-if="row.installed" class="link-icon" @click="goDir(row.name)"><FolderOpened /></el-icon>
          </template>
        </el-table-column>
        <el-table-column label="状态" width="80" align="center">
          <template #default="{ row }">
            <span v-if="row.installed" class="status-installed">已安装</span>
            <span v-else class="status-missing">未安装</span>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="150" align="center">
          <template #default="{ row }">
            <el-button v-if="!row.installed" size="small" type="primary" @click="install(row)">安装</el-button>
            <template v-else>
              <el-button v-if="row.upgradable" size="small" type="warning" @click="install(row)">更新</el-button>
              <el-button size="small" type="danger" @click="uninstall(row)">卸载</el-button>
            </template>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { RefreshRight, FolderOpened } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { apiFetch } from '@/utils/api'

interface PluginItem {
  title: string
  name: string
  desc: string
  versions: string
  author: string
  home: string
  logo: string
  installed: boolean
  upgradable: boolean
}

const router = useRouter()
const activeTab = ref('installed')
const localPlugins = ref<PluginItem[]>([])
const remotePlugins = ref<PluginItem[]>([])

const GH_RAW = 'https://raw.githubusercontent.com/svier0/alpanel-plugins/master'

const allPlugins = computed(() => {
  const map = new Map<string, PluginItem>()
  for (const r of remotePlugins.value) map.set(r.name, r)
  for (const l of localPlugins.value) {
    const existing = map.get(l.name)
    if (existing) {
      existing.installed = true
      existing.upgradable = existing.versions !== l.versions
    } else {
      map.set(l.name, { ...l, installed: true, upgradable: false, logo: `/static/img/plugins/icon/${l.name}.png` })
    }
  }
  return [...map.values()]
})

const filtered = computed(() => {
  if (activeTab.value === 'installed') return allPlugins.value.filter(p => p.installed)
  if (activeTab.value === 'upgradable') return allPlugins.value.filter(p => p.upgradable)
  return allPlugins.value
})

async function loadAll() {
  try {
    localPlugins.value = (await apiFetch('/api/plugins')).map((p: any) => ({
      ...p, installed: true, upgradable: false, logo: `/static/img/plugins/icon/${p.name}.png`
    }))
  } catch { localPlugins.value = [] }

  try {
    remotePlugins.value = (await apiFetch('/api/plugins/remote')).map((p: any) => ({
      ...p, installed: false, upgradable: false, logo: `${GH_RAW}/plugins/${p.name}/icon.png`
    }))
  } catch { remotePlugins.value = [] }
}

async function install(row: PluginItem) {
  try {
    await apiFetch('/api/plugins/install', { method: 'POST', body: JSON.stringify({ name: row.name }) })
    ElMessage.success('安装成功')
    await loadAll()
  } catch { ElMessage.error('安装失败') }
}

async function uninstall(row: PluginItem) {
  try {
    await apiFetch('/api/plugins/uninstall', { method: 'POST', body: JSON.stringify({ name: row.name }) })
    ElMessage.success('卸载成功')
    await loadAll()
  } catch { ElMessage.error('卸载失败') }
}

function goDir(name: string) {
  router.push(`/file?path=/www/server/panel/plugin/${name}`)
}

onMounted(loadAll)
</script>

<style scoped>
.plugin-page { display: flex; flex-direction: column; height: 100%; }
.plugin-header { display: flex; align-items: center; justify-content: space-between; }
.plugin-name-cell { display: flex; align-items: center; gap: 8px; }
.plugin-logo { width: 28px; height: 28px; border-radius: 4px; object-fit: contain; }
.link-icon { cursor: pointer; font-size: 18px; color: var(--el-color-primary); }
.link-icon:hover { opacity: 0.7; }
.status-installed { color: var(--el-color-success); }
.status-missing { color: var(--el-color-info); }
</style>
