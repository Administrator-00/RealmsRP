# Realms 项目执行手册

> **本文件是给你的（Pi）执行手册**。用户已把架构、技术栈、最终设计全部敲定。
> 你的任务: 按照本手册,从 0 行代码开始,把 Realms RP 平台构建出来。
>
> **用户角色**: 监督者,在关键决策点介入。**不参与每步对话**。
>
> **你的角色**: 自主执行。维护状态、分解任务、debug、写代码、写测试、commit。

---

## 0. 项目背景(必读)

### 0.1 项目定位

**Realms** = 多世界 RP 沙盒平台,**SillyTavern 的上位替代**。

```
SillyTavern:  1 world × 1 chat × 1 user persona × N NPC 共享 prompt(串台)
Realms:       N world × N cycle × N perspective(内置+OC) × N NPC × 隔离 subagent
```

### 0.2 核心概念

| 概念 | 定义 |
|---|---|
| **World** | 顶层主体(仙剑奇侠传一 / 自定义世界),含内置 NPC (性格不可变) + 地图 + 规则 + 世界书 |
| **Cycle (周目)** | 同一 World 的一次独立游玩,有自己的 GM、视角、事件、记忆、关系 |
| **GM** | **一段 system_prompt .md 文件**(文风),唯一用户常改的"风格层",不需 first-class 表 |
| **Perspective** | 用户扮演谁,`source='npc'`(内置不可变) 或 `source='oc'`(自定义) |
| **OC 关系模板** | 7 预置(萍水相逢/一见钟情/青梅竹马/宿敌/师徒/命中注定/无关系),开 cycle 时批量初始化 NPC 关系 |
| **弧光 (Arc)** | 角色在某维度的变化轨迹,由 domain_events 派生,不单独存"当前值" |
| **3 层记忆** | episodic(情景) + semantic(语义) + emotional(情感),带重要性衰减 |
| **5 重防护** | 独立 context / hiddenPublicPolicy / tools 白名单 / domain event 归口 / auditor |

### 0.3 技术栈(5 复用 + 1 胶水层,0 修改上游)

| 组件 | 来源 | 角色 |
|---|---|---|
| **AIRP monorepo** | github.com/GhostXia/AIRP (本地 ../AIRP/) | engine + webui + ui + protocol |
| **AIRP-MCP-Server** | github.com/GhostXia/AIRP-MCP-Server (本地 ../AIRP-MCP-Server/) | RP 数据后端 (38 工具) |
| **AIRP-State-Protocol** | github.com/GhostXia/AIRP-State-Protocol (本地 ../AIRP-State-Protocol/) | Widget 协议 |
| **tavern2agent** | github.com/Xerxes-2/tavern2agent (本地 ../tavern2agent/) | 多 subagent 架构模板 |
| **SQLite (rusqlite)** | embedded | 长期事实库 |
| **自实现胶水层** | `realms-core/` (本仓) | GM 编排器 (~6000 行) |

**绝对不修改**:`../AIRP/`, `../AIRP-MCP-Server/`, `../AIRP-State-Protocol/`, `../tavern2agent/`

**戒律#6 不可破坏**:`subagent_context_has_no_orchestrator_noise` 测试必须永远通过(虽然你不需要写这个测试,但不能破坏 AIRP 现有的)。

### 0.4 必读文档(按顺序读完)

1. `/home/jhwh/src/tmp/tmp/3.md` (40KB) - **最终融合产品设计 (权威)**
2. `/home/jhwh/src/tmp/tmp/README.md` (13KB) - 项目入口
3. `/home/jhwh/src/tmp/tmp/1.md` (8KB) - 原始架构图(理想视图)
4. `/home/jhwh/src/tmp/tmp/2.md` (47KB) - 技术能力详细(参考用,大量细节可借鉴)
5. `/home/jhwh/src/tmp/tmp/STATUS.md` - 当前进度(本仓维护)

---

## 1. 你的工作原则(7 条)

### 1.1 自主性

- **能自己决定的就自己决定**——不要问"要不要加测试"这种问题
- **只在以下情况问用户**:
  - 架构级别变更(改了 3.md)
  - 引入了新的重型依赖
  - 发现 3.md 自相矛盾
  - 卡住超过 30 分钟

### 1.2 顺序性

- 严格按 M0 → M1 → ... → M11 顺序执行
- **不要跳 M**
- 同一个 M 内的子任务可以微调顺序,但不要跳

### 1.3 完整性

- 每个子任务必须有:实现 + 单元测试 + 集成测试(如果跨模块)
- 不要"先实现,后补测试"——**实现完立刻写测试**
- 跑 `cargo test` 通过才算完成

### 1.4 文档同步

- 完成的子任务**立刻**更新 STATUS.md
- 重要决策**立刻**写 ADR (`docs/adr/NNNN-title.md`)
- 不要"最后再补文档"——会忘

### 1.5 简洁性

- 单个文件 < 500 行(超出就拆分)
- pub fn < 50 行(超出就拆 helper)
- 不要 over-engineer

### 1.6 复用优先

- 能用现成 crate 的就用(serde, tokio, rusqlite, thiserror, tracing, chrono)
- 不要自己造轮子(JSON 解析、HTTP 客户端等)
- **绝对不复制上游代码到 realms-core**

### 1.7 安全意识

- 错误用 `thiserror` + `Result<T, RealmsError>`,不用 panic
- 不用 `unwrap()` 在生产代码(测试可以)
- 不用 `unsafe`
- 用户输入要校验

---

## 2. 启动流程(每个 session 必走)

### 2.1 新 session 第一件事

