# Realms TODO — 完整体验开发任务

> **上下文**: M0-M11 全部完成 (208 tests)。当前缺失的是「打开浏览器就能玩」的完整体验。
> 本文档告诉你怎么从当前状态推进到可游玩产品。
>
> **最终目标**: **开箱即用**。用户 clone 项目后只需配一个环境变量 (如 `DEEPSEEK_KEY=xxx`)，跑 `./start.sh`，即可在浏览器中游玩完整的 Realms RP 平台。
> 不需要手动开终端、支持 OpenAI/Anthropic 两种主流 LLM 格式，用户自由切换。
>
> **达成标准**: `./start.sh` → 浏览器打开 `http://localhost:5173` → 选世界 → 选角色 → 选 GM → 开周目 → 玩。

---

## 0. 必读 (新 Session 第一件事)

```bash
# 1. 读架构文档 (3.md 是唯一权威)
read /home/jhwh/src/Realms/3.md

# 2. 了解当前进度
read /home/jhwh/src/Realms/STATUS.md

# 3. 确认环境
cargo test --manifest-path /home/jhwh/src/Realms/realms-core/Cargo.toml 2>&1 | grep "test result"
# 预期: 158 passed (147 unit + 4 crosstalk + 2 e2e + 3 demo + 2 doctest)

# 4. 确认 Vue 测试
cd /home/jhwh/src/Realms/realms-core/webui && npx vitest run 2>&1 | grep "Tests"
# 预期: 50 passed

# 5. 检查目录结构
ls /home/jhwh/src/Realms/realms-core/src/
ls /home/jhwh/src/Realms/realms-core/webui/src/widgets/
```

---

## 1. 已有什么 (不必重新做)

| 层 | 模块 | 文件位置 |
|---|---|---|
| **Rust 引擎** | prompt_assembler, gm_router(三重防护), gm_dispatcher(LLM provider+parse+audit), 4 reducer(state/rel/arc/memory), event_reducer主入口 | `src/engine/orchestrator/`, `src/engine/reducers/` |
| **数据层** | 11表schema + migration + pool + 5 query模块(cycle/char_state/event/rel/cross_cycle) | `src/db/` |
| **World/周目管理** | cycle_manager, world_loader, gm_loader, oc_initializer, perspective(锁定) | `src/engine/cycle/` |
| **Vue widget** | 12个组件 (State/Affinity/Inventory/ArcTimeline/EmotionalMap/Chat + WorldSelector/NpcSelector/OcCreator/GmSwitcher/PerspectiveSwitcher/CycleList + CycleCompare/RelationshipMatrix) | `webui/src/widgets/` |
| **LLM 连接** | HttpLlmProvider (通过 AIRP engine 网关层调 LLM), 已验证 DeepSeek 通过 | `src/engine/orchestrator/gm_dispatcher.rs` |
| **仙剑数据** | worlds/xiatian_qixia_1/{setting.md,npcs/,lorebook.json,map.json,rules.json,gms/} + gms/{default,gulong,humor,horror}.md + perspectives/ (4 NPC + 3 OC .json) | `worlds/`, `gms/`, `perspectives/` |

**不需要改：**
- 所有 `src/engine/` 和 `src/db/` 下的 Rust 模块 — 它们已经完整且测试通过。
- 所有 `webui/src/widgets/` 下的 Vue 组件 — 它们是可用的组件，只欠组装。

---

## 2. 三个缺口 (按优先级)

| # | 缺口 | 做什么 |
|---|---|---|
| **G0** | 配置靠环境变量，不够开箱即用 | WebUI 设置页: 填 provider/endpoint/key/model → 保存到 config.json → 实时生效 (调 engine /v1/settings) |
| **G2** | Rust 引擎没有 HTTP 接口 | 用 axum 写 HTTP server |
| **G1** | Vue widget 是孤儿组件 | 写 Vue Router + 主布局 (含设置页) |
| **G4** | 需要手动开终端 | 写 start.sh 一键启动 (启动 engine + realms-server + webui) |

执行顺序: G0 -> G2 -> G1 -> G4。最终验收: ./start.sh, 浏览器打开 localhost:5173, 在设置页填 API key → 选世界 → 选角色 → 选 GM → 开周目 → 玩。

**关键原则: 所有配置均通过 WebUI 完成，用户不碰文件、不设环境变量。**

---

