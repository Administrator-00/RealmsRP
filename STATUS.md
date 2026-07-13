# Realms 项目状态

> 最后更新: 2026-07-13, Session #2 (M0 完成)

## 当前阶段

**M1 完成** ✅ 4 个 subagent 模板就绪,准备进入 M2 (DM 编排器)

## 已完成

- [x] Session #1 (2026-07-13)
  - [x] 架构设计 (1.md / 2.md / 3.md / README.md / VIBE_CODING.md)
  - [x] 技术栈选型 (5 复用 + 1 胶水层)
  - [x] 工具链验证 (rustc 1.89 / node 26 / sqlite 3.53 / git 2.55)
  - [x] 4 个上游项目克隆 (gh-proxy.com 镜像)
  - [x] realms-core 目录骨架
  - [x] SKILL.md + .gitignore
- [x] Session #2 (2026-07-13) **— M0 + M1 全部完成**
  - [x] M0.1 AIRP engine (PID 16807, :8000)
  - [x] M0.2 MCP-Server (PID 20644, :3001)
  - [x] M0.3 webui (PID 16989, **:9001**)
  - [x] M0.4 端到端连通测试通过
  - [x] M0.5 仙剑示例数据 7 个 .md
  - [x] **M1.1** tavern2agent 软链到 ~/.pi/agent/skills/
  - [x] **M1.2** 4 个 V2 JSON 角色卡 (li_xiaoyao / lin_yueru / zhao_linger / npc_innkeeper),extract_card.py 验证通过
  - [x] **M1.3** 4 个 .pi/agents/*.md subagent 模板 (含 frontmatter + system prompt + 5 重防护约束)
  - [x] **M1.4** verify_agents.py 验证脚本 — 4 个模板全部通过
  - [x] ADR 0001/0002/0003

## 进行中

- 无 (M0 + M1 收尾,待 M2 启动)

## 待开始

- **M2**: DM 编排器 (11 个子任务, ~3 天) — **重头戏**
  - M2.1 Cargo.toml + lib.rs 骨架
  - M2.2 events/domain_event.rs (TypedEvent, 9 个 variant)
  - **M2.3 prompt_assembler.rs (GM 拼接核心, 3.md §1.4)**
  - M2.4-M2.6 dm_router.rs 三重防护
  - M2.7-M2.10 4 个 reducer
  - M2.11 event_reducer.rs 主入口
- M3: 基础 widget
- M4: World/Cycle 数据模型
- ... (M5-M11)

## 决策记录

- 见 `docs/adr/`
- **ADR-0001**: webui 实际端口 9001
- **ADR-0002**: M0 用 debug build
- **ADR-0003**: M1.3 手写 .pi/agents/*.md 而非 pi 嵌套编译
- **ADR-0004** (隐式): 1.md / 2.md 不在"必读"之列,仅按需查 (3.md 是唯一权威)
- **ADR-0005** (隐式): 网络不可达 GitHub,使用 gh-proxy.com 镜像 (Session #1)

## 已知问题

- 无 API key 配 LLM provider — 这是用户责任,需要时给
  - Settings: `POST /v1/settings` 可热重载,见 VIBE_CODING.md §3.1

## 当前运行中的服务

| 服务 | PID | 端口 | 状态 |
|---|---|---|---|
| AIRP engine (debug) | 16807 | 8000 | ✅ |
| MCP-Server (debug) | 20644 | 3001 | ✅ |
| AIRP webui | 16989 | 9001 | ✅ |

## 下次 session 起点

**M2.1**: 创建 realms-core Cargo 项目骨架 (Cargo.toml + lib.rs + error.rs)
