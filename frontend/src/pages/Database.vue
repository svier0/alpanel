<template>
  <div class="database-manager">
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
      <template v-if="activeTab === 'mysql'">
        <div v-if="!mysqlReady" class="install-mask">
          <el-button type="primary" :loading="true">检测中...</el-button>
        </div>
        <div v-else-if="!mysqlInstalled" class="install-mask install-prompt">
          <p class="mask-tip">数据库管理需要 MySQL 服务</p>
          <template v-if="!installingMysql">
            <el-button type="primary" size="small" :loading="installingMysql" @click="installMysql">安装 MySQL</el-button>
          </template>
          <template v-else>
            <p style="margin:0 0 12px;font-size:13px">正在安装 MySQL，请稍候...</p>
            <el-progress :percentage="installProgressMysql" :stroke-width="6" style="width:300px" />
          </template>
          <div v-if="installErrorMysql" class="mask-error">
            {{ installErrorMysql }}
            <el-button size="small" link type="primary" @click="installErrorMysql = ''">重试</el-button>
          </div>
        </div>
        <template v-else>
          <div class="toolbar-row">
            <div class="toolbar-left">
              <el-button size="small" type="primary" @click="showAddDbDialog">
                <el-icon><Plus /></el-icon>添加数据库
              </el-button>
              <el-button size="small" @click="openRootPw">
                <el-icon><Key /></el-icon>root密码
              </el-button>
            </div>
            <div class="toolbar-right">
              <el-input
                v-model="searchQuery"
                size="small"
                class="search-input"
                placeholder="搜索数据库名 / 用户名 / 备注"
                clearable
              >
                <template #prefix><el-icon><Search /></el-icon></template>
              </el-input>
              <el-button size="small" class="toolbar-btn" @click="refreshTable">
                <el-icon><RefreshRight /></el-icon>
              </el-button>
            </div>
          </div>

          <el-table
            :data="pagedDb"
            size="small"
            class="db-table"
            empty-text="暂无数据库"
            :cell-style="{ padding: '4px 0' }"
          >
            <el-table-column label="数据库名" width="220" show-overflow-tooltip>
              <template #default="{ row }"><span class="link-cell">{{ row.name }}</span></template>
            </el-table-column>
            <el-table-column label="用户名" width="160" show-overflow-tooltip>
              <template #default="{ row }">{{ row.user }}</template>
            </el-table-column>
            <el-table-column label="密码" width="180" show-overflow-tooltip>
              <template #default="{ row }">
                <span class="pw-cell">{{ row.password }}</span>
              </template>
            </el-table-column>
            <el-table-column label="备注" min-width="200">
              <template #default="{ row }">
                <el-input v-model="row.ps" size="small" class="ps-input" @blur="savePs(row)" />
              </template>
            </el-table-column>
            <el-table-column label="操作" width="240" fixed="right">
              <template #default="{ row }">
                <el-button size="small" link type="primary" @click="dbPriv(row)">权限</el-button>
                <el-button size="small" link type="primary" @click="dbTool(row)">工具</el-button>
                <el-button size="small" link type="primary" @click="dbChangePw(row)">改密</el-button>
                <el-button size="small" link type="danger" @click="dbDelete(row)">删除</el-button>
              </template>
            </el-table-column>
          </el-table>
          <el-pagination
            small
            layout="prev, pager, next, total"
            :total="filteredDb.length"
            :page-size="pageSize"
            v-model:current-page="page"
            class="db-pagination"
          />
        </template>
      </template>

      <template v-if="activeTab === 'redis'">
        <div v-if="!redisReady" class="install-mask">
          <el-button type="primary" :loading="true">检测中...</el-button>
        </div>
        <div v-else-if="!redisInstalled" class="install-mask install-prompt">
          <p class="mask-tip">Redis 管理需要 Redis 服务</p>
          <template v-if="!installingRedis">
            <el-button type="primary" size="small" :loading="installingRedis" @click="installRedis">安装 Redis</el-button>
          </template>
          <template v-else>
            <p style="margin:0 0 12px;font-size:13px">正在安装 Redis，请稍候...</p>
            <el-progress :percentage="installProgressRedis" :stroke-width="6" style="width:300px" />
          </template>
          <div v-if="installErrorRedis" class="mask-error">
            {{ installErrorRedis }}
            <el-button size="small" link type="primary" @click="installErrorRedis = ''">重试</el-button>
          </div>
        </div>
        <div v-else class="redis-placeholder">
          <el-empty description="Redis 管理（待实现）" />
        </div>
      </template>
    </div>

    <el-dialog v-model="addDbDialog.visible" title="添加数据库" width="480px" append-to-body>
      <el-form label-width="80px">
        <el-form-item label="数据库名" required>
          <el-input v-model="addDbDialog.name" placeholder="请输入数据库名称" />
        </el-form-item>
        <el-form-item label="用户名" required>
          <el-input v-model="addDbDialog.user" placeholder="请输入用户名" />
        </el-form-item>
        <el-form-item label="密码" required>
          <el-input v-model="addDbDialog.password" placeholder="请输入密码" />
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="addDbDialog.ps" placeholder="请输入备注,可为空" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="addDbDialog.visible = false">取消</el-button>
        <el-button type="primary" @click="handleAddDb">确定</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="rootPwDialog.visible" title="修改root密码" width="420px" append-to-body>
      <el-form label-width="90px">
        <el-form-item label="root密码" required>
          <el-input v-model="rootPwDialog.password" placeholder="请输入或生成 root 密码">
            <template #append>
              <el-button @click="genRootPw" title="随机生成">
                <el-icon><Refresh /></el-icon>
              </el-button>
            </template>
          </el-input>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="rootPwDialog.visible = false">取消</el-button>
        <el-button type="primary" @click="handleChangeRootPw">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, reactive, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { Plus, Search, RefreshRight, Key, Refresh } from '@element-plus/icons-vue'