```bash
# 1. 读必读文档
read /home/jhwh/src/tmp/tmp/3.md
read /home/jhwh/src/tmp/tmp/README.md
read /home/jhwh/src/tmp/tmp/STATUS.md

# 2. 检查环境
ls /home/jhwh/src/tmp/    # 应该看到 AIRP/, AIRP-MCP-Server/, AIRP-State-Protocol/, tavern2agent/, tmp/
ls /home/jhwh/src/tmp/tmp/  # 应该看到 1.md, 2.md, 3.md, README.md, VIBE_CODING.md, STATUS.md

# 3. 检查 realms-core
ls /home/jhwh/src/tmp/tmp/realms-core/ 2>/dev/null || echo "realms-core not created yet"
```

### 2.2 自我报告

启动后,**向用户报告**:
```
【Session 启动报告】
- 读完文档: 3.md / README.md / STATUS.md
- 当前阶段: M[X].Y
- 本 session 目标: [从 §6 选]
- 计划步骤: [列出]
- 需要的环境依赖: [列出]
- 预计完成时间: [估]
```

### 2.3 如果是首次启动(0 行代码)

按 §3 执行环境准备。

### 2.4 如果 STATUS.md 显示中断

跳到 STATUS.md "下次 session 起点",继续。

---

## 3. 首次启动:环境准备(用户已做,你要验证)

> **预期**: 用户已按 Part 1 执行,但你要验证一切就绪。

### 3.1 验证清单

```bash
# 1. 工具链
rustc --version    # ≥ 1.70
node --version     # ≥ 18
sqlite3 --version  # ≥ 3.35
git --version      # ≥ 2.30

# 2. 上游项目
ls /home/jhwh/src/tmp/AIRP/
ls /home/jhwh/src/tmp/AIRP-MCP-Server/
ls /home/jhwh/src/tmp/AIRP-State-Protocol/
ls /home/jhwh/src/tmp/tavern2agent/

# 3. 我们的文档
ls /home/jhwh/src/tmp/tmp/*.md
ls /home/jhwh/src/tmp/tmp/realms-core/ 2>/dev/null

# 4. 验证 AIRP 编译
cd /home/jhwh/src/tmp/AIRP && cargo build --release -p airp-core 2>&1 | tail -5
```

### 3.2 缺失处理

| 缺失项 | 你的动作 |
|---|---|
| 工具链缺 | 告诉用户安装命令 |
| 上游项目缺 | 告诉用户 git clone 命令 |
| realms-core 缺 | 你来创建(`mkdir -p`) |
| 编译失败 | 贴错误给用户,等指示 |

### 3.3 初始化 realms-core 目录

```bash
cd /home/jhwh/src/tmp/tmp
mkdir -p realms-core/{engine,db,webui,worlds,gms,perspectives}
mkdir -p realms-core/engine/{orchestrator,reducers,events,cycle}
mkdir -p realms-core/db/{migrations,queries}
mkdir -p realms-core/webui/{widgets,components,pages,utils}
mkdir -p docs/adr

# 创建 SKILL.md(下次 session 自动加载)
# 内容见 §3.4
```

### 3.4 创建 realms-core/SKILL.md

```markdown
# Realms RP 平台 Skill Guide

## 必读
- /home/jhwh/src/tmp/tmp/3.md (权威设计)
- /home/jhwh/src/tmp/tmp/README.md
- /home/jhwh/src/tmp/tmp/VIBE_CODING.md (本仓执行手册)

## 硬性约束
1. **不修改** ../AIRP/ ../AIRP-MCP-Server/ ../AIRP-State-Protocol/ ../tavern2agent/
2. 只在 realms-core/ 写代码
3. 戒律#6 不可破坏
4. 不引入新重型依赖(用现有: serde/tokio/rusqlite/thiserror/tracing/chrono)
5. 每个 pub fn 配 doc comment + 测试
6. 错误用 Result<T, RealmsError>, 不用 panic/unwrap/unsafe

## 常用命令
- cargo test -p realms-core
- cargo run -p realms-core --bin dev-server
- cd ../AIRP && cargo run -p airp-core -- daemon --port 8000
- cd ../AIRP-MCP-Server && ./target/release/airp-mcp serve --bind 127.0.0.1:3001 --data-dir ./data
- cd ../AIRP/webui && node serve.js

## 数据目录
- realms-core/data/memory.db (SQLite, FTS5, WAL)
- 跟 ../AIRP-MCP-Server/data/ 错开

## 端口分配
- AIRP engine: 8000
- MCP-Server: 3001
- webui: 5173
- Tauri 桌面 (可选): 1420
- realms-core 自有服务(后期): 8001
```

### 3.5 创建 STATUS.md (如果用户没创建)

```markdown
# Realms 项目状态

> 最后更新: [ISO date], [session 主题]

## 当前阶段
M0 - 基础环境验证

## 已完成
- [x] 架构设计 (1.md/2.md/3.md/README.md/VIBE_CODING.md)
- [x] 技术栈选型

## 进行中
- [ ] M0 - 基础环境验证

## 待开始
- M1 角色卡编译
- M2 GM 编排器
- ...

## 决策记录
- 见 docs/adr/

## 下次 session 起点
M0 完成后,继续 M1
```

---

## 4. 工作流(每个子任务的标准循环)

### 4.1 单任务 6 步循环

```
1. 设计 (Design)
   └ 自己列函数签名 + 数据结构 + 边界 + 错误处理
2. 骨架 (Skeleton)
   └ 写函数签名 + TODO 注释 + 测试空壳
   └ cargo build 确认编译
3. 实现 (Implement)
   └ 逐函数填实现
   └ 每函数写完 cargo test
4. 集成 (Integrate)
   └ 接入已有模块
   └ cargo test 全跑
5. 自检 (Self-Check)
   └ cargo clippy
   └ cargo fmt
   └ 覆盖率检查
6. 收尾 (Wrap-up)
   └ 更新 STATUS.md
   └ git add + commit
   └ 写 ADR(如果重要决策)
```

