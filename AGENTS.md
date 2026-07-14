# Alpanel — AI 项目笔记

## 一句话

Linux 服务器管理面板（类宝塔），Rust + Axum 后端 + Vue 3 前端，Alpine musl 部署。

---

## 后端结构

`main.rs` 初始化路由、数据库、静态文件服务。关键链路：

```
main.rs (axum::Router::new)
  ├── .layer(AuthLayer)        — JWT 验证 middleware，check_auth() 被14处引用
  ├── routes/auth_routes       — /api/login, /api/verify
  ├── routes/file_routes       — /api/files/* (14个端点)
  ├── routes/settings_routes   — /api/settings
  └── frontend.rs              — rust_embed 嵌入前端 dist，SPA 兜底
```

- `handlers/` 处理 HTTP 入参出参，调 `services/`
- `services/` 业务逻辑（file_service 是大头，含 sanitize_path、list_dir、read_file 等）
- `sanitize_path` 是热点（10处调用），处理 Windows/WSL 路径转换
- `db/pool.rs` 初始化 SQLite（单连接, r2d2+rusqlite）
- SQLite 静态链接：`build.rs` 从 Alpine apk 下载 `libsqlite3.a`，不需 gcc
- `.env` 在二进制同目录，dotenvy 读取

## 前端结构

`router/index.ts` 定义路由，全局 JWT 守卫：

```
/login          → Login.vue        登录页，token 存 localStorage
/ → DefaultLayout
  ├── /         → Home.vue         空白占位
  ├── /website  → Website.vue      网站管理（三标签表格）
  ├── /file     → File.vue         文件管理器（多标签 + 持久化）
  ├── /database → Database.vue     空白占位
  ├── /cron     → Cron.vue         空白占位
  ├── /settings → Settings.vue     面板设置
  └── /logout   → Logout.vue       清除 token 跳转登录
```

- `apiFetch()` 封装 fetch，自动带 JWT `Authorization: Bearer xxx`
- `stores/settings.ts` 管理主题/标题，从 `.env` 读取，localStorage 缓存
- `App.vue` 包 `<el-config-provider :locale="zhCn">` + 监听系统颜色主题

## Website.vue 要点

- 三个 el-table 标签页：普通项目 / 其它项目 / 反向代理
- 备注列行内编辑：`<el-input v-model="row.ps" @blur="savePs(row, tab)" />`
- 状态列颜色：运行中绿色 `▶` / 已停止橙色 `⏸`
- SSL 列：有天数蓝 link / 未部署橙 link
- 根目录列：`<span class="link-cell" @click="goFile(row.root)">` → router.push 到 `/file?path=...`

## File.vue 要点

### 核心数据结构

```ts
interface BrowserTab { id, title, type:'browser', path, files:FileItem[], loading, selectedFile }
interface EditorTab  { id, title, type:'editor', path, content, original, saving }
const tabs = ref<Tab[]>([])     // 持久化到 localStorage 'alpanel_file_tabs'
const activeTab = ref('')       // 持久化
const pathInput = ref('/')      // 当前活跃标签的路径输入框
```

- localStorage 保存标签路径、编辑器内容；browser tab 恢复后重新 fetch 文件列表
- 从站点头跳转：`?path=` 查询参数 → `addBrowserTabAt(path)` 新增标签
- 右键菜单 `ctxMenu` 用 `reactive({})` 管理
- 剪贴板 `clipboard` 用 `reactive({ paths, cut })`
- 内联重命名：v-if 切换 input，聚焦用 `document.querySelector('.rename-inline input')`
- 编辑器标签：按 `item.path` 做 id 去重（openEditor 函数）

### 踩坑记录

- `ref([])` push 的对象自动成为 reactive proxy，但局部变量不跟踪变更
- `el-table` v-for slot 中拿 tab 需 `tabs.value.find()`，闭包捕获的 tab 可能过期
- `el-table` 默认撑满容器，加 `table-layout: fixed` 按列宽分配
- `text-overflow: ellipsis` 对 flex 容器不生效，文件名用普通 inline
- 右键菜单 document click 自动关闭，`confirmRename` 不能放在 `closeCtxMenu` 中

### 右键菜单清单

| 区域 | 选项 |
|------|------|
| 空白 | 刷新、上传、新建文件/文件夹、URL下载、终端 |
| 文件夹 | 打开、在新标签打开、复制、剪切、粘贴、重命名、删除、压缩、属性 |
| 文件 | 编辑、下载、权限、复制、剪切、粘贴、重命名、删除、压缩、属性 |

## 部署模型

```
/www/
├── wwwlogs/
├── wwwroot/        → 站点目录
└── server/
    ├── nginx/php(74/82)/mysql/data/redis/bun/cron/
    └── panel/
        ├── alpanel        # 二进制
        ├── .env           # PANEL_PORT, USER, PASSWORD, TITLE, THEME
        └── alpanel.db     # SQLite
```

## 数据库（alpanel.db）

```sql
users    (id, username, password(md5(md5(pw)+salt)), login_ip, login_time, phone, email, salt)
sites    (id, name, path, status, index, ps, addtime)
domain   (id, pid→sites.id, name, port, addtime)
```

## 构建命令

| 用途 | 命令 |
|------|------|
| 前端开发热更新 | `cd frontend && bun run dev` (Vite :5173) |
| 前端生产构建 | `cd frontend && bun run build` |
| 完整调试（构建→推WSL→运行） | `cd frontend && bun run backend` (调 wsl-run.ps1) |
| 生产发布包 | `scripts/build-release.ps1` → `releases/alpanel-<ver>-<target>.tar.gz` |

- 默认 target 为 Linux musl，`.cargo/config.toml` 控制双架构
- 前端改完必须重编后端（rust_embed 静态嵌入）

## alp 管理命令（scripts/alp.sh）

```
alp     → 帮助菜单
alp 11  → 启动 alpanel
alp 12  → 停止
alp 13  → 重启
alp 21  → 改账号
alp 22  → 改密码
alp 31  → 改端口
alp 0   → 取消
```

## 重要规则

- 改代码前先用 `search_graph` / `trace_path` 定位相关函数和调用链，不盲目通读文件
- 禁止修改用户环境（安装/卸载软件）
- 用户没说"开始"就不改文件
- WSL 单核低内存，禁止在里面编译
- 用户有疑问先回复再改代码