## 3. G0: WebUI 可配置的 LLM Provider (config.json + 实时生效)

### 3.0 原则

用户**不碰文件、不设环境变量**。打开 WebUI → 设置页 → 填 provider/endpoint/key/model → 保存 → 即刻生效。配置持久化到 `data/config.json`，启动时自动加载。

### 3.1 配置存储

`data/config.json` (首次运行自动生成默认值):

```json
{
  "llm": {
    "provider": "OpenAI",
    "endpoint": "https://api.deepseek.com/v1/chat/completions",
    "api_key": "",
    "model": "deepseek-chat"
  }
}
```

### 3.2 Rust 侧: `src/server/config.rs` (新建)

```rust
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "OpenAI".into(),
            endpoint: "https://api.deepseek.com/v1/chat/completions".into(),
            api_key: String::new(),
            model: "deepseek-chat".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub llm: LlmConfig,
}

impl AppConfig {
    /// 从 data/config.json 加载; 不存在则创建默认
    pub fn load(data_dir: &Path) -> Result<Self> { ... }
    /// 保存到 data/config.json
    pub fn save(&self, data_dir: &Path) -> Result<()> { ... }
}
```

### 3.3 API 端点 (加在 G2 的 handlers 中)

| 方法 | 路径 | 功能 |
|---|---|---|
| GET | `/api/config` | 返回当前配置 (api_key 脱敏: `sk-***xxx`) |
| PUT | `/api/config` | 更新配置 → 保存 config.json → 调 engine /v1/settings 热重载 |

PUT 请求体:
```json
{
  "llm": {
    "provider": "Anthropic",
    "endpoint": "https://api.anthropic.com/v1/chat/completions",
    "api_key": "sk-ant-xxx",
    "model": "claude-sonnet-4-20250514"
  }
}
```

处理逻辑:
1. 验证 provider 是 "OpenAI" 或 "Anthropic"
2. 验证 endpoint 非空
3. 保存到 data/config.json
4. POST engine /v1/settings 热重载
5. 返回更新后的配置 (api_key 脱敏)

### 3.4 WebUI 设置页 (加在 G1 的 pages 中)

`SettingsPage.vue`:

```
┌────────────────────────────────────────┐
│  设置                                   │
├────────────────────────────────────────┤
│  LLM Provider                          │
│  ┌──────────────────────────────────┐  │
│  │ Provider:  [OpenAI ▾]            │  │
│  │ Endpoint:  [https://api...     ] │  │
│  │ API Key:   [••••••••••••••••  ] │  │
│  │ Model:     [deepseek-chat      ] │  │
│  └──────────────────────────────────┘  │
│                                        │
│  [测试连接]  [保存配置]                  │
│                                        │
│  状态: ✅ 连接成功 / ❌ 未配置 / ⏳ 测试中 │
└────────────────────────────────────────┘
```

"测试连接" 按钮: 调 `GET /api/config/test` (后端发一个最小 chat 请求验证 key 有效)。

### 3.5 start.sh 更新

不再需要环境变量。`realms-server` 启动时自动从 `data/config.json` 加载配置 (如果 api_key 为空则跳过 engine 配置, WebUI 会提示用户填写)。

### 3.6 文件

- 新建 `src/server/config.rs`
- 修改 `src/server/state.rs` (加 `config: AppConfig`)
- 修改 `src/server/handlers.rs` (加 GET/PUT /api/config)
- 修改 `src/bin/server.rs` (启动时 load config, 如果 api_key 非空则 configure_engine)
- 新建 `webui/src/pages/SettingsPage.vue`

---

## 4. G2: Rust HTTP Server (axum)

### 3.1 新增依赖

在 `realms-core/Cargo.toml` 的 `[dependencies]` 里加:

```toml
# HTTP server
axum = "0.7"
tower-http = { version = "0.5", features = ["cors"] }
tokio = { version = "1", features = ["full"] }  # 已经在 Cargo.toml
```

### 3.2 创建文件

在 `src/` 下新建 `server/` 目录:

```
src/server/
├── mod.rs          # 路由注册 + 启动函数
├── handlers.rs     # 请求处理函数
└── state.rs        # AppState (共享 DB pool + paths)
```

### 3.3 `src/server/state.rs` — 共享状态

