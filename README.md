# Realms · 多世界 RP 沙盒平台

> **选世界 → 选角色 → 选 GM → 开周目 → 玩**

让玩家像 GM 一样**自由构建世界**、扮演**内置 NPC 或自定义 OC**、切换**不同文风**，
同一世界下用**不同身份**体验**多次独立剧情**。

---

## 快速开始

### 前置条件

- **Rust** 1.70+ (`cargo`)
- **Node.js** 18+ (`npm`)
- **LLM API Key**（OpenAI 兼容 / Anthropic）

### 克隆并启动

```bash
git clone git@github.com:Administrator-00/RealmsRP.git
cd RealmsRP
./start.sh
```

> `start.sh` 会自动克隆上游依赖 (AIRP engine 等) 并启动全部服务。

浏览器打开 **http://localhost:5173**：
1. **设置页** — 填写 LLM API Key → 保存
2. **选世界** — 内置"仙剑奇侠传一"
3. **选角色** — 内置 NPC（李逍遥/林月如/赵灵儿）或 创建 OC
4. **选 GM** — 林月如文风 / 古龙武侠 / 诙谐吐槽 / 恐怖氛围
5. **开周目** — 开始玩！

### 手动启动

```bash
# 终端 1：AIRP Engine（LLM 网关，port 8000）
cd AIRP
cargo build -p airp-core
cargo run -p airp-core -- daemon --port 8000

# 终端 2：Realms API Server（胶水层，port 3000）
cd ../realms-core
cargo run --bin realms-server

# 终端 3：WebUI（port 5173）
cd realms-core/webui
npm install
npx vite --port 5173
```

---

## 核心概念

### 🌏 World（世界）

不是"创建一个存档"，而是"**进入一个世界**"：

```
worlds/xiatian_qixia_1/     ← 仙剑奇侠传一
├── setting.md              世界观
├── npcs/                   4 个内置 NPC
├── gms/                    专属 GM 模板
├── lorebook.json           世界书
├── rules.json              规则
└── map.json                地图
```

### 🔄 Cycle（周目）

同一世界可多次独立游玩，每次完全隔离：

```
仙剑奇侠传一
├── Cycle 1：扮演李逍遥，GM 林月如文风
├── Cycle 2：扮演林月如，GM 古龙武侠
└── Cycle 3：扮演 OC "师兄"，GM 诙谐吐槽
```

### 🎨 GM（文风）

GM 就是一段 system_prompt `.md` 文件，切换不影响数据：

```markdown
# gms/style_gulong.md — 古龙武侠
- 短句为主，单句不超过 15 字
- 大量留白和省略号"……"
- 战斗极简，重在氛围和心理
```

内置 4 种文风：**默认** · **林月如**（细腻情感）· **古龙**（短句凌厉）· **恐怖**（悬念压迫）

### 👤 Perspective（扮演视角）

| 类型 | 含义 | 人设来源 | 可编辑 |
|---|---|---|---|
| `source='npc'` | 扮演内置 NPC | npc_base（锁死） | ❌ |
| `source='oc'` | 自定义角色 | 用户输入 | ✅ |

OC 关系模板（快速开局）：青梅竹马 / 一见钟情 / 宿敌 / 师徒 / 萍水相逢。

---

## 架构

```
浏览器 (Vue 3 + Vite, port 5173)
    │ HTTP
    ▼
realms-server (Rust + axum, port 3000)  ← 胶水层
    │
    ├── /api/config      ← 设置页读写 LLM 配置
    ├── /api/worlds      ← 世界列表 / 详情
    ├── /api/gms         ← GM 模板列表
    ├── /api/perspectives ← 视角管理
    ├── /api/cycles      ← 周目 CRUD
    └── /api/cycles/:id/turn ← 单轮对话
    │
    ├── LLM 调用 ──→ AIRP Engine (port 8000) ← 网关层
    │                    │
    ├── SQLite (memory.db)                    ├── /v1/chat/completions
    │   ├── cycles                           ├── /v1/settings (热重载)
    │   ├── character_states                 └── → OpenAI / Anthropic
    │   ├── npc_user_relationships
    │   ├── domain_events（事件溯源）
    │   └── episodic / semantic / emotional memories
    │
    └── 文件系统 (worlds/ gms/ perspectives/)
```

| 层 | 组件 | 角色 |
|---|---|---|
| 前端 | Vue 3 + Vite + 14 widgets | 浏览器 UI |
| 胶水 | **realms-server** (本 repo) | API + cycle 管理 + GM 编排 |
| 网关 | **AIRP Engine** | LLM 调用 + settings 热重载 |
| 数据 | SQLite + 文件系统 | 状态 / 记忆 / 世界数据 |

---

## 测试

```bash
# Rust（177 tests）
cargo test --manifest-path realms-core/Cargo.toml

# Vue（50 tests）
cd realms-core/webui && npx vitest run
```

---

## 项目结构

```
realms-core/
├── src/
│   ├── bin/server.rs           # HTTP server 入口
│   ├── server/
│   │   ├── config.rs           # 配置管理（data/config.json）
│   │   ├── state.rs            # AppState（DB + 路径 + RwLock<AppConfig>）
│   │   ├── handlers.rs         # 全部 API 端点
│   │   └── mod.rs              # axum 路由
│   ├── engine/
│   │   ├── orchestrator/       # prompt_assembler + gm_router + gm_dispatcher
│   │   ├── reducers/           # arc / relationship / memory / state
│   │   └── cycle/              # 周目管理 + OC 初始化
│   ├── db/                     # SQLite schema + pool + 5 查询模块
│   └── events/                 # DomainEvent 类型化枚举
├── webui/
│   ├── src/
│   │   ├── App.vue             # 主布局（顶部 Tab）
│   │   ├── router.ts           # Vue Router 配置
│   │   ├── api.ts              # HTTP API 封装
│   │   ├── widgets/            # 14 个独立组件
│   │   │   ├── ChatWidget      • StateWidget
│   │   │   ├── AffinityWidget  • InventoryWidget
│   │   │   ├── ArcTimeline     • EmotionalMap
│   │   │   ├── WorldSelector   • NpcSelector
│   │   │   ├── OcCreator       • GmSwitcher
│   │   │   ├── PerspectiveSwitcher  • CycleList
│   │   │   ├── CycleCompare    • RelationshipMatrix
│   │   └── pages/              # 7 个路由页面
│   └── index.html
├── worlds/                     # 世界数据（仙剑示例）
├── gms/                        # 全局 GM 模板
├── perspectives/               # 用户视角
├── start.sh                    # 一键启动脚本
└── Cargo.toml
```

---

## License

MIT
