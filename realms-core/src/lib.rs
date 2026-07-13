//! Realms RP 平台核心库 (胶水层)
//!
//! Realms = 多世界 RP 沙盒平台, SillyTavern 的上位替代.
//! 本库 (realms-core) 是 DM 编排器 + reducer + cycle 管理的胶水层,
//! 与上游 AIRP engine / AIRP-MCP-Server / AIRP-State-Protocol / tavern2agent 协作.
//!
//! # 模块组织
//!
//! - `engine::orchestrator`: DM 编排器 (prompt_assembler, dm_router)
//! - `engine::reducers`: 状态变更处理 (arc, relationship, memory, state)
//! - `engine::cycle`: 周目管理 (创建, 加载, 切换, OC 初始化)
//! - `events`: 领域事件 (DomainEvent 类型化枚举)
//! - `db`: SQLite 长期事实库 (schema, queries, pool)
//!
//! # 架构位置
//!
//! 3.md §11 描述的 5 复用 + 1 胶水层, 本仓 = 第 6 块 (胶水层, ~6000 行).
//!
//! # 硬性约束
//!
//! 1. 不修改 ../AIRP/ ../AIRP-MCP-Server/ ../AIRP-State-Protocol/ ../tavern2agent/
//! 2. 戒律#6: subagent_context_has_no_orchestrator_noise
//! 3. 错误用 `Result<T, RealmsError>`, 不用 `panic` / `unwrap` / `unsafe`
//! 4. 依赖仅限: serde / tokio / rusqlite / r2d2 / thiserror / tracing / chrono / async-trait
//! 5. 单文件 < 500 行, pub fn < 50 行

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod db;
pub mod engine;
pub mod error;
pub mod events;

pub use error::RealmsError;
pub use events::DomainEvent;

/// Realms Core 版本号
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        // VERSION 直接来自 Cargo.toml, 锁定预期值即可
        assert_eq!(VERSION, "0.1.0");
    }

    #[test]
    fn error_module_exports_realms_error() {
        // 编译期检查 RealmsError 在 pub use 列表中
        let _: fn(RealmsError) -> String = |e| e.to_string();
    }
}
