<template>
  <div class="website-manager">
    <div v-if="!ngReady" class="install-mask">
      <el-button type="primary" :loading="true">检测中...</el-button>
    </div>
    <div v-if="blockedByNginx && ngReady" class="install-mask install-prompt">
      <p style="margin:0 0 16px;color:var(--el-text-color-secondary);font-size:13px">网站管理功能需要 Nginx Web 服务器</p>
      <template v-if="!installing">
        <el-button type="primary" size="small" @click="doInstall" :loading="installing">安装 Nginx</el-button>
      </template>
      <template v-else>
        <p style="margin:0 0 12px;font-size:13px">正在安装 Nginx，请稍候...</p>
        <el-progress :percentage="installProgress" :stroke-width="6" style="width:300px" />
      </template>
      <div v-if="installError" style="color:var(--el-color-danger);font-size:13px;margin-top:12px">
        {{ installError }}
        <el-button size="small" link type="primary" @click="installError = ''" style="margin-left:8px">重试</el-button>
      </div>
    </div>
    <template v-if="ngReady && !blockedByNginx">
      <div class="tab-bar">
        <div
          v-for="tab in tabs"
          :key="tab.key"
          class="tab-item"
          :class="{ active: activeTab === tab.key }"
          @click="activeTab = tab.key"
        >{{ tab.label }}</div>
      </div>

      <div class="content-area">
        <div class="toolbar-row">
          <div class="toolbar-left">
            <el-button size="small" type="primary" @click="showAddSiteDialog">
              <el-icon><Plus /></el-icon>添加站点
            </el-button>
            <el-dropdown size="small" trigger="hover" @command="handleNginxCmd">
              <el-button size="small" :type="nginxRunning ? 'default' : 'danger'">
                Nginx{{ nginxVersion ? ' [' + nginxVersion + ']' : '' }} {{ nginxRunning ? '\u25b6' : '\u23f8' }}
              </el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="start" v-if="!nginxRunning">启动</el-dropdown-item>
                  <el-dropdown-item command="stop" v-if="nginxRunning">停止</el-dropdown-item>
                  <el-dropdown-item command="restart">重启</el-dropdown-item>
                  <el-dropdown-item command="reload">重载</el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </div>
          <div class="toolbar-right">
            <el-input
              v-model="searchQuery"
              size="small"
              class="search-input"
              placeholder="搜索..."
              clearable
            >
              <template #prefix><el-icon><Search /></el-icon></template>
            </el-input>
            <el-button size="small" class="toolbar-btn" @click="refreshTable">
              <el-icon><RefreshRight /></el-icon>
            </el-button>
          </div>
        </div>

        <template v-if="activeTab === 'PHP'">
          <el-table
            :data="pagedNormal"
            size="small"
            class="site-table"
            empty-text="暂无站点"
            :cell-style="{ padding: '4px 0' }"
          >
            <el-table-column label="站点名称" width="200" show-overflow-tooltip>
              <template #default="{ row }"><span class="link-cell" @click="">{{ row.name }}</span></template>
            </el-table-column>
            <el-table-column label="状态" width="80">
              <template #default="{ row }">
                <span :class="row.status === '运行中' ? 'status-running' : 'status-stopped'">{{ row.status === '运行中' ? '运行中▶' : '已停止⏸' }}</span>
              </template>
            </el-table-column>
            <el-table-column label="根目录" min-width="200" show-overflow-tooltip>
              <template #default="{ row }"><span class="link-cell" @click="goFile(row.root)">{{ row.root }}</span></template>
            </el-table-column>
            <el-table-column label="备注" min-width="160">
              <template #default="{ row }">
                <el-input v-model="row.ps" size="small" class="ps-input" @blur="savePs(row, 'normal')" />
              </template>
            </el-table-column>
            <el-table-column label="PHP版本" width="100">
              <template #default="{ row }"><span class="link-cell" @click="">{{ row.php ? row.php : '纯静态' }}</span></template>
            </el-table-column>
            <el-table-column label="SSL证书" width="110">
              <template #default="{ row }">
                <span v-if="row.ssl" class="link-cell" @click="">剩余{{ row.sslDays }}天</span>
                <span v-else class="link-cell link-action" @click="">未部署</span>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="150" fixed="right">
              <template #default="{ row }">
                <el-button size="small" link type="primary">设置</el-button>
                <el-button size="small" link type="danger" @click="handleDelete(row)">删除</el-button>
              </template>
            </el-table-column>
          </el-table>
          <el-pagination
            small
            layout="prev, pager, next, total"
            :total="filteredNormal.length"
            :page-size="pageSize"
            v-model:current-page="page"
            class="site-pagination"
          />
        </template>

        <template v-if="activeTab === 'Other'">
          <el-table
            :data="pagedOther"
            size="small"
            class="site-table"
            empty-text="暂无项目"
            :cell-style="{ padding: '4px 0' }"
          >
            <el-table-column label="站点名称" width="200" show-overflow-tooltip>
              <template #default="{ row }"><span class="link-cell" @click="">{{ row.name }}</span></template>
            </el-table-column>
            <el-table-column label="状态" width="80">
              <template #default="{ row }">
                <span :class="row.status === '运行中' ? 'status-running' : 'status-stopped'">{{ row.status === '运行中' ? '运行中▶' : '已停止⏸' }}</span>
              </template>
            </el-table-column>
            <el-table-column label="运行端口" width="100">
              <template #default="{ row }">{{ row.port }}</template>
            </el-table-column>
            <el-table-column label="根目录" min-width="200" show-overflow-tooltip>
              <template #default="{ row }"><span class="link-cell" @click="goFile(row.root)">{{ row.root }}</span></template>
            </el-table-column>
            <el-table-column label="备注" min-width="160">
              <template #default="{ row }">
                <el-input v-model="row.ps" size="small" class="ps-input" @blur="savePs(row, 'other')" />
              </template>
            </el-table-column>
            <el-table-column label="SSL证书" width="110">
              <template #default="{ row }">
                <span v-if="row.ssl" class="link-cell" @click="">剩余{{ row.sslDays }}天</span>
                <span v-else class="link-cell link-action" @click="">未部署</span>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="150" fixed="right">
              <template #default="{ row }">
                <el-button size="small" link type="primary">设置</el-button>
                <el-button size="small" link type="danger" @click="handleDelete(row)">删除</el-button>
              </template>
            </el-table-column>
          </el-table>
          <el-pagination
            small
            layout="prev, pager, next, total"
            :total="filteredOther.length"
            :page-size="pageSize"
            v-model:current-page="page"
            class="site-pagination"
          />
        </template>

        <template v-if="activeTab === 'Proxy'">
          <el-table
            :data="pagedProxy"
            size="small"
            class="site-table"
            empty-text="暂无代理"
            :cell-style="{ padding: '4px 0' }"
          >
            <el-table-column label="域名" width="200" show-overflow-tooltip>
              <template #default="{ row }"><span class="link-cell" @click="">{{ row.domain }}</span></template>
            </el-table-column>
            <el-table-column label="状态" width="80">
              <template #default="{ row }">
                <span :class="row.status === '运行中' ? 'status-running' : 'status-stopped'">{{ row.status === '运行中' ? '运行中▶' : '已停止⏸' }}</span>
              </template>
            </el-table-column>
            <el-table-column label="代理地址" width="200" show-overflow-tooltip>
              <template #default="{ row }"><span class="link-cell" @click="">{{ row.proxyPass }}</span></template>
            </el-table-column>
            <el-table-column label="备注" min-width="160">
              <template #default="{ row }">
                <el-input v-model="row.ps" size="small" class="ps-input" @blur="savePs(row, 'proxy')" />
              </template>
            </el-table-column>
            <el-table-column label="SSL证书" width="110">
              <template #default="{ row }">
                <span v-if="row.ssl" class="link-cell" @click="">剩余{{ row.sslDays }}天</span>
                <span v-else class="link-cell link-action" @click="">未部署</span>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="150" fixed="right">
              <template #default>
                <el-button size="small" link type="primary">设置</el-button>
                <el-button size="small" link type="primary">删除</el-button>
              </template>
            </el-table-column>
          </el-table>
          <el-pagination
            small
            layout="prev, pager, next, total"
            :total="filteredProxy.length"
            :page-size="pageSize"
            v-model:current-page="page"
            class="site-pagination"
          />
        </template>
      </div>
    </template>

    <el-dialog v-model="addSiteDialog.visible" title="添加站点" width="560px" append-to-body>
      <el-form label-width="70px">
        <el-form-item label="域名" required>
          <el-input
            v-model="addSiteDialog.domain"
            type="textarea"
            :rows="5"
            placeholder="如需填写多个域名，请换行填写，每行一个域名，默认为80端口&#10;IP地址格式：192.168.1.199&#10;泛解析添加方法 *.domain.com&#10;如另加端口格式为 www.domain.com:88&#10;ipv6格式：[2001:db8:85a3::8a2e:370:7334]:88"
            @input="onDomainInput"
          />
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="addSiteDialog.ps" placeholder="请输入备注,可为空" />
        </el-form-item>
        <el-form-item label="根目录" required>
          <el-input v-model="addSiteDialog.root" placeholder="/www/wwwroot/">
            <template #append>
              <el-button @click="openDirPicker">浏览</el-button>
            </template>
          </el-input>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="addSiteDialog.visible = false">取消</el-button>
        <el-button type="primary" @click="handleAddSite">确定</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="addOtherDialog.visible" title="添加其它项目" width="560px" append-to-body>
      <el-form label-width="80px">
        <el-form-item label="根目录" required>
          <el-input v-model="addOtherDialog.root" placeholder="/www/wwwroot/">
            <template #append>
              <el-button @click="openOtherDirPicker">浏览</el-button>
            </template>
          </el-input>
        </el-form-item>
        <el-form-item label="项目名称" required>
          <el-input v-model="addOtherDialog.name" placeholder="如 node-app" />
        </el-form-item>
        <el-form-item label="执行命令" required>
          <el-input
            v-model="addOtherDialog.cmd"
            type="textarea"
            :rows="3"
            placeholder="如 node server.js 或 /www/wwwroot/app/start.sh"
          />
        </el-form-item>
        <el-form-item label="运行用户">
          <el-select v-model="addOtherDialog.runUser" filterable placeholder="选择运行用户" style="width:100%">
            <el-option v-for="u in systemUsers" :key="u" :label="u" :value="u" />
          </el-select>
        </el-form-item>
        <el-form-item label="开机启动">
          <el-checkbox v-model="addOtherDialog.onpower">是否设置开机启动</el-checkbox>
        </el-form-item>
        <el-form-item label="项目端口">
          <el-input v-model="addOtherDialog.port" placeholder="如 3000，可为空" />
        </el-form-item>
        <el-form-item label="项目备注">
          <el-input v-model="addOtherDialog.ps" placeholder="请输入备注,可为空" />
        </el-form-item>
        <el-form-item label="绑定域名">
          <el-input
            v-model="addOtherDialog.domain"
            type="textarea"
            :rows="4"
            placeholder="每行一个域名，默认为80端口&#10;如另加端口格式为 www.domain.com:88"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="addOtherDialog.visible = false">取消</el-button>
        <el-button type="primary" @click="handleAddOther">确定</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="dirPicker.visible" title="选择目录" width="500px" append-to-body>
      <div style="margin-bottom:8px;color:var(--el-text-color-secondary);font-size:12px">{{ dirPicker.currentPath }}</div>
      <div style="display:flex;gap:4px;margin-bottom:8px">
        <el-input v-model="dirPicker.newDir" placeholder="新建子目录名称" size="small" @keyup.enter="createDir" />
        <el-button size="small" type="primary" @click="createDir" :loading="dirPicker.creating">新建</el-button>
      </div>
      <div style="max-height:300px;overflow-y:auto;border:1px solid var(--el-border-color-lighter);border-radius:4px">
        <div
          v-for="item in dirPicker.items"
          :key="item.path"
          style="padding:6px 12px;cursor:pointer;font-size:13px;display:flex;align-items:center;gap:6px"
          @click="enterDir(item)"
        >
          <span style="color:#e6a23c">📁</span>
          <span>{{ item.name }}</span>
        </div>
        <div v-if="dirPicker.items.length === 0" style="padding:12px;color:var(--el-text-color-secondary);font-size:12px;text-align:center">无子目录</div>
      </div>
      <template #footer>
        <el-button @click="dirPickerGoUp">返回上级</el-button>
        <el-button @click="dirPicker.visible = false">取消</el-button>
        <el-button type="primary" @click="dirPickerConfirm">选择当前目录</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, Search, RefreshRight } from '@element-plus/icons-vue'
