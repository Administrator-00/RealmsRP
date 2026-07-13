# Realms · 仙剑 RP 平台

> **多世界 RP 沙盒 · SillyTavern 上位替代**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Status: MVP](https://img.shields.io/badge/status-MVP-yellow.svg)]()
[![Stack: Rust+TS](https://img.shields.io/badge/stack-Rust%20%2B%20TypeScript-orange.svg)]()

让玩家像 GM 一样**自由构建世界**、**定义 NPC**、**创建自定义角色**,在同一世界下用**不同身份和文风**体验**多次独立剧情**。

---

## 一句话定位

**SillyTavern 是单线程 chat 工具,Realms 是多世界 RP 沙盒平台。**

| 维度 | SillyTavern | Realms |
|---|---|---|
| **世界** | 1 个 (角色卡 = 世界) | **N 个** World,各自独立世界观/地图/规则/内置 NPC |
| **游玩单位** | 1 chat | **N 个 Cycle (周目)**,同世界可多次独立游玩 |
| **角色** | 1 user persona + N NPC (串台) | **N Perspective** (内置 NPC 不可变 / OC 自定义) + 关系模板快速开局 |
| **记忆** | 关键词 lorebook + 手动 summary | **3 层记忆模型** (episodic/semantic/emotional) + FTS5 + 衰减 |
| **角色成长** | ❌ 静态卡片 | ✅ **弧光增量** (事件溯源 + 状态投影) |
| **GM (文风)** | ⚠️ preset 太弱 | ✅ **一段 system_prompt .md** (切换不改数据) |
| **多端** | 桌面客户端 | **PC + 手机** 响应式 webui + PWA |
| **实时变量** | HTML 状态栏 (易碎) | ✅ reducer 流水线 + RFC6902 patch |

---

## 核心特性

### 1. 🌏 World 优先 (世界是顶层主体)

不是"创建一个新存档",而是"**进入一个世界**":

```
worlds/
├── xiatian_qixia_1/        ← 仙剑奇侠传一
│   ├── setting.md          世界观
│   ├── map.json            地图
│   ├── rules.json          规则
│   ├── lorebook.json       世界书
│   ├── npcs/               内置 NPC (性格不可变)
│   └── gms/                该世界专属 GM 模板
├── xiatian_qixia_2/        ← 仙剑二
└── my_custom_world/        ← 用户自定义世界
```

### 2. 🔄 Cycle (周目) = 同一世界多次独立游玩

```
仙剑奇侠传一
├── Cycle 1: 扮演李逍遥,GM 林月如文风
├── Cycle 2: 扮演林月如,GM 古龙武侠
├── Cycle 3: 扮演 OC "师兄",GM 诙谐吐槽
└── Cycle 4: 扮演李逍遥,GM 林月如文风 (走不同分支)

→ 每个 Cycle 独立: 事件/记忆/关系/弧光
→ 跨 Cycle 对比 / 复制 / 分享
```

### 3. 🎨 GM (Game Master) = 一段 system_prompt

**GM 不是 first-class 实体,就是一段文风 system_prompt .md 文件**。

```markdown
<!-- gms/style_linyueru.md -->
# 林月如文风
你是一位细腻情感向的叙事者:
- 句子长度中等 (15-30 字)
- 大量使用通感和环境描写
- 角色对话带轻微娇嗔和试探
- 战斗以"心理压力"和"权衡"为主
```

切换 GM = 选一个不同的 .md 文件 → **不影响 cycle 数据,只影响后续生成的文风**。

### 4. 💕 OC 关系模板 (快速开局)

创建自定义角色 (OC) 时,为每个内置 NPC 选一个**关系模板**——**不再靠 LLM 在第一回合自创关系**。

| 模板 | 初始好感 | 信任 | 亲密 | initial_type |
|---|---:|---:|---:|---|
| 萍水相逢 | 0 | 0 | 0 | stranger |
| **一见钟情** | 70 | 30 | 50 | lover |
| **青梅竹马** | 60 | 80 | 70 | close_friend |
| 宿敌 | -80 | 10 | 0 | enemy |
| 师徒 | 40 | 90 | 50 | master |
| 命中注定 | 50 | 50 | 50 | destined |
| 无关系 | — | — | — | — |

### 5. 🛡️ 内置 NPC 性格不可变

- 世界作者定义的 NPC 性格**锁死**,所有 cycle 共享
- 玩家**可以扮演**内置 NPC (source='npc'),但不能改他们性格
- 玩家也可以创建 OC (source='oc'),**自由编辑**
- 跨 World 可复用同一 perspective

### 6. 🧠 5 重防护 (多人物不串台)

SillyTavern 群聊官方明文警告 "characters being confused about themselves, having merged personalities"。Realms 解决:

1. **独立 context** — 每个角色 subagent 全新 system prompt (戒律#6 测试守护)
2. **hiddenPublicPolicy** — DM 派发前过滤"角色不该看的信息"
3. **工具白名单** — 角色 subagent 只能 lookup + suggest_event,不能 update_state
4. **domain event 归口** — 所有状态变更走 reducer
5. **auditor subagent** — 异步检查"是否知道太多/秘密是否串层"

### 7. 📚 超长记忆 (三层模型 + FTS5)

```
L1 短期: chat.jsonl     (最近 20-50 条)
L2 中期: volume/*.md   (current.md + vol_XXX.md + index.md)
L3 长期: SQLite + FTS5  (episodic / semantic / emotional)
                         + 衰减 (vividness × importance × access_count)
```

### 8. 📱 PC + 手机 双端 webui

- 3 断点响应式: 手机 (<768px) / 平板 (≥768px) / PC (≥1024px)
- 触屏: 按钮 ≥44px (iOS HIG)
- PWA: 装到手机主屏 + 离线 + 推送
- 通知: 状态变化 / 弧光里程碑推送

---

## 架构

```
┌──────────────────────────────────────────────────────────────────┐
│  ① 前端层 (双形态 GUI)                                            │
│  webui (浏览器原生,PC+手机 PWA) + ui (Tauri+Vue 桌面)             │
│  widgets: WorldSelector / CycleList / OcCreator /                │
│           NpcSelector / GmSwitcher / PerspectiveSwitcher /        │
│           RelationshipMatrix / ArcTimeline / EmotionalMap / ...   │
└──────────────────────────────┬───────────────────────────────────┘
                               │ HTTPS + WSS + Envelope
┌──────────────────────────────▼───────────────────────────────────┐
│  ② 网关层 = AIRP engine (Rust daemon, axum)                      │
│  /v1/chat/completions · /v1/agent/run · /v1/settings (热重载)     │
│  + prompt_assembler (GM 拼接核心)                                  │
└──────────────────────────────┬───────────────────────────────────┘
                               │ HTTP
┌──────────────────────────────▼───────────────────────────────────┐
│  ③ 数据层 (双源)                                                   │
│  ┌───────────────────────┐  ┌──────────────────────────────────┐  │
│  │ AIRP-MCP-Server       │  │ SQLite memory.db                 │  │
│  │ 文件:                 │  │ - cycles (周目)                  │  │
│  │ - worlds/             │  │ - character_states               │  │
│  │ - perspectives/       │  │ - npc_user_relationships         │  │
│  │ - gms/                │  │ - domain_events (事件溯源)        │  │
│  │ - chat.jsonl (短期)   │  │ - 3 层记忆 (episodic/semantic/  │  │
│  │ - volume/*.md (中期)  │  │   emotional)                     │  │
│  │ - plugin 命名空间     │  │ - items / locations              │  │
│  │                       │  │ - FTS5 全文检索                  │  │
│  └───────────────────────┘  └──────────────────────────────────┘  │
└──────────────────────────────┬───────────────────────────────────┘
                               │
┌──────────────────────────────▼───────────────────────────────────┐
│  ④ 核心引擎层 = 胶水层 (自实现, ~6000 行)                          │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ DM 编排器: prompt_assembler + dm_router + 5 重防护         │  │
│  │ Reducer 流水线: arc / relationship / memory / state / event │  │
│  │ Cycle 管理: 创建/加载/切换/复制/对比                         │  │
│  │ 角色 subagent: tavern2agent 模板 + AIRP 戒律#6              │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘

外部: LLM Provider (OpenAI 兼容 + Anthropic) + 可选向量库 (sqlite-vec)
```

---

## 技术栈: 5 复用 + 1 胶水层

**0 修改上游**,只组合现成项目。

| 组件 | 来源 | 角色 | 复用度 |
|---|---|---|:---:|
| **AIRP monorepo** | [GhostXia/AIRP](https://github.com/GhostXia/AIRP) | engine + webui + ui + protocol | 0 修改 |
| **AIRP-MCP-Server** | [GhostXia/AIRP-MCP-Server](https://github.com/GhostXia/AIRP-MCP-Server) | RP 数据后端 (38 工具) | 0 修改 |
| **AIRP-State-Protocol** | [GhostXia/AIRP-State-Protocol](https://github.com/GhostXia/AIRP-State-Protocol) | Widget 协议 (Blueprint + RFC6902 patch) | 0 修改 |
| **tavern2agent** | [Xerxes-2/tavern2agent](https://github.com/Xerxes-2/tavern2agent) | 多 subagent 架构模板 | 0 修改 |
| **SQLite (via rusqlite)** | embedded | 长期事实库 (ACID + FTS5 + JSON1) | 0 集成 |
| **(自实现) DM 编排器** | this repo | 胶水层 (~6000 行) | — |

**为什么这套组合**:
- AIRP engine — 唯一已落地的 Rust HTTP agent loop (M_AGENT-1, **299 passing tests**),戒律#6 是测试守护的代码级不变式
- AIRP-MCP-Server — 唯一 RP 数据后端,plugin 命名空间支持自定义存档元数据
- AIRP-State-Protocol — 唯一支持 Blueprint 模式的 widget 协议,RFC6902 patch 增量同步
- tavern2agent — 唯一明确"subagent 不写 state"宪法的项目,fate-sandbox 13 时间线已生产验证
- SQLite — 嵌入/单文件/ACID/FTS5,对单机 RP 足 99% 场景

**为什么不自己造**:
- Pi — 缺 widget / MCP / 多端共享
- SillyTavern 群聊 — 官方明文警告会串台
- SillyTavern preset — 缺 first-class GM 维度

---

## 快速开始

### 前置条件

- Rust 1.70+ (`cargo`)
- Node.js 18+ (`npm`)
- Python 3.9+ (tavern2agent)
- LLM API key (OpenAI 兼容 / Anthropic)

### 1. 克隆所有依赖

```bash
mkdir realms && cd realms

# 4 个核心上游项目
git clone https://github.com/GhostXia/AIRP.git
git clone https://github.com/GhostXia/AIRP-MCP-Server.git
git clone https://github.com/GhostXia/AIRP-State-Protocol.git
git clone https://github.com/Xerxes-2/tavern2agent.git

# Realms 本体 (胶水层)
git clone https://github.com/your-org/realms-core.git
```

### 2. 启动 AIRP engine

```bash
cd AIRP
cargo build --release -p airp-core

# 启动 daemon (默认 :8000)
cargo run -p airp-core -- daemon --port 8000
```

### 3. 启动 MCP-Server

```bash
cd ../AIRP-MCP-Server
cargo build --release
./target/release/airp-mcp serve \
    --bind 127.0.0.1:3001 \
    --data-dir ./data
```

### 4. 启动 webui

```bash
cd ../AIRP/webui
node serve.js
# 浏览器打开 http://localhost:5173
```

### 5. 配置 LLM Provider

```bash
# 编辑 AIRP/data/settings.json
{
  "provider": "OpenAI",
  "endpoint": "https://api.openai.com/v1/chat/completions",
  "api_key": "sk-...",
  "model": "gpt-4o"
}

# 或运行时热重载
curl -X POST http://localhost:8000/v1/settings \
  -H "Content-Type: application/json" \
  -d '{"api_key": "sk-..."}'
```

### 6. 打开 webui,选世界

浏览器访问 `http://localhost:5173`,选择"仙剑奇侠传一",创建 OC 或扮演内置 NPC,选 GM,**开新周目**。

---

## 文档导航

| 文档 | 角色 |
|---|---|
| **[README.md](README.md)** (本文件) | 项目入口 |
| **[1.md](1.md)** | 原始架构图 (理想视图) |
| **[2.md](2.md)** | 技术能力详细文档 (含 webui/Pi 对比) |
| **[3.md](3.md)** | **最终融合产品设计 (权威)** |

**3.md 优先**: 新决策以 3.md 为准,1.md/2.md 作为历史参考。

---

## 路线图

| 阶段 | 时间 | 关键产出 | 状态 |
|---|---|---|:---:|
| M0 基础验证 | 0.5 天 | engine + webui + MCP-Server 跑通 | 🔜 |
| M1 角色卡编译 | 0.5 天 | tavern2agent 编译仙剑角色卡 | 🔜 |
| M2 DM 编排器 | 2-3 天 | dm_router + reducers + prompt_assembler | 🔜 |
| M3 基础 widget | 2-3 天 | Chat / State / Affinity / Inventory / ArcTimeline | 🔜 |
| M4 World/Cycle 数据模型 | 1 天 | SQLite schema + 文件系统布局 | 🔜 |
| M5 NPC/OC/GM 数据层 | 1.5 天 | perspective + npc_base + gm_prompts 管理 | 🔜 |
| M6 OC 关系模板 | 1 天 | 模板 + 初始化流程 | 🔜 |
| M7 DM_Router 完整 | 2-3 天 | 角色 subagent + 5 重防护 + reducer 完整 | 🔜 |
| M8 跨周目对比 | 1 天 | SQL + widget | 🔜 |
| M9 WebUI 完整 | 3-4 天 | 7 个新增 widget + 响应式 + PWA | 🔜 |
| M10 质量保证 | 2-3 天 | 串台/弧光跳变检测 + 审计 | 🔜 |
| M11 仙剑完整 demo | 2 天 | 1 World + 4 NPC + 3 OC + 5 GM + 3 Cycle | 🔜 |

**完整 MVP 预计 18-24 天 (单人)**。

---

## 与 SillyTavern 详细对比

| 能力 | SillyTavern | Realms |
|---|:---:|:---:|
| 多 World (独立世界观/地图/规则) | ❌ | ✅ |
| 多周目 (cycle) 同世界独立 | ⚠️ 多 chat | ✅ 产品级 |
| 角色成长 (弧光) | ❌ | ✅ 事件溯源 |
| 跨 chat/cycle 记忆延续 | ❌ | ✅ SQLite 投影 |
| OC 关系模板 (快速开局) | ❌ | ✅ 7 预置 |
| 内置 NPC 性格锁死 | ❌ | ✅ |
| GM (system_prompt) 维度 | ⚠️ preset 太弱 | ✅ first-class |
| 多人物不串台 | ❌ 官方警告 | ✅ 5 重防护 |
| PC + 手机 webui | ⚠️ 第三方 | ✅ 响应式 + PWA |
| 实时变量 (HP/好感/物品) | ⚠️ HTML 状态栏 | ✅ reducer + RFC6902 patch |
| 跨周目对比/分享 | ❌ | ✅ |
| 插件/MCP 生态 | ✅ 社区大 | ⚠️ 早期 |
| 桌面 GUI | ✅ 强 | ✅ Tauri (可选) |
| TUI | ✅ 强 | ❌ 浏览器 only |
| 部署门槛 | 中 (Node + 后端) | 中 (Rust + Node) |

---

## 仙剑示例世界 (内置)

Realms 默认带 **仙剑奇侠传一** 示例世界,开箱即用:

```
worlds/xiatian_qixia_1/
├── setting.md              "神州浩土,仙魔纷争..."
├── map.json                余杭镇 / 苏州 / 锁妖塔 / 圣姑巢穴 / ...
├── rules.json              战斗 / 修炼 / 等级
├── lorebook.json           仙剑世界观/门派/技能
├── npcs/
│   ├── li_xiaoyao.md       李逍遥 (内置,不可变)
│   ├── lin_yueru.md        林月如
│   ├── zhao_linger.md      赵灵儿
│   └── npc_innkeeper.md    客栈老板
└── gms/
    └── style_linyueru.md   林月如文风
```

预置 **5 个 GM** 模板:
- 林月如文风 (细腻情感)
- 古龙武侠 (短句凌厉)
- 诙谐吐槽 (现代幽默)
- 恐怖氛围 (悬念压力)
- 默认 (中性)

预置 **7 个 OC 关系模板**:
- 萍水相逢 / 一见钟情 / 青梅竹马 / 宿敌 / 师徒 / 命中注定 / 无关系

---

## 项目结构

```
realms/
├── AIRP/                    # 复用: engine + webui + ui + protocol
├── AIRP-MCP-Server/         # 复用: RP 数据后端
├── AIRP-State-Protocol/     # 复用: widget 协议
├── tavern2agent/            # 复用: 多 subagent 架构模板
│
├── realms-core/             # 🆕 本项目: 胶水层
│   ├── engine/
│   │   ├── orchestrator/
│   │   │   ├── prompt_assembler.rs    # GM 拼接核心
│   │   │   ├── dm_router.rs           # 角色调度
│   │   │   └── cycle_manager.rs       # 周目管理
│   │   ├── reducers/
│   │   │   ├── arc_reducer.rs         # 弧光
│   │   │   ├── relationship_reducer.rs
│   │   │   ├── memory_reducer.rs
│   │   │   ├── state_reducer.rs
│   │   │   └── event_reducer.rs
│   │   ├── events/
│   │   │   └── domain_event.rs        # TypedEvent
│   │   └── cycle/
│   │       └── oc_initializer.rs      # OC 初始化
│   ├── db/
│   │   ├── migrations/                # SQLite schema
│   │   ├── queries/                   # 预编译查询
│   │   └── pool.rs
│   ├── worlds/
│   │   └── xiatian_qixia_1/           # 仙剑示例
│   ├── gms/                           # 全局 GM 模板
│   ├── perspectives/                  # 用户视角
│   └── webui/
│       ├── widgets/
│       │   ├── WorldSelector.vue
│       │   ├── CycleList.vue
│       │   ├── OcCreator.vue          # 3 步式向导
│       │   ├── NpcSelector.vue
│       │   ├── GmSwitcher.vue
│       │   ├── PerspectiveSwitcher.vue
│       │   ├── RelationshipMatrix.vue
│       │   ├── CycleCompare.vue
│       │   ├── ChatWidget.vue
│       │   ├── StateWidget.vue
│       │   ├── AffinityWidget.vue
│       │   ├── InventoryWidget.vue
│       │   ├── ArcTimelineWidget.vue
│       │   └── EmotionalMap.vue
│       └── ...
│
├── docs/                    # 设计文档
│   ├── 1.md                 # 原始架构图
│   ├── 2.md                 # 技术能力详细
│   ├── 3.md                 # 最终融合设计
│   └── adr/                 # 架构决策记录
│
└── README.md                # 本文件
```

---

## 致谢

Realms 站在以下项目肩膀上:

- **[AIRP monorepo](https://github.com/GhostXia/AIRP)** by GhostXia — engine daemon + webui + Tauri 桌面 + 协议
- **[AIRP-MCP-Server](https://github.com/GhostXia/AIRP-MCP-Server)** by GhostXia — RP 数据后端 (38 工具/19 资源/12 prompt)
- **[AIRP-State-Protocol](https://github.com/GhostXia/AIRP-State-Protocol)** by GhostXia — Widget Registry + Blueprint + RFC6902 patch
- **[tavern2agent](https://github.com/Xerxes-2/tavern2agent)** by Xerxes-2 — 多 subagent 架构 (subagent 不写 state 宪法)
- **[fate-sandbox](https://github.com/lolo-s-Cosmos/fate-sandbox)** — tavern2agent 生产验证 (Type-Moon 13 时间线)

特别感谢 **AIRP 戒律#6** `subagent_context_has_no_orchestrator_noise` 测试,这是"多人物不串台"的代码级不变式。

---

## 参与贡献

欢迎 PR! 重点方向:
- **新 GM 模板** (`.md` 文件) — 不同文风/规则
- **新 World** — 仙剑/古龙/金庸/原创
- **新 widget** — 用 AIRP-State-Protocol 的 Blueprint 协议扩展
- **跨周目对比功能** — 玩家爱看
- **质量保证** — 串台检测 / 弧光跳变检测

**协议**: 不修改上游项目,只在本项目 `realms-core/` 加胶水层代码 + 配置文件 + widget。

---

## License

本项目采用 **MIT License**。

上游项目各自许可证:
- AIRP: MIT OR Apache-2.0
- tavern2agent: (待查)
- SillyTavern: AGPL-3.0 (本项目不基于 ST 源码)

---

## Star History

如果这个项目对你有帮助,请 ⭐️ star 一下,这是我们持续开发的动力。

---

> **Realms** — 让每一个世界都有自己的 GM
