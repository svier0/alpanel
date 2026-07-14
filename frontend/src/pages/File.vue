<template>
  <div class="file-manager">
    <div class="tab-bar">
      <div
        v-for="tab in tabs"
        :key="tab.id"
        class="tab-item"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
      >
        <span class="tab-title">{{ tab.title }}</span>
        <span v-if="tab.type === 'editor'" class="tab-subtitle">{{ tab.path.split('/').pop() }}</span>
        <el-icon v-if="tabs.length > 1" class="tab-close" @click.stop="removeTab(tab.id)"><Close /></el-icon>
      </div>
      <div class="tab-add" @click="addBrowserTab">
        <el-icon><Plus /></el-icon>
      </div>
    </div>

    <template v-for="tab in tabs" :key="tab.id">
      <div v-if="activeTab === tab.id && tab.type === 'browser'" class="browser-content" @contextmenu.prevent="(e: MouseEvent) => onBrowserContextMenu(e, tab)">
        <div class="path-bar">
          <el-button size="small" class="path-back-btn" :disabled="!canGoUp(tab)" @click="goUp(tab)">
            <el-icon><Back /></el-icon>
          </el-button>
          <div class="path-breadcrumb">
            <template v-for="(seg, i) in getSegments(tab.path)" :key="i">
              <span v-if="i > 0" class="path-sep">&gt;</span>
              <span class="seg-pill" @click="navigateTab(tab, seg.fullPath)">{{ seg.name }}</span>
            </template>
          </div>
          <el-input
            v-model="pathInput"
            size="small"
            class="path-input"
            placeholder="输入路径按回车跳转"
            @keyup.enter="navigatePathInput(tab)"
          >
            <template #prefix><el-icon><Search /></el-icon></template>
          </el-input>
          <el-button size="small" class="path-refresh-btn" @click="refreshTab(tab)">
            <el-icon><RefreshRight /></el-icon>
          </el-button>
        </div>
        <div class="toolbar-row">
          <el-button size="small" @click="refreshTab(tab)">刷新</el-button>
          <el-button size="small" @click="openCreate(tab, false)">新建文件</el-button>
          <el-button size="small" @click="openCreate(tab, true)">新建目录</el-button>
          <el-button size="small" :disabled="!tab.selectedFile" @click="openRename(tab)">重命名</el-button>
          <el-button size="small" :disabled="!tab.selectedFile" type="danger" @click="confirmDelete(tab)">删除</el-button>
        </div>
        <el-table
          v-loading="tab.loading"
          :data="tab.files"

          highlight-current-row
          @current-change="(row: FileItem | null) => tab.selectedFile = row"
          size="small"
          class="file-table"
          empty-text="暂无文件"
          :cell-style="{ padding: '4px 0' }"
        >
          <el-table-column label="名称" width="300" :show-overflow-tooltip="true">
            <template #default="{ row }">
              <div v-if="renamingPath === row.path" class="rename-inline">
                <el-input
                  v-model="renamingValue"
                  size="small"
                  autofocus
                  @keyup.enter="confirmRename"
                  @keyup.escape="cancelRename"
                  @blur="confirmRename"
                />
              </div>
              <span v-else class="file-name" :class="{ 'file-selected': tab.selectedFile?.path === row.path }" @dblclick.stop="onRowDoubleClick(tab, row)">
                <el-icon v-if="row.is_dir" size="14"><FolderOpened /></el-icon>
                <el-icon v-else-if="row.is_link" size="14"><Link /></el-icon>
                <el-icon v-else size="14"><Document /></el-icon>
                {{ row.name }}
              </span>
            </template>
          </el-table-column>
          <el-table-column label="权限/所有者" width="120">
            <template #default="{ row }">{{ row.mode }}<template v-if="row.owner"> / {{ row.owner }}</template></template>
          </el-table-column>
          <el-table-column label="大小" width="90">
            <template #default="{ row }">
              <template v-if="row.is_dir">
                <el-icon v-if="row._calculating" class="is-loading" size="14"><Loading /></el-icon>
                <el-button v-else-if="row._size === undefined" size="small" link type="primary" @click="calcDirSize(tab, row)">计算</el-button>
                <span v-else>{{ formatSize(row._size, false) }}</span>
              </template>
              <span v-else>{{ formatSize(row.size, false) }}</span>
            </template>
          </el-table-column>
          <el-table-column label="修改时间" width="150">
            <template #default="{ row }">{{ formatTime(row.modified) }}</template>
          </el-table-column>
          <el-table-column label="备注" min-width="100" />
        </el-table>
      </div>

      <div v-else-if="activeTab === tab.id && tab.type === 'editor'" class="editor-panel">
        <div class="editor-toolbar">
          <span class="editor-path">{{ tab.path }}</span>
          <div>
            <el-button size="small" @click="revertFile(tab)" :disabled="tab.content === tab.original">撤销</el-button>
            <el-button size="small" type="primary" :loading="tab.saving" @click="saveFile(tab)">保存</el-button>
          </div>
        </div>
        <textarea v-model="tab.content" class="editor-textarea" spellcheck="false"></textarea>
      </div>
    </template>

    <!-- Context menus -->
    <Teleport to="body">
      <div v-if="ctxMenu.visible" class="ctx-menu" :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }" @click="ctxMenu.visible = false">
        <template v-if="ctxMenu.type === 'blank'">
          <div class="ctx-item" @click="refreshTab(ctxMenu.tab!)">刷新</div>
          <div class="ctx-item disabled">上传</div>
          <div class="ctx-divider" />
          <div class="ctx-item" @click="openCreate(ctxMenu.tab!, false)">新建 - 文件</div>
          <div class="ctx-item" @click="openCreate(ctxMenu.tab!, true)">新建 - 文件夹</div>
          <div class="ctx-item disabled">新建 - 软连接</div>
          <div class="ctx-divider" />
          <div class="ctx-item" @click="openDownload(ctxMenu.tab!)">从URL下载</div>
          <div class="ctx-divider" />
          <div class="ctx-item disabled">终端</div>
          <template v-if="clipboard.paths.length">
            <div class="ctx-divider" />
            <div class="ctx-item" @click="ctxPaste">粘贴</div>
          </template>
        </template>
        <template v-else-if="ctxMenu.type === 'dir'">
          <div class="ctx-item" @click="navigateTab(ctxMenu.tab!, ctxMenu.filePath!)">打开</div>
          <div class="ctx-item" @click="openInNewTab(ctxMenu.filePath!)">在新标签打开</div>
          <div class="ctx-divider" />
          <div class="ctx-item disabled">权限</div>
          <div class="ctx-divider" />
          <div class="ctx-item" @click="ctxCopy(ctxMenu.filePath!)">复制</div>
          <div class="ctx-item" @click="ctxCut(ctxMenu.filePath!)">剪切</div>
          <div v-if="clipboard.paths.length" class="ctx-item" @click="ctxPaste">粘贴</div>
          <div class="ctx-divider" />
          <div class="ctx-item" @click="ctxRename(ctxMenu.filePath!)">重命名</div>
          <div class="ctx-item" @click="ctxDelete(ctxMenu.filePath!, ctxMenu.fileName!)">删除</div>
          <div class="ctx-divider" />
          <div class="ctx-item disabled">创建压缩</div>
          <div class="ctx-divider" />
          <div class="ctx-item disabled">属性</div>
        </template>
        <template v-else-if="ctxMenu.type === 'file'">
          <div class="ctx-item" @click="ctxOpenEditor">编辑</div>
          <div class="ctx-item" @click="ctxDownload">下载</div>
          <div class="ctx-divider" />
          <div class="ctx-item disabled">权限</div>
          <div class="ctx-divider" />
          <div class="ctx-item" @click="ctxCopy(ctxMenu.filePath!)">复制</div>
          <div class="ctx-item" @click="ctxCut(ctxMenu.filePath!)">剪切</div>
          <div v-if="clipboard.paths.length" class="ctx-item" @click="ctxPaste">粘贴</div>
          <div class="ctx-divider" />
          <div class="ctx-item" @click="ctxRename(ctxMenu.filePath!)">重命名</div>
          <div class="ctx-item" @click="ctxDelete(ctxMenu.filePath!, ctxMenu.fileName!)">删除</div>
          <div class="ctx-divider" />
          <div class="ctx-item disabled">创建压缩</div>
          <div class="ctx-divider" />
          <div class="ctx-item disabled">属性</div>
        </template>
      </div>
    </Teleport>

    <el-dialog v-model="createDialog.visible" :title="createDialog.isDir ? '新建目录' : '新建文件'" width="400px" append-to-body>
      <el-form @submit.prevent="handleCreate">
        <el-form-item :label="createDialog.isDir ? '目录名' : '文件名'">
          <el-input v-model="createDialog.name" placeholder="请输入名称" @keyup.enter="handleCreate" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="createDialog.visible = false">取消</el-button>
        <el-button type="primary" @click="handleCreate">确定</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="deleteDialog.visible" title="确认删除" width="400px" append-to-body>
      <p>确定要删除 <strong>{{ deleteDialog.name }}</strong> 吗？</p>
      <p v-if="deleteDialog.isDir" style="color:#e6a23c;font-size:12px;margin-top:4px">目录将递归删除所有内容，此操作不可恢复。</p>
      <template #footer>
        <el-button @click="deleteDialog.visible = false">取消</el-button>
        <el-button type="danger" @click="handleDelete">删除</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="downloadDialog.visible" title="从URL下载" width="450px" append-to-body>
      <el-form @submit.prevent="handleDownload">
        <el-form-item label="下载地址">
          <el-input v-model="downloadDialog.url" placeholder="请输入URL" @keyup.enter="handleDownload" />
        </el-form-item>
        <el-form-item label="保存到">
          <el-input v-model="downloadDialog.path" readonly />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="downloadDialog.visible = false">取消</el-button>
        <el-button type="primary" @click="handleDownload" :loading="downloadDialog.loading">下载</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import { FolderOpened, Document, Link, Search, Close, Plus, Back, RefreshRight, Loading } from '@element-plus/icons-vue'
