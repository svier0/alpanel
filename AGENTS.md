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
- 本项目禁止使用npm，应使用pnpm
- **改文件即时提交**：高频小步提交，改一处/验证一处/提交一处，一个功能可拆成多次原子提交（不攒批，保证可回溯节点细粒度）
- git push 必须关证书校验：`git -c http.sslVerify=false push`（Windows schannel 吊销检查失败）；**禁止指定远程分支名**（本地分支已跟踪远程同名分支，直接 push 即可；曾因 `push origin main:master` 在远程凭空建出 master 分支造成混乱，已改回 master 单一分支）；无 GPG key，提交为 unsigned
- 重新部署二进制到 WSL 后必须 `chmod +x`（Windows→WSL 复制会丢执行权限，否则 alp 提示"已启动"但进程起不来）
- 禁止 `apk add`。patchelf 仅可临时用：mktemp 目录 + `apk fetch --recursive patchelf`，用 `LD_LIBRARY_PATH=$tmp/usr/lib:$tmp/lib` 调用，用完 `rm -rf`

## 知识图维护

- 每次会话开始时先运行 `codebase-memory_index_repository` 重新索引知识图
- 修改代码结构后（新增/重命名函数、文件、路由），运行 `codebase-memory_index_repository` 重新索引知识图
- 只改函数内部实现无需重索引

---

## 后端结构

`main.rs` 初始化路由、数据库、静态文件服务。关键链路：

```
main.rs → routes::routes() (routes/mod.rs 里 merge 全部子路由).fallback(serve_frontend)
  ├── .layer(AuthLayer)        — JWT 验证 middleware，check_auth() 被14处引用
  ├── routes/auth_routes       — /api/login, /api/verify
  ├── routes/settings_routes   — /api/settings
  ├── routes/file_routes       — /api/files/* (16个端点)
  ├── routes/plugin_routes     — /api/plugins/*（action/list/remote）
  ├── routes/site_routes       — /api/sites/{id}/files|logs（站点修改弹框各 tab 数据源）
  ├── routes/system_routes     — /api/system/{users,info,stat,kill/{pid},php-versions}（php-versions 扫描已装 PHP 目录）
  └── frontend.rs              — 从文件系统 dist/ 目录读取静态文件，SPA 兜底
```

- `handlers/` 处理 HTTP 入参出参，调 `services/` 或 `repositories/`
- `services/` 业务逻辑（file_service 是大头，含 sanitize_path、list_dir、read_file 等）
- `repositories/` DB 访问层（user/site/domain repo）；`dto/` 请求/响应结构；`models/` 表实体
- 分层：`routes → handlers → repositories/services → db::pool`
- `sanitize_path` 是热点（10处调用），处理 Windows/WSL 路径转换
- `db/pool.rs` 初始化 SQLite（单连接, r2d2+rusqlite）
- SQLite 静态链接：`build.rs` 从 Alpine apk 下载 `libsqlite3.a`，不需 gcc
- `.env` 在二进制同目录，dotenvy 读取

## 前端结构

`router/index.ts` 定义路由，全局 JWT 守卫：

```
/login          → Login.vue        登录页，token 存 localStorage
/ → DefaultLayout
  ├── /         → Home.vue         仪表盘（负载/CPU/内存/挂载点园环、监控折线图、系统信息、应用状态）
  ├── /website  → Website.vue      网站管理（动态标签表格，由 /api/sites/types 生成）
  ├── /file     → File.vue         文件管理器（多标签 + 持久化）
  ├── /database → Database.vue     数据库管理（MySQL/Redis 双标签）
  ├── /cron     → Cron.vue         空白占位
  ├── /settings → Settings.vue     面板设置
  └── /logout   → Logout.vue       清除 token 跳转登录
```

