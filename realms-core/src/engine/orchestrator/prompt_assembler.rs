//! Prompt assembler (M2.3) — GM 拼接核心
//!
//! 按 3.md §1.4 的拼接机制组装 system_prompt:
//!
//! 1. `# 角色`         — 角色人设 (来自 perspective, 不可变或自定义)
//! 2. `# 世界观`       — 世界设定 (来自 world)
//! 3. `# 文风指引`     — GM 文风 system_prompt (唯一用户可改的"风格层")
//! 4. `# 当前状态`     — 状态投影 (来自 SQLite)
//! 5. `# 弧光`         — 弧光阶段 + 弧光上下文
//! 6. `# 相关记忆`     — 记忆检索结果 (超长自动截断)
//! 7. `# 规则`         — 规则约束 (来自 world.rules)
//!
//! 段间用 `\n\n---\n\n` 分隔 (与 3.md §1.4 一致).
//!
//! 设计要点:
//! - `PromptContext` 持 7 段字符串, 不直接依赖 `Cycle` / `GmPrompt` 复合类型
//!   (那些类型在 M4-M5 才落地), 解耦编排器与数据层.
//! - `gm_content` 为空时用占位符 (保持中性叙事, 不让模型自行臆造文风).
//! - `relevant_memories` 超过 [`MAX_RELEVANT_MEMORIES_CHARS`] 字符时按 char 边界截断,
//!   避免破坏 UTF-8 以及喂爆 context window.

/// 相关记忆在 system_prompt 中保留的最大字符数 (按 char 计, 非字节).
///
/// 超过的尾部用 [`MEMORY_TRUNCATION_SUFFIX`] 替换.
/// 默认 8000 字符, 约合混合中英 16k token 量级的一半, 留余量给其他段.
pub const MAX_RELEVANT_MEMORIES_CHARS: usize = 8000;

/// 记忆截断时的尾部提示.
pub const MEMORY_TRUNCATION_SUFFIX: &str = "\n\n（相关记忆过长，已截断，仅展示最近/最重要的部分）";

/// 7 段拼接的分隔符 (与 3.md §1.4 一致).
pub const SECTION_SEPARATOR: &str = "\n\n---\n\n";

/// `gm_content` 为空时使用的占位文风.
pub const DEFAULT_GM_PLACEHOLDER: &str = "你是一位中性的第三人称叙事者。保持克制的描写风格，\
依据「# 世界观」与「# 规则」推进剧情，不要自行臆造文风。";

/// 7 段标题 (顺序 = 3.md §1.4).
const SECTION_TITLES: [&str; 7] = [
    "# 角色",
    "# 世界观",
    "# 文风指引",
    "# 当前状态",
    "# 弧光",
    "# 相关记忆",
    "# 规则",
];

/// 拼装 system_prompt 所需的 7 段上下文.
///
/// 字段语义见模块级文档. 所有字段均为 `String`, 由上层 (cycle_manager / world_loader
/// / gm_loader / reducer 投影) 填充后再交给 [`build_system_prompt`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PromptContext {
    /// 角色人设 (perspective.persona_data).
    pub perspective_persona: String,
    /// 世界观正文 (world.setting).
    pub world_setting: String,
    /// GM 文风 system_prompt (唯一用户可改的"风格层"); 空时用占位符.
    pub gm_content: String,
    /// 当前状态投影 (HP/MP/位置/flag, 拼成的多行字符串).
    pub current_state: String,
    /// 当前弧光阶段 (如 "friend"/"lover"/"stranger"); `None` 表示无弧光数据.
    pub arc_phase: Option<String>,
    /// 弧光上下文 (各维度的当前值 + 最近里程碑).
    pub arc_context: String,
    /// 相关记忆检索结果 (超长自动截断).
    pub relevant_memories: String,
    /// 规则约束 (world.rules).
    pub world_rules: String,
}

impl PromptContext {
    /// 构造空的 `PromptContext` (所有字段为空 / None).
    pub fn new() -> Self {
        Self::default()
    }

    /// 段数 (固定 7).
    pub fn section_count() -> usize {
        SECTION_TITLES.len()
    }
}

