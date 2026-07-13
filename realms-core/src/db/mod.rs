//! SQLite 数据库模块
//!
//! 负责 realms-core 私有 SQLite 长期事实库 (memory.db).
//! 数据目录与 AIRP-MCP-Server 错开 (SKILL.md 已规定).
//!
//! 详见 3.md §2.2 完整 schema.

pub mod pool;

pub use pool::{init_pool, DbPool};
