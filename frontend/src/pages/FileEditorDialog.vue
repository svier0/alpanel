<template>
  <Teleport to="body">
    <div v-if="visible" class="ed-window" :class="{ maximized: isMaximized }" :style="windowStyle" @mousedown="bringFront">
      <div class="ed-titlebar" @mousedown="onTitleMouseDown">
        <span class="ed-title">在线编辑器</span>
        <div class="ed-title-spacer" />
        <div class="ed-win-btns">
          <button class="ed-win-btn" title="最小化" @click.stop="minimize"><el-icon><Minus /></el-icon></button>
          <button class="ed-win-btn" :title="isMaximized ? '还原' : '最大化'" @click.stop="toggleMaximize">
            <el-icon v-if="isMaximized"><CopyDocument /></el-icon>
            <el-icon v-else><FullScreen /></el-icon>
          </button>
          <button class="ed-win-btn ed-win-close" title="关闭" @click.stop="close"><el-icon><Close /></el-icon></button>
        </div>
      </div>

      <div class="ed-body">
        <div class="ed-tree">
          <div class="ed-tree-head">
            <span class="ed-tree-path" :title="treePath">{{ treePath }}</span>
          </div>
          <div class="ed-tree-toolbar">
            <el-tooltip content="上一级" placement="bottom" :z-index="5000">
              <button class="ed-tool" :disabled="!canGoUp" @click="goUp"><el-icon><Top /></el-icon></button>
            </el-tooltip>
            <el-tooltip content="刷新" placement="bottom" :z-index="5000">
              <button class="ed-tool" @click="refreshTree"><el-icon><Refresh /></el-icon></button>
            </el-tooltip>
            <el-tooltip content="新建文件" placement="bottom" :z-index="5000">
              <button class="ed-tool" @click="openCreate(false)"><el-icon><DocumentAdd /></el-icon></button>
            </el-tooltip>
            <el-tooltip content="新建目录" placement="bottom" :z-index="5000">
              <button class="ed-tool" @click="openCreate(true)"><el-icon><FolderAdd /></el-icon></button>
            </el-tooltip>
          </div>
          <div class="ed-tree-body">
            <FileTree
              :nodes="treeNodes"
              :loading="treeLoading"
              :active-path="activePath"
              :open-paths="openPaths"
              :renaming-path="renamingPath"
              :renaming-value="renamingValue"
              @toggle="onToggleNode"
              @select="onSelectFile"
              @ctx="onTreeCtx"
              @rename="onTreeRename"
              @cancel-rename="cancelRename"
            />
          </div>
        </div>

        <div class="ed-main">
          <div class="ed-tabs">
            <div v-for="tab in tabs" :key="tab.path" class="ed-tab" :class="{ active: activePath === tab.path }" @click="switchTab(tab.path)">
              <span class="ed-tab-name">{{ tab.name }}</span>
              <span v-if="tab.dirty" class="ed-tab-dirty">●</span>
              <el-icon class="ed-tab-close" @click.stop="closeTab(tab.path)"><Close /></el-icon>
            </div>
            <div class="ed-tabs-spacer" />
          </div>

          <div class="ed-editor-wrap">
            <div v-if="tabs.length === 0" class="ed-empty">
              <el-empty description="双击左侧文件打开编辑" />
            </div>
            <template v-for="tab in tabs" :key="tab.path">
              <div v-show="activePath === tab.path" class="ed-cm-host" :ref="setCmHostRef">
                <CodeMirrorHost
                  :value="tab.content"
                  :language="tab.language"
                  :dark="isDark"
                  @update:value="onContentChange(tab.path, $event)"
                  @cursor="onCursor(tab.path, $event)"
                />
              </div>
            </template>
            <div v-if="activeTab" class="ed-statusbar">
              <span class="ed-status-item">行:{{ activeTab.cursor.line }} 列:{{ activeTab.cursor.col }}</span>
              <span class="ed-status-item">UTF-8</span>
              <span class="ed-status-item">{{ activeTab.language }}</span>
              <span class="ed-status-item ed-status-right">{{ activeTab.dirty ? '未保存' : '已保存' }}</span>
            </div>
          </div>
        </div>
      </div>

      <div class="ed-resize" @mousedown.stop="onResizeMouseDown" />

      <div
        v-if="ctxMenu.visible"
        class="ed-ctx"
        :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px', zIndex: zIndex + 10 }"
        @click.stop
      >
        <template v-if="ctxMenu.isDir">
          <div class="ed-ctx-item" @click="ctxRefreshDir">刷新目录</div>
          <div class="ed-ctx-item" @click="ctxOpenSubdir">打开子目录</div>
          <div class="ed-ctx-item" @click="ctxNewDir">新建目录</div>
          <div class="ed-ctx-item" @click="ctxNewFile">新建文件</div>
          <div class="ed-ctx-item" @click="ctxRename">重命名</div>
        </template>
        <template v-else>
          <div class="ed-ctx-item" @click="ctxRename">重命名</div>
          <div class="ed-ctx-item" @click="ctxDownload">下载</div>
        </template>
        <div class="ed-ctx-sep" />
        <div class="ed-ctx-item ed-ctx-danger" @click="ctxDelete">删除</div>
      </div>
    </div>

    <div v-if="visible && isMinimized" class="ed-minbar" :style="{ zIndex: zIndex }" @click="restore">
      <el-icon><Document /></el-icon>
      <span>在线编辑器</span>
    </div>

    <el-dialog v-model="createDialog.visible" :title="createDialog.isDir ? '新建目录' : '新建文件'" width="400px" append-to-body :z-index="zIndex + 100" @closed="createDialog.name=''">
      <el-form @submit.prevent="handleCreate">
        <el-form-item label="位置">
          <span class="ed-create-path">{{ createDialog.targetDir || treePath }}</span>
        </el-form-item>
        <el-form-item :label="createDialog.isDir ? '目录名' : '文件名'">
          <el-input v-model="createDialog.name" placeholder="请输入名称" @keyup.enter="handleCreate" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="createDialog.visible = false">取消</el-button>
        <el-button type="primary" @click="handleCreate">确定</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="deleteDialog.visible" title="删除" width="400px" append-to-body :z-index="zIndex + 100">
      <p>确定删除以下项目吗？此操作不可恢复。</p>
      <ul class="ed-delete-list">
        <li v-for="item in deleteDialog.items" :key="item.path">
          <el-icon v-if="item.is_dir"><Folder /></el-icon>
          <el-icon v-else><Document /></el-icon>
          <span>{{ item.name }}</span>
        </li>
      </ul>
      <template #footer>
        <el-button @click="deleteDialog.visible = false">取消</el-button>
        <el-button type="danger" @click="handleDelete">删除</el-button>
      </template>
    </el-dialog>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, ComponentPublicInstance } from 'vue'