import { apiFetch, authHeaders, checkRes401 } from '@/utils/api'

interface FileItem {
  name: string
  path: string
  size: number
  is_dir: boolean
  is_link: boolean
  mode: string
  modified: number
  _size?: number
  _calculating?: boolean
}

interface BrowserTab {
  id: string
  title: string
  type: 'browser'
  path: string
  files: FileItem[]
  loading: boolean
  selectedFile: FileItem | null
}

interface EditorTab {
  id: string
  title: string
  type: 'editor'
  path: string
  content: string
  original: string
  saving: boolean
}

type Tab = BrowserTab | EditorTab

const tabs = ref<Tab[]>([])
const activeTab = ref('')
const pathInput = ref('/')
const route = useRoute()
const renamingPath = ref('')
const renamingValue = ref('')
const renamingTab = ref<BrowserTab | null>(null)

const ctxMenu = reactive({
  visible: false,
  x: 0,
  y: 0,
  type: '' as '' | 'blank' | 'dir' | 'file',
  tab: null as BrowserTab | null,
  filePath: '',
  fileName: '',
})

const clipboard = reactive({
  paths: [] as string[],
  cut: false,
})

const STORAGE_KEY = 'alpanel_file_tabs'

interface StoredTab {
  id: string
  title: string
  type: 'browser' | 'editor'
  path: string
  content?: string
  original?: string
}

