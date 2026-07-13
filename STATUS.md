# Realms 项目状态

> 最后更新: 2026-07-13, Session #3 (M2.3-M2.5 完成)

## 当前阶段

**M2.5 完成** ✅ gm_router 前两重防护就绪, 准备 M2.6 (tools 白名单)

## 已完成

- [x] Session #1 (2026-07-13) — 架构/技术栈/工具链/克隆/骨架
- [x] Session #2 (2026-07-13) — M0 + M1 全部完成
- [x] Session #3 (2026-07-13)
  - [x] 术语统一 DM→GM, 命名收口 `gm_router.rs`
  - [x] **M2.1** Cargo 骨架 (RealmsError 10 variant)
  - [x] **M2.2** DomainEvent 9 variant + validate() + AppliedTemplate + is_system_actor()
  - [x] **M4.2** SQLite 连接池
  - [x] **M2.3** prompt_assembler (7 段 GM 拼接, PromptContext, ADR-0004)
  - [x] **M2.4** gm_router 第1重防护: 独立 context (build_subagent_context + filter_events_for_role)
    - 14 单测 + 1 doctest (ADR-0005)
  - [x] **M2.5** gm_router 第2重防护: hiddenPublicPolicy
    - `PublicPolicy` 三层可配置 (can_see_secrets / can_see_other_intent / can_see_dm_internal)
    - `filter_events_with_policy()` 叠加过滤 (M2.4 基础 + M2.5 policy 放行)
    - DomainEvent::is_system_actor() 辅助方法
    - 9 单测 + 1 doctest
  - [x] ADR-0004 (PromptContext 解耦), ADR-0005 (gm_router pure functions)

## 进行中

- 无

## 待开始

- **M2.6**: tools 白名单 (RoleCapability 枚举)
- M2.7-M2.10: 4 个 reducer
- M2.11: event_reducer 主入口
- M3-M11 ...

## 决策记录

见 `docs/adr/` — ADR-0001..0005

## 累计测试

| 模块 | 测试数 |
|---|---|
| error.rs | 5 |
| domain_event.rs | 8 + 1 is_system_actor |
| pool.rs | 3 |
| prompt_assembler.rs | 12 (含 1 ignore) |
| gm_router.rs | 23 + 2 doctest |
| lib.rs | 2 |
| **合计** | **55 unit + 2 doctest** |

## 已知问题

- 无 LLM API key — M2.4 dispatch 用桩, M2.11 集成测试时需接入
- 上游服务 (engine/MCP/webui) Session #2 后未重启; M2 纯库开发暂不依赖

## 下次 session 起点

**M2.6**: gm_router — `RoleCapability` 枚举 (LookupSelf / LookupLocation / LookupRule / SuggestEvent),
限制角色 subagent 只能 lookup + suggest, 不能 update_state. 2+ 测试.