import { apiFetch } from '@/utils/api'

const router = useRouter()

const projectTypes = ref<{ name: string; title: string; visibled: number }[]>([])
const tabs = computed(() =>
  projectTypes.value.filter(t => t.visibled === 1).map(t => ({ key: t.name, label: t.title }))
)

async function fetchProjectTypes() {
  try {
    const data = await apiFetch('/api/sites/types')
    if (Array.isArray(data)) projectTypes.value = data
  } catch {}
}

const activeTab = ref('PHP')
const searchQuery = ref('')
const nginxInstalled = ref(false)
const ngReady = ref(false)
const blockedByNginx = ref(false)
const installing = ref(false)
const installProgress = ref(0)
const installError = ref('')
const nginxRunning = ref(false)
const nginxVersion = ref('')

async function checkNginx() {
  ngReady.value = false
  try {
    const data = await apiFetch('/api/nginx/status')
    nginxInstalled.value = data.installed
    nginxRunning.value = data.running
    nginxVersion.value = data.version || ''
    if (!data.installed) {
      blockedByNginx.value = true
    }
  } catch {
    nginxInstalled.value = false
    blockedByNginx.value = true
  } finally {
    ngReady.value = true
  }
}

async function fetchNginxStatus() {
  try {
    const data = await apiFetch('/api/nginx/status')
    nginxRunning.value = data.running
  } catch {}
}

