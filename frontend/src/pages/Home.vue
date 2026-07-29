<template>
  <div class="home-layout">
    <div class="home-left">
      <el-card class="section-card">
        <template #header><span>状态</span></template>
        <div class="rings-row">
          <div v-for="r in rings" :key="r.label" class="ring-item">
            <el-progress type="dashboard" :percentage="r.pct" :color="ringColor(r.pct)" :width="90" :stroke-width="8">
              <template #default>{{ r.pct.toFixed(1) }}%</template>
            </el-progress>
            <div class="ring-label">{{ r.label }}</div>
            <div class="ring-desc">{{ r.desc }}</div>
          </div>
        </div>
      </el-card>

      <el-card class="section-card">
        <template #header><span>概览</span></template>
        <div class="overview-row">
          <div class="overview-item">
            <div class="ov-num">{{ overview.sites }}</div>
            <div class="ov-label">站点</div>
          </div>
          <div class="overview-item">
            <div class="ov-num">{{ overview.databases }}</div>
            <div class="ov-label">数据库</div>
          </div>
          <div class="overview-item">
            <div class="ov-num">{{ overview.apps }}</div>
            <div class="ov-label">应用</div>
          </div>
        </div>
      </el-card>

      <el-card class="section-card chart-card">
        <template #header>
          <div class="chart-header">
            <span>监控</span>
            <el-radio-group v-model="chartMode" size="small">
              <el-radio-button value="cpu">CPU</el-radio-button>
              <el-radio-button value="mem">内存</el-radio-button>
            </el-radio-group>
          </div>
        </template>
        <div class="chart-wrap">
          <canvas ref="chartCanvas" width="700" height="200"></canvas>
        </div>
      </el-card>
    </div>

    <div class="home-right">
      <el-card class="section-card">
        <template #header><span>系统信息</span></template>
        <div class="sysinfo-list">
          <div class="si-row"><span class="si-l">主机名称</span><span class="si-v">{{ info.hostname }}</span></div>
          <div class="si-row"><span class="si-l">发行版本</span><span class="si-v">{{ info.os_pretty }}</span></div>
          <div class="si-row"><span class="si-l">系统架构</span><span class="si-v">{{ info.os_arch }}</span></div>
          <div class="si-row"><span class="si-l">内核版本</span><span class="si-v">{{ info.kernel }}</span></div>
          <div class="si-row"><span class="si-l">主机地址</span><span class="si-v">{{ info.ip }}</span></div>
          <div class="si-row"><span class="si-l">启动时间</span><span class="si-v">{{ info.boot_time }}</span></div>
          <div class="si-row"><span class="si-l">运行时间</span><span class="si-v">{{ info.os_uptime }}</span></div>
        </div>
      </el-card>

      <el-card class="section-card">
        <template #header><span>备忘录</span></template>
        <el-input v-model="memo" type="textarea" :rows="4" placeholder="在此记录（本地存储）" @blur="saveMemo" />
      </el-card>

      <el-card class="section-card">
        <template #header><span>应用</span></template>
        <div v-for="app in apps" :key="app.name" class="app-row">
          <span class="app-name">{{ app.name }}</span>
          <span class="app-status" :class="{ running: app.running, stopped: !app.running }">
            {{ app.running ? '▶' : '⏸' }}
          </span>
          <el-button v-if="!app.running" size="small" type="primary" @click="startApp(app)">启动</el-button>
          <el-button v-else size="small" type="warning" @click="stopApp(app)">停止</el-button>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { apiFetch } from '@/utils/api'

// ── Types ──
interface OsInfo {
  os_id: string; os_name: string; os_version: string; os_pretty: string
  os_arch: string; os_uptime: string; hostname: string; kernel: string; ip: string; boot_time: string
}
interface SystemStat {
  loadavg: { load1: number; load5: number; load15: number }
  cpu: { name: string; physical_count: number; core_count: number; logical_count: number; usage_percent: number }
  mem: { total: number; used: number; percent: number }
  disks: { mount: string; total: number; used: number; percent: number }[]
}
interface AppInfo { name: string; running: boolean }

// ── State ──
const info = ref<OsInfo>({
  os_id: '', os_name: '', os_version: '', os_pretty: '', os_arch: '',
  os_uptime: '', hostname: '', kernel: '', ip: '', boot_time: ''
})
const stat = ref<SystemStat | null>(null)
const apps = ref<AppInfo[]>([
  { name: 'Nginx', running: false },
  { name: 'MySQL', running: false },
  { name: 'Redis', running: false },
])
const overview = ref({ sites: 0, databases: 0, apps: 0 })
const memo = ref(localStorage.getItem('alpanel_memo') || '')
const chartMode = ref('cpu')
const chartCanvas = ref<HTMLCanvasElement | null>(null)

// ── Rings ──
const rings = ref<{ label: string; pct: number; desc: string }[]>([])
function ringColor(pct: number) {
  if (pct >= 90) return '#f56c6c'
  if (pct >= 60) return '#e6a23c'
  return '#67c23a'
}
function updateRings(s: SystemStat) {
  const loadPct = s.cpu.logical_count > 0 ? (s.loadavg.load1 / s.cpu.logical_count) * 100 : 0
  let diskPct = 0
  if (s.disks.length > 0) diskPct = s.disks[0].percent
  rings.value = [
    { label: '负载', pct: Math.min(loadPct, 100), desc: `${s.loadavg.load1.toFixed(2)} / ${s.cpu.logical_count}核心` },
    { label: 'CPU', pct: s.cpu.usage_percent, desc: `${s.cpu.logical_count}核心` },
    { label: '内存', pct: s.mem.percent, desc: fmtSize(s.mem.used) + ' / ' + fmtSize(s.mem.total) },
    { label: '磁盘', pct: diskPct, desc: fmtSize(s.disks[0]?.used || 0) + ' / ' + fmtSize(s.disks[0]?.total || 0) },
  ]
}