/// 组装最终 system_prompt.
///
/// 严格按 3.md §1.4 的 7 段顺序拼接, 段间用 [`SECTION_SEPARATOR`].
/// `gm_content` 空时填入 [`DEFAULT_GM_PLACEHOLDER`] (避免模型自创文风).
/// `relevant_memories` 超长时按 char 边界截断 + 尾部标记.
///
/// # 示例
///
/// ```ignore
/// use realms_core::engine::orchestrator::prompt_assembler::{PromptContext, build_system_prompt};
/// let ctx = PromptContext {
///     perspective_persona: "我是李逍遥".into(),
///     world_setting: "神州浩土...".into(),
///     gm_content: "短句凌厉".into(),
///     ..PromptContext::new()
/// };
/// let prompt = build_system_prompt(&ctx);
/// assert!(prompt.contains("# 文风指引\n短句凌厉"));
/// ```
pub fn build_system_prompt(ctx: &PromptContext) -> String {
    let parts: [String; 7] = [
        format!("# 角色\n{}", ctx.perspective_persona),
        format!("# 世界观\n{}", ctx.world_setting),
        format!("# 文风指引\n{}", gm_section(&ctx.gm_content)),
        format!("# 当前状态\n{}", ctx.current_state),
        format!("# 弧光\n{}", arc_section(&ctx.arc_phase, &ctx.arc_context)),
        format!("# 相关记忆\n{}", truncate_memories(&ctx.relevant_memories)),
        format!("# 规则\n{}", ctx.world_rules),
    ];
    parts.join(SECTION_SEPARATOR)
}

/// 组装 `# 文风指引` 段: 空内容用占位符.
fn gm_section(gm_content: &str) -> String {
    if gm_content.trim().is_empty() {
        DEFAULT_GM_PLACEHOLDER.to_string()
    } else {
        gm_content.to_string()
    }
}

/// 组装 `# 弧光` 段.
///
/// - 同时有 phase + context: `当前阶段: {phase}\n{context}`
/// - 仅 phase: `当前阶段: {phase}`
/// - 仅 context: 直接输出 context
/// - 都空: `（无弧光数据）`
fn arc_section(arc_phase: &Option<String>, arc_context: &str) -> String {
    let phase = arc_phase
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let ctx_trim = arc_context.trim();
    match (phase, ctx_trim) {
        (Some(p), "") => format!("当前阶段: {p}"),
        (Some(p), c) => format!("当前阶段: {p}\n{c}"),
        (None, "") => "（无弧光数据）".to_string(),
        (None, c) => c.to_string(),
    }
}