```rust
use std::path::PathBuf;
use std::sync::Arc;
use crate::db::pool::DbPool;

/// 所有 handler 共享的状态
pub struct AppState {
    pub pool: DbPool,
    pub worlds_dir: PathBuf,
    pub gms_dir: PathBuf,
    pub perspectives_dir: PathBuf,
    pub engine_url: String,          // "http://127.0.0.1:8000"
}

pub type SharedState = Arc<AppState>;
```

### 3.4 `src/server/handlers.rs` — API 端点

#### 端点清单

| 方法 | 路径 | 功能 |
|---|---|---|
| GET | `/api/worlds` | 列出所有 world (调用 `world_loader::list_worlds`) |
| GET | `/api/worlds/:id` | 获取 world 详情 (setting + npc 列表) |
| GET | `/api/gms` | 列出所有 GM 模板 (调用 `gm_loader::list_gms`) |
| GET | `/api/perspectives` | 列出所有视角 (调用 `perspective::list_perspectives`) |
| POST | `/api/cycles` | 创建新周目 |
| GET | `/api/cycles` | 列出某 world 下的周目 (query: `?world_id=xxx`) |
| GET | `/api/cycles/:id` | 获取周目详情 + 当前状态 |
| POST | `/api/cycles/:id/turn` | **核心**: 处理一轮玩家输入 |
| DELETE | `/api/cycles/:id` | 删除周目 |

#### `POST /api/cycles` 请求体

```json
{
  "world_id": "xiatian_qixia_1",
  "perspective_id": "pov_npc_li_xiaoyao",
  "gm_id": "style_linyueru",
  "cycle_name": "周目1: 李逍遥线",
  "oc_selections": {               // 可选; source='npc' 时省略
    "lin_yueru": "tpl_childhood_friend",
    "li_xiaoyao": null,
    "zhao_linger": null
  }
}
```

处理逻辑:
1. 调用 `world_loader::load_world` 校验 world 存在
2. 调用 `gm_loader::load_gm` 校验 gm 存在
3. 调用 `perspective::load_perspective` 校验视角存在
4. 调用 `cycle::create_cycle` 写 cycles 表
5. 遍历 world.npc_personas, 为每个 npc INSERT character_states
6. 如果 oc_selections 有内容, 调用 `oc_initializer::initialize_cycle_relationships`
7. 返回创建的 cycle 信息

#### `POST /api/cycles/:id/turn` — 核心端点

请求体:
```json
{
  "user_message": "我走进余杭镇，想找一家客栈休息"
}
```

处理逻辑 (对应 3.md §4.2 的完整流程):

```
1. 读 cycle → 获取 world_id, perspective_id, gm_id
2. 加载 perspective (persona_data)
3. 加载 world.setting
4. 加载 gm.content
5. 从 SQLite 投影当前状态
6. 从 SQLite 检索相关记忆
7. 组装 system_prompt → 调用 prompt_assembler 或手写拼接
8. 调 `state.llm.chat(system_prompt, user_message).await?` (通过 AIRP engine 网关)
9. 用 `parse_subagent_output` 解析 LLM 回复
10. 从 visible_response 构造 DomainEvent::Dialogue
11. 调用 process_event 写状态
```

返回格式 (SSE):
```
event: narrative
data: {"text": "客栈老板: 客官里边请!本店上房干净,饭菜可口!"}

event: patch
data: {"op":"add","path":"/events/demo_c1","value":{...}}

event: state
data: {"character_states":{...}, "relationships":{...}}

event: done
data: {}
```

#### `GET /api/cycles/:id` 响应格式

```json
{
  "cycle": { "cycle_id": "...", "cycle_name": "...", ... },
  "character_states": [ {"npc_id":"lin_yueru","hp":80,...}, ... ],
  "relationships": [ {"npc_id":"lin_yueru","affinity":60,...}, ... ],
  "events": [ {"id":1,"summary":"...","event_type":"dialogue"}, ... ]
}
```

### 3.5 `src/server/mod.rs` — 路由

```rust
pub mod handlers;
pub mod state;

use axum::{routing::{get, post, delete}, Router};
use tower_http::cors::{CorsLayer, Any};
use state::SharedState;

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/api/worlds", get(handlers::list_worlds))
        .route("/api/worlds/:id", get(handlers::get_world))
        .route("/api/gms", get(handlers::list_gms))
        .route("/api/perspectives", get(handlers::list_perspectives))
        .route("/api/cycles", get(handlers::list_cycles).post(handlers::create_cycle))
        .route("/api/cycles/:id", get(handlers::get_cycle).delete(handlers::delete_cycle))
        .route("/api/cycles/:id/turn", post(handlers::process_turn))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

pub async fn serve(state: SharedState, port: u16) {
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await.unwrap();
    println!("Realms API server on http://localhost:{port}");
    axum::serve(listener, app).await.unwrap();
}
```