function saveTabs() {
  const data: StoredTab[] = tabs.value.map(t => ({
    id: t.id,
    title: t.title,
    type: t.type,
    path: t.path,
    ...(t.type === 'editor' ? { content: t.content, original: t.original } : {}),
  }))
  localStorage.setItem(STORAGE_KEY, JSON.stringify({ tabs: data, activeTab: activeTab.value, tabIdSeq }))
}

function restoreTabs() {
  const raw = localStorage.getItem(STORAGE_KEY)
  if (!raw) return false
  try {
    const saved = JSON.parse(raw)
    if (!saved.tabs?.length) return false
    tabIdSeq = saved.tabIdSeq || 0
    const rest: Tab[] = saved.tabs.map((st: StoredTab) => {
      if (st.type === 'editor') {
        return { id: st.id, title: st.title, type: 'editor' as const, path: st.path, content: st.content || '', original: st.original || '', saving: false }
      }
      return { id: st.id, title: st.title, type: 'browser' as const, path: st.path, files: [], loading: false, selectedFile: null }
    })
    // fetch data first, then assign to tabs.value so Vue tracks from the start
    const browserTabs = rest.filter((t): t is BrowserTab => t.type === 'browser')
    Promise.all(browserTabs.map(t =>
      apiFetch(`/api/files/list?path=${encodeURIComponent(t.path)}`).then(data => {
        if (data?.path) t.path = data.path
        t.title = t.path === '/' ? '根目录' : t.path.split('/').filter(Boolean).pop() || '根目录'
        t.files = data?.items || []
      }).catch(() => { t.files = [] })
    )).then(() => {
      tabs.value = rest
      activeTab.value = saved.activeTab || rest[0]?.id || ''
      pathInput.value = (rest.find(t => t.id === activeTab.value) as BrowserTab)?.path || '/'
    })
    return true
  } catch { return false }
}

