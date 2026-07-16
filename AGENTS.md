# Alpanel — AI 项目笔记

## 一句话

Linux 服务器管理面板（类宝塔），Rust + Axum 后端 + Vue 3 前端，Alpine musl 部署。

---

## 重要规则

- 改代码前先用 `search_graph` / `trace_path` 定位相关函数和调用链，不盲目通读文件
- 禁止修改用户环境（安装/卸载软件）
- 用户没说"开始"就不改文件
- WSL 单核低内存，禁止在里面编译
- 用户有疑问先回复再改代码
- 禁止把用户电脑当wsl(例如，需要操作wsl中的/www目录，却访问d:\www)
- 本项目禁止使用node/npm
- **改文件即时提交**（每改完一个文件立刻 commit）
- git push 必须关证书校验：`git -c http.sslVerify=false push origin master`（Windows schannel 吊销检查失败）；无 GPG key，提交为 unsigned
- 禁止 `apk add`。patchelf 仅可临时用：mktemp 目录 + `apk fetch --recursive patchelf`，用 `LD_LIBRARY_PATH=$tmp/usr/lib:$tmp/lib` 调用，用完 `rm -rf`

## 知识图维护

- 每次会话开始时先运行 `codebase-memory_index_repository` 重新索引知识图
- 修改代码结构后（新增/重命名函数、文件、路由），运行 `codebase-memory_index_repository` 重新索引知识图
- 只改函数内部实现无需重索引

---

## 后端结构

`main.rs` 初始化路由、数据库、静态文件服务。关键链路：

```
main.rs (axum::Router::new)
  ├── .layer(AuthLayer)        — JWT 验证 middleware，check_auth() 被14处引用
  ├── routes/auth_routes       — /api/login, /api/verify
  ├── routes/file_routes       — /api/files/* (16个端点)
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
  ├── /database → Database.vue     数据库管理（MySQL/Redis 双标签）
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

## Database.vue 要点

- 两个 el-tab：MySQL / Redis
- MySQL 标签：工具栏「添加数据库」「root密码」按钮 + 搜索/刷新；表格 数据库名|用户名|密码|备注|操作(权限/工具/改密/删除)，备注列 min-width 不限制，操作列 fixed 右侧，底部分页（10/页）
- root密码弹框：标题「修改root密码」，`*root密码` 输入框后缀刷新图标 → 随机生成 16 位（大小写字母+数字）密码
- 未安装按 tab 显示蒙版：MySQL 检测 `/www/server/mysql/bin/mariadbd`，Redis 检测 `/www/server/redis/bin/redis-server`；蒙版只盖 tab 内容（非整页），提示安装 + 安装按钮
- 前端 `apiFetch` 非 2xx（含 404）抛错 → `catch` 置 `installed=false`；旧后端缺 `/api/mysql/status` 会让已装 MySQL 也显示蒙版，换新后端即正常

## File.vue 要点

### 核心数据结构

```ts
interface BrowserTab { id, title, type:'browser', path, files:FileItem[], loading, selectedFile, selectedRows }
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

### checkbox 多选

- el-table 添加 `type="selection"` 列，`@selection-change` 更新 `tab.selectedRows`
- 右键菜单：先 `clearSelection()` 再 `toggleRowSelection(row, true)`
- 工具栏"更多"下拉菜单：复制/剪切/压缩/权限/删除
- 复制/剪切后显示粘贴按钮

### 备注功能

- 后端存储：`/www/server/panel/data/files_ps/{MD5(路径)}` 文件内容为备注
- 前端内置默认值列表 `DEFAULT_PS`，后端返回空时自动填充
- 备注列行内编辑，`@blur` 自动保存

### 压缩/解压

- 压缩：工具栏更多菜单，调用系统 `tar -czf`，支持多选
- 解压：.tar.gz 文件右键菜单，调用 `tar -xzf`，支持密码
- 压缩路径：`目录名_4位随机后缀.tar.gz`

### 踩坑记录