### 3.6 添加 binary target

在 `Cargo.toml` 加:

```toml
[[bin]]
name = "realms-server"
path = "src/bin/server.rs"
```

`src/bin/server.rs`:
```rust
use std::path::PathBuf;
use std::sync::Arc;
use realms_core::db::pool::init_pool;
use realms_core::server::{state::AppState, serve};

#[tokio::main]
async fn main() {
    let data_dir = PathBuf::from("./data");
    let pool = init_pool(&data_dir).expect("Failed to init DB");

    let state = Arc::new(AppState {
        pool,
        worlds_dir: PathBuf::from("./worlds"),
        gms_dir: PathBuf::from("./gms"),
        perspectives_dir: PathBuf::from("./perspectives"),
        engine_url: "http://127.0.0.1:8000".to_string(),
    });

    serve(state, 3000).await;
}
```

### 3.7 测试

写 `src/server/handlers.rs` 的单元测试:

```rust
#[cfg(test)]
mod tests {
    // 1. GET /api/worlds → 返回 worlds/ 下目录列表
    // 2. POST /api/cycles → 创建 cycle + 验证 character_states 初始化
    // 3. GET /api/cycles/:id → 返回 cycle + state + rels
    // 4. POST /api/cycles/:id/turn → 模拟一轮 (用 MockLlmProvider), 验证 state 变化
}
```

用 `axum::test` 的 `TestClient` 做 HTTP 层测试, 不需要启动真实 server。

### 3.8 lib.rs 更新

在 `src/lib.rs` 加 `pub mod server;`

---

## 4. G1: WebUI 主界面

### 4.1 依赖

已有 Vue 3 + Vite + vitest 全家桶, 不需新增依赖。但需新增 `vue-router`:

```bash
cd webui && npm install vue-router
```

### 4.2 创建文件

```
webui/src/
├── main.ts           # Vue 入口, 挂载 router
├── App.vue           # 主布局: 顶部 Tab + router-view
├── router.ts         # 路由配置
├── api.ts            # HTTP 请求封装 (调 G2 的 API)
├── types.ts          # 已有, 不修改
└── pages/
    ├── WorldSelectPage.vue    # 使用 WorldSelector widget
    ├── CharacterSelectPage.vue # 使用 NpcSelector + 创建 OC 入口
    ├── OcCreatePage.vue       # 使用 OcCreator widget
    ├── GmSelectPage.vue       # 使用 GmSwitcher widget
    ├── CycleListPage.vue      # 使用 CycleList widget
    └── GamePlayPage.vue       # 核心: 聊天 + 右侧 widget 面板
```

### 4.3 `src/router.ts`

```typescript
import { createRouter, createWebHistory } from 'vue-router'

const routes = [
  { path: '/',              component: () => import('./pages/WorldSelectPage.vue') },
  { path: '/character',     component: () => import('./pages/CharacterSelectPage.vue') },
  { path: '/oc/create',     component: () => import('./pages/OcCreatePage.vue') },
  { path: '/gm',            component: () => import('./pages/GmSelectPage.vue') },
  { path: '/cycles',        component: () => import('./pages/CycleListPage.vue') },
  { path: '/play/:cycleId', component: () => import('./pages/GamePlayPage.vue') },
]

export default createRouter({ history: createWebHistory(), routes })
```

### 4.4 `src/App.vue` — 主布局

```
┌─────────────────────────────────────────────┐
│  [世界] [角色] [GM] [周目] [游玩]             │ ← 顶部 Tab
├─────────────────────────────────────────────┤
│                                             │
│          <router-view />                     │ ← 页面内容
│                                             │
└─────────────────────────────────────────────┘
```

响应式:
- PC (≥1024px): Tab 在顶部水平排列
- 手机 (<768px): Tab 在底部固定, 内容区滚动

### 4.5 `src/api.ts` — HTTP 封装