import { ElMessage } from 'element-plus'
import { Refresh, Close, Minus, FullScreen, CopyDocument, Document, Top, DocumentAdd, FolderAdd } from '@element-plus/icons-vue'
import { apiFetch, authHeaders } from '@/utils/api'
import { checkRes401 } from '@/utils/api'
import FileTree, { type TreeNode } from './FileTree.vue'
import CodeMirrorHost, { type CursorPos } from './CodeMirrorHost.vue'

const props = defineProps<{ modelValue: boolean; rootPath?: string; initialFile?: string }>()
const emit = defineEmits<{ 'update:modelValue': [boolean] }>()

const visible = computed({
  get: () => props.modelValue,
  set: (v) => emit('update:modelValue', v),
})

const isDark = ref(false)

// window geometry
const isMaximized = ref(false)
const isMinimized = ref(false)
const zIndex = ref(3000)
let zCounter = 3000
const win = reactive({ x: 60, y: 60, w: 1200, h: 740 })
function centerWindow() {
  const sw = window.innerWidth
  const sh = window.innerHeight
  win.x = Math.max(0, Math.round((sw - win.w) / 2))
  win.y = Math.max(0, Math.round((sh - win.h) / 2))
}
const windowStyle = computed(() => {
  if (isMaximized.value) return { zIndex: zIndex.value }
  if (isMinimized.value) return { display: 'none' }
  return {
    left: win.x + 'px',
    top: win.y + 'px',
    width: win.w + 'px',
    height: win.h + 'px',
    zIndex: zIndex.value,
  }
})

