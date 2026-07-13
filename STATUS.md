# Realms 项目状态

> 最后更新: 2026-07-13, Session #3 (M2 + M3 完成)

## 当前阶段

**M3 ✅ 完成** — 6 个 widget 就绪 (Vue 3 + TypeScript + vitest)

## 已完成

| M# | 内容 | 测试 |
|---|---|---|
| M2.1-M2.11 | GM 编排器完整 (prompt_assembler + gm_router + 4 reducer + event_reducer + pool) | 101 |
| M3.1 | StateWidget (HP/MP bars + location/mood/arc phase/flags) | 6 |
| M3.2 | AffinityWidget (NPC 关系列表 + 好感条) | 4 |
| M3.3 | InventoryWidget (分组物品栏) | 4 |
| M3.4 | ArcTimelineWidget (弧光时间线) | 3 |
| M3.5 | EmotionalMap (情感地图) | 2 |
| M3.6 | ChatWidget (聊天 + 发送 emit) | 5 |
| — | webui 工程骨架 (Vue 3 + Vite + vitest + vue-tsc) | — |

| **合计** | **17 模块** | **101 Rust + 24 Vue = 125 tests** |

## 技术栈

| 层 | 技术 |
|---|---|
| 后端 | Rust (realms-core, rusqlite + r2d2) |
| 前端 | Vue 3 + TypeScript + Vite (realms-core/webui/) |
| 协议 | AIRP-State-Protocol Blueprint (RFC6902 patch) |
| 测试 | cargo test 101 + vitest 24 |

## 待开始

- **M4**: World/Cycle 数据模型 (完整 SQLite schema 替代 M2 测试 DDL)
- M5-M11 ...

## 下次 session 起点

**M4.1**: 完整 `0001_init.sql` schema（替换 M2 的 4 个测试用 DDL）— meta/cycles/character_states/npc_user_relationships/relationship_history/domain_events/3层记忆/FTS5/触发器