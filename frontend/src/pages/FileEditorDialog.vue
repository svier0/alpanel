<template>
  <el-dialog
    v-model="visible"
    fullscreen
    :show-close="false"
    append-to-body
    class="editor-dialog"
    @open="onOpen"
    @closed="onClosed"
  >
    <template #header>
      <div class="ed-header">
        <span class="ed-title">在线编辑器</span>
        <div class="ed-header-spacer" />
        <el-button size="small" @click="close">关闭</el-button>
      </div>
    </template>

    <div class="ed-body">
      <!-- 目录树 -->
      <div class="ed-tree">
        <div class="ed-tree-head">
          <span>目录</span>
          <el-button size="small" text :icon="Refresh" @click="loadTree(rootPath)" />
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

      <!-- 编辑区 -->
      <div class="ed-main">
        <div class="ed-tabs">
          <div
            v-for="tab in tabs"
            :key="tab.path"
            class="ed-tab"
            :class="{ active: activePath === tab.path }"
            @click="switchTab(tab.path)"
          >
            <span class="ed-tab-name">{{ tab.name }}</span>
            <span class="ed-tab-dirty" v-if="tab.dirty">●</span>
            <el-icon class="ed-tab-close" @click.stop="closeTab(tab.path)"><Close /></el-icon>
          </div>
          <div class="ed-tabs-spacer" />
        </div>

        <div class="ed-editor-wrap">
          <div v-if="tabs.length === 0" class="ed-empty">
            <el-empty description="双击左侧文件打开编辑" />
          </div>
          <template v-for="tab in tabs" :key="tab.path">
            <div v-show="activePath === tab.path" class="ed-cm-host" :ref="(el) => setCmHost(el, tab.path)">
              <CodeMirrorHost
                v-if="activePath === tab.path"
                :value="tab.content"
                :language="tab.language"
                :dark="isDark"
                @update:value="(v: string) => onContentChange(tab.path, v)"
                @cursor="(p: CursorPos) => onCursor(tab.path, p)"
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
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { Refresh, Close } from '@element-plus/icons-vue'
import { apiFetch } from '@/utils/api'
import FileTree, { type TreeNode } from './FileTree.vue'
import CodeMirrorHost, { type CursorPos } from './CodeMirrorHost.vue'

const props = defineProps<{ modelValue: boolean; rootPath?: string; initialFile?: string }>()
const emit = defineEmits<{ 'update:modelValue': [boolean] }>()

const visible = computed({
  get: () => props.modelValue,
  set: (v) => emit('update:modelValue', v),
})

const rootPath = props.rootPath || '/www'
const isDark = ref(false)

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
const openPaths = reactive(new Set<string>([rootPath]))
const treeNodes = ref<TreeNode[]>([])
const treeLoading = ref(false)
const cmHosts = new Map<string, any>()

const activeTab = computed(() => tabs.find((t) => t.path === activePath.value) || null)

function setCmHost(el: any, path: string) {
  cmHosts.set(path, el)
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
    const items: any[] = data.items || []
    const children: TreeNode[] = items
      .filter((i) => i.is_dir)
      .sort((a, b) => a.name.localeCompare(b.name, 'zh'))
      .map((i) => ({ name: i.name, path: i.path, is_dir: true, loaded: false, children: [], expanded: openPaths.has(i.path) }))
    const files: TreeNode[] = items
      .filter((i) => !i.is_dir)
      .sort((a, b) => a.name.localeCompare(b.name, 'zh'))
      .map((i) => ({ name: i.name, path: i.path, is_dir: false, loaded: true, children: [], expanded: false }))
    setTreeNodes(path, [...children, ...files])
  } catch (e: any) {
    ElMessage.error(e?.message || '目录加载失败')
  } finally {
    treeLoading.value = false
  }
}

function setTreeNodes(path: string, children: TreeNode[]) {
  if (path === rootPath) {
    treeNodes.value = [{ name: rootPath, path: rootPath, is_dir: true, loaded: true, expanded: true, children }]
    return
  }
  const node = findNode(treeNodes.value, path)
  if (node) {
    node.children = children
    node.loaded = true
  }
}

function findNode(nodes: TreeNode[], path: string): TreeNode | null {
  for (const n of nodes) {
    if (n.path === path) return n
    if (n.children?.length) {
      const found = findNode(n.children, path)
      if (found) return found
    }
  }
  return null
}

async function onToggleNode(node: TreeNode) {
  if (!node.is_dir) {
    openFile(node.path, node.name)
    return
  }
  node.expanded = !node.expanded
  if (node.expanded) {
    openPaths.add(node.path)
    if (!node.loaded || !node.children?.length) await loadTree(node.path)
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
  activePath.value = path
  await loadFileContent(tab)
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

function onOpen() {
  const theme = document.documentElement.classList.contains('dark') ||
    getComputedStyle(document.body).backgroundColor.includes('rgb(0, 0, 0)')
  isDark.value = !!theme
  loadTree(rootPath)
  if (props.initialFile) {
    const name = props.initialFile.split('/').filter(Boolean).pop() || props.initialFile
    openFile(props.initialFile, name)
  }
  window.addEventListener('keydown', onKeydown)
}

function onClosed() {
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
}

watch(() => props.modelValue, (v) => {
  if (v) onOpen()
})
</script>

<style scoped>
.ed-body {
  display: flex;
  height: calc(100vh - 54px);
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
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  font-size: 12px;
  font-weight: 600;
  border-bottom: 1px solid var(--el-border-color-lighter);
  flex-shrink: 0;
}

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

.ed-header { display: flex; align-items: center; width: 100%; }
.ed-title { font-weight: 600; font-size: 14px; }
.ed-header-spacer { flex: 1; }
.ed-tabs-spacer { flex: 1; }
</style>

<style>
.editor-dialog .el-dialog__header {
  margin: 0;
  padding: 10px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}
.editor-dialog .el-dialog__body {
  padding: 0;
}
</style>
