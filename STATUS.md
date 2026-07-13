# Realms 项目状态

> 最后更新: 2026-07-13, Session #3 (M2.1 + M2.2 + M2.3 + M4.2 完成)

## 当前阶段

**M2.3 完成** ✅ prompt_assembler GM 拼接核心就绪,准备进入 M2.4 gm_router 三重防护

## 已完成

- [x] Session #1 (2026-07-13)
  - [x] 架构设计 (1.md / 2.md / 3.md / README.md / VIBE_CODING.md)
  - [x] 技术栈选型 (5 复用 + 1 胶水层)
  - [x] 工具链验证 (rustc 1.89 / node 26 / sqlite 3.53 / git 2.55)
  - [x] 4 个上游项目克隆 (gh-proxy.com 镜像)
  - [x] realms-core 目录骨架
  - [x] SKILL.md + .gitignore
- [x] Session #2 (2026-07-13) — M0 + M1 全部完成
  - [x] M0.1 AIRP engine (PID 16807, :8000)
  - [x] M0.2 MCP-Server (PID 20644, :3001)
  - [x] M0.3 webui (PID 16989, **:9001**)
  - [x] M0.4 端到端连通测试通过
  - [x] M0.5 仙剑示例数据 7 个 .md
  - [x] M1.1-M1.4 tavern2agent + 4 角色卡 + 4 .pi/agents/*.md + 验证脚本
  - [x] ADR 0001/0002/0003
- [x] Session #3 (2026-07-13) — **M2.1 / M2.2 / M2.3 / M4.2 完成**
  - [x] 术语统一 DM→GM (3.md / README / VIBE_CODING), 路由模块命名收口为 `gm_router.rs`
  - [x] M2.1 Cargo 工程骨架 (Cargo.toml + lib.rs + error.rs)：10 variant `RealmsError`
  - [x] M2.2 DomainEvent 9 variant + validate() + EventType 往返 + `AppliedTemplate` 子结构
  - [x] M4.2 SQLite 连接池 `pool.rs`（内存/文件双模式, WAL + foreign_keys, migrations 钩子留空待 M4.1）
  - [x] **M2.3 prompt_assembler (3.md §1.4 GM 拼接核心)**
    - 7 段顺序: 角色 → 世界观 → 文风指引(GM) → 当前状态 → 弧光 → 相关记忆 → 规则
    - `\n\n---\n\n` 分隔,空 GM 填 `DEFAULT_GM_PLACEHOLDER`,超长记忆按 char 截断+标记
    - 31 个单测全部通过,clippy `-D warnings` 干净
    - 决策: 独立 `PromptContext`(7 段 String), 不直接耦合未落地的 `Cycle`/`GmPrompt` (ADR-0004)
  - [x] ADR-0004 (PromptContext 解耦决策)

## 进行中

- 无 (M2.3 收尾)

## 待开始

- **M2.4-M2.6**: gm_router.rs 三重防护 (独立 context / hiddenPublicPolicy / tools 白名单)
- **M2.7-M2.10**: 4 个 reducer (state / relationship / arc / memory)
- **M2.11**: event_reducer 主入口 (串起 9 variant, 单事务)
- M3: 基础 widget (6 个 .vue)
- M4: World/Cycle 数据模型 (M4.1 SQL schema, M4.3-M4.10 其余 CRUD)
- M5-M11 ...

## 决策记录

- 见 `docs/adr/`
- **ADR-0001**: webui 实际端口 9001
- **ADR-0002**: M0 用 debug build
- **ADR-0003**: M1.3 手写 .pi/agents/*.md 而非 pi 嵌套编译
- **ADR-0004**: prompt_assembler 用独立 PromptContext 解耦 Cycle/GmPrompt
- (隐式) ADR: 1.md / 2.md 不在"必读"之列,仅按需查 (3.md 唯一权威)
- (隐式) ADR: 网络不可达 GitHub,使用 gh-proxy.com 镜像

## 已知问题

- 无 API key 配 LLM provider — 用户责任,需要时给
  - Settings: `POST /v1/settings` 可热重载,见 VIBE_CODING.md §3.1
- 4 个上游服务在 Session #2 后未再启动;若 M2 之后要做端到端连通 (M2.11 集成测试),需重启 engine/MCP/webui

## 当前运行中的服务

| 服务 | PID | 端口 | 状态 |
|---|---|---|---|
| AIRP engine (debug) | 16807 | 8000 | ⚠️ Session #2 残留,可能已退出 |
| MCP-Server (debug) | 20644 | 3001 | ⚠️ 同上 |
| AIRP webui | 16989 | 9001 | ⚠️ 同上 |

> 当前 session 未验服务存活,M2 阶段纯库开发,暂不依赖实时服务.

## 下次 session 起点

**M2.4**: gm_router.rs 第 1 重防护 — 角色 subagent 独立 context
(读 npc_base.persona 锁死, 过滤 visible_context, 拼装纯净 subagent context, 模拟戒律#6 测试)