### 4.2 任务启动 checklist

开始任何子任务前,确认:
- [ ] 已读 3.md 对应章节
- [ ] 已看类似代码风格(如果有)
- [ ] 已列出函数签名(写在 prompt 里)
- [ ] 已列出测试用例
- [ ] 知道 DoD(完成定义)

### 4.3 任务完成 checklist

- [ ] 所有 fn 实现
- [ ] 所有 fn 配 doc comment
- [ ] 所有 fn 配 ≥ 1 测试
- [ ] cargo test 通过
- [ ] cargo clippy 无 warning
- [ ] cargo fmt 已跑
- [ ] STATUS.md 更新
- [ ] git commit (格式见 §9.2)
- [ ] ADR 写(如果改了架构/选了重要技术)

---

## 5. M0-M11 执行剧本(详细)

> **每个 M 阶段有**: 目标 / 子任务 / 每个子任务的目标+步骤+DoD / 验收 / 出错处理

---

### M0: 基础环境验证(0.5 天)

#### 目标
确认 4 个上游项目能跑通,3 个服务能连通。

#### 子任务

##### M0.1 启动 AIRP engine

**目标**: `cargo run -p airp-core -- daemon --port 8000` 成功,/health 返回 ok

**步骤**:
```bash
cd /home/jhwh/src/tmp/AIRP
# 后台启动,日志输出到文件
nohup cargo run -p airp-core -- daemon --port 8000 > /tmp/engine.log 2>&1 &
sleep 10
curl -s http://localhost:8000/health
```

**DoD**:
- [ ] curl 返回 `{"status":"ok"}` 或类似
- [ ] `tail /tmp/engine.log` 没 panic/error
- [ ] 进程 PID 记录到 STATUS.md

**出错处理**:
- 编译失败 → cargo update 重试
- 端口占用 → lsof -i :8000 查谁
- panic → 贴日志问用户

##### M0.2 启动 MCP-Server

**步骤**:
```bash
cd /home/jhwh/src/tmp/AIRP-MCP-Server
mkdir -p ./data
nohup ./target/release/airp-mcp serve --bind 127.0.0.1:3001 --data-dir ./data > /tmp/mcp.log 2>&1 &
sleep 3
curl -s http://localhost:3001/health
```

**DoD**: curl 返回 ok

##### M0.3 启动 webui

**步骤**:
```bash
cd /home/jhwh/src/tmp/AIRP/webui
nohup node serve.js > /tmp/webui.log 2>&1 &
sleep 3
curl -s http://localhost:5173/ | head -5
```

**DoD**: 浏览器(playwright/headless chrome)能打开

##### M0.4 端到端连通测试

**步骤**:
- webui 连 engine
- engine 调 MCP (用 list_characters 测试)
- webui 导入 1 个简单角色卡
- 发 1 条消息收到回复(可 mock)

**DoD**: 全链路通

##### M0.5 仙剑示例数据预置(最小版)

**目标**: 仙剑世界最小可用数据集

**步骤**:
```bash
mkdir -p realms-core/worlds/xiatian_qixia_1/npcs
mkdir -p realms-core/worlds/xiatian_qixia_1/gms
mkdir -p realms-core/gms
```

**文件**(每文件 100-200 字即可,后续 M11 扩展):
- `worlds/xiatian_qixia_1/setting.md` - 仙剑世界简介
- `worlds/xiatian_qixia_1/npcs/li_xiaoyao.md` - 李逍遥
- `worlds/xiatian_qixia_1/npcs/lin_yueru.md` - 林月如
- `worlds/xiatian_qixia_1/npcs/zhao_linger.md` - 赵灵儿
- `worlds/xiatian_qixia_1/npcs/npc_innkeeper.md` - 客栈老板
- `worlds/xiatian_qixia_1/gms/style_linyueru.md` - 林月如文风
- `gms/default.md` - 默认 GM

**格式参考 3.md §1.1**

**DoD**: 7 个文件存在,内容 ≥ 100 字

#### M0 验收

- [ ] 3 个服务在跑
- [ ] 端到端连通测试通过
- [ ] 仙剑数据 7 个文件存在
- [ ] STATUS.md 标记 M0 完成
- [ ] git commit

---

### M1: 角色卡编译(0.5 天)