async function handleNginxCmd(cmd: string) {
  try {
    await apiFetch(`/api/nginx/${cmd}`, { method: 'POST' })
    const msgs: Record<string, string> = { start: 'Nginx 已启动', stop: 'Nginx 已停止', restart: 'Nginx 已重启', reload: 'Nginx 已重载' }
    ElMessage.success(msgs[cmd] || '操作成功')
    setTimeout(fetchNginxStatus, 1000)
  } catch {
    ElMessage.error('请求失败，请检查服务端连接')
  }
}

async function doInstall() {
  installing.value = true
  installError.value = ''
  installProgress.value = 0
  const iv = setInterval(() => {
    installProgress.value = Math.min(installProgress.value + 5, 90)
  }, 600)
  try {
    await apiFetch('/api/nginx/install', { method: 'POST' })
    clearInterval(iv)
    installProgress.value = 100
    setTimeout(() => {
      blockedByNginx.value = false
      nginxInstalled.value = true
    }, 600)
  } catch {
    clearInterval(iv)
    installError.value = '请求失败，请检查服务端连接'
  } finally {
    installing.value = false
  }
}

onMounted(() => {
  checkNginx()
  fetchProjectTypes().then(() => {
    if (tabs.value.length && !tabs.value.some(t => t.key === activeTab.value)) {
      activeTab.value = tabs.value[0].key
    }
  })
  fetchSites()
})

