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
                    <el-dropdown v-if="tab.selectedRows.length" @command="(cmd: string) => handleToolbar(cmd, tab)">
                        <el-button size="small">
                            更多 <el-icon class="el-icon--right"><ArrowDown /></el-icon>
                        </el-button>
                        <template #dropdown>
                            <el-dropdown-menu>
                                <el-dropdown-item command="copy">复制</el-dropdown-item>
                                <el-dropdown-item command="cut">剪切</el-dropdown-item>
                                <el-dropdown-item command="compress">压缩</el-dropdown-item>
                                <el-dropdown-item command="chmod">权限</el-dropdown-item>
                                <el-dropdown-item command="delete" divided>删除</el-dropdown-item>
                            </el-dropdown-menu>
                        </template>
                    </el-dropdown>
                    <el-button v-if="clipboard.paths.length" size="small" type="warning" @click="toolbarPaste(tab)">粘贴</el-button>
                </div>
                <el-table
                    v-loading="tab.loading"
                    :data="tab.files"
                    ref="fileTableRef"

                    highlight-current-row
                    @current-change="(row: FileItem | null) => tab.selectedFile = row"
                    @selection-change="(rows: FileItem[]) => tab.selectedRows = rows"
                    @sort-change="onSortChange(tab, $event)"
                    :default-sort="{ prop: 'name', order: 'ascending' }"
                    size="small"
                    class="file-table"
                    empty-text="暂无文件"
                    :cell-style="{ padding: '4px 0' }"
                >
                    <el-table-column type="selection" width="40" />
                    <el-table-column
                        label="名称"
                        width="300"
                        :show-overflow-tooltip="true"
                        sortable="custom"
                        :sort-orders="['ascending', 'descending']"
                        prop="name"
                    >
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
                                <span class="file-name-text">{{ row.name }}</span>
                                <template v-if="row.is_link && row.link_target">
                                    <span class="link-arrow"> -> </span>
                                    <span class="link-target">{{ row.link_target }}</span>
                                </template>
                            </span>
                        </template>
                    </el-table-column>
                    <el-table-column label="权限/所有者" width="120">
                        <template #default="{ row }">{{ row.mode }}<template v-if="row.owner"> / {{ row.owner }}</template></template>
                    </el-table-column>
                    <el-table-column
                        label="大小"
                        width="90"
                        sortable="custom"
                        :sort-orders="['ascending', 'descending']"
                        prop="sort_size"
                        :sort-method="sortBySize"
                    >
                        <template #default="{ row }">
                            <template v-if="row.is_dir">
                                <el-icon v-if="row._calculating" class="is-loading" size="14"><Loading /></el-icon>
                                <el-button v-else-if="row._size === undefined" size="small" link type="primary" @click="calcDirSize(tab, row)">计算</el-button>
                                <span v-else>{{ formatSize(row._size, false) }}</span>
                            </template>
                            <span v-else>{{ formatSize(row.size, false) }}</span>
                        </template>
                    </el-table-column>
                    <el-table-column
                        label="修改时间"
                        width="150"
                        sortable="custom"
                        :sort-orders="['ascending', 'descending']"
                        prop="modified"
                    >
                        <template #default="{ row }">{{ formatTime(row.modified) }}</template>
                    </el-table-column>
                    <el-table-column label="备注" min-width="160">
                        <template #default="{ row }">
                            <el-input v-model="row.ps" size="small" class="ps-input" @blur="savePs(row, tab)" />
                        </template>
                    </el-table-column>
                </el-table>
            </div>
        </template>

        <!-- Context menus -->
        <Teleport to="body">
            <div v-if="ctxMenu.visible" ref="ctxMenuRef" class="ctx-menu" :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }" @click="ctxMenu.visible = false">
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
                    <div class="ctx-item" @click="ctxChmod">权限</div>
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
                    <div class="ctx-item" @click="ctxChmod">权限</div>
                    <div class="ctx-divider" />
                    <div class="ctx-item" @click="ctxCopy(ctxMenu.filePath!)">复制</div>
                    <div class="ctx-item" @click="ctxCut(ctxMenu.filePath!)">剪切</div>
                    <div v-if="clipboard.paths.length" class="ctx-item" @click="ctxPaste">粘贴</div>
                    <div class="ctx-divider" />
                    <div class="ctx-item" @click="ctxRename(ctxMenu.filePath!)">重命名</div>
                    <div class="ctx-item" @click="ctxDelete(ctxMenu.filePath!, ctxMenu.fileName!)">删除</div>
                    <div class="ctx-divider" />
                    <div class="ctx-item disabled">创建压缩</div>
                    <div v-if="ctxMenu.fileName?.endsWith('.tar.gz')" class="ctx-item" @click="openExtractDialog(ctxMenu.filePath!, ctxMenu.fileName!)">解压</div>
                    <div class="ctx-divider" />
                    <div class="ctx-item" @click="ctxStat">属性</div>
                </template>
            </div>
        </Teleport>

        <el-dialog v-model="createDialog.visible" class="file-create-dialog" :title="createDialog.isDir ? '新建目录' : '新建文件'" width="400px" append-to-body @opened="focusCreateInput">
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
            <p>确定要删除选中的 <strong>{{ deleteDialog.items.length }}</strong> 个文件/目录吗？</p>
            <p style="font-size:12px;color:var(--el-text-color-secondary);margin-top:4px;max-height:120px;overflow-y:auto;">
                {{ deleteDialog.items.map(i => i.name).join('、') }}
            </p>
            <p v-if="deleteDialog.items.some(i => i.is_dir)" style="color:#e6a23c;font-size:12px;margin-top:4px">目录将递归删除所有内容，此操作不可恢复。</p>
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

        <el-dialog v-model="compressDialog.visible" title="压缩文件" width="500px" append-to-body>
            <el-form label-width="80px">
                <el-form-item label="压缩类型">
                    <el-select v-model="compressDialog.type" style="width:100%">
                        <el-option label="tar.gz(推荐)" value="tar.gz" />
                    </el-select>
                </el-form-item>
                <el-form-item label="压缩路径">
                    <div style="display:flex;gap:4px;width:100%">
                        <el-input v-model="compressDialog.path" readonly />
                        <el-button @click="openCompressDirPicker">浏览</el-button>
                    </div>
                </el-form-item>
            </el-form>
            <template #footer>
                <el-button @click="compressDialog.visible = false">取消</el-button>
                <el-button type="primary" @click="handleCompress" :loading="compressDialog.loading">压缩</el-button>
            </template>
        </el-dialog>

        <el-dialog v-model="extractDialog.visible" :title="`解压文件[${extractDialog.fileName}]`" width="500px" append-to-body>
            <el-form label-width="80px">
                <el-form-item label="文件名">
                    <el-input :model-value="extractDialog.fileName" readonly />
                </el-form-item>
                <el-form-item label="解压到">
                    <div style="display:flex;gap:4px;width:100%">
                        <el-input v-model="extractDialog.dest" />
                        <el-button @click="openExtractDirPicker">浏览</el-button>
                    </div>
                </el-form-item>
                <el-form-item label="解压密码">
                    <el-input v-model="extractDialog.password" placeholder="无密码则留空" />
                </el-form-item>
            </el-form>
            <template #footer>
                <el-button @click="extractDialog.visible = false">取消</el-button>
                <el-button type="primary" @click="handleExtract" :loading="extractDialog.loading">解压</el-button>
            </template>
        </el-dialog>

        <el-dialog v-model="chmodDialog.visible" :title="chmodDialog.isBatch ? '设置权限-批量' : '设置权限'" width="420px" append-to-body>
            <el-form label-width="70px">
                <el-form-item label="所有者" style="margin-bottom: 10px">
                    <div class="perm-row">
                        <el-checkbox v-model="chmodDialog.owner.r" @change="syncMode">读取</el-checkbox>
                        <el-checkbox v-model="chmodDialog.owner.w" @change="syncMode">写入</el-checkbox>
                        <el-checkbox v-model="chmodDialog.owner.x" @change="syncMode">执行</el-checkbox>
                    </div>
                </el-form-item>
                <el-form-item label="用户组" style="margin-bottom: 10px">
                    <div class="perm-row">
                        <el-checkbox v-model="chmodDialog.group.r" @change="syncMode">读取</el-checkbox>
                        <el-checkbox v-model="chmodDialog.group.w" @change="syncMode">写入</el-checkbox>
                        <el-checkbox v-model="chmodDialog.group.x" @change="syncMode">执行</el-checkbox>
                    </div>
                </el-form-item>
                <el-form-item label="公共" style="margin-bottom: 10px">
                    <div class="perm-row">
                        <el-checkbox v-model="chmodDialog.other.r" @change="syncMode">读取</el-checkbox>
                        <el-checkbox v-model="chmodDialog.other.w" @change="syncMode">写入</el-checkbox>
                        <el-checkbox v-model="chmodDialog.other.x" @change="syncMode">执行</el-checkbox>
                    </div>
                </el-form-item>
                <el-form-item label="权限">
                    <el-input v-model="chmodDialog.mode" style="width: 90px" @input="syncChecks" />
                </el-form-item>
                <el-form-item label="所有者">
                    <el-select v-model="chmodDialog.ownerName" style="width: 160px">
                        <el-option v-for="u in userList" :key="u" :label="u" :value="u" />
                    </el-select>
                </el-form-item>
                <el-form-item v-if="chmodDialog.isDir" label="应用到">
                    <el-checkbox v-model="chmodDialog.recursive">应用到子目录</el-checkbox>
                </el-form-item>
            </el-form>
            <template #footer>
                <el-button @click="chmodDialog.visible = false">取消</el-button>
                <el-button type="primary" @click="handleChmod" :loading="chmodDialog.loading">确定</el-button>
            </template>
        </el-dialog>

        <el-dialog v-model="statDialog.visible" :title="`文件属性[${statDialog.name}]`" width="400px" append-to-body>
            <div v-loading="statDialog.loading" class="stat-box">
                <div class="stat-row">
                    <span class="stat-label">文件类型</span>
                    <span>{{ statDialog.file_type }}</span>
                </div>
                <div class="stat-row">
                    <span class="stat-label">位置</span>
                    <span class="stat-value">{{ statLocation }}</span>
                    <el-icon class="stat-copy" @click="copyText(statLocation)"><CopyDocument /></el-icon>
                </div>
                <div class="stat-row">
                    <span class="stat-label">路径</span>
                    <span class="stat-value">{{ statDialog.path }}</span>
                    <el-icon class="stat-copy" @click="copyText(statDialog.path)"><CopyDocument /></el-icon>
                </div>
                <div class="stat-row">
                    <span class="stat-label">大小</span>
                    <span>{{ statDialog.size_str }}</span>
                </div>
                <el-divider style="margin: 8px 0" />
                <div class="stat-row">
                    <span class="stat-label">MD5</span>
                    <span class="stat-vc"><span class="stat-value stat-hash">{{ statDialog.md5 || '-' }}</span><el-icon class="stat-copy" @click="copyText(statDialog.md5)"><CopyDocument /></el-icon></span>
                </div>
                <div class="stat-row">
                    <span class="stat-label">SHA1</span>
                    <span class="stat-vc"><span class="stat-value stat-hash">{{ statDialog.sha1 || '-' }}</span><el-icon class="stat-copy" @click="copyText(statDialog.sha1)"><CopyDocument /></el-icon></span>
                </div>
                <div class="stat-row">
                    <span class="stat-label">SHA256</span>
                    <span class="stat-vc"><span class="stat-value stat-hash">{{ statDialog.sha256 || '-' }}</span><el-icon class="stat-copy" @click="copyText(statDialog.sha256)"><CopyDocument /></el-icon></span>
                </div>
                <el-divider style="margin: 8px 0" />
                <div class="stat-row">
                    <span class="stat-label">创建时间</span>
                    <span>{{ formatTime(statDialog.created) }}</span>
                </div>
                <div class="stat-row">
                    <span class="stat-label">修改时间</span>
                    <span>{{ formatTime(statDialog.modified) }}</span>
                </div>
                <div class="stat-row">
                    <span class="stat-label">访问时间</span>
                    <span>{{ formatTime(statDialog.accessed) }}</span>
                </div>
            </div>
            <template #footer>
                <el-button @click="statDialog.visible = false">关闭</el-button>
            </template>
        </el-dialog>

        <DirPicker v-model="dirPickerVisible" :initial-path="dirPickerInitial" @confirm="dirPickerConfirm" />

        <FileEditorDialog v-model="editorDialog.visible" :root-path="editorDialog.rootPath" :initial-file="editorDialog.file" />
    </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import { FolderOpened, Document, Link, Search, Close, Plus, Back, RefreshRight, Loading, ArrowDown, CopyDocument } from '@element-plus/icons-vue'