let dragState: { ox: number; oy: number; mx: number; my: number } | null = null
function onTitleMouseDown(e: MouseEvent) {
  if (isMaximized.value) return
  dragState = { ox: win.x, oy: win.y, mx: e.clientX, my: e.clientY }
  document.addEventListener('mousemove', onDragMove)
  document.addEventListener('mouseup', onDragUp)
}
function onDragMove(e: MouseEvent) {
  if (!dragState) return
  win.x = Math.max(0, dragState.ox + (e.clientX - dragState.mx))
  win.y = Math.max(0, dragState.oy + (e.clientY - dragState.my))
}
function onDragUp() {
  dragState = null
  document.removeEventListener('mousemove', onDragMove)
  document.removeEventListener('mouseup', onDragUp)
}

let resizeState: { ow: number; oh: number; mx: number; my: number } | null = null
function onResizeMouseDown(e: MouseEvent) {
  if (isMaximized.value) return
  resizeState = { ow: win.w, oh: win.h, mx: e.clientX, my: e.clientY }
  document.addEventListener('mousemove', onResizeMove)
  document.addEventListener('mouseup', onResizeUp)
}
function onResizeMove(e: MouseEvent) {
  if (!resizeState) return
  win.w = Math.max(480, resizeState.ow + (e.clientX - resizeState.mx))
  win.h = Math.max(320, resizeState.oh + (e.clientY - resizeState.my))
}
function onResizeUp() {
  resizeState = null
  document.removeEventListener('mousemove', onResizeMove)
  document.removeEventListener('mouseup', onResizeUp)
}

function bringFront() {
  zCounter += 1
  zIndex.value = zCounter
}
function toggleMaximize() {
  isMaximized.value = !isMaximized.value
}
function minimize() {
  isMinimized.value = true
}
function restore() {
  isMinimized.value = false
  bringFront()
}

interface EditTab {
  name: string
  path: string
  content: string
  original: string
  language: string
  dirty: boolean
  cursor: CursorPos
  loaded: boolean
  loading: boolean
}

const tabs = reactive<EditTab[]>([])
const activePath = ref('')
const treePath = ref(props.rootPath || '/')
const openPaths = reactive(new Set<string>())
const treeNodes = ref<TreeNode[]>([])
const treeLoading = ref(false)
const cmHosts = new Map<string, any>()
const renamingPath = ref('')
const renamingValue = ref('')

const activeTab = computed(() => tabs.find((t) => t.path === activePath.value) || null)

function canGoUp(): boolean {
  const p = treePath.value.replace(/\/$/, '')
  return p !== '' && p !== '/'
}

function setCmHostRef(el: Element | ComponentPublicInstance | null) {
  if (el) cmHosts.set(activePath.value, el)
}

function detectLanguage(path: string): string {
  const base = path.split('/').filter(Boolean).pop() || ''
  const lower = base.toLowerCase()
  if (lower === 'dockerfile') return 'dockerfile'
  if (lower === 'nginx.conf' || lower.endsWith('.conf')) return 'nginx'
  const ext = lower.includes('.') ? lower.split('.').pop()! : ''
  const map: Record<string, string> = {
    php: 'php', js: 'javascript', ts: 'javascript', mjs: 'javascript', cjs: 'javascript',
    css: 'css', scss: 'css', less: 'css', html: 'html', htm: 'html', vue: 'html', xml: 'html',
    json: 'json', sh: 'shell', bash: 'shell', zsh: 'shell', yml: 'yaml', yaml: 'yaml',
    py: 'python', ini: 'ini', conf: 'ini', cfg: 'ini', sql: 'sql', toml: 'toml',
    env: 'ini', txt: 'text', log: 'text', md: 'markdown',
  }
  return map[ext] || 'text'
}

