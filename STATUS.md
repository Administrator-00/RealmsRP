# Realms 项目状态

> 最后更新: 2026-07-13, Session #3 (M2 进度 7/11)

## 当前阶段

**M2.7 完成** ✅ state_reducer 就绪, 准备 M2.8 relationship_reducer

## 已完成

- [x] M2.1-M2.6: Cargo 骨架 + DomainEvent + pool + prompt_assembler + gm_router 三重防护
- [x] **M2.7** state_reducer — 处理 StateChange, 写 character_states
  - 6 字段: hp/mp (i64) / location_id / arc_phase / mood / status_flag (JSON merge)
  - UPSERT 模式: INSERT OR IGNORE 默认值 → UPDATE 指定字段
  - 产出 PatchOp (RFC6902 replace, JSON Pointer 路径)
  - 10 单元测试 (CRUD + 错误) + 4 merge_json_flags 纯函数测试
- [x] PatchOp 共享类型 (reducers/mod.rs), CHARACTER_STATES_DDL 测试用 schema

## 待开始

- **M2.8** relationship_reducer — RelationshipChange → npc_user_relationships + history
- **M2.9** arc_reducer — ArcIncrement + 判定 arc_phase
- **M2.10** memory_reducer — 3 层记忆写入
- **M2.11** event_reducer — 主入口串接 9 variant + 单事务
- M3-M11 ...

## 累计测试

| 模块 | 测试数 |
|---|---|
| error / domain_event / pool / lib | 19 |
| prompt_assembler.rs | 12 |
| gm_router.rs | 27 + 2 doctest |
| state_reducer.rs | 14 |
| **合计** | **73 unit + 2 doctest** |

## 决策记录

见 `docs/adr/` — ADR-0001..0006

## 下次 session 起点

**M2.8 relationship_reducer**: 处理 RelationshipChange, 写 npc_user_relationships +
relationship_history. 3+ 单测 (单事务 / 产出 patch / history 追加).