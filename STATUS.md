# Realms 项目状态

> 最后更新: 2026-07-13, Session #3 (**M2 完成！11/11**)

## 当前阶段

**M2 ✅ 完成** — GM 编排器全部就绪, 准备 M3 基础 widget 或 M4 数据模型

## 已完成

| M# | 模块 | 测试 |
|---|---|---|
| M2.1 | Cargo 骨架 + RealmsError | 5 |
| M2.2 | DomainEvent 9 variant | 9 |
| M2.3 | prompt_assembler (7 段 GM 拼接) | 12 |
| M2.4 | gm_router 独立 context | 15 |
| M2.5 | PublicPolicy 叠加过滤 | 10 |
| M2.6 | RoleCapability 工具白名单 | 4 |
| M2.7 | state_reducer | 14 |
| M2.8 | relationship_reducer | 8 |
| M2.9 | arc_reducer | 11 |
| M2.10 | memory_reducer | 5 |
| M2.11 | event_reducer 主入口 | 5 |
| M4.2 | SQLite pool | 3 |
| — | 术语同步 + 修复 | — |

| **合计** | **12 个模块** | **101 unit + 1 doc** |

## 架构摘要

```
event_reducer::process_event(pool, event)
  │
  ├→ 写 domain_events 行
  ├→ 分发子 reducer (match event_type):
  │   ├ StateChange      → state_reducer     → character_states
  │   ├ RelationshipChange → relationship_reducer → npc_user_relationships + history
  │   └ ArcIncrement     → arc_reducer       → arcs JSON + arc_phase
  ├→ auto_record memories (episodic + emotional)
  └→ 单事务提交, 返回 Vec<PatchOp>
```

## 决策记录

见 `docs/adr/` — ADR-0001..0006

## 待开始

- M3: 基础 widget (6 个 .vue)
- M4: World/Cycle 数据模型 (SQLite schema + CRUD)
- M5-M11 ...

## 下次 session 起点

**M3.1 StateWidget** 或 **M4.1 SQL schema** — 视优先级而定.
建议 M4 先补齐完整 schema (M2 的 DDL 都是测试用最小版), 再上 M3 widget.