async function loadTree(path: string) {
  if (treeLoading.value) return
  treeLoading.value = true
  try {
    const data = await apiFetch(`/api/files/list?path=${encodeURIComponent(path)}`)
    treePath.value = data.path || path
    const items: any[] = data.items || []
    const children: TreeNode[] = items
      .filter((i) => i.is_dir)
      .sort((a, b) => a.name.localeCompare(b.name, 'zh'))
      .map((i) => ({ name: i.name, path: i.path, is_dir: true, loaded: false, children: [], expanded: openPaths.has(i.path) }))
    const files: TreeNode[] = items
      .filter((i) => !i.is_dir)
      .sort((a, b) => a.name.localeCompare(b.name, 'zh'))
      .map((i) => ({ name: i.name, path: i.path, is_dir: false, loaded: true, children: [], expanded: false }))
    treeNodes.value = [...children, ...files]
  } catch (e: any) {
    ElMessage.error(e?.message || '目录加载失败')
  } finally {
    treeLoading.value = false
  }
}

async function refreshTree() {
  await loadTree(treePath.value)
}

async function goUp() {
  if (!canGoUp()) return
  const p = treePath.value.replace(/\/$/, '')
  const idx = p.lastIndexOf('/')
  const parent = idx <= 0 ? '/' : p.substring(0, idx) || '/'
  await loadTree(parent)
}

async function onToggleNode(node: TreeNode) {
  if (!node.is_dir) {
    openFile(node.path, node.name)
    return
  }
  node.expanded = !node.expanded
  if (node.expanded) {
    openPaths.add(node.path)
    if (!node.loaded || !node.children?.length) {
      try {
        const data = await apiFetch(`/api/files/list?path=${encodeURIComponent(node.path)}`)
        const items: any[] = data.items || []
        node.children = [
          ...items.filter((i) => i.is_dir).sort((a, b) => a.name.localeCompare(b.name, 'zh'))
            .map((i) => ({ name: i.name, path: i.path, is_dir: true, loaded: false, children: [], expanded: openPaths.has(i.path) })),
          ...items.filter((i) => !i.is_dir).sort((a, b) => a.name.localeCompare(b.name, 'zh'))
            .map((i) => ({ name: i.name, path: i.path, is_dir: false, loaded: true, children: [], expanded: false })),
        ]
        node.loaded = true
      } catch (e: any) {
        ElMessage.error(e?.message || '目录加载失败')
      }
    }
  } else {
    openPaths.delete(node.path)
  }
}

async function onSelectFile(node: TreeNode) {
  if (!node.is_dir) openFile(node.path, node.name)
}

async function openFile(path: string, name: string) {
  const existing = tabs.find((t) => t.path === path)
  if (existing) {
    if (!existing.loaded && !existing.loading) await loadFileContent(existing)
    activePath.value = path
    return
  }
  const tab: EditTab = {
    name, path, content: '', original: '', language: detectLanguage(path),
    dirty: false, cursor: { line: 1, col: 1 }, loaded: false, loading: false,
  }
  tabs.push(tab)
  await loadFileContent(tab)
  activePath.value = path
}

async function loadFileContent(tab: EditTab) {
  tab.loading = true
  try {
    const data = await apiFetch(`/api/files/read?path=${encodeURIComponent(tab.path)}`)
    tab.content = data.content ?? ''
    tab.original = tab.content
    tab.dirty = false
    tab.loaded = true
    if (tab.language === 'text') {
      const lang = detectShebang(tab.content)
      if (lang) tab.language = lang
    }
  } catch (e: any) {
    ElMessage.error(e?.message || '读取失败')
    tabs.splice(tabs.indexOf(tab), 1)
  } finally {
    tab.loading = false
  }
}