function closeCtxMenu() {
  ctxMenu.visible = false
}

watch([tabs, activeTab], () => { saveTabs() }, { deep: true })

onMounted(() => {
  document.addEventListener('click', closeCtxMenu)
  const restored = restoreTabs()
  if (!restored) addBrowserTab()
  // handle query param after restore
  const pathQ = route.query.path as string | undefined
  if (pathQ) {
    addBrowserTabAt(pathQ)
    // clean query to avoid re-process on re-mount
    window.history.replaceState(null, '', '/#/file')
  }
})

onUnmounted(() => {
  document.removeEventListener('click', closeCtxMenu)
  saveTabs()
})

function onBrowserContextMenu(e: MouseEvent, tab: BrowserTab) {
  if (renamingPath.value) confirmRename()
  const rowEl = (e.target as HTMLElement).closest('.el-table__row')
  if (rowEl) {
    const tableEl = rowEl.closest('.el-table')
    const rows = tableEl ? Array.from(tableEl.querySelectorAll('.el-table__row')) : []
    const idx = rows.indexOf(rowEl)
    if (idx >= 0 && idx < tab.files.length) {
      onRowContextMenu(e, tab, tab.files[idx])
      return
    }
  }
  onTableContextMenu(e, tab)
}

function onTableContextMenu(e: MouseEvent, tab: BrowserTab) {
  e.preventDefault()
  ctxMenu.x = e.clientX
  ctxMenu.y = e.clientY
  ctxMenu.type = 'blank'
  ctxMenu.tab = tab
  ctxMenu.filePath = ''
  ctxMenu.fileName = ''
  ctxMenu.visible = true
}

function onRowContextMenu(e: MouseEvent, tab: BrowserTab, row: FileItem) {
  e.preventDefault()
  e.stopPropagation()
  ctxMenu.x = e.clientX
  ctxMenu.y = e.clientY
  ctxMenu.type = row.is_dir ? 'dir' : 'file'
  ctxMenu.tab = tab
  ctxMenu.filePath = row.path
  ctxMenu.fileName = row.name
  ctxMenu.visible = true
}

function ctxCopy(path: string) {
  clipboard.paths = [path]
  clipboard.cut = false
}

function ctxCut(path: string) {
  clipboard.paths = [path]
  clipboard.cut = true
}

async function ctxPaste() {
  if (!clipboard.paths.length || !ctxMenu.tab) return
  const tab = ctxMenu.tab
  for (const src of clipboard.paths) {
    const name = src.split('/').filter(Boolean).pop() || src
    let destDir = tab.path
    let destName = name

    if (src === (destDir === '/' ? `/${name}` : `${destDir}/${name}`)) {
      const dot = name.lastIndexOf('.')
      if (dot > 0) {
        destName = name.substring(0, dot) + '(1)' + name.substring(dot)
      } else {
        destName = name + '(1)'
      }
    }

    const dest = destDir === '/' ? `/${destName}` : `${destDir}/${destName}`

    try {
      if (clipboard.cut) {
        await apiFetch('/api/files/rename', {
          method: 'POST',
          body: JSON.stringify({ path: src, new_name: destName }),
        })
      } else {
        await apiFetch('/api/files/copy', {
          method: 'POST',
          body: JSON.stringify({ src, dest }),
        })
      }
    } catch (e: any) {
      ElMessage.error(e?.message || '操作失败')
    }
  }
  clipboard.paths = []
  clipboard.cut = false
  fetchTabList(tab)
}