- `apiFetch()` 封装 fetch，自动带 JWT `Authorization: Bearer xxx`，非 2xx 抛异常，JSON 外返回 `text()`
- `stores/settings.ts` 管理主题/标题，从 `.env` 读取，localStorage 缓存
- `App.vue` 包 `<el-config-provider :locale="zhCn">` + 监听系统颜色主题
- `PluginMarket.vue` — 插件市场，`openPlugin(name)` 动态加载 index.js，Vue 弹窗渲染（非 iframe）；仅已安装插件可点击打开

## 插件系统（Plugin API）

`frontend/src/utils/plugin.ts` 提供插件沙箱：

```ts
// 插件 JS DSL（由 new Function('Plugin', code) 执行）
Plugin({
  plugin_name: 'nginx',
  title: 'Nginx 插件',     // 弹窗标题，默认 plugin_name
  width: 800,          // 可选，数字=px，字符串原样；默认 620px
  height: 700,         // 默认 620px
  layout: 'tabpages',  // 可选；'tabpages'=侧面tab+内容区多页结构（tabs/pages 生效），缺省 'none'=render
  tabs: { fastcgi: 'FastCgi', log: '网站日志', ... },  // layout='tabpages' 时的 tab 键名→标题
  pages: { fastcgi: { onLoad(ctx,state){...}, render(h,state){...} }, ... },  // 每 tab 的加载/渲染
  setup(ctx) { ... },  // Vue Composition API 风格，返回 state
  render(h, state) { ... },  // 返回 VNode（layout='none' 时用）
  style() { return 'css string' },  // 自动以 .plugin-dlg 前缀 scope
}).show()
```

### tabpages 布局要点

- `layout:'tabpages'` 时渲染为：左侧 `.side` tab 栏 + 右侧 `.content` 内容区，切换 tab 时触发对应 page 的 `onLoad(ctx, state)`（懒加载，切到才加载）再 `render(h, state)`；内置 TABPAGES_CSS 拼在插件 style 前一并 scope
- 页面根 div 是 `.content` 的直接子元素，命中 `.page>div` flex 高度链规则（见下）

### ctx 上下文

| 属性 | 说明 |
|------|------|
| `ctx.api(action, opts?)` | 调后端；`'status'` → `POST /api/plugins/action/{name}/status`；`'/api/xxx'` → 完整路径 |
| `ctx.ref` / `ctx.reactive` / `ctx.computed` | Vue 3 响应式 API |
| `ctx.onMounted(fn)` / `ctx.onUnmounted(fn)` | 生命周期钩子 |
| `ctx.toast(msg, type?)` | 顶部通知，type=`'ok'`（绿）/`'err'`（红），3s 消失 |
| `ctx.Editor` | CodeMirror 6 编辑器组件，`h(ctx.Editor, { modelValue, language, readonly, height, 'onUpdate:modelValue' })` |
| `ctx.plugin_name` | 插件名 |

### 插件 CSS 自动 scope

插件 `style()` 返回的 CSS 自动加 `.plugin-dlg` 前缀（如 `.app{...}` → `.plugin-dlg .app{...}`），弹窗根元素带 `class="plugin-dlg"`，避免样式泄露到页面其他元素。

### 插件弹窗 flex 高度链

tabpages 结构（`.app`/`.content`/`.page`）与编辑器滚动依赖一条标准 flex 高度链：

- `.app{display:flex;flex:1;min-height:0}` → `.content{flex:1;...;display:flex;flex-direction:column}` → `.page{display:flex;flex-direction:column;flex:1;min-height:0}` → `.page>div{flex:1;display:flex;flex-direction:column;min-height:0;overflow:auto}`
- `.page>div` 直接子元素选择器是**双刃剑**：它让编辑器 `flex:1` 生效撑满剩余空间、`cm-scroller` 内部滚动，但也会把**页面根 div 的任何横向布局**强制改成竖排居中（`.fcgi-row` 曾中招）
- 规避：页面内容若需横向 flex（如 label+select+button 一行），在页面根 div 内**再包一层普通 div**，使 `.page>div` 命中外层、内层保持自身布局
- `ctx.Editor` 渲染为 `.plugin-editor`：默认固定 320px 内部滚动；传 `height`（数字=px/字符串原样）可覆盖；`.cm-editor{height:100%}` + `.cm-scroller{overflow-y:auto}` 使滚动条确定可用
- 编辑器扩展：`indentUnit.of('    ')`（4 空格，CM6 默认 2 会导致缩进辅助线错位）+ `@replit/codemirror-indentation-markers` 缩进辅助线