function detectShebang(content: string): string | null {
  const nl = content.indexOf('\n')
  const first = (nl >= 0 ? content.slice(0, nl) : content).trimEnd()
  if (!first.startsWith('#!')) return null
  const lower = first.toLowerCase()
  if (lower.includes('bash') || lower.includes('sh') || lower.includes('zsh')) return 'shell'
  if (lower.includes('python')) return 'python'
  if (lower.includes('php')) return 'php'
  if (lower.includes('perl')) return 'python'
  if (lower.includes('node') || lower.includes('javascript')) return 'javascript'
  return 'shell'
}

function onContentChange(path: string, val: string) {
  const tab = tabs.find((t) => t.path === path)
  if (!tab) return
  tab.content = val
  tab.dirty = val !== tab.original
}

function onCursor(path: string, pos: CursorPos) {
  const tab = tabs.find((t) => t.path === path)
  if (tab) tab.cursor = pos
}

function switchTab(path: string) {
  activePath.value = path
}

function closeTab(path: string) {
  const idx = tabs.findIndex((t) => t.path === path)
  if (idx === -1) return
  const wasActive = activePath.value === path
  tabs.splice(idx, 1)
  cmHosts.delete(path)
  if (wasActive) {
    activePath.value = tabs.length ? tabs[Math.max(0, idx - 1)].path : ''
  }
}

const ctxMenu = reactive({
  visible: false,
  x: 0,
  y: 0,
  node: null as TreeNode | null,
  isDir: false,
})

function onTreeCtx(node: TreeNode, e: MouseEvent) {
  closeCtxMenu()
  ctxMenu.node = node
  ctxMenu.isDir = node.is_dir
  ctxMenu.x = e.clientX
  ctxMenu.y = e.clientY
  ctxMenu.visible = true
  document.addEventListener('mousedown', onDocMouseDown, true)
  document.addEventListener('contextmenu', onDocCtx, true)
}

function onDocMouseDown(e: MouseEvent) {
  if (!(e.target as HTMLElement)?.closest('.ed-ctx')) closeCtxMenu()
}
function onDocCtx(e: MouseEvent) {
  if ((e.target as HTMLElement)?.closest('.ed-tree') || (e.target as HTMLElement)?.closest('.ed-ctx')) return
  closeCtxMenu()
}

function closeCtxMenu() {
  if (!ctxMenu.visible && !document.contains(document.querySelector('.ed-ctx'))) {
    document.removeEventListener('mousedown', onDocMouseDown, true)
    document.removeEventListener('contextmenu', onDocCtx, true)
    return
  }
  ctxMenu.visible = false
  document.removeEventListener('mousedown', onDocMouseDown, true)
  document.removeEventListener('contextmenu', onDocCtx, true)
}

function ctxRefreshDir() {
  const node = ctxMenu.node
  closeCtxMenu()
  if (!node || !node.is_dir) return
  if (!node.expanded) {
    onToggleNode(node)
  } else {
    loadSubdir(node)
  }
}

async function loadSubdir(node: TreeNode) {
  node.expanded = true
  openPaths.add(node.path)
  if (!node.loaded || !node.children?.length) {
    try {
      const data = await apiFetch(`/api/files/list?path=${encodeURIComponent(node.path)}`)
      const items: any[] = data.items || []
      node.children = [
        ...items.filter((i) => i.is_dir).sort((a, b) => a.name.localeCompare(b.name, 'zh'))
          .map((i) => ({ name: i.name, path: i.path, is_dir: true, loaded: false, children: [], expanded: openPaths.has(i.path) })),
        ...items.filter((i) => !i.is_dir).sort((a, b) => a.name.localeCompare(b.name, 'zh'))
          .map((i) => ({ name: i.name, path: i.path, is_dir: false, loaded: true, children: [], expanded: false })),
      ]
      node.loaded = true
    } catch (e: any) {
      ElMessage.error(e?.message || '目录加载失败')
    }
  }
}

