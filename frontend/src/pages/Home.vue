<template>
  <div class="home-page">
    <div class="home-header">
      <div class="header-left">
        <img v-if="osLogo" :src="osLogo" class="os-logo" @error="osLogo = ''" />
        <el-icon v-else class="os-logo-icon"><Monitor /></el-icon>
        <el-tooltip placement="bottom">
          <template #content>
            <div>系统：{{ info.os_pretty }} {{ info.os_arch }}</div>
            <div>持续运行：{{ info.os_uptime }}</div>
          </template>
          <span class="header-os-name">{{ info.os_id.charAt(0).toUpperCase() + info.os_id.slice(1) }} {{ info.os_version }}</span>
        </el-tooltip>
      </div>
      <div class="header-right">
        <span class="header-version">v0.1.0</span>
        <el-button size="small" disabled>
          <el-icon><RefreshRight /></el-icon>
          重启
        </el-button>
      </div>
    </div>

    <div class="home-layout">
      <div class="home-left">
        <el-card class="section-card" shadow="never">
          <template #header><span class="sect-title">状态</span></template>
<div class="rings-row">
  <div v-for="(r, idx) in rings" :key="r.label" class="ring-item">
    <el-tooltip v-if="idx === 0" placement="bottom" :disabled="!r.tooltip">
      <template #content>
        <div v-for="line in r.tooltip" :key="line">{{ line }}</div>
      </template>
      <el-progress type="dashboard" :percentage="r.pct" :color="ringColor(r.pct)" :width="100" :stroke-width="10">
        <template #default>{{ r.pct.toFixed(1) }}%</template>
      </el-progress>
    </el-tooltip>
    <el-progress v-else type="dashboard" :percentage="r.pct" :color="ringColor(r.pct)" :width="100" :stroke-width="10">
      <template #default>{{ r.pct.toFixed(1) }}%</template>
    </el-progress>
    <div class="ring-label">{{ r.label }}</div>
    <div class="ring-desc">{{ r.desc }}</div>
  </div>
</div>
        </el-card>

        <el-card class="section-card" shadow="never">
          <template #header><span class="sect-title">概览</span></template>
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

<el-card class="section-card chart-card" shadow="never">
  <template #header>
    <div class="chart-header">
      <span class="sect-title">监控</span>
      <div class="chart-tools">
        <el-radio-group v-model="chartMode" size="small">
          <el-radio-button value="net">流量</el-radio-button>
          <el-radio-button value="disk">磁盘</el-radio-button>
        </el-radio-group>
        <el-select v-model="chartIface" size="small" style="width:100px" placeholder="所有">
          <el-option label="所有" value="" />
          <el-option v-for="n in netIfaces" :key="n" :label="n" :value="n" />
        </el-select>
      </div>
    </div>
  </template>
  <div class="chart-wrap">
    <canvas ref="chartCanvas" width="700" height="200"></canvas>
  </div>
</el-card>
      </div>

      <div class="home-right">
        <el-card class="section-card" shadow="never">
          <template #header><span class="sect-title">系统信息</span></template>
          <div class="sysinfo-list">
            <div class="si-row">
              <span class="si-l">主机名称</span>
              <span class="si-v">{{ info.hostname }}</span>
            </div>
            <div class="si-row">
              <span class="si-l">发行版本</span>
              <span class="si-v">{{ info.os_pretty }}</span>
            </div>
            <div class="si-row">
              <span class="si-l">系统架构</span>
              <span class="si-v">{{ info.os_arch }}</span>
            </div>
            <div class="si-row">
              <span class="si-l">内核版本</span>
              <span class="si-v">{{ info.kernel }}</span>
            </div>
            <div class="si-row">
              <span class="si-l">主机地址</span>
              <span class="si-v">{{ info.ip }}</span>
            </div>
            <div class="si-row">
              <span class="si-l">启动时间</span>
              <span class="si-v">{{ info.boot_time }}</span>
            </div>
            <div class="si-row">
              <span class="si-l">运行时间</span>
              <span class="si-v">{{ info.os_uptime }}</span>
            </div>
          </div>
        </el-card>

        <el-card class="section-card" shadow="never">
          <template #header><span class="sect-title">备忘录</span></template>
          <el-input v-model="memo" type="textarea" :rows="3" placeholder="在此记录（本地存储）" @blur="saveMemo" />
        </el-card>

        <el-card class="section-card" shadow="never">
          <template #header><span class="sect-title">应用</span></template>
          <div class="app-list">
            <div v-for="app in apps" :key="app.name" class="app-row">
              <span class="app-name">{{ app.name }}</span>
              <el-dropdown size="small" trigger="hover" @command="(c: string) => handleSrvCmd(app, c)">
                <el-button size="small" :type="app.running ? 'default' : 'danger'">
                  {{ app.name }} {{ app.running ? '▶' : '⏸' }}
                </el-button>
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item v-if="!app.running" command="start">启动</el-dropdown-item>
                    <el-dropdown-item v-if="app.running" command="stop">停止</el-dropdown-item>
                    <el-dropdown-item command="restart">重启</el-dropdown-item>
                    <el-dropdown-item command="reload">重载</el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
            </div>
          </div>
        </el-card>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { RefreshRight, Monitor } from '@element-plus/icons-vue'
