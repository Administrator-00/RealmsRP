# Realms 项目状态

> 最后更新: 2026-07-13, Session #3 (M2.3 + M2.4 完成)

## 当前阶段

**M2.4 完成** ✅ gm_router 第1重防护 - 独立 context, 准备进入 M2.5 hiddenPublicPolicy

## 已完成

- [x] Session #1 (2026-07-13) — 架构/技术栈/工具链/克隆/骨架
- [x] Session #2 (2026-07-13) — M0 + M1 全部完成 (engine/MCP/webui 连通, 4 subagent 模板)
- [x] Session #3 (2026-07-13)
  - [x] 术语统一 DM→GM, 路由模块命名收口为 `gm_router.rs`
  - [x] **M2.1** Cargo 工程骨架 (RealmsError 10 variant)
  - [x] **M2.2** DomainEvent 9 variant + validate() + AppliedTemplate
  - [x] **M4.2** SQLite 连接池 (内存/文件双模式, WAL+foreign_keys)
  - [x] **M2.3** prompt_assembler — 7 段 GM 拼接核心 (3.md §1.4)
    - PromptContext (7 段 String + arc_phase), 解耦 Cycle/GmPrompt (ADR-0004)
    - 空 GM → DEFAULT_GM_PLACEHOLDER; 长记忆 → 按 char 截断+标记
    - 11 单测 (段顺序/分隔符/GM 段位置/截断/弧光组合)
  - [x] **M2.4** gm_router 第1重防护 — 角色 subagent 独立 context
    - `build_subagent_context()`: 4 段 (你是→世界→情境→玩家), 仅含 NPC 感知内信息
    - `filter_events_for_role()`: 按 actor/target 筛选, Setup/TimeAdvance 环境级公开
    - `dispatch_role_subagent()`: 组合上述 + LLM 桩 (M2 stub, 不含真实 LLM 调用)
    - 14 单测: 2 NPC context 不互泄 / 戒律#6 无 orchestrator noise / persona 原文不动
      / 段顺序 / 事件可见性 (含/不含/混合/环境级)
    - 1 doctest (filter_events_for_role)
  - [x] ADR-0004 (PromptContext 解耦)

## 进行中

- **M2.5**: gm_router 第2重防护 — `PublicPolicy` 三层过滤 (secrets / intent / dm_internal)

## 待开始

- M2.6 tools 白名单 (RoleCapability 枚举)
- M2.7-M2.10: 4 个 reducer
- M2.11: event_reducer 主入口
- M3-M11 ...

## 决策记录

见 `docs/adr/` — ADR-0001..0004

## 已知问题

- 无 LLM API key — M2.4 用桩, M2.11 集成测试时需接入
- 上游服务 (engine/MCP/webui) Session #2 后未重启; M2 纯库开发暂不依赖

## 当前运行中的服务

无 (M2 阶段纯库开发)

## 下次 session 起点

**M2.5**: gm_router — PublicPolicy 三层过滤, 在 `filter_events_for_role` 基础上叠加
`can_see_secrets` / `can_see_other_intent` / `can_see_dm_internal` 策略