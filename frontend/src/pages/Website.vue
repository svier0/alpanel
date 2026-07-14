<template>
  <div class="website-manager">
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
          <el-button size="small" type="primary">
            <el-icon><Plus /></el-icon>添加站点
          </el-button>
          <el-dropdown size="small" trigger="click">
            <el-button size="small">
              Nginx<el-icon class="el-icon--right"><ArrowDown /></el-icon>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item>停止</el-dropdown-item>
                <el-dropdown-item>重启</el-dropdown-item>
                <el-dropdown-item>重载</el-dropdown-item>
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

      <template v-if="activeTab === 'normal'">
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
            <template #default="{ row }"><span class="link-cell" @click="">{{ row.root }}</span></template>
          </el-table-column>
          <el-table-column label="备注" width="160">
            <template #default="{ row }">
              <el-input v-model="row.ps" size="small" class="ps-input" @blur="savePs(row, 'normal')" />
            </template>
          </el-table-column>
          <el-table-column label="PHP版本" width="100">
            <template #default="{ row }"><span class="link-cell" @click="">{{ row.php ?? '-' }}</span></template>
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
          :total="filteredNormal.length"
          :page-size="pageSize"
          v-model:current-page="page"
          class="site-pagination"
        />
      </template>

      <template v-if="activeTab === 'other'">
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
            <template #default="{ row }"><span class="link-cell" @click="">{{ row.root }}</span></template>
          </el-table-column>
          <el-table-column label="备注" width="160">
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
            <template #default>
              <el-button size="small" link type="primary">设置</el-button>
              <el-button size="small" link type="primary">删除</el-button>
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

      <template v-if="activeTab === 'proxy'">
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
          <el-table-column label="备注" width="160">
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Plus, ArrowDown, Search, RefreshRight } from '@element-plus/icons-vue'

const tabs = [
  { key: 'normal', label: '普通项目' },
  { key: 'other', label: '其它项目' },
  { key: 'proxy', label: '反向代理' },
]

const activeTab = ref('normal')
const searchQuery = ref('')

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

const normalList = ref<NormalItem[]>([
  { id: 1, name: 'example.com', status: '运行中', root: '/www/wwwroot/example.com', ps: '公司官网', php: '82', ssl: true, sslDays: 31 },
  { id: 2, name: 'blog.example.com', status: '运行中', root: '/www/wwwroot/blog.example.com', ps: '个人博客', php: '74', ssl: true, sslDays: 15 },
  { id: 3, name: 'test.example.com', status: '停止', root: '/www/wwwroot/test.example.com', ps: '', php: '82', ssl: false, sslDays: 0 },
  { id: 4, name: 'admin.example.com', status: '运行中', root: '/www/wwwroot/admin.example.com', ps: '后台管理', php: '74', ssl: true, sslDays: 89 },
])

const otherList = ref<OtherItem[]>([
  { id: 1, name: 'node-app', status: '运行中', port: 3001, root: '/www/wwwroot/node-app', ps: 'Node.js 服务', ssl: false, sslDays: 0 },
  { id: 2, name: 'python-api', status: '运行中', port: 5000, root: '/www/wwwroot/python-api', ps: '', ssl: false, sslDays: 0 },
  { id: 3, name: 'go-service', status: '停止', port: 8080, root: '/www/wwwroot/go-service', ps: 'Go 后端', ssl: true, sslDays: 120 },
])

const proxyList = ref<ProxyItem[]>([
  { id: 1, domain: 'api.example.com', status: '运行中', proxyPass: 'http://localhost:8080', ps: 'API 代理', ssl: true, sslDays: 45 },
  { id: 2, domain: 'ws.example.com', status: '运行中', proxyPass: 'http://localhost:3000', ps: '', ssl: true, sslDays: 7 },
  { id: 3, domain: 'cdn.example.com', status: '停止', proxyPass: 'http://localhost:9000', ps: 'CDN 代理', ssl: false, sslDays: 0 },
])

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
  return proxyList.value.filter(x => x.domain.toLowerCase().includes(q) || x.proxyPass.toLowerCase().includes(q) || x.ps.toLowerCase().includes(q))
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
}

function savePs(_row: any, _tab: string) {
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
</style>