function ctxRename(path: string) {
  if (!ctxMenu.tab) return
  renamingPath.value = path
  renamingValue.value = ctxMenu.fileName
  renamingTab.value = ctxMenu.tab
  nextTick(() => {
    document.querySelector<HTMLInputElement>('.rename-inline input')?.focus()
  })
}

function ctxDelete(path: string, name: string) {
  if (!ctxMenu.tab) return
  deleteDialog.name = name
  deleteDialog.isDir = ctxMenu.type === 'dir'
  deleteDialog.path = path
  deleteDialog.targetTab = ctxMenu.tab
  deleteDialog.visible = true
}

function ctxOpenEditor() {
  if (ctxMenu.type !== 'file' || !ctxMenu.filePath) return
  const item: FileItem = { name: ctxMenu.fileName, path: ctxMenu.filePath, size: 0, is_dir: false, is_link: false, mode: '', modified: 0 }
  openEditor(item)
}

async function ctxDownload() {
  if (!ctxMenu.filePath) return
  try {
    const res = await fetch(`/api/files/stream?path=${encodeURIComponent(ctxMenu.filePath)}`, {
      headers: authHeaders(),
    })
    checkRes401(res)
    if (!res.ok) {
      const text = await res.text()
      throw new Error(text || res.statusText)
    }
    const blob = await res.blob()
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = ctxMenu.fileName || 'download'
    a.click()
    URL.revokeObjectURL(url)
  } catch (e: any) {
    if (e?.message !== 'unauthorized') ElMessage.error(e?.message || '下载失败')
  }
}

function openInNewTab(path: string) {
  const id = `browser-${++tabIdSeq}`
  tabs.value.push({
    id,
    title: path === '/' ? '根目录' : path.split('/').filter(Boolean).pop() || '根目录',
    type: 'browser',
    path,
    files: [],
    loading: false,
    selectedFile: null,
  })
  activeTab.value = id
  const tab = tabs.value.find(t => t.id === id) as BrowserTab
  fetchTabList(tab)
}

function removeTab(id: string) {
  const idx = tabs.value.findIndex(t => t.id === id)
  if (idx === -1) return
  tabs.value.splice(idx, 1)
  if (tabs.value.length === 0) {
    addBrowserTab()
  } else if (activeTab.value === id) {
    activeTab.value = tabs.value[Math.min(idx, tabs.value.length - 1)].id
  }
}

async function openEditor(item: FileItem) {
  const existing = tabs.value.find(t => t.type === 'editor' && t.id === item.path) as EditorTab | undefined
  if (existing) {
    activeTab.value = existing.id
    return
  }
  const etab: EditorTab = {
    id: item.path,
    title: item.name,
    type: 'editor',
    path: item.path,
    content: '',
    original: '',
    saving: false,
  }
  tabs.value.push(etab)
  activeTab.value = etab.id
  try {
    const data = await apiFetch(`/api/files/read?path=${encodeURIComponent(item.path)}`)
    etab.content = data.content
    etab.original = data.content
  } catch {
    ElMessage.error('无法读取文件')
    tabs.value = tabs.value.filter(t => t.id !== etab.id)
  }
}

async function saveFile(tab: EditorTab) {
  tab.saving = true
  try {
    await apiFetch('/api/files/write', {
      method: 'POST',
      body: JSON.stringify({ path: tab.path, content: tab.content }),
    })
    tab.original = tab.content
    ElMessage.success('已保存')
  } catch (e: any) {
    ElMessage.error(e?.message || '保存失败')
  } finally {
    tab.saving = false
  }
}

function revertFile(tab: EditorTab) {
  tab.content = tab.original
}

const createDialog = reactive({
  visible: false,
  name: '',
  isDir: false,
  targetTab: null as BrowserTab | null,
})

const deleteDialog = reactive({
  visible: false,
  name: '',
  isDir: false,
  path: '',
  targetTab: null as BrowserTab | null,
})