#### 目标
用 tavern2agent 把 4 个 ST 格式角色卡编译为 .pi/agents/*.md

#### 子任务

##### M1.1 安装 tavern2agent

**步骤**:
- 读 `/home/jhwh/src/tmp/tavern2agent/README.md` 找安装方式
- 通常是 `npx skills add Xerxes-2/tavern2agent` 或软链
- 验证: `pi --list-skills` 看到 tavern2agent

**DoD**: tavern2agent skill 可用

##### M1.2 准备 4 个 ST 格式角色卡

**方案**(选最容易的):
- **方案 A(最快)**: 直接创建 4 个 JSON 角色卡(绕过 PNG)
  - 格式: V2 角色卡 JSON
  - 字段: name, description, personality, first_mes, example_dialogs, creator_notes
  - 最小可工作: name + description + first_mes(每字段 50-100 字)
- **方案 B**: 从 SillyTavern 社区下载 4 个 PNG
- **方案 C**: 用工具生成(略复杂)

**推荐方案 A**(最快,无外部依赖)。

**DoD**: 4 个 .json 或 .png 文件在 `realms-core/.cards/`

##### M1.3 编译角色卡

**步骤**:
- 对每个角色卡,启动 pi 让它编译
- 输出: `realms-core/.pi/agents/{name}.md`

**DoD**: 4 个 .pi/agents/*.md 存在,每个含 frontmatter (name, description, tools) + system prompt

##### M1.4 验证模板

**步骤**: 启动 pi,说"列出所有 agent 模板",看到 4 个

**DoD**: 4 个模板可被加载

#### M1 验收

- [ ] tavern2agent 安装
- [ ] 4 个角色卡文件
- [ ] 4 个 .pi/agents/*.md
- [ ] 验证加载成功
- [ ] STATUS.md 标记 M1 完成
- [ ] git commit

---

### M2: GM 编排器(3 天)

#### 目标
实现核心编排器: prompt_assembler + gm_router + 4 reducer + event_reducer 串起来

#### 子任务(11 个,严格按顺序)

##### M2.1 Cargo.toml + lib.rs 骨架

**目标**: realms-core 可编译 + 跑空测试

**Cargo.toml** (放在 realms-core/):
```toml
[package]
name = "realms-core"
version = "0.1.0"
edition = "2021"

[lib]
name = "realms_core"
path = "src/lib.rs"

[dependencies]
tokio = { version = "1", features = ["full"] }
rusqlite = { version = "0.32", features = ["bundled", "json1", "fts5"] }
r2d2 = "0.8"
r2d2_sqlite = "0.25"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
chrono = { version = "0.4", features = ["serde"] }
async-trait = "0.1"

[dev-dependencies]
tempfile = "3"
```

**src/lib.rs**:
```rust
pub mod engine;
pub mod db;
pub mod error;
pub mod events;

pub use error::RealmsError;
```

**src/error.rs**:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RealmsError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("internal: {0}")]
    Internal(String),
}
```

**DoD**:
- [ ] cargo build 成功
- [ ] cargo test 成功(空测试 fn it_works)
- [ ] cargo clippy 无 warning

##### M2.2 events/domain_event.rs (TypedEvent)

**目标**: 定义所有事件类型

**结构**:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainEvent {
    Setup { /* 字段 */ },
    ArcIncrement { /* 字段 */ },
    RelationshipChange { /* 字段 */ },
    StateChange { /* 字段 */ },
    ItemChange { /* 字段 */ },
    TimeAdvance { /* 字段 */ },
    Dialogue { /* 字段 */ },
    Discovery { /* 字段 */ },
    Combat { /* 字段 */ },
}

impl DomainEvent {
    pub fn validate(&self) -> Result<(), RealmsError> { /* 校验 */ }
}
```

**字段定义参考 3.md §2.2 domain_events 表**

**DoD**:
- [ ] 9 个 variant 全实现
- [ ] validate() 函数校验必填字段
- [ ] 5+ 单元测试(每个 variant 至少 1 个,validate 边界测试)

##### M2.3 prompt_assembler.rs (GM 拼接核心!)

**目标**: 按 3.md §1.4 机制组装 system_prompt

**结构**:
```rust
pub struct PromptContext {
    pub perspective_persona: String,    // 角色人设
    pub world_setting: String,          // 世界观
    pub gm_content: String,             // ← 唯一用户可改的"风格层"
    pub current_state: String,          // 当前状态
    pub arc_context: String,            // 弧光上下文
    pub relevant_memories: String,      // 记忆检索
    pub world_rules: String,            // 规则
}

pub fn build_system_prompt(ctx: &PromptContext) -> String { /* 7 段拼接 */ }
```

**拼接顺序(严格按 3.md §1.4)**:
1. 角色人设
2. 世界观
3. GM 文风(用户可改)
4. 当前状态
5. 弧光
6. 记忆
7. 规则

**用 `\n\n---\n\n` 分隔**

**DoD**:
- [ ] build_system_prompt 实现
- [ ] 4+ 单元测试:
  - 7 段顺序正确
  - GM 段位置正确
  - 空 GM 用占位符
  - 超长记忆截断
  - arc_phase 出现
- [ ] 写一个 mock GM 读测试
- [ ] cargo test 通过

**这是核心,先做!**

##### M2.4-M2.6 gm_router.rs 三重防护

**M2.4 独立 context**:

```rust
pub struct RoleDispatchRequest {
    pub cycle_id: String,
    pub npc_id: String,
    pub user_message: String,
    pub visible_context: Vec<DomainEvent>,
}

pub struct RoleDispatchResult {
    pub visible_response: String,
    pub private_intent: String,
    pub suggested_events: Vec<DomainEvent>,
}

pub async fn dispatch_role_subagent(req: RoleDispatchRequest) -> Result<RoleDispatchResult, RealmsError> {
    // 1. 读 npc_base.persona(锁死,不可变)
    // 2. 读 world.setting(过滤秘密)
    // 3. 过滤 visible_context(只留角色该看的)
    // 4. 拼装 subagent context(只含以上 4 块)
    // 5. 调 LLM
    // 6. 解析输出为 RoleDispatchResult
}
```

**DoD**:
- [ ] 5+ 测试(2 角色 subagent 不互相泄漏)
- [ ] 验证 subagent context 不含协调器变量(模拟戒律#6)

**M2.5 hiddenPublicPolicy**:

```rust
pub struct PublicPolicy {
    pub can_see_secrets: bool,
    pub can_see_other_intent: bool,
    pub can_see_dm_internal: bool,
}

pub fn filter_events_for_role(events: &[DomainEvent], policy: &PublicPolicy) -> Vec<DomainEvent> {
    // 默认 policy: 全部 false
}
```

**DoD**:
- [ ] 默认 policy 三 false
- [ ] 3+ 测试(秘密/内心独白/GM 内部都被过滤)

**M2.6 tools 白名单**:

```rust
pub enum RoleCapability {
    LookupSelf,
    LookupLocation,
    LookupRule,
    SuggestEvent,
}