// ── Chart (rolling 15 min = 300 points at 3s) ──
const MAX_POINTS = 300
const chartData: number[] = []
function drawChart() {
  const cvs = chartCanvas.value
  if (!cvs) return
  const ctx = cvs.getContext('2d')
  if (!ctx) return
  const w = cvs.width, h = cvs.height
  ctx.clearRect(0, 0, w, h)

  const data = chartData.slice(-MAX_POINTS)
  if (data.length < 2) return

  const pad = 20
  const cw = w - pad * 2
  const ch = h - pad * 2
  const max = 100
  const min = 0

  // grid
  ctx.strokeStyle = '#e0e0e0'
  ctx.lineWidth = 0.5
  for (let i = 0; i <= 4; i++) {
    const y = pad + (ch / 4) * i
    ctx.beginPath()
    ctx.moveTo(pad, y)
    ctx.lineTo(w - pad, y)
    ctx.stroke()
    ctx.fillStyle = '#999'
    ctx.font = '10px sans-serif'
    ctx.textAlign = 'right'
    ctx.fillText(`${(max - (max - min) / 4 * i).toFixed(0)}%`, pad - 4, y + 3)
  }

  // line
  const step = cw / (MAX_POINTS - 1)
  const offset = MAX_POINTS - data.length
  ctx.strokeStyle = '#409eff'
  ctx.lineWidth = 1.5
  ctx.beginPath()
  for (let i = 0; i < data.length; i++) {
    const x = pad + (offset + i) * step
    const y = pad + ch - (data[i] / max) * ch
    i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y)
  }
  ctx.stroke()
}

function fmtSize(bytes: number): string {
  if (bytes >= 1073741824) return (bytes / 1073741824).toFixed(1) + ' GB'
  if (bytes >= 1048576) return (bytes / 1048576).toFixed(1) + ' MB'
  if (bytes >= 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return bytes + ' B'
}

// ── Memo ──
function saveMemo() {
  localStorage.setItem('alpanel_memo', memo.value)
}

// ── App status ──
async function checkAppStatus(a: AppInfo) {
  const name = a.name.toLowerCase()
  try {
    const data = await apiFetch(`/api/${name}/status`)
    a.running = data?.running ?? false
  } catch { a.running = false }
}
async function startApp(a: AppInfo) {
  const name = a.name.toLowerCase()
  try {
    await apiFetch(`/api/${name}/start`, { method: 'POST' })
    await checkAppStatus(a)
  } catch {}
}
async function stopApp(a: AppInfo) {
  const name = a.name.toLowerCase()
  try {
    await apiFetch(`/api/${name}/stop`, { method: 'POST' })
    await checkAppStatus(a)
  } catch {}
}

// ── Polling ──
let timer: ReturnType<typeof setInterval> | null = null

onMounted(async () => {
  // fetch static info once
  try {
    const data = await apiFetch('/api/system/info')
    info.value = data
  } catch {}

  // initial poll
  const poll = async () => {
    try {
      const s: SystemStat = await apiFetch('/api/system/stat')
      stat.value = s
      updateRings(s)
      chartData.push(chartMode.value === 'cpu' ? s.cpu.usage_percent : s.mem.percent)
      if (chartData.length > MAX_POINTS) chartData.splice(0, chartData.length - MAX_POINTS)
      nextTick(() => drawChart())
    } catch {}
    // check app status
    for (const a of apps.value) await checkAppStatus(a)
    // overview
    try {
      const sites = await apiFetch('/api/sites')
      overview.value.sites = Array.isArray(sites) ? sites.length : 0
    } catch {}
    try {
      const dbs = await apiFetch('/api/mysql/databases')
      overview.value.databases = Array.isArray(dbs) ? dbs.length : 0
    } catch {}
    overview.value.apps = apps.value.filter(a => a.running).length
  }
  await poll()
  timer = setInterval(poll, 3000)
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<style scoped>
.home-layout {
  display: flex;
  gap: 12px;
  height: 100%;
}
.home-left {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
}
.home-right {
  width: 300px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.section-card { flex: 1; }
.section-card :deep(.el-card__body) { padding: 12px; }

/* Rings */
.rings-row {
  display: flex;
  justify-content: space-around;
  gap: 8px;
}
.ring-item { text-align: center; }
.ring-label { font-size: 13px; font-weight: 600; margin-top: 4px; }
.ring-desc { font-size: 11px; color: var(--el-text-color-secondary); margin-top: 2px; }

/* Overview */
.overview-row {
  display: flex;
  justify-content: space-around;
  gap: 16px;
}
.overview-item { text-align: center; }
.ov-num { font-size: 28px; font-weight: 700; color: var(--el-color-primary); }
.ov-label { font-size: 12px; color: var(--el-text-color-secondary); }

/* Chart */
.chart-card { flex: 2; }
.chart-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.chart-wrap { width: 100%; }
.chart-wrap canvas { width: 100%; height: 200px; }

/* Sysinfo */
.sysinfo-list { font-size: 12px; }
.si-row {
  display: flex;
  justify-content: space-between;
  padding: 3px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
}
.si-l { color: var(--el-text-color-secondary); white-space: nowrap; }
.si-v { color: var(--el-text-color-primary); text-align: right; margin-left: 8px; }

/* App */
.app-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
}
.app-name { flex: 1; font-size: 13px; }
.app-status { font-size: 14px; }
.app-status.running { color: #67c23a; }
.app-status.stopped { color: #e6a23c; }
</style>