import { apiFetch } from '@/utils/api'

interface OsInfo {
  os_id: string; os_name: string; os_version: string; os_pretty: string
  os_arch: string; os_uptime: string; hostname: string; kernel: string; ip: string; boot_time: string
}
interface SystemStat {
  loadavg: { load1: number; load5: number; load15: number }
  cpu: { name: string; physical_count: number; core_count: number; logical_count: number; usage_percent: number }
  mem: { total: number; used: number; percent: number }
  disks: { mount: string; total: number; used: number; percent: number }[]
  net: { name: string; rx_bytes: number; tx_bytes: number }[]
  disk_io: { name: string; read_bytes: number; write_bytes: number }
  overview: { sites: number; databases: number; apps: number }
}
interface AppInfo { name: string; running: boolean }

const info = ref<OsInfo>({
  os_id: '', os_name: '', os_version: '', os_pretty: '', os_arch: '',
  os_uptime: '', hostname: '', kernel: '', ip: '', boot_time: ''
})
const osLogo = ref('')
const apps = ref<AppInfo[]>([
  { name: 'Nginx', running: false },
  { name: 'MySQL', running: false },
  { name: 'Redis', running: false },
])
const overview = ref({ sites: 0, databases: 0, apps: 0 })
const memo = ref(localStorage.getItem('alpanel_memo') || '')
const chartCanvas = ref<HTMLCanvasElement | null>(null)
const chartMode = ref('net')
const chartIface = ref('')
const netIfaces = ref<string[]>([])

const logoMap: Record<string, string> = {
  debian: 'https://www.debian.org/logos/openlogo-nd.svg',
  alpine: 'https://cdn.jsdelivr.net/gh/homarr-labs/dashboard-icons/svg/alpine-linux.svg',
  ubuntu: 'https://ubuntu.com/static/images/logos/logo-ubuntu.svg',
}

// ── Rings ──
const rings = ref<{ label: string; pct: number; desc: string; tooltip: string[] }[]>([])
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
    { label: '负载', pct: Math.min(loadPct, 100), desc: `${s.loadavg.load1.toFixed(2)} / ${s.cpu.logical_count}核心`,
      tooltip: [
        `最近 1 分钟平均负载${s.loadavg.load1.toFixed(2)}`,
        `最近 5 分钟平均负载${s.loadavg.load5.toFixed(2)}`,
        `最近 15 分钟平均负载${s.loadavg.load15.toFixed(2)}`,
      ] },
    { label: 'CPU', pct: s.cpu.usage_percent, desc: `${s.cpu.logical_count}核心`, tooltip: [] },
    { label: '内存', pct: s.mem.percent, desc: fmtSize(s.mem.used) + ' / ' + fmtSize(s.mem.total), tooltip: [] },
    { label: '磁盘', pct: diskPct, desc: fmtSize(s.disks[0]?.used || 0) + ' / ' + fmtSize(s.disks[0]?.total || 0), tooltip: [] },
  ]
}

// ── Chart ──
const MAX_POINTS = 300
interface ChartPoint { rx: number; tx: number; rd: number; wr: number }
const chartData: ChartPoint[] = []
let prevNet: Record<string, { rx: number; tx: number }> = {}
let prevDiskIo = { rd: 0, wr: 0 }

function getChartLine(): { data: number[]; label: string; color: string }[] {
  if (chartMode.value === 'net') {
    const iface = chartIface.value
    const data = chartData.map(p => iface ? p.rx : p.rx + p.tx)
    return [{ data, label: iface || '总流量', color: '#409eff' }]
  }
  return [
    { data: chartData.map(p => p.rd), label: '读取', color: '#67c23a' },
    { data: chartData.map(p => p.wr), label: '写入', color: '#e6a23c' },
  ]
}