### 后端 action 白名单

固定方法 `install|uninstall|start|stop|restart|reload|status` + `info.json` 的 `func` 字段（`|` 分隔，如 `"func":"get_version|get_nginx_value"`）。

### nginx 插件设计参考

- 服务控制：`ctx.api('status')` / `ctx.api('start')` / `ctx.api('stop')` ...
- 配置文件：`ctx.api('/api/files/read?path=...', {method:'GET'})` 读 / `POST /api/files/write` 写
- 性能调整：sh 侧 `get_nginx_value` / `set_nginx_value`（读 config 解析 JSON / 写 JSON 到 `/tmp/nginx_perf.json` 后 sed 更新并重载）
- 负载状态：sh 侧 `get_nginx_status`（读 `/proc/{pid}/status` + curl stub_status）
- 日志：读 `/www/wwwlogs/nginx_error.log`

### nginx 插件 conf 目录（配置即文件）

插件仓库 `plugins/nginx/conf/` 放配置模板（明文单文件，非压缩包），`install()` 逐个 wget 从仓库拉取：

- URL 拼法（读 .env 的 GHPROXY 加前缀，同 alp.sh）：`NGINX_RAW="${GH_PROXY}https://raw.githubusercontent.com/svier0/alpanel-plugins/master/plugins/nginx"`，再 `wget "$NGINX_RAW/conf/xxx.conf" -O ...`
- 禁止改 alp.sh 插件下载机制（固定拉 info.json/name.sh/icon.png/index.js）；conf 目录是插件自身行为，由插件脚本自己拉取
- `conf/nginx.conf`：主配置，`http{ include mime.types; include proxy.conf; ... }`
- `conf/proxy.conf`：proxy_temp_path / proxy_cache_path / proxy_* 超时缓存等，被 nginx.conf include
- `conf/php-{ver}.conf`：PHP 解析 location 块（ver=00/74/75/80/81/82/83/84/85，fastcgi_pass unix:/tmp/php-cgi-{ver}.sock），`php-00.conf` 为空占位（纯静态站点用）

### 站点 vhost 模板占位符

`panel/vhost/template/nginx/site.conf` 模板占位符（后端 `generate_site_vhost` 替换）：

- `{$listen_ports}` / `{$domains}` / `{$site_path}` / `{$site_name}` / `{$php_version}`
- PHP include 行：`include {$php_version}.conf;`，后端 `php_version_tag()` 把站点 phpversion（`7.4`→`php-74`）转成 conf 文件名，空→`php-00`

## Home.vue 要点

- 两列布局：左宽（状态园环+概览+监控折线图）右窄（系统信息+备忘录+应用列表）
- 园环 grid 一行4列（负载/CPU/内存 + N个挂载点各一个），颜色 <60% 绿 <90% 黄 >=90% 红
- 监控折线图 15 分钟窗口（180点 @5s），流量/磁盘切换，网卡选择
- 系统信息：hostname/OS/arch/kernel/IP/开机时间/运行时长
- 备忘录：localStorage textarea 持久化
- 应用列表：每个应用调 `/api/plugins/action/<svc>/status` 获 installed/running/version，按钮与 Website/Database 页完全一致（版本号+▶/⏸+启动/停止/重启/重载下拉）
- 园环 tooltip：负载(1/5/15min)、CPU(型号*频率+per-core%+8项breakdown+Top5进程+killbtn)、内存(total/used/avail/free/cached/shared+Top5)、磁盘(设备/fs_type/总量/已用/可用/占用率+inode信息)

## Website.vue 要点