```typescript
const BASE = 'http://localhost:3000/api';

export async function listWorlds(): Promise<WorldInfo[]> { ... }
export async function getWorld(id: string): Promise<WorldDetail> { ... }
export async function listGms(): Promise<GmTemplate[]> { ... }
export async function createCycle(data: CreateCycleReq): Promise<Cycle> { ... }
export async function getCycle(id: string): Promise<CycleState> { ... }
export async function processTurn(cycleId: string, msg: string): Promise<ReadableStream> { ... }
```

`processTurn` 用 `fetch` + `ReadableStream` 接收 SSE 流, 返回一个 async iterable。

### 4.6 页面实现要点

#### WorldSelectPage.vue
- 调用 `listWorlds()` → 传给 WorldSelector widget
- 选好后 → `router.push('/character')`

#### CharacterSelectPage.vue
- 显示内置 NPC (调用 `getWorld()`) + "创建 OC" 按钮
- 选内置 NPC → `router.push('/gm')`
- 创建 OC → `router.push('/oc/create')`

#### OcCreatePage.vue
- 使用已有的 OcCreator widget
- 完成创建 → `router.push('/gm')`

#### GmSelectPage.vue
- `listGms()` → GmSwitcher widget
- 选好 → `router.push('/cycles')`

#### CycleListPage.vue
- 同时展示已有周目 (CycleList) + "开新周目" 按钮
- 点周目 → `router.push('/play/:id')`

#### GamePlayPage.vue ← **最关键的页面**

三栏布局 (PC):
```
┌──────────┬───────────────────┬──────────────────┐
│ 左侧角色  │  中间聊天区        │  右侧 Widget 面板  │
│          │                   │                  │
│ 角色列表  │  历史消息滚动      │  StateWidget     │
│ NPC 好感  │                   │  AffinityWidget  │
│          │  输入框 + 发送      │  ArcTimeline     │
│ [切GM]   │                   │  Inventory       │
│          │                   │  EmotionalMap    │
└──────────┴───────────────────┴──────────────────┘
```

手机 (<768px):
- 聊天区全宽, 底部输入框
- 底部 Tab: [聊天] [状态] [物品] [关系]
- 每个 Tab 对应一个 widget 面板

**核心逻辑**:
1. 页面加载 → `getCycle(id)` 获取初始状态 → 渲染 widget
2. 用户输入 → `processTurn(id, msg)` → SSE 流
3. 每收到 `event: narrative` → 追加到聊天区
4. 每收到 `event: patch` → 更新对应 widget
5. 每收到 `event: state` → 刷新全部 widget
6. `event: done` → 解锁输入框

### 4.7 测试

至少写 GamePlayPage 的测试:
- 初始加载显示 cycle 信息
- 发送消息后聊天区更新
- SSE 流处理正确

---

## 8. G4: 一键启动脚本 (开箱即用)

### 5.1 目标

用户 clone 项目后只需跑一条命令: `./start.sh`

脚本自动完成:
1. 编译并启动 AIRP engine (后台)
2. 编译并启动 realms-server (后台, 自动从 data/config.json 加载 LLM 配置并调 engine /v1/settings)
3. 安装 webui 依赖 (首次) + 启动 vite dev server
4. 打印 `打开浏览器 http://localhost:5173 -> 设置页填 API key -> 开始玩`

### 8.2 创建 `realms-core/start.sh`

```bash
#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REALMS_DIR="$(dirname "$SCRIPT_DIR")"
AIRP_DIR="$REALMS_DIR/AIRP"
WEBUI_DIR="$SCRIPT_DIR/webui"
ENGINE_PORT=8000
API_PORT=3000
WEBUI_PORT=5173

echo "=== Realms 启动 ==="

# 1. AIRP Engine (网关层)
echo "[1/3] AIRP Engine (port $ENGINE_PORT)..."
cd "$AIRP_DIR"
cargo build -p airp-core 2>/dev/null
nohup cargo run -p airp-core -- daemon --port $ENGINE_PORT > /tmp/realms-engine.log 2>&1 &
ENGINE_PID=$!
sleep 8
curl -s http://localhost:$ENGINE_PORT/health > /dev/null || { echo "Engine 启动失败"; exit 1; }
echo "  OK (PID $ENGINE_PID)"

# 2. Realms API Server (从 config.json 加载 LLM 配置并 configure_engine)
echo "[2/3] Realms API Server (port $API_PORT)..."
cd "$SCRIPT_DIR"
cargo build --bin realms-server 2>/dev/null
nohup cargo run --bin realms-server > /tmp/realms-api.log 2>&1 &
API_PID=$!
sleep 5
echo "  OK (PID $API_PID)"

# 3. WebUI
echo "[3/3] WebUI (port $WEBUI_PORT)..."
cd "$WEBUI_DIR"
[ -d node_modules ] || npm install --silent 2>/dev/null
npx vite --port $WEBUI_PORT --host > /tmp/realms-webui.log 2>&1 &
WEBUI_PID=$!
sleep 2

echo ""
echo "=============================================="
echo "  Realms RP 平台已就绪!"
echo "  打开浏览器: http://localhost:$WEBUI_PORT"
echo "  -> 设置页 填写 LLM API Key -> 保存"
echo "  -> 选世界 -> 选角色 -> 选 GM -> 开周目 -> 玩"
echo "  停止: kill $ENGINE_PID $API_PID $WEBUI_PID"
echo "=============================================="

trap "kill $ENGINE_PID $API_PID $WEBUI_PID 2>/dev/null; exit" INT TERM
wait
```