/// 截断超长的相关记忆 (按 char 边界).
///
/// 字符数 (按 `char_indices`) 不超过 [`MAX_RELEVANT_MEMORIES_CHARS`] 时原样返回;
/// 否则取前 [`MAX_RELEVANT_MEMORIES_CHARS`] 个 char 并追加 [`MEMORY_TRUNCATION_SUFFIX`].
fn truncate_memories(relevant_memories: &str) -> String {
    let char_count = relevant_memories.chars().count();
    if char_count <= MAX_RELEVANT_MEMORIES_CHARS {
        return relevant_memories.to_string();
    }
    let truncated: String = relevant_memories
        .chars()
        .take(MAX_RELEVANT_MEMORIES_CHARS)
        .collect();
    format!("{truncated}{MEMORY_TRUNCATION_SUFFIX}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with_all() -> PromptContext {
        PromptContext {
            perspective_persona: "PERS".into(),
            world_setting: "WORLD".into(),
            gm_content: "GM".into(),
            current_state: "STATE".into(),
            arc_phase: Some("lover".into()),
            arc_context: "ARC".into(),
            relevant_memories: "MEM".into(),
            world_rules: "RULES".into(),
        }
    }

    #[test]
    fn seven_sections_appear_in_order() {
        let prompt = build_system_prompt(&ctx_with_all());
        let mut last: Option<usize> = None;
        for title in SECTION_TITLES {
            let pos = prompt
                .find(title)
                .unwrap_or_else(|| panic!("missing title {title} in prompt:\n{prompt}"));
            if let Some(l) = last {
                assert!(pos > l, "title {title} 出现顺序错乱 at {pos} (prev {l})");
            }
            last = Some(pos);
        }
    }

    #[test]
    fn section_separator_count_is_six() {
        let prompt = build_system_prompt(&ctx_with_all());
        let sep_count = prompt.matches(SECTION_SEPARATOR).count();
        assert_eq!(sep_count, 6, "7 段之间应有 6 个分隔符, 实际 {sep_count}");
    }

    #[test]
    fn gm_section_between_world_and_state() {
        let prompt = build_system_prompt(&ctx_with_all());
        let world_pos = prompt.find("# 世界观").unwrap();
        let gm_pos = prompt.find("# 文风指引\nGM").unwrap();
        let state_pos = prompt.find("# 当前状态").unwrap();
        assert!(world_pos < gm_pos, "GM 段应在世界观之后");
        assert!(gm_pos < state_pos, "GM 段应在当前状态之前");
    }

    #[test]
    fn empty_gm_uses_placeholder() {
        let mut ctx = ctx_with_all();
        ctx.gm_content = String::new();
        let prompt = build_system_prompt(&ctx);
        assert!(
            prompt.contains(&format!("# 文风指引\n{DEFAULT_GM_PLACEHOLDER}")),
            "空 GM 应填占位符"
        );
        assert!(
            !prompt.contains("# 文风指引\nGM"),
            "空 GM 不应残留原 GM 内容"
        );
    }

    #[test]
    fn whitespace_only_gm_uses_placeholder() {
        let mut ctx = ctx_with_all();
        ctx.gm_content = "   \n\t ".into();
        let prompt = build_system_prompt(&ctx);
        assert!(
            prompt.contains(&format!("# 文风指引\n{DEFAULT_GM_PLACEHOLDER}")),
            "纯空白 GM 应视作空"
        );
    }

    #[test]
    fn long_memories_are_truncated() {
        let mut ctx = ctx_with_all();
        // 构造远超上限的中文记忆; 验证不破坏 UTF-8 + 出现截断标记
        let unit = "景天元年三月初七，余杭镇雨后初晴。";
        ctx.relevant_memories = unit.repeat(MAX_RELEVANT_MEMORIES_CHARS * 2 + 100);
        let prompt = build_system_prompt(&ctx);

        // 找到 # 相关记忆 段, 截到下一个分隔符前
        let start = prompt.find("# 相关记忆\n").unwrap();
        let section_after = &prompt[start..];
        let end = section_after
            .find(SECTION_SEPARATOR)
            .unwrap_or(section_after.len());
        let section = &section_after[..end];
        assert!(section.contains(MEMORY_TRUNCATION_SUFFIX), "缺少截断标记");
        // 截断后段内字符数不应超过 MAX + suffix
        let body = section.strip_prefix("# 相关记忆\n").unwrap_or(section);
        assert!(
            body.chars().count()
                <= MAX_RELEVANT_MEMORIES_CHARS + MEMORY_TRUNCATION_SUFFIX.chars().count(),
            "截断后字符数仍超标"
        );
    }

    #[test]
    fn short_memories_not_truncated() {
        let ctx = ctx_with_all();
        let prompt = build_system_prompt(&ctx);
        assert!(
            prompt.contains("# 相关记忆\nMEM"),
            "短记忆应原样输出 (含截断说明反而错误)"
        );
        assert!(!prompt.contains(MEMORY_TRUNCATION_SUFFIX));
    }

    #[test]
    fn arc_phase_appears_in_arc_section() {
        let ctx = ctx_with_all();
        let prompt = build_system_prompt(&ctx);
        assert!(
            prompt.contains("# 弧光\n当前阶段: lover\nARC"),
            "弧光段应含 arc_phase=lover 与 arc_context"
        );
    }

    #[test]
    fn arc_phase_only_shows_phase() {
        let mut ctx = ctx_with_all();
        ctx.arc_context = String::new();
        ctx.arc_phase = Some("friend".into());
        let prompt = build_system_prompt(&ctx);
        assert!(
            prompt.contains("# 弧光\n当前阶段: friend"),
            "仅有 phase 时输出阶段"
        );
        assert!(!prompt.contains("ARC"), "arc_context 为空时不应输出 ARC");
    }

    #[test]
    fn no_arc_data_emits_placeholder() {
        let mut ctx = ctx_with_all();
        ctx.arc_phase = None;
        ctx.arc_context = String::new();
        let prompt = build_system_prompt(&ctx);
        assert!(prompt.contains("# 弧光\n（无弧光数据）"));
    }

    #[test]
    fn empty_context_does_not_panic() {
        let ctx = PromptContext::new();
        let prompt = build_system_prompt(&ctx);
        // 即使全空, 7 段标题仍应齐
        for title in SECTION_TITLES {
            assert!(prompt.contains(title), "空 ctx 缺段 {title}");
        }
        assert_eq!(prompt.matches(SECTION_SEPARATOR).count(), 6);
    }

    #[test]
    fn section_count_is_seven() {
        assert_eq!(PromptContext::section_count(), 7);
    }

    #[test]
    fn gm_placeholder_is_nonempty() {
        assert!(!DEFAULT_GM_PLACEHOLDER.trim().is_empty());
    }
}