interface NormalItem {
  id: number
  name: string
  status: string
  root: string
  ps: string
  php: string
  ssl: boolean
  sslDays: number
}

interface OtherItem {
  id: number
  name: string
  status: string
  port: number
  root: string
  ps: string
  ssl: boolean
  sslDays: number
}

interface ProxyItem {
  id: number
  domain: string
  status: string
  proxyPass: string
  ps: string
  ssl: boolean
  sslDays: number
}

const normalList = ref<NormalItem[]>([])
const otherList = ref<OtherItem[]>([])
const proxyList = ref<ProxyItem[]>([])

async function fetchSites() {
  try {
    const data = await apiFetch('/api/sites')
    normalList.value = (data || [])
      .filter((s: any) => (s.project_type || 'PHP') === 'PHP')
      .map((s: any) => ({
        id: s.id,
        name: s.name,
        status: s.status === '1' ? '运行中' : '已停止',
        root: s.path,
        ps: s.ps || '',
        php: '',
        ssl: false,
        sslDays: 0,
      }))
    // 其它项目 / 反向代理 暂未实现
    otherList.value = (data || [])
      .filter((s: any) => (s.project_type || '') === 'Other')
      .map((s: any) => ({
        id: s.id,
        name: s.name,
        status: s.status === '1' ? '运行中' : '已停止',
        port: 0,
        root: s.path,
        ps: s.ps || '',
        ssl: false,
        sslDays: 0,
      }))
    proxyList.value = (data || [])
      .filter((s: any) => (s.project_type || '') === 'Proxy')
      .map((s: any) => ({
        id: s.id,
        domain: s.domains?.[0]?.name || s.name,
        status: s.status === '1' ? '运行中' : '已停止',
        proxyPass: '',
        ps: s.ps || '',
        ssl: false,
        sslDays: 0,
      }))
  } catch {
    ElMessage.error('获取站点列表失败')
  }
}