// 角色 subagent 启动时 tools = [allowed capabilities]
```

**DoD**:
- [ ] 默认 NPC 只有 lookup + suggest
- [ ] 2+ 测试(不能调用 update_state)

##### M2.7 - M2.10 4 个 reducer

每个 reducer 模式相同:
```rust
pub fn apply_xxx_change(pool: &DbPool, event: &DomainEvent) -> Result<XxxPatch, RealmsError> {
    // 1. 开事务
    // 2. 处理 event
    // 3. 写 SQLite
    // 4. 产出 RFC6902 patch
    // 5. commit
}
```

- **M2.7 state_reducer**: 处理 StateChange,写 character_states
- **M2.8 relationship_reducer**: 处理 RelationshipChange,写 npc_user_relationships + relationship_history
- **M2.9 arc_reducer**: 处理 ArcIncrement,写 character_states.arcs + 判定 arc_phase
- **M2.10 memory_reducer**: 处理各种 event,写 3 层记忆(episodic/semantic/emotional)

**DoD(每个 reducer)**:
- [ ] 3+ 测试
- [ ] 单事务
- [ ] 产出 patch

##### M2.11 event_reducer.rs (主入口,串起来)

**目标**: 一个函数分发所有 event 到对应 reducer,全在大事务中

```rust
pub async fn process_event(pool: &DbPool, event: DomainEvent) -> Result<Vec<PatchOp>, RealmsError> {
    let mut tx = pool.begin()?;
    let mut all_patches = vec![];
    
    // 1. 写 domain_events 表
    tx.execute("INSERT INTO domain_events ...", ...)?;
    
    // 2. 根据 event.type 分发
    match event {
        DomainEvent::StateChange { .. } => {
            let patches = state_reducer::apply_with_tx(&mut tx, &event)?;
            all_patches.extend(patches);
        }
        DomainEvent::RelationshipChange { .. } => { /* ... */ }
        DomainEvent::ArcIncrement { .. } => { /* ... */ }
        // ...
    }
    
    // 3. 自动写 episodic_memories(每个 event 都生成记忆)
    let memory_patches = memory_reducer::auto_record(&mut tx, &event)?;
    all_patches.extend(memory_patches);
    
    tx.commit()?;
    Ok(all_patches)
}
```

**DoD**:
- [ ] 9 个 variant 全部处理
- [ ] 集成测试: 一次 process_event 触发多个 reducer,全部成功
- [ ] 事务原子性验证(中途失败回滚)

#### M2 验收

- [ ] 11 个子任务全完成
- [ ] cargo test 全通过
- [ ] cargo clippy 无 warning
- [ ] cargo fmt
- [ ] commit: "feat(orchestrator): M2 - GM 编排器 + 4 reducer + 三重防护"
- [ ] STATUS.md 标记 M2 完成

---

### M3: 基础 widget(2-3 天)

#### 子任务

```
M3.1 StateWidget
M3.2 AffinityWidget
M3.3 InventoryWidget
M3.4 ArcTimelineWidget
M3.5 EmotionalMap
M3.6 ChatWidget (可能 airp 已有,确认即可)
```

#### 每个 widget 的标准结构

```vue
<!-- webui/widgets/StateWidget.vue -->
<script setup lang="ts">
import { ref, computed } from 'vue';
import type { CharacterState } from '@/types';

const props = defineProps<{
  state: CharacterState;
}>();

const hpPercent = computed(() => 
  props.state.hp_max > 0 ? (props.state.hp / props.state.hp_max) * 100 : 0
);
</script>

<template>
  <div class="state-widget">
    <div class="hp-bar">
      <div class="hp-fill" :style="{ width: hpPercent + '%' }"></div>
      <span>HP: {{ state.hp }} / {{ state.hp_max }}</span>
    </div>
    <div class="info">
      <div>位置: {{ state.location_id }}</div>
      <div>情绪: {{ state.perceived_mood }}</div>
    </div>
  </div>
</template>

<style scoped>
.state-widget { padding: 12px; }
.hp-bar { /* ... */ }
/* 响应式: @media (min-width: 768px) { ... } */
/* 触屏: button { min-height: 44px; } */
</style>
```

**每个 widget 的 DoD**:
- [ ] .vue 文件存在
- [ ] TypeScript 类型定义
- [ ] 响应式 3 断点
- [ ] 触屏按钮 ≥44px
- [ ] 接入 AIRP-State-Protocol Blueprint
- [ ] 1+ vitest 测试

#### M3 验收

- [ ] 6 个 widget
- [ ] 每个 ≥1 测试
- [ ] 响应式
- [ ] commit

---

### M4: World/Cycle 数据模型(1.5 天)

#### 子任务

```
M4.1 db/migrations/0001_init.sql (完整 schema)
M4.2 db/pool.rs
M4.3 db/migration.rs
M4.4 db/queries/cycle.rs
M4.5 db/queries/character_state.rs
M4.6 db/queries/event.rs
M4.7 db/queries/relationship.rs
M4.8 cycle/cycle_manager.rs
M4.9 cycle/world_loader.rs
M4.10 cycle/gm_loader.rs
```

#### M4.1 SQL schema (完整)

**要求**: 完整实现 3.md §2.2 所有表

**核心表**:
- meta (schema_version)
- cycles (cycle_id, world_id, perspective_id, gm_id, cycle_name, current_world_time, ...)
- character_states (cycle_id, npc_id, arcs JSON, hp, mp, location_id, arc_phase, ...)
- npc_user_relationships (cycle_id, npc_id, affinity, trust, ..., initial_template, ...)
- relationship_history
- domain_events (cycle_id, event_type, actor_id, ..., arc_dimension, delta, importance, ...)
- episodic_memories / semantic_memories / emotional_memories
- items / locations
- arc_dimensions
- FTS5 虚表(events_fts, episodic_fts, semantic_fts)
- 触发器(FTS 同步)

**参考**: 3.md §2.2 完整 SQL(200+ 行)

**DoD**:
- [ ] SQL 文件可执行
- [ ] migration runner 跑两次不出错(idempotent)
- [ ] schema 跟 3.md §2.2 一致
- [ ] 3+ 测试

#### M4.2 pool.rs

```rust
pub type DbPool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