import { apiFetch } from '@/utils/api'

const tabs = [
  { key: 'mysql', label: 'MySQL' },
  { key: 'redis', label: 'Redis' },
]

const activeTab = ref('mysql')
const searchQuery = ref('')
const pageSize = 10
const page = ref(1)

watch(activeTab, () => { page.value = 1 })
watch(searchQuery, () => { page.value = 1 })

const mysqlReady = ref(false)
const mysqlInstalled = ref(false)
const installingMysql = ref(false)
const installProgressMysql = ref(0)
const installErrorMysql = ref('')

const redisReady = ref(false)
const redisInstalled = ref(false)
const installingRedis = ref(false)
const installProgressRedis = ref(0)
const installErrorRedis = ref('')

async function checkMysql() {
  mysqlReady.value = false
  try {
    const data = await apiFetch('/api/mysql/status')
    mysqlInstalled.value = data.installed
  } catch {
    mysqlInstalled.value = false
  } finally {
    mysqlReady.value = true
  }
}

async function checkRedis() {
  redisReady.value = false
  try {
    const data = await apiFetch('/api/redis/status')
    redisInstalled.value = data.installed
  } catch {
    redisInstalled.value = false
  } finally {
    redisReady.value = true
  }
}

async function installMysql() {
  installingMysql.value = true
  installErrorMysql.value = ''
  installProgressMysql.value = 0
  const iv = setInterval(() => {
    installProgressMysql.value = Math.min(installProgressMysql.value + 3, 90)
  }, 800)
  try {
    await apiFetch('/api/mysql/install', { method: 'POST' })
    clearInterval(iv)
    installProgressMysql.value = 100
    setTimeout(() => { mysqlInstalled.value = true }, 600)
  } catch {
    clearInterval(iv)
    installErrorMysql.value = '安装失败，请检查服务端连接'
  } finally {
    installingMysql.value = false
  }
}