function ctxOpenSubdir() {
  const node = ctxMenu.node
  closeCtxMenu()
  if (!node || !node.is_dir) return
  loadSubdir(node)
  loadTree(node.path)
}

function ctxNewDir() {
  const node = ctxMenu.node
  closeCtxMenu()
  if (!node) return
  const base = node.is_dir ? node.path : node.path.replace(/\/[^/]*$/, '') || '/'
  createDialog.name = ''
  createDialog.isDir = true
  createDialog.targetDir = base
  createDialog.visible = true
}

function ctxNewFile() {
  const node = ctxMenu.node
  closeCtxMenu()
  if (!node) return
  const base = node.is_dir ? node.path : node.path.replace(/\/[^/]*$/, '') || '/'
  createDialog.name = ''
  createDialog.isDir = false
  createDialog.targetDir = base
  createDialog.visible = true
}

function ctxRename() {
  const node = ctxMenu.node
  closeCtxMenu()
  if (!node) return
  renamingPath.value = node.path
  renamingValue.value = node.name
}

function cancelRename() {
  renamingPath.value = ''
  renamingValue.value = ''
}

async function onTreeRename(node: TreeNode) {
  const oldPath = node.path
  const newName = renamingValue.value.trim()
  const oldName = oldPath.split('/').filter(Boolean).pop() || ''
  cancelRename()
  if (!newName || newName === oldName) return
  const parent = oldPath.replace(/\/[^/]*$/, '') || ''
  const newPath = parent ? `${parent}/${newName}` : `/${newName}`
  try {
    await apiFetch('/api/files/rename', {
      method: 'POST',
      body: JSON.stringify({ path: oldPath, new_name: newName }),
    })
    ElMessage.success('已重命名')
    if (node.is_dir) {
      openPaths.delete(oldPath)
      openPaths.add(newPath)
    }
    refreshTree()
    updateTabPaths(oldPath, newPath)
  } catch (e: any) {
    ElMessage.error(e?.message || '重命名失败')
  }
}

function updateTabPaths(oldPath: string, newPath: string) {
  for (const tab of tabs) {
    if (tab.path === oldPath) {
      tab.path = newPath
      tab.name = newPath.split('/').filter(Boolean).pop() || newPath
    } else if (oldPath.endsWith('/') && tab.path.startsWith(oldPath)) {
      const np = newPath + tab.path.slice(oldPath.length)
      tab.path = np
      tab.name = np.split('/').filter(Boolean).pop() || np
    }
  }
}

function ctxDelete() {
  const node = ctxMenu.node
  closeCtxMenu()
  if (!node) return
  deleteDialog.items = [{ name: node.name, path: node.path, is_dir: node.is_dir }]
  deleteDialog.visible = true
}

