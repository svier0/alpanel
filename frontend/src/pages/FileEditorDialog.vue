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
              @toggle="onToggleNode"
              @select="onSelectFile"
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
    </div>

    <div v-if="visible && isMinimized" class="ed-minbar" :style="{ zIndex: zIndex }" @click="restore">
      <el-icon><Document /></el-icon>
      <span>在线编辑器</span>
    </div>

    <el-dialog v-model="createDialog.visible" :title="createDialog.isDir ? '新建目录' : '新建文件'" width="400px" append-to-body @closed="createDialog.name=''">
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
  </Teleport>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, ComponentPublicInstance } from 'vue'
import { ElMessage } from 'element-plus'
import { Refresh, Close, Minus, FullScreen, CopyDocument, Document, Top, DocumentAdd, FolderAdd } from '@element-plus/icons-vue'
import { apiFetch } from '@/utils/api'
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

const activeTab = computed(() => tabs.find((t) => t.path === activePath.value) || null)

function canGoUp(): boolean {
  const p = treePath.value.replace(/\/$/, '')
  return p !== '' && p !== '/'
}

function setCmHostRef(el: Element | ComponentPublicInstance | null) {
  if (el) cmHosts.set(activePath.value, el)
}

function detectLanguage(path: string): string {
  const ext = path.split('.').pop()?.toLowerCase() || ''
  const map: Record<string, string> = {
    php: 'php', js: 'javascript', ts: 'javascript', mjs: 'javascript', cjs: 'javascript',
    css: 'css', scss: 'css', less: 'css', html: 'html', htm: 'html', vue: 'html', xml: 'html',
    json: 'json', md: 'markdown', txt: 'text', log: 'text',
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
  } catch (e: any) {
    ElMessage.error(e?.message || '读取失败')
    tabs.splice(tabs.indexOf(tab), 1)
  } finally {
    tab.loading = false
  }
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
})

function openCreate(isDir: boolean) {
  createDialog.name = ''
  createDialog.isDir = isDir
  createDialog.visible = true
}

async function handleCreate() {
  if (!createDialog.name.trim()) return
  const p = treePath.value.endsWith('/') ? treePath.value + createDialog.name : treePath.value + '/' + createDialog.name
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

function onOpen() {
  const theme = document.documentElement.classList.contains('dark') ||
    getComputedStyle(document.body).backgroundColor.includes('rgb(0, 0, 0)')
  isDark.value = !!theme
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
</style>
