# Realms 项目状态

> 最后更新: 2026-07-13, Session #3 (M2 + M3 + M4.1-M4.4 完成)

## 当前阶段

**M4 进度 4/10** — 完整 schema + migration runner + cycle CRUD 就绪

## 已完成

| M# | 内容 | 测试 |
|---|---|---|
| M2.1-M2.11 | GM 编排器 | 101 |
| M3.1-M3.6 | Vue widgets | 24 |
| **M4.1** | `0001_init.sql` 完整 schema (11 表 + FTS5 + 触发器) | — |
| M4.2 | SQLite pool (WAL + foreign_keys) | 3 |
| **M4.3** | Migration runner (幂等, include_str! 嵌入) | 3 |
| **M4.4** | Cycle CRUD (create/load/list/touch/delete 级联) | 5 |

| **合计** | **21 模块** | **109 Rust + 24 Vue = 133 tests** |

## 待开始

- M4.5 character_state queries
- M4.6 event queries
- M4.7 relationship queries
- M4.8 cycle_manager (World/Cycle/GM 编排)
- M4.9 world_loader
- M4.10 gm_loader
- M5-M11 ...

## 下次 session 起点

**M4.5**: character_state CRUD 查询