async function ctxDownload() {
  const node = ctxMenu.node
  closeCtxMenu()
  if (!node || node.is_dir) return
  try {
    const res = await fetch(`/api/files/stream?path=${encodeURIComponent(node.path)}`, {
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
    a.download = node.name
    a.click()
    URL.revokeObjectURL(url)
  } catch (e: any) {
    if (e?.message !== 'unauthorized') ElMessage.error(e?.message || '下载失败')
  }
}

async function saveTab(path: string) {
  const tab = tabs.find((t) => t.path === path)
  if (!tab || !tab.dirty) return
  try {
    await apiFetch('/api/files/write', {
      method: 'POST',
      body: JSON.stringify({ path: tab.path, content: tab.content }),
    })
    tab.original = tab.content
    tab.dirty = false
    ElMessage.success('已保存')
  } catch (e: any) {
    ElMessage.error(e?.message || '保存失败')
  }
}

const createDialog = reactive({
  visible: false,
  name: '',
  isDir: false,
  targetDir: '',
})

function openCreate(isDir: boolean) {
  createDialog.name = ''
  createDialog.isDir = isDir
  createDialog.targetDir = treePath.value
  createDialog.visible = true
}

async function handleCreate() {
  if (!createDialog.name.trim()) return
  const base = createDialog.targetDir || treePath.value
  const p = base.endsWith('/') ? base + createDialog.name : base + '/' + createDialog.name
  try {
    await apiFetch('/api/files/create', {
      method: 'POST',
      body: JSON.stringify({ path: p, type: createDialog.isDir ? 'dir' : 'file' }),
    })
    ElMessage.success(createDialog.isDir ? '目录已创建' : '文件已创建')
    createDialog.visible = false
    refreshTree()
  } catch (e: any) {
    ElMessage.error(e?.message || '创建失败')
  }
}

const deleteDialog = reactive({
  visible: false,
  items: [] as { name: string; path: string; is_dir: boolean }[],
})

async function handleDelete() {
  try {
    for (const item of deleteDialog.items) {
      await apiFetch('/api/files/delete', {
        method: 'POST',
        body: JSON.stringify({ path: item.path }),
      })
      for (let i = tabs.length - 1; i >= 0; i--) {
        const t = tabs[i]
        if (t.path === item.path || (item.is_dir && t.path.startsWith(item.path + '/'))) {
          tabs.splice(i, 1)
          cmHosts.delete(t.path)
        }
      }
    }
    ElMessage.success('已删除')
    deleteDialog.visible = false
    if (activePath.value === '') {
      activePath.value = tabs.length ? tabs[0].path : ''
    }
    refreshTree()
  } catch (e: any) {
    ElMessage.error(e?.message || '删除失败')
  }
}

function onOpen() {
  const isDarkMode =
    document.documentElement.classList.contains('dark') ||
    document.documentElement.classList.contains('html-dark') ||
    window.matchMedia('(prefers-color-scheme: dark)').matches
  isDark.value = !!isDarkMode
  centerWindow()
  treePath.value = props.rootPath || '/'
  openPaths.clear()
  loadTree(treePath.value)
  if (props.initialFile) {
    const name = props.initialFile.split('/').filter(Boolean).pop() || props.initialFile
    openFile(props.initialFile, name)
  }
  window.addEventListener('keydown', onKeydown)
}

function onClose() {
  window.removeEventListener('keydown', onKeydown)
}

function onKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
    e.preventDefault()
    if (activePath.value) saveTab(activePath.value)
  }
}

function close() {
  visible.value = false
  onClose()
}

watch(() => props.modelValue, (v) => {
  if (v) {
    isMinimized.value = false
    onOpen()
  } else {
    onClose()
  }
})
</script>

<style scoped>
.ed-window {
  position: fixed;
  display: flex;
  flex-direction: column;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color);
  border-radius: 6px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25);
  overflow: hidden;
  min-width: 480px;
  min-height: 320px;
}
.ed-window.maximized {
  left: 0 !important;
  top: 0 !important;
  width: 100vw !important;
  height: 100vh !important;
  border-radius: 0;
  border: none;
}

.ed-titlebar {
  display: flex;
  align-items: center;
  height: 38px;
  padding: 0 8px 0 12px;
  background: var(--el-fill-color-light);
  border-bottom: 1px solid var(--el-border-color-lighter);
  cursor: move;
  flex-shrink: 0;
  user-select: none;
}
.ed-title { font-weight: 600; font-size: 13px; }
.ed-title-spacer { flex: 1; }