async function installRedis() {
  installingRedis.value = true
  installErrorRedis.value = ''
  installProgressRedis.value = 0
  const iv = setInterval(() => {
    installProgressRedis.value = Math.min(installProgressRedis.value + 5, 90)
  }, 600)
  try {
    await apiFetch('/api/redis/install', { method: 'POST' })
    clearInterval(iv)
    installProgressRedis.value = 100
    setTimeout(() => { redisInstalled.value = true }, 600)
  } catch {
    installErrorRedis.value = '安装失败，请检查服务端连接'
  } finally {
    installingRedis.value = false
  }
}

onMounted(() => {
  checkMysql()
  checkRedis()
})

interface DbItem {
  id: number
  name: string
  user: string
  password: string
  ps: string
}

const dbList = ref<DbItem[]>([
  { id: 1, name: 'wordpress', user: 'wp_user', password: 'Wp@2024xQ9', ps: '博客主库' },
  { id: 2, name: 'shop', user: 'shop_user', password: 'Shop#8831aa', ps: '商城数据库' },
  { id: 3, name: 'auth', user: 'auth_user', password: 'Auth_77Bb', ps: '认证服务' },
  { id: 4, name: 'log', user: 'log_user', password: 'Log@2024cc', ps: '' },
  { id: 5, name: 'test', user: 'test_user', password: 'Test@1234', ps: '测试库' },
])

const filteredDb = computed(() => {
  if (!searchQuery.value) return dbList.value
  const q = searchQuery.value.toLowerCase()
  return dbList.value.filter(
    x => x.name.toLowerCase().includes(q) || x.user.toLowerCase().includes(q) || x.ps.toLowerCase().includes(q)
  )
})

const pagedDb = computed(() => {
  const start = (page.value - 1) * pageSize
  return filteredDb.value.slice(start, start + pageSize)
})

function refreshTable() {}

function savePs(_row: DbItem) {}

function dbPriv(_row: DbItem) {
  ElMessage.info('权限（待实现）')
}
function dbTool(_row: DbItem) {
  ElMessage.info('工具（待实现）')
}
function dbChangePw(_row: DbItem) {
  ElMessage.info('改密（待实现）')
}
function dbDelete(_row: DbItem) {
  ElMessage.info('删除（待实现）')
}

const addDbDialog = reactive({
  visible: false,
  name: '',
  user: '',
  password: '',
  ps: '',
})

function showAddDbDialog() {
  addDbDialog.name = ''
  addDbDialog.user = ''
  addDbDialog.password = ''
  addDbDialog.ps = ''
  addDbDialog.visible = true
}

function handleAddDb() {
  addDbDialog.visible = false
  ElMessage.success('已添加（演示）')
}

const rootPwDialog = reactive({
  visible: false,
  password: '',
})

function genRootPw() {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'
  let s = ''
  for (let i = 0; i < 16; i++) s += chars[Math.floor(Math.random() * chars.length)]
  rootPwDialog.password = s
}

function openRootPw() {
  genRootPw()
  rootPwDialog.visible = true
}

function handleChangeRootPw() {
  if (!rootPwDialog.password) {
    ElMessage.warning('请输入 root 密码')
    return
  }
  rootPwDialog.visible = false
  ElMessage.success('root 密码已修改（演示）')
}
</script>

<style scoped>
.database-manager {
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
  width: 240px;
}
.db-table {
  flex: 1;
  min-height: 0;
  width: 100%;
}
.pw-cell {
  font-family: monospace;
  font-size: 12px;
  color: var(--el-text-color-secondary);
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
.link-cell {
  color: var(--el-color-primary);
  cursor: pointer;
}
.link-cell:hover {
  text-decoration: underline;
}
.db-pagination {
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
.redis-placeholder {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 300px;
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
  gap: 12px;
}
.mask-tip {
  margin: 0;
  color: var(--el-text-color-secondary);
  font-size: 13px;
}
.mask-error {
  color: var(--el-color-danger);
  font-size: 13px;
}
</style>