pub fn init(data_dir: &Path) -> Result<DbPool> {
    std::fs::create_dir_all(data_dir)?;
    let db_path = data_dir.join("memory.db");
    let manager = SqliteConnectionManager::file(&db_path)
        .with_init(|c| {
            c.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
        });
    let pool = r2d2::Pool::builder().max_size(8).build(manager)?;
    crate::db::migration::run(&pool)?;
    Ok(pool)
}
```

**DoD**:
- [ ] init() 实现
- [ ] WAL 模式
- [ ] 外键开启
- [ ] 1 测试(并发访问)

#### M4.4 cycle_manager.rs (核心 CRUD)

```rust
pub async fn create_cycle(pool: &DbPool, world_id: &str, perspective_id: &str, gm_id: &str, cycle_name: &str) -> Result<Cycle, RealmsError>;
pub async fn load_cycle(pool: &DbPool, cycle_id: &str) -> Result<Cycle, RealmsError>;
pub async fn list_cycles(pool: &DbPool, world_id: &str) -> Result<Vec<Cycle>, RealmsError>;
pub async fn switch_cycle(pool: &DbPool, cycle_id: &str) -> Result<(), RealmsError>;
pub async fn duplicate_cycle(pool: &DbPool, cycle_id: &str, new_name: &str) -> Result<Cycle, RealmsError>;
pub async fn delete_cycle(pool: &DbPool, cycle_id: &str) -> Result<(), RealmsError>;
```

**DoD**:
- [ ] 6 个 fn 全实现
- [ ] 6 个测试
- [ ] duplicate 复制所有 cycle 数据
- [ ] delete 级联删除(单事务)

#### M4 验收

- [ ] 10 个子任务完成
- [ ] SQL 跟 3.md 一致
- [ ] cycle CRUD 全部工作
- [ ] 复制/删除事务原子性
- [ ] commit

---

### M5: NPC/OC/GM 数据层(1.5 天)

#### 子任务

```
M5.1 cycle/oc_initializer.rs (产品创新点!)
M5.2 source='npc' 视角锁死处理
M5.3 source='oc' 视角正常处理
M5.4 跨周目共享/隔离验证
M5.5 perspective CRUD
```

#### M5.1 oc_initializer.rs

**目标**: 7 个关系模板,开 cycle 时批量初始化

```rust
pub struct OcRelationshipSetup {
    pub oc_perspective_id: String,
    pub templates: HashMap<String, String>,  // npc_id -> template_id
}