const pageSize = 10
const page = ref(1)

watch(activeTab, () => { page.value = 1 })
watch(searchQuery, () => { page.value = 1 })

const filteredNormal = computed(() => {
  if (!searchQuery.value) return normalList.value
  const q = searchQuery.value.toLowerCase()
  return normalList.value.filter(x => x.name.toLowerCase().includes(q) || x.root.toLowerCase().includes(q) || x.ps.toLowerCase().includes(q))
})

const filteredOther = computed(() => {
  if (!searchQuery.value) return otherList.value
  const q = searchQuery.value.toLowerCase()
  return otherList.value.filter(x => x.name.toLowerCase().includes(q) || x.root.toLowerCase().includes(q) || x.ps.toLowerCase().includes(q))
})

const filteredProxy = computed(() => {
  if (!searchQuery.value) return proxyList.value
  const q = searchQuery.value.toLowerCase()
  return proxyList.value.filter(x => x.domain.toLowerCase().includes(q) || x.ps.toLowerCase().includes(q))
})

const pagedNormal = computed(() => {
  const start = (page.value - 1) * pageSize
  return filteredNormal.value.slice(start, start + pageSize)
})

const pagedOther = computed(() => {
  const start = (page.value - 1) * pageSize
  return filteredOther.value.slice(start, start + pageSize)
})

const pagedProxy = computed(() => {
  const start = (page.value - 1) * pageSize
  return filteredProxy.value.slice(start, start + pageSize)
})

function refreshTable() {
  fetchSites()
}

async function handleDelete(row: any) {
  try {
    await ElMessageBox.confirm(`确定删除站点「${row.name}」吗？`, '删除确认', {
      type: 'warning',
      confirmButtonText: '删除',
      cancelButtonText: '取消',
    })
  } catch {
    return
  }
  try {
    await apiFetch(`/api/sites/${row.id}`, { method: 'DELETE' })
    ElMessage.success('已删除')
    refreshTable()
  } catch {
    ElMessage.error('删除失败')
  }
}

function parseDomains(input: string): { name: string; port: number | null }[] {
  return input
    .split('\n')
    .map(s => s.trim())
    .filter(s => s.length > 0)
    .map(s => {
      const idx = s.lastIndexOf(':')
      if (idx > 0 && /^\d+$/.test(s.slice(idx + 1))) {
        return { name: s.slice(0, idx), port: parseInt(s.slice(idx + 1), 10) }
      }
      return { name: s, port: null }
    })
}

async function savePs(row: any, _tab: string) {
  try {
    await apiFetch(`/api/sites/${row.id}`, {
      method: 'PUT',
      body: JSON.stringify({ ps: row.ps }),
    })
  } catch {
    ElMessage.error('保存备注失败')
  }
}

function goFile(path: string) {
  router.push({ name: 'file', query: { path } })
}

const addSiteDialog = reactive({
  visible: false,
  domain: '',
  ps: '',
  root: '/www/wwwroot/',
})

