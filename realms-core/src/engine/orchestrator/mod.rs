//! GM 编排器子模块 (M2.3 / M2.4-M2.6 / M7)
//!
//! - `prompt_assembler`: 7 段 GM 拼接核心 (3.md §1.4)
//! - `gm_router`: 角色调度 + 三重防护 (M2.4-M2.6)
//! - `gm_dispatcher`: 完整 dispatch 管线 (M7)

pub mod gm_dispatcher;
pub mod gm_router;
pub mod prompt_assembler;

// prompt_assembler
pub use prompt_assembler::{build_system_prompt, PromptContext};

// gm_router (M2.4-M2.6)
pub use gm_router::{
    build_subagent_context, dispatch_role_subagent, filter_events_for_role,
    filter_events_with_policy, PublicPolicy, RoleCapability, RoleDispatchRequest,
    RoleDispatchResult, RoleToolWhitelist,
};

// gm_dispatcher (M7)
pub use gm_dispatcher::{
    audit_for_crosstalk, full_dispatch, parse_subagent_output, FullDispatchResult, HttpLlmProvider,
    LlmProvider, MockLlmProvider, SuggestedEvent,
};