pub async fn initialize_cycle_with_templates(
    pool: &DbPool,
    cycle_id: &str,
    setup: OcRelationshipSetup,
) -> Result<(), RealmsError> {
    // 1. 读所有模板(从 arc_dimensions 或新表 oc_relationship_templates)
    // 2. 对每个 npc, 批量 INSERT npc_user_relationships
    // 3. INSERT domain_events: event_type='setup'
}
```

**7 个预置模板**:
- tpl_stranger (萍水相逢) - affinity 0, trust 0, intimacy 0, type 'stranger'
- tpl_love_at_first_sight (一见钟情) - 70/30/50, type 'lover'
- tpl_childhood_friend (青梅竹马) - 60/80/70, type 'close_friend'
- tpl_sworn_enemy (宿敌) - -80/10/0, type 'enemy'
- tpl_master_student (师徒) - 40/90/50, type 'master'
- tpl_destined (命中注定) - 50/50/50, type 'destined'
- tpl_no_relation (无关系) - 全部 NULL/空

**DoD**:
- [ ] 7 模板的初始值正确
- [ ] 7+ 测试(每模板一个)
- [ ] 集成测试: 创建 OC 选 3 模板,验证 3 个 NPC 关系被初始化
- [ ] commit

---

### M6: 并入 M5(预置模板验证)

---

### M7: GM Router 完整(2-3 天)

#### 子任务

```
M7.1 完整 dispatch(M2.4-2.6 集成)
M7.2 visibleResponse + privateIntent 协议
M7.3 suggestedEvents 收集 + GM 决策
M7.4 auditor subagent
M7.5 集成测试: 完整单轮流程
```

#### M7.5 集成测试(必须通过)

```rust
// tests/full_cycle_test.rs
#[tokio::test]
async fn test_full_single_turn() {
    // 1. 创建仙剑 world
    // 2. 创建 OC "师兄"
    // 3. 创建 cycle, 选 [青梅竹马, 萍水相逢, 一见钟情, 萍水相逢]
    // 4. user 发消息: "我走进余杭镇"
    // 5. 验证: domain_events +1
    // 6. 验证: npc_user_relationships 按模板
    // 7. 验证: character_states 有初始值
    // 8. 验证: episodic_memories 创建
    // 整个测试 < 5 秒
}
```

**DoD**:
- [ ] 集成测试通过
- [ ] < 5 秒
- [ ] commit

---

### M8: 跨周目对比(1 天)

#### 子任务

```
M8.1 SQL: 同 NPC 跨 cycle
M8.2 SQL: 同 cycle 跨 NPC 矩阵
M8.3 webui: CycleCompare.vue
M8.4 webui: RelationshipMatrix.vue
```

**DoD**:
- [ ] 2 SQL 正确
- [ ] 2 widget 渲染
- [ ] 截图(可选)

---

### M9: WebUI 完整(3-4 天)

#### 子任务

```
M9.1 WorldSelector.vue
M9.2 NpcSelector.vue
M9.3 OcCreator.vue (3 步式向导,产品创新)
M9.4 GmSwitcher.vue
M9.5 PerspectiveSwitcher.vue
M9.6 CycleList.vue
M9.7 响应式 CSS 整理
M9.8 PWA manifest + service worker
M9.9 SSE 断线重连
M9.10 Notification API
```

#### M9.3 OcCreator 详细

**3 步式向导**:
- Step 1: 填 OC 基本信息(name, background, personality, avatar)
- Step 2: 选关系模板(对每个内置 NPC 选 7 模板之一)
- Step 3: 选 GM(列出 gms/*.md,预览)

**进度条 + 步骤间校验 + 触屏友好**

**DoD**:
- [ ] 3 步流程跑通
- [ ] 步骤校验
- [ ] 响应式
- [ ] 3+ vitest 测试

---

### M10: 质量保证(2-3 天)

#### 子任务

```
M10.1 串台检测 (50 轮 × 100 次)
M10.2 arc 跳变检测
M10.3 reducer 覆盖率 > 90%
M10.4 集成测试覆盖
M10.5 auditor subagent 完整
M10.6 benchmark (100 万行 < 50ms)
M10.7 端到端 demo
```

#### M10.1 串台检测(必须通过)

```rust
// tests/crosstalk_test.rs
#[tokio::test]
async fn test_no_crosstalk_in_100_runs() {
    let mut failures = 0;
    for _ in 0..100 {
        // 50 轮对话, 4 NPC
        // 跑完检查每个 NPC 的人设
        // 不能说成其他 NPC 的话
        // 不能知道其他 NPC 的 private_intent
    }
    assert_eq!(failures, 0, "串台 {} 次", failures);
}
```

**DoD**: 0 串台

---

### M11: 仙剑完整 demo(2 天)

#### 子任务

```
M11.1 setting.md (完整 2000 字)
M11.2 npcs/*.md (4 个, 每个 1500 字)
M11.3 lorebook.json (50 条)
M11.4 map.json (20 个地点)
M11.5 rules.json (战斗 + 修炼)
M11.6 style_linyueru.md GM
M11.7 4 个全局 GM (gulong, humor, horror, default)
M11.8 pov_npc_*.json (4 个内置视角)
M11.9 pov_oc_*.json (3 个 OC demo)
M11.10 跑 3 个完整 cycle demo
M11.11 README 截图
```

**DoD**:
- [ ] 4 NPC 完整人设
- [ ] 5 GM 模板
- [ ] 7 OC 关系模板(从 M5)
- [ ] 3 cycle demo
- [ ] README 更新
- [ ] commit "feat: M11 - 仙剑完整 demo"

**MVP 完成!**

---

## 6. 代码规范(强制)

### 6.1 Rust

```rust
//! 模块级 doc comment
//! 说明这个文件做什么

// imports 按 std / 3rd / crate
use std::sync::Arc;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::db::pool::DbPool;
use crate::error::RealmsError;

/// 公开函数必须有 doc comment
/// 
/// # Errors
/// 
/// 当 world_id 不存在时返回 `RealmsError::NotFound`
pub async fn create_cycle(
    pool: &DbPool,
    world_id: &str,
) -> Result<Cycle, RealmsError> {
    // ...
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_cycle_ok() {
        // ...
    }
}
```

**规则**:
- ✅ 每个 pub fn 有 doc comment
- ✅ 每个 pub fn 配 ≥1 测试
- ✅ 错误用 `thiserror`,不用 String/panic
- ❌ 不用 `unwrap()` 在生产代码
- ❌ 不用 `unsafe`
- ❌ 不引新依赖,除非必要

### 6.2 TypeScript/Vue

```typescript
// 函数式优先
export interface CycleConfig {
  cycleId: string;
  worldId: string;
}

export async function createCycle(config: CycleConfig): Promise<Cycle> {
  // ...
}

// 错误用 discriminated union
export type CycleError =
  | { kind: 'NotFound'; cycleId: string }
  | { kind: 'Database'; cause: string };
```

```vue
<script setup lang="ts">
const props = defineProps<{ cycle: Cycle }>();
const emit = defineEmits<{ select: [id: string] }>();
</script>

<template>
  <button @click="emit('select', cycle.id)">{{ cycle.name }}</button>
</template>

<style scoped>
button { min-height: 44px; padding: 8px 16px; }
@media (min-width: 768px) { /* tablet */ }
@media (min-width: 1024px) { /* PC */ }
</style>
```

### 6.3 SQL

```sql
-- Migration: 0003
-- Description: add user_perspectives.source
-- Up: ...
ALTER TABLE user_perspectives ADD COLUMN source TEXT NOT NULL DEFAULT 'oc';
UPDATE meta SET value = '3' WHERE key = 'schema_version';
-- Down: ALTER TABLE user_perspectives DROP COLUMN source;
```

---

## 7. 测试策略

### 7.1 覆盖率目标

| 模块 | 目标 |
|---|---|
| reducer | 100% |
| prompt_assembler | 90% |
| gm_router | 80% |
| cycle_manager | 80% |
| oc_initializer | 90% |
| widget | 60% |

### 7.2 单元测试模式

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn setup_test_db() -> DbPool {
        let pool = r2d2::Pool::builder()
            .max_size(2)
            .build(r2d2_sqlite::SqliteConnectionManager::memory())
            .unwrap();
        // 跑 migrations
        crate::db::migration::run(&pool).unwrap();
        pool
    }
    
    #[tokio::test]
    async fn test_xxx() {
        let pool = setup_test_db();
        // ...
    }
}
```

### 7.3 集成测试

放 `tests/` 目录,文件名 `*_test.rs`:
- `tests/full_cycle_test.rs` - 端到端
- `tests/crosstalk_test.rs` - 串台检测
- `tests/arc_test.rs` - 弧光跳变

---

## 8. Git 工作流

### 8.1 分支策略

```
main (稳定, 永远 cargo test 通过)
├── feat/M0-env
├── feat/M2-orchestrator
│   ├── M2.1-skeleton
│   ├── M2.2-events
│   └── ...
```