function drawChart() {
  const cvs = chartCanvas.value
  if (!cvs) return
  const ctx = cvs.getContext('2d')
  if (!ctx) return
  const w = cvs.width, h = cvs.height
  ctx.clearRect(0, 0, w, h)

  const lines = getChartLine()
  const allData = lines.flatMap(l => l.data).filter(v => v > 0)
  if (allData.length < 2) return

  const maxVal = Math.max(...allData) * 1.1
  const pad = 20
  const cw = w - pad * 2
  const ch = h - pad * 2

  // grid
  ctx.strokeStyle = '#e0e0e0'
  ctx.lineWidth = 0.5
  for (let i = 0; i <= 4; i++) {
    const y = pad + (ch / 4) * i
    ctx.beginPath()
    ctx.moveTo(pad, y)
    ctx.lineTo(w - pad, y)
    ctx.stroke()
  }

  // lines
  const step = cw / (MAX_POINTS - 1)
  const offset = MAX_POINTS - chartData.length
  for (const line of lines) {
    ctx.strokeStyle = line.color
    ctx.lineWidth = 1.5
    ctx.beginPath()
    let drawn = false
    for (let i = 0; i < line.data.length; i++) {
      if (line.data[i] === 0) continue
      const x = pad + (offset + i) * step
      const y = pad + ch - (line.data[i] / maxVal) * ch
      if (!drawn) { ctx.moveTo(x, y); drawn = true }
      else ctx.lineTo(x, y)
    }
    if (drawn) ctx.stroke()
  }
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

// ── App ──
async function handleSrvCmd(app: AppInfo, cmd: string) {
  const name = app.name.toLowerCase()
  try {
    await apiFetch(`/api/${name}/${cmd}`, { method: 'POST' })
    const data = await apiFetch(`/api/${name}/status`)
    app.running = data?.running ?? false
  } catch {}
}

// ── Polling ──
let timer: ReturnType<typeof setInterval> | null = null

onMounted(async () => {
  try {
    const data = await apiFetch('/api/system/info')
    info.value = data
    osLogo.value = logoMap[data.os_id] || ''
  } catch {}

  const poll = async () => {
    try {
      const s: SystemStat = await apiFetch('/api/system/stat')
      updateRings(s)
      overview.value = s.overview

      // net delta
      const ifaces = s.net.map(n => n.name)
      if (netIfaces.value.length === 0) netIfaces.value = ifaces
      if (ifaces.length > 0 && !chartIface.value && ifaces[0]) chartIface.value = ifaces[0]
      let rx = 0, tx = 0
      const targetIface = chartIface.value || ''
      for (const n of s.net) {
        if (targetIface && n.name !== targetIface) continue
        const prev = prevNet[n.name]
        if (prev) {
          rx += n.rx_bytes - prev.rx
          tx += n.tx_bytes - prev.tx
        }
        prevNet[n.name] = { rx: n.rx_bytes, tx: n.tx_bytes }
      }
      // disk delta
      const prev = prevDiskIo
      let rd = 0, wr = 0
      if (s.disk_io.name) {
        if (prev.rd > 0) rd = s.disk_io.read_bytes - prev.rd
        if (prev.wr > 0) wr = s.disk_io.write_bytes - prev.wr
        prevDiskIo = { rd: s.disk_io.read_bytes, wr: s.disk_io.write_bytes }
      }

      chartData.push({ rx, tx, rd, wr })
      if (chartData.length > MAX_POINTS) chartData.splice(0, chartData.length - MAX_POINTS)
      drawChart()
    } catch {}
  }
  await poll()
  timer = setInterval(poll, 5000)
  // check app status once
  for (const a of apps.value) {
    try {
      const data = await apiFetch(`/api/${a.name.toLowerCase()}/status`)
      a.running = data?.running ?? false
    } catch {}
  }
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<style scoped>
.home-page {
  display: flex;
  flex-direction: column;
  gap: 12px;
  height: 100%;
}
.home-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 0;
}
.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}
.os-logo { width: 22px; height: 22px; object-fit: contain; }
.os-logo-icon { font-size: 22px; color: var(--el-color-primary); }
.header-os-name { font-size: 14px; font-weight: 600; }
.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}
.header-version { font-size: 13px; color: var(--el-text-color-secondary); }
.home-layout {
  display: flex;
  gap: 12px;
  flex: 1;
  min-height: 0;
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
.section-card { flex: none; }
.section-card.chart-card { flex: 1; }
.section-card :deep(.el-card__body) { padding: 12px 16px; }
.sect-title { font-size: 14px; font-weight: 600; }

/* Rings */
.rings-row {
  display: flex;
  justify-content: space-around;
  gap: 8px;
  padding: 4px 0;
}
.ring-item { text-align: center; }
.ring-label { font-size: 13px; font-weight: 600; margin-top: 6px; }
.ring-desc { font-size: 11px; color: var(--el-text-color-secondary); margin-top: 2px; }

/* Overview */
.overview-row {
  display: flex;
  justify-content: space-around;
  align-items: center;
  gap: 16px;
  padding: 8px 0;
}
.overview-item { text-align: center; }
.ov-num { font-size: 28px; font-weight: 700; color: var(--el-color-primary); line-height: 1.2; }
.ov-label { font-size: 12px; color: var(--el-text-color-secondary); margin-top: 2px; }

/* Chart */
.chart-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.chart-tools {
  display: flex;
  gap: 8px;
  align-items: center;
}
.chart-wrap { width: 100%; }
.chart-wrap canvas { width: 100%; height: 200px; }

/* Sysinfo */
.sysinfo-list { font-size: 13px; }
.si-row {
  display: flex;
  justify-content: space-between;
  padding: 6px 0;
}
.si-l { color: var(--el-text-color-secondary); white-space: nowrap; }
.si-v { color: var(--el-text-color-primary); text-align: right; margin-left: 12px; }

/* App */
.app-list { display: flex; flex-direction: column; gap: 6px; }
.app-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.app-name { font-size: 13px; }
</style>