.ed-win-btns { display: flex; gap: 2px; }
.ed-win-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 26px;
  border: none;
  background: transparent;
  color: var(--el-text-color-regular);
  cursor: pointer;
  border-radius: 4px;
  font-size: 14px;
}
.ed-win-btn:hover { background: var(--el-fill-color-dark); }
.ed-win-close:hover { background: var(--el-color-danger); color: #fff; }

.ed-resize {
  position: absolute;
  right: 0;
  bottom: 0;
  width: 14px;
  height: 14px;
  cursor: nwse-resize;
}

.ed-minbar {
  position: fixed;
  right: 16px;
  bottom: 16px;
  display: flex;
  align-items: center;
  gap: 6px;
  height: 34px;
  padding: 0 14px;
  background: var(--el-color-primary);
  color: #fff;
  font-size: 12px;
  border-radius: 17px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
  cursor: pointer;
}

.ed-body {
  display: flex;
  flex: 1;
  min-height: 0;
  background: var(--el-bg-color);
}

.ed-tree {
  width: 260px;
  flex-shrink: 0;
  border-right: 1px solid var(--el-border-color-lighter);
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.ed-tree-head {
  padding: 8px 12px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  flex-shrink: 0;
}
.ed-tree-path {
  font-size: 12px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ed-tree-toolbar {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 4px 8px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  flex-shrink: 0;
}
.ed-tool {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 26px;
  border: none;
  background: transparent;
  color: var(--el-text-color-regular);
  cursor: pointer;
  border-radius: 4px;
  font-size: 15px;
}
.ed-tool:hover:not(:disabled) { background: var(--el-fill-color-dark); color: var(--el-color-primary); }
.ed-tool:disabled { color: var(--el-text-color-disabled); cursor: not-allowed; }

.ed-tree-body {
  flex: 1;
  overflow: auto;
  padding: 4px 0;
}

.ed-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}

.ed-tabs {
  display: flex;
  align-items: stretch;
  background: var(--el-fill-color);
  border-bottom: 1px solid var(--el-border-color-lighter);
  flex-shrink: 0;
  overflow-x: auto;
}

.ed-tab {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0 8px 0 12px;
  height: 34px;
  font-size: 12px;
  cursor: pointer;
  border-right: 1px solid var(--el-border-color-lighter);
  white-space: nowrap;
  color: var(--el-text-color-regular);
  user-select: none;
  flex-shrink: 0;
}

.ed-tab:hover { background: var(--el-fill-color-light); }
.ed-tab.active {
  background: var(--el-bg-color);
  color: var(--el-color-primary);
  border-bottom: 2px solid var(--el-color-primary);
  margin-bottom: -1px;
}

.ed-tab-name { font-weight: 500; }
.ed-tab-dirty { color: var(--el-color-danger); font-size: 10px; }
.ed-tab-close {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  border-radius: 50%;
  padding: 2px;
}
.ed-tab-close:hover { color: var(--el-color-danger); background: var(--el-color-danger-light-9); }

.ed-editor-wrap {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  position: relative;
}

.ed-cm-host {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.ed-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.ed-statusbar {
  display: flex;
  align-items: center;
  gap: 16px;
  height: 26px;
  padding: 0 12px;
  font-size: 12px;
  background: var(--el-fill-color);
  border-top: 1px solid var(--el-border-color-lighter);
  color: var(--el-text-color-secondary);
  flex-shrink: 0;
}

.ed-status-right { margin-left: auto; }
.ed-tabs-spacer { flex: 1; }

.ed-ctx {
  position: fixed;
  min-width: 140px;
  background: var(--el-bg-color-overlay);
  border: 1px solid var(--el-border-color-light);
  border-radius: 6px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.18);
  padding: 4px;
  font-size: 12px;
  user-select: none;
}
.ed-ctx-item {
  padding: 6px 12px;
  cursor: pointer;
  border-radius: 4px;
  color: var(--el-text-color-regular);
  white-space: nowrap;
}
.ed-ctx-item:hover { background: var(--el-color-primary-light-9); color: var(--el-color-primary); }
.ed-ctx-danger { color: var(--el-color-danger); }
.ed-ctx-danger:hover { background: var(--el-color-danger-light-9); color: var(--el-color-danger); }
.ed-ctx-sep { height: 1px; background: var(--el-border-color-lighter); margin: 4px 0; }

.ed-create-path {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  word-break: break-all;
}

.ed-delete-list {
  list-style: none;
  margin: 8px 0 0;
  padding: 0;
  max-height: 200px;
  overflow: auto;
}
.ed-delete-list li {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 0;
  font-size: 13px;
}
.ed-delete-list .el-icon { color: var(--el-text-color-secondary); }
</style>