- tab 由 `/api/sites/types` 动态生成（过滤 visibled=1，PHP/Other 显示，Proxy 暂未做）；activeTab 用 project_type 名
- 备注列行内编辑：`<el-input v-model="row.ps" @blur="savePs(row, tab)" />`
- 状态列颜色：运行中绿色 `▶` / 已停止橙色 `⏸`
- SSL 列：有天数蓝 link / 未部署橙 link
- 根目录列：显示 `row.root`（后端 path 字段映射为 root），`<span class="link-cell" @click="goFile(row.root)">` → router.push 到 `/file?path=...`

### 站点修改弹框（siteConfig.ts）

- `openSiteConfig(site)` 用 `Plugin({ layout:'tabpages', width:800, height:620, tabs, pages })` 实现，tab 键名→标题：domain 域名管理 / directory 网站目录 / rewrite 伪静态 / config 配置文件 / ssl SSL证书 / fastcgi FastCgi / proxy 反向代理 / log 网站日志 / other 其它设置
- 数据源：`/api/sites/{id}`（域名/网站目录/运行目录/PHP版本）、`/api/sites/{id}/files`（rewrite/config 内容）、`/api/sites/{id}/logs?type=access|error`（log tab，响应/错误日志子 tab）
- FastCgi tab：PHP版本下拉由 `GET /api/system/php-versions` 动态生成（纯静态=0 + php{ver}），`saveFcgi()` PUT `{phpversion}`，后端 `7.4`→`php-74` 写 vhost 并 reload nginx
- 运行目录：`onRunPicked` 存 `path.slice(siteRoot.length)` 相对路径，PUT `{php_run_path}` 保存；仅 PHP 站点生效（后端显式判断 project_type）
- 伪静态/配置文件/日志编辑器用 `ctx.Editor` 传 `height`（rewrite/config=480，log=520），保存按钮在编辑器下方 `.row` 内（当前 saveRewrite/saveConfig 仅 toast 占位，未落盘）

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

### 在线编辑器（FileEditorDialog）

- 双击文件 / 右键"编辑" 打开 `FileEditorDialog.vue`（Teleport 到 body 的自定义浮窗，非全屏，可拖拽/缩放/最小化/最大化）
- 左目录树 `FileTree.vue` 递归组件；右多标签 CodeMirror `CodeMirrorHost.vue`（CodeMirror 6 + 官方 `@codemirror/theme-one-dark`）
- 目录树右键菜单：目录→刷新目录/打开子目录/新建目录/新建文件/重命名/删除；文件→重命名/下载/删除
  - "打开子目录"= 修改树上方目录地址；"新建"在右击目录下；重命名为内联编辑框（框内 ✓ 确认，回车/失焦确认，Esc 取消）
  - 重命名同步更新已打开标签页路径；删除同步关闭相关标签页