### 5.3 测试

```bash
cd /home/jhwh/src/Realms/realms-core
chmod +x start.sh
./start.sh
# 浏览器打开 http://localhost:5173
# 设置页填 API key -> 保存 -> 测试连接 -> 开始玩
```

---

## 6. 验证清单

完成 G2+G1+G3+G4 后:

```bash
# 一条命令启动全部服务
cd /home/jhwh/src/Realms/realms-core
./start.sh

# 浏览器打开 http://localhost:5173
# 选世界 -> 选角色 -> 选 GM -> 开周目 -> 玩
```

全量测试:
```bash
cargo test                                # 158+ handler 测试
cd webui && npx vitest run                # 50+ 页面测试
```

---

## 7. 约束与注意事项

1. **不修改上游**: `../AIRP/`, `../AIRP-MCP-Server/`, `../AIRP-State-Protocol/`, `../tavern2agent/` 绝对不改。
2. **只改 realms-core/**: 所有新代码在 `realms-core/src/server/` 和 `realms-core/webui/src/` 下。
3. **不破坏现有测试**: 每次代码变更后跑 `cargo test` 确认 158 个测试全过。
4. **测试驱动**: 每个新 handler 配 ≥1 个 axum test。
5. **LLM 测试用 MockLlmProvider**: G2 的 turn 处理测试不要调真实 LLM, 用已有的 `MockLlmProvider`。
6. **单文件 < 500 行**: server 代码拆分到 handler/state/mod 三个文件, 不在一个文件里全塞。

---

## 8. 优先级速览

| 顺序 | 任务 | 文件 | 预计 |
|---|---|---|---|
| 1 | G0: config.rs (AppConfig load/save) | `src/server/config.rs` (新建) | 30 min |
| 2 | G0: GET/PUT /api/config + configure_engine | `handlers.rs` | 30 min |
| 3 | G0: SettingsPage.vue (WebUI 设置页) | `webui/src/pages/` (新建) | 30 min |
| 4 | G2: 加 axum 依赖 | `Cargo.toml` | 5 min |
| 5 | G2: 写 server/state.rs (AppState 含 config + pool + paths) | 新建 | 10 min |
| 6 | G2: 写 server/handlers.rs (全部端点) | 新建 | 2-3 hr |
| 7 | G2: 写 server/mod.rs (路由) | 新建 | 15 min |
| 8 | G2: 写 bin/server.rs (入口: load config + init DB + configure_engine + serve) | 新建 | 15 min |
| 9 | 更新 lib.rs 加 pub mod server | 修改 | 1 min |
| 10 | 写 handler 测试 | handlers.rs 底部 | 1 hr |
| 11 | G1: 装 vue-router | npm install | 2 min |
| 12 | G1: 写 router.ts + api.ts | 新建 | 30 min |
| 13 | G1: 写 App.vue (主布局, 顶部 Tab 含设置) | 新建 | 30 min |
| 14 | G1: 写 6 个 Page 组件 | 新建 | 2 hr |
| 15 | G1: 写 GamePlayPage 测试 | 新建 | 30 min |
| 16 | G4: 写 start.sh 一键启动 | start.sh (新建) | 20 min |
| 17 | 端到端验证 (./start.sh → 浏览器 → 设置页填 key → 玩) | 手动 | 15 min |

**总计: ~9-11 小时 (2 天)**