import { apiFetch, authHeaders, checkRes401 } from '@/utils/api'
import FileEditorDialog from '@/components/FileEditorDialog.vue'
import DirPicker from '@/components/DirPicker.vue'

interface FileItem {
    name: string
    path: string
    size: number
    is_dir: boolean
    is_link: boolean
    link_target: string
    mode: string
    owner: string
    modified: number
    ps: string
    _size?: number
    _calculating?: boolean
}

const DEFAULT_PS: Record<string, string> = {
    '/www': 'PS: Alpanel面板程序目录',
    '/www/wwwlogs': 'PS: 网站日志目录',
    '/www/server': 'PS: Alpanel软件安装目录',
    '/www/server/stop': '网站停用页面目录,请勿删除!',
    '/www/server/mysql': 'MySQL程序目录',
    '/www/server/nginx': 'Nginx程序目录',
    '/www/server/php': 'PHP目录',
    '/www/server/redis': 'Redis程序目录',
    '/www/server/cron': '计划任务脚本与日志目录',
    '/www/server/data': 'MySQL数据目录',
    '/www/server/panel': 'PS: Alpanel主程序目录',
}

interface BrowserTab {
    id: string
    title: string
    type: 'browser'
    path: string
    files: FileItem[]
    loading: boolean
    selectedFile: FileItem | null
    selectedRows: FileItem[]
    sortProp: 'name' | 'sort_size' | 'modified'
    sortOrder: 'ascending' | 'descending'
}

