//! DM 编排器模块
//!
//! 包含:
//! - `orchestrator`: prompt_assembler + dm_router (GM 拼接 + 角色调度)
//! - `reducers`: 4 个 reducer (arc / relationship / memory / state)
//! - `cycle`: 周目管理 (创建 / 加载 / 切换 / OC 初始化)

pub mod cycle;
pub mod orchestrator;
pub mod reducers;