**main 规则**:
- 永远 `cargo build` + `cargo test` 通过
- 每 M 阶段 merge 一次
- tag: v0.1.0-m0, v0.2.0-m1, ...

### 8.2 Commit 信息

```bash
<type>(<scope>): <subject>

<body>

Refs: 3.md §X.Y
```

**type**: feat / fix / refactor / test / docs / chore
**scope**: orchestrator / reducers / db / webui / cycle

**示例**:
```bash
git commit -m "feat(orchestrator): add prompt_assembler with 7-section composition

- Implement build_system_prompt with persona+world+gm+state+arc+memory+rules
- GM section is the only user-editable 'style layer'
- Add 4 unit tests

Refs: 3.md §1.4
Co-Authored-By: Claude <noreply@anthropic.com>"
```

### 8.3 .gitignore

```gitignore
target/
node_modules/
data/
*.db
*.db-*
.env
.vscode/
*.log
nohup.out
```

---

## 9. 状态管理

### 9.1 STATUS.md 模板

每次 session 结束,更新:

```markdown
# Realms 项目状态

> 最后更新: [ISO date], [session 主题]

## 当前阶段
M[X].Y (子任务名) - 状态

## 已完成
- [x] M0 (2026-07-12) - 基础环境验证
- [x] M1 (2026-07-12) - 角色卡编译
- [x] M2.1 (2026-07-13) - Cargo 骨架
- [x] M2.2 (2026-07-13) - DomainEvent
- [x] M2.3 (2026-07-14) - prompt_assembler

## 进行中
- [ ] M2.4 (90%) - gm_router 独立 context

## 待开始
- M2.5 - hiddenPublicPolicy
- M2.6 - tools 白名单
- ...

## 决策记录
- 见 docs/adr/
- [2026-07-14] prompt_assembler 用 7 段拼接, 不动态改段数

## 已知问题
- #1 ...

## 下次 session 起点
M2.5 gm_router hiddenPublicPolicy, 从这里继续
```

### 9.2 ADR 模板

每次重要决策,写 `docs/adr/NNNN-title.md`:

```markdown
# ADR NNNN: [决策标题]

## Context
[背景, 什么问题需要决策]

## Options
[候选方案, ≥ 2 个]

## Decision
[选了哪个, 简短理由]

## Consequences
[影响, + 优点 - 缺点]

## Status
[Accepted / Superseded by NNNN]
```

---

## 10. 出错处理(自主决策树)

### 10.1 cargo build 失败

```bash
1. cargo clean
2. cargo update
3. cargo build 2>&1 | tail -50
4. 分析错误
   - 依赖问题 → 改 Cargo.toml 或加 version
   - 类型问题 → 修代码
   - 路径问题 → 检查 ../AIRP 等
5. 如果自己搞不定,问用户
```

### 10.2 cargo test 失败

```bash
1. cargo test test_name -- --nocapture
2. 看输出
3. 分析 root cause(不瞎试)
4. 修
5. 跑全测试确认
```

### 10.3 上游编译失败(你没改上游)

```bash
# 这是上游的问题,不是你的问题
1. git log ../AIRP 看最新 commit
2. 看上游 issue
3. 必要时:在 realms-core 用 wrapper 绕开
4. 或: 提 issue 给上游
5. 如果阻塞 → 问用户
```

### 10.4 webui 跑不起来

```bash
1. 看 /tmp/webui.log
2. 检查端口冲突
3. 重启
4. 如果是上游问题,问用户
```

### 10.5 卡住超过 30 分钟

**强制**:
1. 停下来
2. 重新读 3.md 相关章节
3. 看类似代码
4. 列出"我已知"和"我未知"
5. 如果关键未知,问用户(简短提问)
6. 如果只是实现细节,自己想办法(简化方案)

### 10.6 发现自己引入新依赖

**强制停下**:
1. 评估能否用现有依赖实现
2. 评估依赖的维护性/成熟度
3. 如果确实需要,**告诉用户,等同意**
4. 不需要就改用现有依赖

### 10.7 发现自己要改上游

**立即停止**:
1. 不能改
2. 改在 realms-core/ 加 wrapper
3. 或问用户是否可以提 PR

---

## 11. 用户介入的判断标准

### 11.1 必须问用户(等指示)

- 架构级别变更(改了 3.md)
- 引入新重型依赖
- 上游 bug 阻塞
- 跨 M 阶段决策
- 用户主动询问进度

### 11.2 自己决定(不需问)

- 内部命名
- 代码组织
- 测试用例设计
- 错误信息措辞
- 中间步骤优化
- 调试技巧

### 11.3 默认行为(用户不在时)

按本手册执行,出错按 §10 处理,**只在必须时**打断用户。

---

## 12. 自我监督(每个 session 结束)

### 12.1 收尾 checklist

```bash
# 1. 全跑测试
cd realms-core && cargo test 2>&1 | tail -20

# 2. clippy
cargo clippy --all-targets -- -D warnings 2>&1 | tail -20

# 3. fmt
cargo fmt --check

# 4. git status
git status

# 5. 写 commit
git add -A
git commit -m "..."

# 6. 更新 STATUS.md

# 7. 向用户报告
```

### 12.2 给用户的报告格式

```
【Session 完成报告】
- 完成子任务: M[X].Y (名字)
- 耗时: X 小时
- 产出:
  - 文件: [list]
  - 测试: X 个 pass
  - commit: [hash] [message]
- 决策: [列出,含 ADR 链接]
- 问题: [如果有]
- 下次起点: M[X].Z
```

---

## 13. 立即开始

如果你已经读完 §0-§12,环境已就绪,**从 M0 开始**。

如果你是首次启动(0 行代码),先执行 §3。

任何问题,先看 §10 出错处理,再问用户。

🚀 开始构建 Realms!