type Tab = BrowserTab

const tabs = ref<Tab[]>([])
const activeTab = ref('')
const pathInput = ref('/')
const route = useRoute()
const renamingPath = ref('')
const renamingValue = ref('')
const renamingTab = ref<BrowserTab | null>(null)
const fileTableRef = ref()

const ctxMenu = reactive({
    visible: false,
    x: 0,
    y: 0,
    type: '' as '' | 'blank' | 'dir' | 'file',
    tab: null as BrowserTab | null,
    filePath: '',
    fileName: '',
})

const ctxMenuRef = ref<HTMLElement | null>(null)

const clipboard = reactive({
    paths: [] as string[],
    cut: false,
})

const STORAGE_KEY = 'alpanel_file_tabs'

interface StoredTab {
    id: string
    title: string
    type: 'browser'
    path: string
}

function saveTabs() {
    const data: StoredTab[] = tabs.value.map(t => ({
        id: t.id,
        title: t.title,
        type: t.type,
        path: t.path,
    }))
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ tabs: data, activeTab: activeTab.value, tabIdSeq }))
}

function restoreTabs(): Promise<boolean> {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return Promise.resolve(false)
    try {
        const saved = JSON.parse(raw)
        if (!saved.tabs?.length) return Promise.resolve(false)
        tabIdSeq = saved.tabIdSeq || 0
        const rest: Tab[] = saved.tabs.map((st: StoredTab) => {
            return { id: st.id, title: st.title, type: 'browser' as const, path: st.path, files: [], loading: false, selectedFile: null, selectedRows: [], sortProp: 'name', sortOrder: 'ascending' }
        })
        // fetch data first, then assign to tabs.value so Vue tracks from the start
        const browserTabs = rest.filter((t): t is BrowserTab => t.type === 'browser')
        return Promise.all(browserTabs.map(t =>
            apiFetch(`/api/files/list?path=${encodeURIComponent(t.path)}`).then(data => {
                if (data?.path) t.path = data.path
                t.title = t.path === '/' ? '根目录' : t.path.split('/').filter(Boolean).pop() || '根目录'
                t.files = data?.items || []
            }).catch(() => { t.files = [] })
        )).then(() => {
            tabs.value = rest
            activeTab.value = saved.activeTab || rest[0]?.id || ''
            pathInput.value = (rest.find(t => t.id === activeTab.value) as BrowserTab)?.path || '/'
            return true
        })
    } catch { return Promise.resolve(false) }
}