const downloadDialog = reactive({
  visible: false,
  url: '',
  path: '',
  loading: false,
})



let tabIdSeq = 0

function addBrowserTab() {
  const id = `browser-${++tabIdSeq}`
  tabs.value.push({
    id,
    title: 'www',
    type: 'browser',
    path: '/www',
    files: [],
    loading: false,
    selectedFile: null,
  })
  activeTab.value = id
  pathInput.value = '/www'
  const tab = tabs.value.find(t => t.id === id) as BrowserTab
  fetchTabList(tab)
}

function addBrowserTabAt(path: string) {
  const id = `browser-${++tabIdSeq}`
  tabs.value.push({
    id,
    title: path.split('/').filter(Boolean).pop() || '根目录',
    type: 'browser',
    path,
    files: [],
    loading: false,
    selectedFile: null,
  })
  activeTab.value = id
  pathInput.value = path
  const tab = tabs.value.find(t => t.id === id) as BrowserTab
  fetchTabList(tab)
}

function getSegments(p: string): { name: string; fullPath: string }[] {
  const segs: { name: string; fullPath: string }[] = []
  p = p.replace(/\\/g, '/')
  if (/^[a-zA-Z]:\/?$/.test(p)) {
    segs.push({ name: p.endsWith('/') ? p : p + '/', fullPath: p.replace(/\/?$/, '/') })
    return segs
  }
  if (p === '/') {
    segs.push({ name: '根目录', fullPath: '/' })
    return segs
  }
  const parts = p.split('/').filter(Boolean)
  segs.push({ name: '根目录', fullPath: '/' })
  let acc = ''
  for (const part of parts) {
    acc += '/' + part
    segs.push({ name: part, fullPath: acc })
  }
  return segs
}

async function fetchTabList(tab: BrowserTab) {
  tab.loading = true
  try {
    const data = await apiFetch(`/api/files/list?path=${encodeURIComponent(tab.path)}`)
    if (data?.path) tab.path = data.path
    tab.title = tab.path === '/' ? '根目录' : tab.path.split('/').filter(Boolean).pop() || '根目录'
    tab.files = data?.items || []
    pathInput.value = tab.path
  } catch (e: any) {
    tab.files = []
    ElMessage.error(e?.message || '加载失败')
  } finally {
    tab.loading = false
  }
}

function navigateTab(tab: BrowserTab, path: string) {
  tab.path = path
  tab.selectedFile = null
  pathInput.value = path
  fetchTabList(tab)
}

function navigatePathInput(tab: BrowserTab) {
  const p = pathInput.value.trim()
  if (p) navigateTab(tab, p)
}

function canGoUp(tab: BrowserTab): boolean {
  const p = tab.path.replace(/\\/g, '/').replace(/\/$/, '')
  if (/^[a-zA-Z]:$/.test(p)) return false
  return p !== '' && p !== '/'
}

function goUp(tab: BrowserTab) {
  const p = tab.path.replace(/\\/g, '/').replace(/\/$/, '')
  const idx = p.lastIndexOf('/')
  if (idx <= 0) {
    if (/^[a-zA-Z]:/.test(p)) {
      navigateTab(tab, p.charAt(0) + ':/')
    } else {
      navigateTab(tab, '/')
    }
  } else {
    navigateTab(tab, p.substring(0, idx) || '/')
  }
}

function refreshTab(tab: BrowserTab) {
  fetchTabList(tab)
}

async function calcDirSize(_tab: BrowserTab, row: FileItem) {
  row._calculating = true
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), 30000)
  try {
    const res = await fetch(`/api/files/dirsize?path=${encodeURIComponent(row.path)}`, {
      headers: authHeaders(),
      signal: controller.signal,
    })
    checkRes401(res)
    if (!res.ok) {
      const text = await res.text()
      throw new Error(text || res.statusText)
    }
    const data = await res.json()
    row._size = data?.size ?? 0
  } catch (e: any) {
    if (e?.message === 'unauthorized') return
    if (e.name === 'AbortError') {
      ElMessage.error('计算超时（30秒），目录过大')
    } else {
      ElMessage.error(e?.message || '计算失败')
    }
  } finally {
    clearTimeout(timer)
    row._calculating = false
  }
}

