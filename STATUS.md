# Realms 项目状态

> 最后更新: 2026-07-13, Session #3 (M2.1-M2.6 + M4.2 完成)

## 当前阶段

**M2 (GM 编排器) 6/11 子任务完成** ✅ gm_router 三重防护就绪, 准备 M2.7 首个 reducer

## 已完成

- [x] Session #1 — 架构/技术栈/工具链/克隆/骨架
- [x] Session #2 — M0 + M1 全部完成
- [x] Session #3
  - [x] 术语统一 DM→GM, `gm_router.rs` 命名收口
  - [x] **M2.1** Cargo 骨架 (RealmsError 10 variant)
  - [x] **M2.2** DomainEvent 9 variant + validate() + is_system_actor()
  - [x] **M4.2** SQLite 连接池 (WAL + foreign_keys)
  - [x] **M2.3** prompt_assembler — 7 段 GM 拼接 (ADR-0004)
  - [x] **M2.4** gm_router 第1重防护 — 独立 context + event filter (ADR-0005)
  - [x] **M2.5** gm_router 第2重防护 — PublicPolicy 叠加过滤 (ADR-0006)
  - [x] **M2.6** gm_router 第3重防护 — RoleCapability 工具白名单
    - 4 项能力 (LookupSelf/Location/Rule + SuggestEvent)
    - NPC 默认无写入能力, 状态变更归口 GM → reducer

## 待开始

- **M2.7** state_reducer — 处理 StateChange, 写 character_states
- **M2.8** relationship_reducer — 处理 RelationshipChange
- **M2.9** arc_reducer — 处理 ArcIncrement + 判定 arc_phase
- **M2.10** memory_reducer — 写 3 层记忆 (episodic/semantic/emotional)
- **M2.11** event_reducer — 主入口串接 9 variant + 单事务
- M3-M11 ...

## 累计测试

| 模块 | 测试数 |
|---|---|
| error.rs | 5 |
| domain_event.rs | 9 (含 is_system_actor) |
| pool.rs | 3 |
| prompt_assembler.rs | 12 (含 1 ignore) |
| gm_router.rs | 27 + 2 doctest |
| lib.rs | 2 |
| **合计** | **59 unit + 2 doctest** |

## 决策记录

见 `docs/adr/` — ADR-0001..0006

## 下次 session 起点

**M2.7 state_reducer**: 处理 `DomainEvent::StateChange`, 写 `character_states` 表 (SQL),
产出 RFC6902 patch. 3+ 单测 (单事务 / 产出 patch).