function closeCtxMenu() {
    ctxMenu.visible = false
}

watch([tabs, activeTab], () => { saveTabs() }, { deep: true })

onMounted(async () => {
    document.addEventListener('click', closeCtxMenu)
    const pathQ = route.query.path as string | undefined
    const restored = await restoreTabs()
    if (!restored) addBrowserTab()
    // handle query param after restore
    if (pathQ) {
        addBrowserTabAt(pathQ)
        // clean query to avoid re-process on re-mount
        window.history.replaceState(null, '', '/file')
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
            const row = tab.files[idx]
            try {
                const table = fileTableRef.value?.[0]
                if (table) {
                    table.clearSelection()
                    table.toggleRowSelection(row, true)
                }
            } catch {}
            onRowContextMenu(e, tab, row)
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
    adjustCtxMenu()
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
    adjustCtxMenu()
}

function adjustCtxMenu() {
    nextTick(() => {
        const el = ctxMenuRef.value
        if (!el) return
        const rect = el.getBoundingClientRect()
        const margin = 4
        const vw = window.innerWidth
        const vh = window.innerHeight
        let x = ctxMenu.x
        let y = ctxMenu.y
        if (y + rect.height > vh - margin) {
            y = Math.max(margin, vh - rect.height - margin)
        }
        if (x + rect.width > vw - margin) {
            x = Math.max(margin, vw - rect.width - margin)
        }
        ctxMenu.x = x
        ctxMenu.y = y
    })
}

function ctxCopy(path: string) {
    clipboard.paths = [path]
    clipboard.cut = false
}

function ctxCut(path: string) {
    clipboard.paths = [path]
    clipboard.cut = true
}

function toolbarCopy(tab: BrowserTab) {
    clipboard.paths = tab.selectedRows.map(r => r.path)
    clipboard.cut = false
    ElMessage.success(`已复制 ${clipboard.paths.length} 项`)
}

function toolbarCut(tab: BrowserTab) {
    clipboard.paths = tab.selectedRows.map(r => r.path)
    clipboard.cut = true
    ElMessage.success(`已剪切 ${clipboard.paths.length} 项`)
}

function handleToolbar(cmd: string, tab: BrowserTab) {
    if (cmd === 'copy') toolbarCopy(tab)
    else if (cmd === 'cut') toolbarCut(tab)
    else if (cmd === 'compress') openCompressDialog(tab)
    else if (cmd === 'chmod') openBatchChmod(tab)
    else if (cmd === 'delete') confirmDelete(tab)
}

async function toolbarPaste(tab: BrowserTab) {
    if (!clipboard.paths.length) return
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
    deleteDialog.items = [{ name, path, is_dir: ctxMenu.type === 'dir' }]
    deleteDialog.targetTab = ctxMenu.tab
    deleteDialog.visible = true
}

function ctxOpenEditor() {
    if (ctxMenu.type !== 'file' || !ctxMenu.filePath) return
    editorDialog.rootPath = ctxMenu.tab?.path || '/www'
    editorDialog.file = ctxMenu.filePath
    editorDialog.visible = true
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
        selectedRows: [],
        sortProp: 'name',
        sortOrder: 'ascending',
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

const createDialog = reactive({
    visible: false,
    name: '',
    isDir: false,
    targetTab: null as BrowserTab | null,
})

const deleteDialog = reactive({
    visible: false,
    items: [] as { name: string; path: string; is_dir: boolean }[],
    targetTab: null as BrowserTab | null,
})

const downloadDialog = reactive({
    visible: false,
    url: '',
    path: '',
    loading: false,
})

const compressDialog = reactive({
    visible: false,
    type: 'tar.gz',
    path: '',
    loading: false,
    tab: null as BrowserTab | null,
})

const extractDialog = reactive({
    visible: false,
    filePath: '',
    fileName: '',
    dest: '',
    password: '',
    loading: false,
})

interface PermBits { r: boolean; w: boolean; x: boolean }
interface PermSet { owner: PermBits; group: PermBits; other: PermBits }

const chmodDialog = reactive({
    visible: false,
    isBatch: false,
    isDir: false,
    recursive: false,
    loading: false,
    paths: [] as string[],
    ownerName: 'root',
    mode: '644',
    owner: { r: true, w: true, x: false } as PermBits,
    group: { r: true, w: true, x: false } as PermBits,
    other: { r: true, w: true, x: false } as PermBits,
})

const userList = ref<string[]>([])

function parseMode(mode: string): PermSet {
    const m = parseInt(mode, 8) || 0
    const bit = (shift: number) => ((m >> shift) & 1) === 1
    return {
        owner: { r: bit(8), w: bit(7), x: bit(6) },
        group: { r: bit(5), w: bit(4), x: bit(3) },
        other: { r: bit(2), w: bit(1), x: bit(0) },
    }
}

function modeFromBits(ps: PermSet): string {
    const v = (b: PermBits) => (b.r ? 4 : 0) + (b.w ? 2 : 0) + (b.x ? 1 : 0)
    return '' + v(ps.owner) + v(ps.group) + v(ps.other)
}

function syncMode() {
    chmodDialog.mode = modeFromBits(chmodDialog)
}

function syncChecks() {
    const bits = parseMode(chmodDialog.mode)
    chmodDialog.owner = bits.owner
    chmodDialog.group = bits.group
    chmodDialog.other = bits.other
}

function openChmod(paths: string[], isDir: boolean, isBatch: boolean, mode: string, ownerName: string) {
    chmodDialog.paths = paths
    chmodDialog.isDir = isDir
    chmodDialog.isBatch = isBatch
    chmodDialog.recursive = false
    chmodDialog.mode = mode || '644'
    chmodDialog.ownerName = ownerName || 'root'
    chmodDialog.visible = true
    syncChecks()
    if (!userList.value.length) loadUsers()
}

async function loadUsers() {
    try {
        const data = await apiFetch('/api/system/users')
        if (Array.isArray(data)) userList.value = data
    } catch {}
}

function ctxChmod() {
    if (!ctxMenu.filePath) return
    openChmod([ctxMenu.filePath], ctxMenu.type === 'dir', false, ctxMenu.tab?.files.find(f => f.path === ctxMenu.filePath)?.mode || '644', ctxMenu.tab?.files.find(f => f.path === ctxMenu.filePath)?.owner || 'root')
}

function openBatchChmod(tab: BrowserTab) {
    if (!tab.selectedRows.length) return
    const anyDir = tab.selectedRows.some(r => r.is_dir)
    openChmod(
        tab.selectedRows.map(r => r.path),
        anyDir,
        true,
        tab.selectedRows[0]?.mode || '644',
        tab.selectedRows[0]?.owner || 'root',
    )
}

async function handleChmod() {
    if (!chmodDialog.paths.length) return
    chmodDialog.loading = true
    try {
        await apiFetch('/api/files/chmod', {
            method: 'POST',
            body: JSON.stringify({
                paths: chmodDialog.paths,
                mode: chmodDialog.mode,
                owner: chmodDialog.ownerName || undefined,
                recursive: chmodDialog.recursive,
            }),
        })
        ElMessage.success(chmodDialog.isBatch ? '批量修改权限完成' : '权限已修改')
        chmodDialog.visible = false
        const tab = tabs.value.find(t => t.id === activeTab.value && t.type === 'browser') as BrowserTab | undefined
        if (tab) fetchTabList(tab)
    } catch (e: any) {
        ElMessage.error(e?.message || '修改权限失败')
    } finally {
        chmodDialog.loading = false
    }
}

const statDialog = reactive({
    visible: false,
    loading: false,
    name: '',
    path: '',
    file_type: '',
    size_str: '',
    md5: '',
    sha1: '',
    sha256: '',
    is_dir: false,
    created: 0,
    modified: 0,
    accessed: 0,
})

function ctxStat() {
    if (!ctxMenu.filePath) return
    statDialog.visible = true
    statDialog.loading = true
    apiFetch(`/api/files/stat?path=${encodeURIComponent(ctxMenu.filePath)}`)
        .then((data: any) => {
            statDialog.name = data?.name || ''
            statDialog.path = data?.path || ''
            statDialog.file_type = data?.file_type || ''
            statDialog.size_str = data?.size_str || ''
            statDialog.md5 = data?.md5 || ''
            statDialog.sha1 = data?.sha1 || ''
            statDialog.sha256 = data?.sha256 || ''
            statDialog.is_dir = !!data?.is_dir
            statDialog.created = data?.created || 0
            statDialog.modified = data?.modified || 0
            statDialog.accessed = data?.accessed || 0
        })
        .catch((e: any) => {
            ElMessage.error(e?.message || '获取文件属性失败')
            statDialog.visible = false
        })
        .finally(() => {
            statDialog.loading = false
        })
}

function copyText(text: string) {
    if (!text) return
    navigator.clipboard.writeText(text).then(() => {
        ElMessage.success('已复制')
    }).catch(() => {
        ElMessage.error('复制失败')
    })
}

const statLocation = computed(() => {
    const p = statDialog.path.replace(/\\/g, '/')
    const idx = p.lastIndexOf('/')
    return idx <= 0 ? '/' : p.substring(0, idx)
})

const dirPickerVisible = ref(false)
const dirPickerInitial = ref('/')
const dirPickerTarget = ref<'extract' | 'compress'>('compress')



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
        selectedRows: [],
        sortProp: 'name',
        sortOrder: 'ascending',
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
        selectedRows: [],
        sortProp: 'name',
        sortOrder: 'ascending',
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

function sortBySize(a: FileItem, b: FileItem): number {
    const sa = a.is_dir ? (a._size ?? -1) : a.size
    const sb = b.is_dir ? (b._size ?? -1) : b.size
    return sa - sb
}

function applySort(tab: BrowserTab) {
    const files = [...tab.files]
    const dirs = files.filter(f => f.is_dir)
    const others = files.filter(f => !f.is_dir)
    const cmp: (a: FileItem, b: FileItem) => number = (a, b) => {
        if (tab.sortProp === 'sort_size') {
            return sortBySize(a, b)
        }
        if (tab.sortProp === 'modified') {
            return a.modified - b.modified
        }
        return a.name.localeCompare(b.name, 'zh-Hans-CN')
    }
    const factor = tab.sortOrder === 'descending' ? -1 : 1
    dirs.sort((a, b) => cmp(a, b) * factor)
    others.sort((a, b) => cmp(a, b) * factor)
    tab.files = [...dirs, ...others]
}

function onSortChange(tab: BrowserTab, e: { prop?: string; order?: 'ascending' | 'descending' | null }) {
    if (!e.prop || !e.order) return
    tab.sortProp = e.prop as BrowserTab['sortProp']
    tab.sortOrder = e.order
    applySort(tab)
}

async function fetchTabList(tab: BrowserTab) {
    tab.loading = true
    try {
        const data = await apiFetch(`/api/files/list?path=${encodeURIComponent(tab.path)}`)
        if (data?.path) tab.path = data.path
        tab.title = tab.path === '/' ? '根目录' : tab.path.split('/').filter(Boolean).pop() || '根目录'
        const items = data?.items || []
        items.forEach((item: FileItem) => {
            if (!item.ps && DEFAULT_PS[item.path]) {
                item.ps = DEFAULT_PS[item.path]
            }
        })
        tab.files = items
        applySort(tab)
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
    } else {
        editorDialog.rootPath = tab.path
        editorDialog.file = row.path
        editorDialog.visible = true
    }
}

const editorDialog = reactive({
    visible: false,
    file: '',
    rootPath: '/www',
})

function openCreate(tab: BrowserTab, isDir: boolean) {
    createDialog.name = ''
    createDialog.isDir = isDir
    createDialog.targetTab = tab
    createDialog.visible = true
}

function focusCreateInput() {
    setTimeout(() => {
        const el = document.querySelector<HTMLInputElement>('.file-create-dialog .el-input__inner')
        if (el) el.focus()
    }, 50)
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
    if (!tab.selectedRows.length) return
    deleteDialog.items = tab.selectedRows.map(r => ({ name: r.name, path: r.path, is_dir: r.is_dir }))
    deleteDialog.targetTab = tab
    deleteDialog.visible = true
}

async function handleDelete() {
    const tab = deleteDialog.targetTab
    if (!tab) return
    try {
        for (const item of deleteDialog.items) {
            await apiFetch('/api/files/delete', {
                method: 'POST',
                body: JSON.stringify({ path: item.path }),
            })
        }
        ElMessage.success('已删除')
        deleteDialog.visible = false
        tab.selectedRows = []
        fetchTabList(tab)
    } catch (e: any) {
        ElMessage.error(e?.message || '删除失败')
    }
}

function genRandomSuffix(): string {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'
    let result = ''
    for (let i = 0; i < 4; i++) {
        result += chars.charAt(Math.floor(Math.random() * chars.length))
    }
    return result
}

function openCompressDialog(tab: BrowserTab) {
    if (!tab.selectedRows.length) return
    const names = tab.selectedRows.map(r => r.name)
    const dirName = names.length === 1 ? names[0] : tab.path.split('/').filter(Boolean).pop() || 'archive'
    const suffix = genRandomSuffix()
    compressDialog.type = 'tar.gz'
    compressDialog.path = `${tab.path}/${dirName}_${suffix}.tar.gz`
    compressDialog.tab = tab
    compressDialog.visible = true
}

async function handleCompress() {
    const tab = compressDialog.tab
    if (!tab || !compressDialog.path) return
    compressDialog.loading = true
    try {
        const paths = tab.selectedRows.map(r => r.path)
        await apiFetch('/api/files/compress', {
            method: 'POST',
            body: JSON.stringify({ paths, dest: compressDialog.path, type: compressDialog.type }),
        })
        ElMessage.success('压缩任务已提交')
        compressDialog.visible = false
        fetchTabList(tab)
    } catch (e: any) {
        ElMessage.error(e?.message || '压缩失败')
    } finally {
        compressDialog.loading = false
    }
}

function openExtractDialog(filePath: string, fileName: string) {
    extractDialog.filePath = filePath
    extractDialog.fileName = fileName
    const parts = filePath.split('/')
    parts.pop()
    extractDialog.dest = parts.join('/') || '/'
    extractDialog.password = ''
    extractDialog.visible = true
}

function openExtractDirPicker() {
    dirPickerTarget.value = 'extract'
    dirPickerInitial.value = extractDialog.dest || '/'
    dirPickerVisible.value = true
}

async function handleExtract() {
    if (!extractDialog.filePath || !extractDialog.dest) return
    extractDialog.loading = true
    try {
        await apiFetch('/api/files/extract', {
            method: 'POST',
            body: JSON.stringify({
                path: extractDialog.filePath,
                dest: extractDialog.dest,
                password: extractDialog.password || undefined,
            }),
        })
        ElMessage.success('解压完成')
        extractDialog.visible = false
        const tab = tabs.value.find(t => t.id === activeTab.value && t.type === 'browser') as BrowserTab | undefined
        if (tab) fetchTabList(tab)
    } catch (e: any) {
        ElMessage.error(e?.message || '解压失败')
    } finally {
        extractDialog.loading = false
    }
}

function openCompressDirPicker() {
    const parts = compressDialog.path.split('/')
    parts.pop()
    dirPickerTarget.value = 'compress'
    dirPickerInitial.value = parts.join('/') || '/'
    dirPickerVisible.value = true
}

function dirPickerConfirm(dir: string) {
    if (dirPickerTarget.value === 'extract') {
        extractDialog.dest = dir
    } else {
        const oldParts = compressDialog.path.split('/')
        const fileName = oldParts.pop() || 'archive.tar.gz'
        compressDialog.path = dir + fileName
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

async function savePs(row: FileItem, _tab: BrowserTab) {
    try {
        const ps = row.ps || ''
        await apiFetch('/api/files/ps', {
            method: 'POST',
            body: JSON.stringify({ path: row.path, ps }),
        })
    } catch (e: any) {
        ElMessage.error(e?.message || '保存备注失败')
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

.file-name .file-name-text {
    font-weight: 500;
}

.file-name .link-arrow {
    color: var(--el-text-color-secondary);
    margin: 0 2px;
}

.file-name .link-target {
    color: var(--el-text-color-secondary);
    font-size: 11px;
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

.perm-row {
    display: flex;
    gap: 16px;
}

.stat-box {
    min-height: 120px;
}

.stat-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 3px 0;
    font-size: 12px;
}

.stat-label {
    width: 70px;
    flex-shrink: 0;
    color: var(--el-text-color-secondary);
}

.stat-value {
    flex: 0 1 auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.stat-vc {
    flex: 1;
    min-width: 0;
    display: inline-block;
    vertical-align: top;
}

.stat-vc .stat-value {
    display: inline;
    white-space: normal;
    word-break: break-all;
    overflow: visible;
    text-overflow: clip;
}

.stat-copy {
    cursor: pointer;
    color: var(--el-text-color-secondary);
    flex-shrink: 0;
    display: inline-flex;
    vertical-align: middle;
    margin-left: 4px;
}

.stat-copy:hover {
    color: var(--el-color-primary);
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