function onRowDoubleClick(tab: BrowserTab, row: FileItem) {
  if (row.is_dir) {
    navigateTab(tab, row.path)
  }
}

function openCreate(tab: BrowserTab, isDir: boolean) {
  createDialog.name = ''
  createDialog.isDir = isDir
  createDialog.targetTab = tab
  createDialog.visible = true
}

async function handleCreate() {
  if (!createDialog.name.trim() || !createDialog.targetTab) return
  const tab = createDialog.targetTab
  const p = tab.path.endsWith('/') ? tab.path + createDialog.name : tab.path + '/' + createDialog.name
  try {
    await apiFetch('/api/files/create', {
      method: 'POST',
      body: JSON.stringify({ path: p, type: createDialog.isDir ? 'dir' : 'file' }),
    })
    ElMessage.success(createDialog.isDir ? '目录已创建' : '文件已创建')
    createDialog.visible = false
    fetchTabList(tab)
  } catch (e: any) {
    ElMessage.error(e?.message || '创建失败')
  }
}

function openRename(tab: BrowserTab) {
  if (!tab.selectedFile) return
  renamingPath.value = tab.selectedFile.path
  renamingValue.value = tab.selectedFile.name
  renamingTab.value = tab
  nextTick(() => {
    document.querySelector<HTMLInputElement>('.rename-inline input')?.focus()
  })
}

async function confirmRename() {
  const tab = renamingTab.value
  const oldPath = renamingPath.value
  const newName = renamingValue.value.trim()
  const oldName = oldPath.split('/').filter(Boolean).pop() || ''
  cancelRename()
  if (!tab || !newName || !oldPath || newName === oldName) return
  try {
    await apiFetch('/api/files/rename', {
      method: 'POST',
      body: JSON.stringify({ path: oldPath, new_name: newName }),
    })
    ElMessage.success('已重命名')
    fetchTabList(tab)
  } catch (e: any) {
    ElMessage.error(e?.message || '重命名失败')
  }
}

function cancelRename() {
  renamingPath.value = ''
  renamingValue.value = ''
  renamingTab.value = null
}

function confirmDelete(tab: BrowserTab) {
  if (!tab.selectedFile) return
  deleteDialog.name = tab.selectedFile.name
  deleteDialog.isDir = tab.selectedFile.is_dir
  deleteDialog.path = tab.selectedFile.path
  deleteDialog.targetTab = tab
  deleteDialog.visible = true
}

async function handleDelete() {
  const tab = deleteDialog.targetTab
  if (!tab) return
  try {
    await apiFetch('/api/files/delete', {
      method: 'POST',
      body: JSON.stringify({ path: deleteDialog.path }),
    })
    ElMessage.success('已删除')
    deleteDialog.visible = false
    tab.selectedFile = null
    fetchTabList(tab)
    tabs.value = tabs.value.filter(t => !(t.type === 'editor' && t.id === deleteDialog.path))
  } catch (e: any) {
    ElMessage.error(e?.message || '删除失败')
  }
}

function openDownload(tab: BrowserTab) {
  downloadDialog.url = ''
  downloadDialog.path = tab.path
  downloadDialog.visible = true
}

async function handleDownload() {
  if (!downloadDialog.url.trim()) {
    ElMessage.warning('请输入下载地址')
    return
  }
  downloadDialog.loading = true
  try {
    await apiFetch('/api/files/download', {
      method: 'POST',
      body: JSON.stringify({ url: downloadDialog.url, path: downloadDialog.path }),
    })
    ElMessage.success('下载任务已提交')
    downloadDialog.visible = false
    const tab = tabs.value.find(t => t.id === activeTab.value && t.type === 'browser') as BrowserTab | undefined
    if (tab) fetchTabList(tab)
  } catch (e: any) {
    ElMessage.error(e?.message || '下载失败')
  } finally {
    downloadDialog.loading = false
  }
}

