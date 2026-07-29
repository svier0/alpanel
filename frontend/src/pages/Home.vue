<template>
  <el-card class="page-card">
    <template #header>
      <div class="card-header-row">
        <div class="header-left">
          <img
            v-if="osLogo"
            :src="osLogo"
            class="os-logo"
            @error="osLogo = ''"
          />
          <el-icon v-else class="os-logo-icon"><Monitor /></el-icon>
          <el-tooltip placement="bottom">
            <template #content>
              <div>系统：{{ osInfo.os_pretty }} {{ osInfo.os_arch }}</div>
              <div>持续运行：{{ osInfo.os_uptime }}</div>
            </template>
            <span>{{ osInfo.os_id.charAt(0).toUpperCase() + osInfo.os_id.slice(1) }} {{ osInfo.os_version }}</span>
          </el-tooltip>
        </div>
        <div class="header-right">
          <span class="header-version">v{{ VERSION }}</span>
          <el-button size="small" disabled>
            <el-icon><RefreshRight /></el-icon>
            {{ t('header.restart') }}
          </el-button>
        </div>
      </div>
    </template>
    <div v-if="stat" class="stat-grid">
      <div class="stat-section">
        <div class="stat-title">CPU</div>
        <div class="stat-row"><span class="stat-label">型号</span><span class="stat-value">{{ stat.cpu.name }}</span></div>
        <div class="stat-row"><span class="stat-label">物理数量</span><span class="stat-value">{{ stat.cpu.physical_count }}</span></div>
        <div class="stat-row"><span class="stat-label">物理核心</span><span class="stat-value">{{ stat.cpu.core_count }}</span></div>
        <div class="stat-row"><span class="stat-label">逻辑核心</span><span class="stat-value">{{ stat.cpu.logical_count }}</span></div>
        <div class="stat-row">
          <span class="stat-label">使用率</span>
          <span class="stat-value" :style="{ color: stat.cpu.usage_percent > 80 ? '#e6a23c' : '#67c23a' }">{{ stat.cpu.usage_percent.toFixed(1) }}%</span>
        </div>
      </div>
      <div class="stat-section">
        <div class="stat-title">内存</div>
        <div class="stat-row"><span class="stat-label">总容量</span><span class="stat-value">{{ fmtSize(stat.mem.total) }}</span></div>
        <div class="stat-row"><span class="stat-label">已用</span><span class="stat-value">{{ fmtSize(stat.mem.used) }}</span></div>
        <div class="stat-row">
          <span class="stat-label">占用率</span>
          <span class="stat-value" :style="{ color: stat.mem.percent > 80 ? '#e6a23c' : '#67c23a' }">{{ stat.mem.percent.toFixed(1) }}%</span>
        </div>
      </div>
      <div class="stat-section">
        <div class="stat-title">硬盘</div>
        <div v-for="d in stat.disks" :key="d.mount" class="stat-row">
          <span class="stat-label">{{ d.mount }}</span>
          <span class="stat-value">{{ fmtSize(d.used) }} / {{ fmtSize(d.total) }}</span>
        </div>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Monitor, RefreshRight } from '@element-plus/icons-vue'
import { apiFetch } from '@/utils/api'

const VERSION = '0.1.0'
const { t } = useI18n()

interface OsInfo {
  os_id: string
  os_name: string
  os_version: string
  os_pretty: string
  os_arch: string
  os_uptime: string
}

interface CpuStat {
  name: string
  physical_count: number
  core_count: number
  logical_count: number
  usage_percent: number
}
interface MemStat {
  total: number
  used: number
  percent: number
}
interface DiskStat {
  mount: string
  total: number
  used: number
  percent: number
}
interface SystemStat {
  cpu: CpuStat
  mem: MemStat
  disks: DiskStat[]
}

const osInfo = ref<OsInfo>({ os_id: '', os_name: '', os_version: '', os_pretty: '', os_arch: '', os_uptime: '' })
const osLogo = ref('')
const stat = ref<SystemStat | null>(null)

const logoMap: Record<string, string> = {
  debian: 'https://www.debian.org/logos/openlogo-nd.svg',
  alpine: 'https://cdn.jsdelivr.net/gh/homarr-labs/dashboard-icons/svg/alpine-linux.svg',
  ubuntu: 'https://ubuntu.com/static/images/logos/logo-ubuntu.svg',
}

function fmtSize(bytes: number): string {
  if (bytes >= 1073741824) return (bytes / 1073741824).toFixed(1) + ' GB'
  if (bytes >= 1048576) return (bytes / 1048576).toFixed(1) + ' MB'
  if (bytes >= 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return bytes + ' B'
}

let timer: ReturnType<typeof setInterval> | null = null

onMounted(async () => {
  try {
    const data = await apiFetch('/api/system/info')
    osInfo.value = data
    osLogo.value = logoMap[data.os_id] || ''
  } catch {}
  const poll = async () => {
    try {
      stat.value = await apiFetch('/api/system/stat')
    } catch {}
  }
  await poll()
  timer = setInterval(poll, 3000)
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<style scoped>
.page-card {
  flex: 1;
  display: flex;
  flex-direction: column;
}
.page-card :deep(.el-card__body) {
  flex: 1;
  display: flex;
  flex-direction: column;
}
.card-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}
.os-logo {
  width: 22px;
  height: 22px;
  object-fit: contain;
}
.os-logo-icon {
  font-size: 22px;
  color: var(--el-color-primary);
}
.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}
.header-version {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}
.stat-grid {
  display: flex;
  gap: 24px;
  flex-wrap: wrap;
}
.stat-section {
  min-width: 240px;
  flex: 1;
}
.stat-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin-bottom: 8px;
  padding-bottom: 4px;
  border-bottom: 1px solid var(--el-border-color-light);
}
.stat-row {
  display: flex;
  justify-content: space-between;
  padding: 4px 0;
  font-size: 13px;
}
.stat-label {
  color: var(--el-text-color-secondary);
}
.stat-value {
  color: var(--el-text-color-primary);
  font-family: monospace;
}
</style>