- `ref([])` push 的对象自动成为 reactive proxy，但局部变量不跟踪变更
- `el-table` v-for slot 中拿 tab 需 `tabs.value.find()`，闭包捕获的 tab 可能过期
- `el-table` 默认撑满容器，加 `table-layout: fixed` 按列宽分配
- `text-overflow: ellipsis` 对 flex 容器不生效，文件名用普通 inline
- 右键菜单 document click 自动关闭，`confirmRename` 不能放在 `closeCtxMenu` 中
- el-table v-for 中 ref 返回数组，需 `fileTableRef.value?.[0]` 访问实例

### 右键菜单清单

| 区域 | 选项 |
|------|------|
| 空白 | 刷新、上传、新建文件/文件夹、URL下载、终端 |
| 文件夹 | 打开、在新标签打开、权限、复制、剪切、粘贴、重命名、删除、创建压缩、属性 |
| 文件 | 编辑、下载、权限、复制、剪切、粘贴、重命名、删除、创建压缩、解压(.tar.gz)、属性 |

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
        └── data/db/
            └── alpanel.db # SQLite
```

## 命名约定（MySQL / MariaDB）

MariaDB 是 MySQL 分支，程序内**一律称 MySQL**，`mariadb` 只作为上游 apk 名出现，禁止在文件/变量/路由/UI 中写 `mariadb`：

- 允许出现 `mariadb` 的：apk 包名（`mariadb mariadb-client`）、apk 二进制（`mariadbd` 守护进程、`mariadb` 客户端、`mariadb-install-db`）、引擎内部目录 `share/mariadb`（初始化必须用此名）
- 一律用 `mysql` 的：已安装目录 `/www/server/mysql`、OpenRC 脚本 `/etc/init.d/mysql`、pid/sock/log（`mysql.pid`/`mysql.sock`/`mysql_error.log`）、软链 `/usr/bin/mysql`、后端文件 `mysql_service.rs`/`mysql_handler.rs`/`mysql_routes.rs`、路由 `/api/mysql/*`、UI 文案

## 服务部署设计（Nginx / MySQL / Redis）

- 每个服务：`/www/server/<svc>/` 放二进制+lib+conf；`/etc/init.d/<svc>` 为 OpenRC 控制脚本（`start/stop/status/restart/reload`，走 `start-stop-daemon`，带 `RC_SVCNAME` 守卫）；`rc-update add <svc> default` 开机自启
- `/usr/bin/<svc>` 是**纯软链**指向真实二进制（nginx→sbin/nginx、mysql→bin/mariadbd、redis→bin/redis-server、php<ver>→bin/php<ver>），无 passthrough 脚本
- 软链能直跑：对二进制用临时 `apk fetch --recursive patchelf` 在 mktemp 目录提取 patchelf，再 `--set-rpath` 嵌入 `/www/server/<svc>/lib`，用完 `rm -rf` 临时目录
- 后端 `services/<svc>_service.rs` 与 `nginx_service.rs` **逐字对齐**（同函数同顺序：init_d / pid_alive / check_installed / check_running / last_error / start / stop / restart / reload / install），handler/routes 亦同
- 后端控制服务一律经 `/etc/init.d/<svc>`（OpenRC），不经裸 `start-stop-daemon`
- 路由：`/api/<svc>/{status,install,start,stop,restart,reload}`（<svc>=nginx|mysql|redis），前端 `apiFetch` 非 2xx 抛错 → 调用方 `catch` 设未安装

## 数据库（alpanel.db）

```sql
users    (id, username, password(md5(md5(pw)+salt)), login_ip, login_time, phone, email, salt)
sites    (id, name, path, status, project_type, ps, addtime)
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
alp 51  → 安装 Nginx（apk fetch → 提取到 /www/server/nginx → 生成 OpenRC）
        站点配置目录: /www/server/panel/vhost/nginx/
alp 52  → 安装 PHP（可多版本, 如 alp 52 82；版本来自 apk 源 php*）
alp 53  → 安装 MySQL（apk 源的 mariadb 包，但程序内一律称 mysql；详见下方命名约定）
alp 54  → 安装 Redis（apk fetch → 提取到 /www/server/redis）
alp 99  → 卸载面板（删 /www 全部、所有服务脚本、www 用户组，不可恢复；需 root 且输入 YES 确认）
alp 0   → 取消
```