function formatSize(size: number, isDir: boolean): string {
  if (isDir) return '-'
  if (size < 1024) return size + ' B'
  if (size < 1024 * 1024) return (size / 1024).toFixed(1) + ' KB'
  if (size < 1024 * 1024 * 1024) return (size / (1024 * 1024)).toFixed(1) + ' MB'
  return (size / (1024 * 1024 * 1024)).toFixed(1) + ' GB'
}

function formatTime(ts: number): string {
  const d = new Date(ts * 1000)
  const pad = (n: number) => n.toString().padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}
</script>

<style scoped>
.file-manager {
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
  gap: 4px;
  padding: 0 12px;
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

.tab-subtitle {
  color: var(--el-text-color-secondary);
  font-size: 11px;
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tab-close {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  border-radius: 50%;
  padding: 1px;
  margin-left: 2px;
}

.tab-close:hover {
  color: var(--el-color-danger);
  background: var(--el-color-danger-light-9);
}

.tab-add {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 34px;
  cursor: pointer;
  color: var(--el-text-color-secondary);
  flex-shrink: 0;
}

.tab-add:hover {
  color: var(--el-color-primary);
  background: var(--el-fill-color-light);
}

.browser-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  padding: 8px;
  box-sizing: border-box;
}

.path-bar {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-bottom: 6px;
  flex-shrink: 0;
  flex-wrap: wrap;
}

.path-back-btn {
  flex-shrink: 0;
}

.path-breadcrumb {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 0;
  flex-wrap: wrap;
}

.path-sep {
  color: var(--el-text-color-secondary);
  font-size: 12px;
  margin: 0 4px;
  flex-shrink: 0;
}

.seg-pill {
  display: inline-flex;
  align-items: center;
  padding: 2px 10px;
  font-size: 12px;
  border-radius: 3px;
  background: var(--el-fill-color-dark);
  color: var(--el-text-color-primary);
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
}

.seg-pill:hover {
  background: var(--el-color-primary);
  color: #fff;
}

.path-input {
  width: 200px;
  flex-shrink: 0;
}

.path-refresh-btn {
  flex-shrink: 0;
}

.toolbar-row {
  display: flex;
  gap: 5px;
  margin-bottom: 6px;
  flex-shrink: 0;
}

.file-table {
  flex: 1;
  min-height: 0;

  :deep(.el-table__inner-wrapper) {
    table {
      table-layout: fixed;
    }
  }
}

.file-name {
  font-size: 12px;
  cursor: pointer;

  .el-icon {
    vertical-align: middle;
  }
}

.file-name:hover {

  color: var(--el-color-primary);
}

.file-name .el-icon {
  flex-shrink: 0;
}

.is-loading {
  animation: rotating 1s linear infinite;
}

@keyframes rotating {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.file-selected {
  color: var(--el-color-primary);
  font-weight: 600;
}

.rename-inline {
  display: flex;
  align-items: center;
}

.rename-inline .el-input {
  width: 280px;
}

.editor-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  padding: 8px;
}

.editor-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
  flex-shrink: 0;
}

.editor-path {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.editor-textarea {
  flex: 1;
  width: 100%;
  border: 1px solid var(--el-border-color);
  border-radius: 4px;
  padding: 8px;
  font-family: 'Cascadia Code', 'Fira Code', Consolas, monospace;
  font-size: 13px;
  line-height: 1.5;
  resize: none;
  background: var(--el-bg-color);
  color: var(--el-text-color-primary);
  outline: none;
  tab-size: 2;
  min-height: 0;
}

.editor-textarea:focus {
  border-color: var(--el-color-primary);
}
</style>

<style>
.ctx-menu {
  position: fixed;
  z-index: 9999;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 4px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.15);
  padding: 4px 0;
  min-width: 160px;
  font-size: 12px;
}

.ctx-item {
  padding: 6px 16px;
  cursor: pointer;
  white-space: nowrap;
  color: var(--el-text-color-primary);
}

.ctx-item:hover {
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
}

.ctx-item.disabled {
  color: var(--el-text-color-disabled);
  cursor: not-allowed;
}

.ctx-item.disabled:hover {
  background: transparent;
  color: var(--el-text-color-disabled);
}

.ctx-divider {
  height: 1px;
  background: var(--el-border-color-lighter);
  margin: 4px 0;
}
</style>