- 右键菜单用显式 `document` mousedown/contextmenu 捕获监听（非 `once`），`onTreeCtx` 先 `closeCtxMenu()` 清旧监听再重开，否则二次右键会被遗留监听瞬间关掉
- 确认弹框（新建/删除 `el-dialog`）加 `:z-index="zIndex + 100"` 防被编辑器窗口遮挡
- `detectLanguage`：php/js/ts/css/html/json/sh/bash/zsh→shell/yml/yaml→yaml/py/ini/conf/cfg/env→ini( properties )/sql→mySQL/toml/md→markdown/txt/log→text/Dockerfile→dockerfile/nginx.conf/*.conf→nginx
- **shebang 推断**：`loadFileContent` 后若 language 为 text，取首行 `#!` 推断（sh/bash/zsh→shell，python/perl→python，php，node→javascript），切换 language 经 `langCompartment` 自动重刷
- shell 高亮使用 `@codemirror/legacy-modes/mode/shell`（StreamLanguage），**按命令词表上色**（`start`/`stop` 等在内故黄，`status` 不在故白），不识别函数定义；此为上游限制，不手写主题覆盖

## 部署模型

```
/www/
├── wwwlogs/
├── wwwroot/        → 站点目录
└── server/
    ├── nginx/php(74/82/83/84/85)/mysql/data/redis/bun/cron/
    └── panel/                  # 由 release 包(panel 整目录)解压而来
        ├── alpanel             # 二进制
        ├── dist/               # 前端静态文件(打包时从 frontend/dist 复制)
        ├── plugin/             # 插件目录(运行时装)
        ├── vhost/
        │   ├── nginx/          # 站点 nginx 配置(运行时生成)
        │   ├── rewrite/        # 伪静态(运行时生成)
        │   ├── ssl/            # SSL 证书(运行时生成)
        │   └── template/
        │       └── nginx/      # nginx 站点模板(随包发布, site.conf/other_http.conf/proxy.conf)
        ├── data/
        │   ├── db/             # SQLite alpanel.db
        │   └── files_ps/       # 文件备注
        └── .env                # PANEL_PORT, USER, PASSWORD, TITLE, THEME
```

## 命名约定（MySQL / MariaDB）

MariaDB 是 MySQL 分支，程序内**一律称 MySQL**，`mariadb` 只作为上游 apk 名出现，禁止在文件/变量/路由/UI 中写 `mariadb`：

- 允许出现 `mariadb` 的：apk 包名（`mariadb mariadb-client`）、apk 二进制（`mariadbd` 守护进程、`mariadb` 客户端、`mariadb-install-db`）、引擎内部目录 `share/mariadb`（初始化必须用此名）
- 一律用 `mysql` 的：已安装目录 `/www/server/mysql`、OpenRC 脚本 `/etc/init.d/mysql`、pid/sock/log（`mysql.pid`/`mysql.sock`/`mysql_error.log`）、软链 `/usr/bin/mysql`、UI 文案

## 服务部署设计（Nginx / MySQL / Redis）

- 每个服务：`/www/server/<svc>/` 放二进制+lib+conf；`/etc/init.d/<svc>` 为 OpenRC 控制脚本（`start/stop/status/restart/reload`，走 `start-stop-daemon`，带 `RC_SVCNAME` 守卫）；`rc-update add <svc> default` 开机自启
- `/usr/bin/<svc>` 是**纯软链**指向真实二进制（nginx→sbin/nginx、mysql→bin/mariadbd、redis→bin/redis-server、php<ver>→bin/php<ver>），无 passthrough 脚本
- 软链能直跑：对二进制用临时 `apk fetch --recursive patchelf` 在 mktemp 目录提取 patchelf，再 `--set-rpath` 嵌入 `/www/server/<svc>/lib`，用完 `rm -rf` 临时目录
- 后端控制服务一律经插件 action 端点：`POST /api/plugins/action/<svc>/{status,install,start,stop,restart,reload}`，`source` 插件脚本后调对应函数
- 后端控制服务一律经 `/etc/init.d/<svc>`（OpenRC），不经裸 `start-stop-daemon`
- 路由：`/api/plugins/action/<svc>/{status,install,start,stop,restart,reload}`（<svc>=nginx|mysql|redis|php74|php82|php83|php84|php85），前端 `apiFetch` 非 2xx 抛错 → 调用方 `catch` 设未安装

## 数据库（alpanel.db）

```sql
users    (id, username, password(md5(md5(pw)+salt)), login_ip, login_time, phone, email, salt)
sites    (id, name, path, status, project_type, phpversion, php_run_path, project_cmd, project_port, run_user, is_onpower, ps, addtime)
domain   (id, pid→sites.id, name, port, addtime)
```

- `project_type`: PHP / Other / Proxy（前端由 `/api/sites/types` 动态获取；该接口每个类型带 `visibled` 字段控制 tab 是否显示，Proxy=0 暂未做）
- `status`: 站点运行状态，`1`(运行中) / `0`(已停止)；新建默认 `0`（未生成 nginx 配置前不视为运行中）
- `phpversion`: 普通项目 PHP 版本，存 `7.4`/`8.2` 等，空为静态
- `php_run_path`: 运行目录（相对 site path，存 `/public` 等，默认空=网站根），仅 PHP 站点生效；`generate_site_vhost` 拼进 root，修改后重新生成 vhost 并 reload
- 其它项目(Other)字段：`project_cmd`(执行命令)、`project_port`(运行端口)、`run_user`(运行用户，默认 www)、`is_onpower`(是否开机启动 1/0)

## 构建命令

| 用途 | 命令 |
|------|------|
| 前端开发热更新 | `cd frontend && pnpm run dev` (Vite :5173) |
| 前端生产构建 | `cd frontend && pnpm run build` |
| 完整调试（构建→推WSL→运行） | `cd frontend && pnpm run backend` (调 wsl-run.ps1) |
| 生产发布包 | `scripts/build-release.ps1` → `releases/alpanel-<ver>-<target>.tar.gz` |

- 默认 target 为 Linux musl，`.cargo/config.toml` 控制双架构
- 前端 dist 从文件系统读取（`{binary_dir}/dist/`），非 rust_embed 嵌入
- **发布包结构**：`releases/alpanel-<ver>-<target>.tar.gz` = **panel 整个目录**（仓库根 `panel/` 母版：二进制 alpanel + 完整前端 dist + vhost/template 模板 + data/db|files_ps、plugin、vhost/nginx|rewrite|ssl 空目录）。build-release.ps1/sh 流程：编译前端 → dist 复制进 panel/dist → 逐 target 编译后端 → alpanel 复制进 panel → 整目录打包
- `panel/` 目录 git 规则：`dist/`、`alpanel`、运行时生成内容（data/db/*、data/files_ps/*、plugin/*、vhost/nginx/*、vhost/rewrite/*、vhost/ssl/*）被 .gitignore 忽略，空目录用 .gitkeep 保留；**vhost/template/ 模板入库**
- **升级覆盖策略**：install.sh 单包下载后 `tar -xzf -C /www/server/panel/`，文件覆盖、已有目录内容保留（不删 `.env`/alpanel.db/已装插件/已生成配置）、缺失目录自动新建

## install.sh

- 函数化结构，脚本顶层 `main "$@"` → `parse_args "$@"` 传递安装参数；支持 `--port`（1-65535 校验）、`--user`、`--pass`；未指定时 port 随机 10000-65535、pass 随机 16 位、user 默认 admin
- `apk add curl sqlite openrc jq vnstat`（root + Alpine 检查后执行）；写 aliyun 镜像源仅在 `GH_PROXY` 非空（网络探测失败）时执行
- 网络探测 `wget --spider google.com` 决定是否走 gh-proxy.com 代理；**下载用 curl**（`curl -fsSL --connect-timeout 10 -o`），因 busybox wget 对多 A 记录解析报 bad address
- 公网 IP 探测：GH_PROXY 非空走 nslookup opendns，否则 `curl --connect-timeout 10 https://ifconfig.me`
- vnstat OpenRC 服务名是 `vnstatd`（不是 `vnstat`），`rc-update add vnstatd default` + `rc-service vnstatd start`
- 脚本带 `set -e`，下载/校验/解压失败均 `exit 1`

## 踩坑

- WSL2 VirtioProxy 内核 bug：`bind()` 返回非标准正值 `2147014883`，musl 不认错，`TcpListener::bind()` 返回 Ok 但端口无效。**默认端口 8888** 已验证可用；`.wslconfig` 不要关闭 VirtioProxy

## alp 管理命令（scripts/alp.sh）

```
alp     → 帮助菜单
alp 11  → 启动 alpanel
alp 12  → 停止
alp 13  → 重启
alp 21  → 改账号
alp 22  → 改密码
alp 31  → 改端口
alp 51  → 查看已安装插件
alp 52  → 插件市场（获取远程插件列表）
alp 53  → 安装插件 (如 alp 53 nginx)
alp 54  → 卸载插件 (如 alp 54 nginx)
alp 99  → 卸载面板（删 /www 全部、所有服务脚本、www 用户组，不可恢复；需 root 且输入 YES 确认）
alp 0   → 取消
```