function showAddSiteDialog() {
  if (activeTab.value === 'Other') {
    showAddOtherDialog()
    return
  }
  addSiteDialog.domain = ''
  addSiteDialog.ps = ''
  addSiteDialog.root = '/www/wwwroot/'
  addSiteDialog.visible = true
}

const systemUsers = ref<string[]>(['www'])

async function fetchUsers() {
  try {
    const data = await apiFetch('/api/system/users')
    if (Array.isArray(data) && data.length) systemUsers.value = data
  } catch {}
}

const addOtherDialog = reactive({
  visible: false,
  root: '/www/wwwroot/',
  name: '',
  cmd: '',
  runUser: 'www',
  onpower: false,
  port: '',
  ps: '',
  domain: '',
})

function showAddOtherDialog() {
  addOtherDialog.root = '/www/wwwroot/'
  addOtherDialog.name = ''
  addOtherDialog.cmd = ''
  addOtherDialog.runUser = 'www'
  addOtherDialog.onpower = false
  addOtherDialog.port = ''
  addOtherDialog.ps = ''
  addOtherDialog.domain = ''
  fetchUsers()
  addOtherDialog.visible = true
}

async function handleAddOther() {
  const name = addOtherDialog.name.trim()
  const root = addOtherDialog.root.trim()
  const cmd = addOtherDialog.cmd.trim()
  if (!root) { ElMessage.error('请填写根目录'); return }
  if (!name) { ElMessage.error('请填写项目名称'); return }
  if (!cmd) { ElMessage.error('请填写执行命令'); return }
  const domains = parseDomains(addOtherDialog.domain)
  try {
    await apiFetch('/api/sites', {
      method: 'POST',
      body: JSON.stringify({
        project_type: 'Other',
        name,
        path: root,
        project_cmd: cmd,
        run_user: addOtherDialog.runUser || 'www',
        is_onpower: addOtherDialog.onpower ? 1 : 0,
        project_port: addOtherDialog.port ? parseInt(addOtherDialog.port, 10) : undefined,
        ps: addOtherDialog.ps || undefined,
        domains,
      }),
    })
    ElMessage.success('项目创建成功')
    addOtherDialog.visible = false
    refreshTable()
  } catch (e: any) {
    ElMessage.error((e && e.message) || '创建项目失败')
  }
}

function openOtherDirPicker() {
  dirPicker.currentPath = addOtherDialog.root || '/www/wwwroot/'
  fetchDirs(dirPicker.currentPath)
  dirPicker.visible = true
}

function onDomainInput() {
  const firstLine = addSiteDialog.domain.split('\n')[0]?.trim() || ''
  addSiteDialog.ps = firstLine
  addSiteDialog.root = '/www/wwwroot/' + firstLine
}

async function handleAddSite() {
  const domains = parseDomains(addSiteDialog.domain)
  if (domains.length === 0) {
    ElMessage.error('请至少填写一个域名')
    return
  }
  const root = addSiteDialog.root.trim()
  if (!root) {
    ElMessage.error('请填写根目录')
    return
  }
  try {
    await apiFetch('/api/sites', {
      method: 'POST',
      body: JSON.stringify({
        project_type: 'PHP',
        domains,
        path: root,
        ps: addSiteDialog.ps || undefined,
      }),
    })
    ElMessage.success('站点创建成功')
    addSiteDialog.visible = false
    refreshTable()
  } catch (e: any) {
    ElMessage.error((e && e.message) || '创建站点失败')
  }
}

const dirPicker = reactive({
  visible: false,
  currentPath: '/www/wwwroot/',
  parentPath: '',
  items: [] as { name: string; path: string; is_dir: boolean }[],
  newDir: '',
  creating: false,
})

async function fetchDirs(path: string) {
  try {
    const data = await apiFetch('/api/files/list?path=' + encodeURIComponent(path))
    dirPicker.items = (data.items || []).filter((i: any) => i.is_dir)
    dirPicker.currentPath = data.path
    dirPicker.parentPath = data.parent || ''
  } catch {}
}

