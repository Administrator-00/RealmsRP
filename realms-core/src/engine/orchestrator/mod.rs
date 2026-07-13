//! DM 编排器子模块 (M2.3 / M2.4-M2.6)
//!
//! - `prompt_assembler`: 7 段 GM 拼接核心 (3.md §1.4)
//! - `gm_router`: 角色调度 + 三重防护 (M2.4-M2.6)

pub mod gm_router;
pub mod prompt_assembler;

// 重新导出 prompt_assembler 的核心 API, 供上层直接用
// `realms_core::engine::orchestrator::build_system_prompt`.
pub use prompt_assembler::{build_system_prompt, PromptContext};
// 重新导出 gm_router 的核心 API
pub use gm_router::{
    build_subagent_context, dispatch_role_subagent, filter_events_for_role, RoleDispatchRequest,
    RoleDispatchResult,
};