function openDirPicker() {
  dirPicker.currentPath = addSiteDialog.root || '/www/wwwroot/'
  fetchDirs(dirPicker.currentPath)
  dirPicker.visible = true
}

function enterDir(item: { name: string; path: string; is_dir: boolean }) {
  if (item.is_dir) fetchDirs(item.path)
}

function dirPickerGoUp() {
  if (dirPicker.parentPath) fetchDirs(dirPicker.parentPath)
}

function dirPickerConfirm() {
  addSiteDialog.root = dirPicker.currentPath.endsWith('/') ? dirPicker.currentPath : dirPicker.currentPath + '/'
  dirPicker.visible = false
}

async function createDir() {
  const name = dirPicker.newDir.trim()
  if (!name) return
  dirPicker.creating = true
  try {
    const p = dirPicker.currentPath.endsWith('/') ? dirPicker.currentPath + name : dirPicker.currentPath + '/' + name
    await apiFetch('/api/files/create', {
      method: 'POST',
      body: JSON.stringify({ path: p, type: 'dir' }),
    })
    dirPicker.newDir = ''
    await fetchDirs(dirPicker.currentPath)
  } finally {
    dirPicker.creating = false
  }
}
</script>

<style scoped>
.website-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}
.tab-bar {
  display: flex;
  align-items: stretch;
  background: var(--el-fill-color);
  border-bottom: 1px solid var(--el-border-color-lighter);
  flex-shrink: 0;
  overflow-x: auto;
}
.tab-item {
  display: flex;
  align-items: center;
  padding: 0 16px;
  height: 34px;
  font-size: 12px;
  cursor: pointer;
  border-right: 1px solid var(--el-border-color-lighter);
  white-space: nowrap;
  color: var(--el-text-color-regular);
  user-select: none;
  flex-shrink: 0;
}
.tab-item:hover {
  background: var(--el-fill-color-light);
}
.tab-item.active {
  background: var(--el-bg-color);
  color: var(--el-color-primary);
  font-weight: 500;
  border-bottom: 2px solid var(--el-color-primary);
  margin-bottom: -1px;
}
.content-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  padding: 8px;
  box-sizing: border-box;
}
.toolbar-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
  flex-shrink: 0;
}
.toolbar-left {
  display: flex;
  align-items: center;
  gap: 6px;
}
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 4px;
}
.search-input {
  width: 200px;
}
.site-table {
  flex: 1;
  min-height: 0;
  width: 100%;
}
.ps-input {
  width: 100%;
}
.ps-input :deep(.el-input__wrapper) {
  background: transparent;
  box-shadow: none;
  padding: 0 4px;
  border: 1px solid transparent;
}
.ps-input :deep(.el-input__wrapper:hover),
.ps-input :deep(.el-input__wrapper.is-focus) {
  border-color: var(--el-border-color);
}
.ps-input :deep(.el-input__inner) {
  font-size: 12px;
  padding: 0;
}
.status-running {
  color: var(--el-color-success);
}
.status-stopped {
  color: var(--el-color-danger);
}
.link-cell {
  color: var(--el-color-primary);
  cursor: pointer;
}
.link-cell:hover {
  text-decoration: underline;
}
.link-action {
  color: var(--el-color-warning);
}
.link-action:hover {
  color: var(--el-color-warning-dark-2);
}
.site-pagination {
  flex-shrink: 0;
  display: flex;
  justify-content: center;
  padding: 8px 0 0;
}
:deep(.el-pager li),
:deep(.btn-prev),
:deep(.btn-next) {
  border: 1px solid var(--el-border-color);
  margin: 0 2px;
  min-width: 24px;
  height: 24px;
  line-height: 22px;
  border-radius: 4px;
  background: var(--el-bg-color);
}
:deep(.btn-prev),
:deep(.btn-next) {
  padding: 0 4px;
}
:deep(.el-pager li.active) {
  border-color: var(--el-color-primary);
  color: var(--el-color-primary);
  font-weight: 600;
}
.install-mask {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 300px;
}
.install-prompt {
  flex-direction: column;
  gap: 0;
}
</style>
