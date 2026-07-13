//! GM Router (M2.4-M2.6) — 角色调度 + 三重防护
//!
//! 3.md §7 的五重防护中前三重:
//!
//! **M2.4 独立 context** (本模块核心):
//! - [`build_subagent_context`] 为 NPC subagent 拼装纯净 system prompt
//! - [`filter_events_for_role`] 从 domain_events 筛该 NPC 可见的部分
//!
//! **M2.5 hiddenPublicPolicy** (占位):
//! - 在 M2.4 的事件过滤之上增加策略层 (secrets / intent / dm_internal)
//!
//! **M2.6 tools 白名单** (占位):
//! - 限制角色 subagent 只能调用 lookup + suggest, 不能 update_state
//!
//! 硬约束 (来自 tavern2agent + AIRP 戒律#6):
//! - subagent context 不含 DM orchestrator 内部变量
//! - subagent 不写 state (只做 lookup + suggest)
//! - subagent context 不含其他 NPC 的 persona
//! - persona 来自 npc_base, 锁死不可变

use crate::error::RealmsError;
use crate::events::domain_event::{DomainEvent, EventType};

/// 角色 subagent 调度请求 (M2.4)
#[derive(Debug, Clone)]
pub struct RoleDispatchRequest {
    /// NPC id (如 "lin_yueru")
    pub npc_id: String,
    /// NPC 人设全文 (来自 npc_base, 锁死不可变)
    pub npc_persona: String,
    /// 世界观设定 (已经过滤该 NPC 不该知晓的秘密)
    pub world_setting: String,
    /// 该 NPC 可见的 domain_events 摘要列表
    pub visible_events: Vec<String>,
    /// 用户最新消息
    pub user_message: String,
}

/// 角色 subagent 返回 (M2.4 — LLM 桩)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleDispatchResult {
    /// NPC id
    pub npc_id: String,
    /// NPC 公开说的话/做的事 (叙事文本)
    pub visible_response: String,
    /// NPC 内心独白 (不透给其他 NPC / 玩家)
    pub private_intent: String,
    /// 建议的领域事件 (stub: 空)
    pub suggested_events: Vec<String>,
}

/// 构建角色 subagent 的独立 system prompt.
///
/// 仅含 4 块:
/// 1. NPC 人设 (来自 npc_base, 原文不动)
/// 2. 世界观 (已过滤秘密)
/// 3. 当前情境 (该 NPC 可见的事件摘要)
/// 4. 玩家最新消息
///
/// 绝对不含:
/// - DM orchestrator 内部状态 (cycle_id, gm_id, prompt_assembler 内部表)
/// - 其他 NPC 的名字/人设
/// - 其他 NPC 的 privateIntent
///
/// 段间用 `\n\n---\n\n` 分隔 (与 prompt_assembler 一致).
pub fn build_subagent_context(req: &RoleDispatchRequest) -> String {
    let mut parts = Vec::with_capacity(4);

    // 1. 角色人设 (从 npc_base 原文不动)
    parts.push(format!("# 你是\n{}", req.npc_persona));

    // 2. 世界观
    if !req.world_setting.trim().is_empty() {
        parts.push(format!("# 你所在的世界\n{}", req.world_setting));
    } else {
        parts.push("# 你所在的世界\n（未知）".to_string());
    }

    // 3. 当前情境 (该 NPC 可见的事件)
    parts.push(format!(
        "# 当前情境\n{}",
        format_visible_events(&req.visible_events)
    ));

    // 4. 玩家的话
    parts.push(format!("# 玩家的话\n{}", req.user_message));

    parts.join("\n\n---\n\n")
}

/// 将可见事件列表格式化为情境描述文本.
fn format_visible_events(events: &[String]) -> String {
    if events.is_empty() {
        return "（暂无事件）".to_string();
    }
    let mut lines = Vec::with_capacity(events.len());
    for (i, ev) in events.iter().enumerate() {
        lines.push(format!("{}. {}", i + 1, ev));
    }
    lines.join("\n")
}

/// 从 domain_events 中筛选该 NPC 可见的事件.
///
/// **可见规则**:
/// - NPC 是 actor 的事件 → 可见
/// - NPC 是 target 的事件 → 可见
/// - `Setup` + `TimeAdvance` 对所有 NPC 可见 (环境级事件)
/// - 其他事件 (NPC 既非 actor 也非 target) → 不可见
///
/// # 示例
///
/// ```
/// use realms_core::events::domain_event::{DomainEvent, ActorType, TargetType};
/// use realms_core::engine::orchestrator::gm_router::filter_events_for_role;
///
/// let ev = DomainEvent::Dialogue {
///     cycle_id: "c1".into(),
///     actor_id: "lin_yueru".into(),
///     actor_type: ActorType::Npc,
///     target_id: Some("xiaoyao".into()),
///     target_type: TargetType::Npc,
///     location_id: None,
///     line: "喂!".into(),
///     world_time: None,
///     importance: 0.3,
///     summary: "林月如喊李逍遥".into(),
/// };
///
/// let events = [ev.clone()];
/// let visible = filter_events_for_role(&events, "lin_yueru");
/// assert_eq!(visible.len(), 1);  // 林月如是 actor, 可见
///
/// let hidden = filter_events_for_role(&events, "zhao_linger");
/// assert_eq!(hidden.len(), 0);    // 赵灵儿既非 actor 也非 target, 不可见
/// ```
pub fn filter_events_for_role<'a>(events: &'a [DomainEvent], npc_id: &str) -> Vec<&'a DomainEvent> {
    events
        .iter()
        .filter(|ev| is_event_visible_to(ev, npc_id))
        .collect()
}

/// 判定单个 domain_event 是否对指定 npc_id 可见.
fn is_event_visible_to(ev: &DomainEvent, npc_id: &str) -> bool {
    // Setup / TimeAdvance: 全局可见
    match ev.event_type() {
        EventType::Setup | EventType::TimeAdvance => return true,
        _ => {}
    }

    let actor = event_actor_id(ev);
    let target = event_target_id(ev);

    // NPC 是 actor 或 target → 可见
    actor == Some(npc_id) || target == Some(npc_id)
}

/// 提取 domain_event 的 actor_id (若适用).
fn event_actor_id(ev: &DomainEvent) -> Option<&str> {
    match ev {
        DomainEvent::Setup { .. } => None,
        DomainEvent::ArcIncrement { actor_id, .. } => Some(actor_id.as_str()),
        DomainEvent::RelationshipChange { npc_id, .. } => Some(npc_id.as_str()),
        DomainEvent::StateChange { actor_id, .. } => Some(actor_id.as_str()),
        DomainEvent::ItemChange { owner_id, .. } => Some(owner_id.as_str()),
        DomainEvent::TimeAdvance { .. } => None,
        DomainEvent::Dialogue { actor_id, .. } => Some(actor_id.as_str()),
        DomainEvent::Discovery { discoverer_id, .. } => Some(discoverer_id.as_str()),
        DomainEvent::Combat { attacker_id, .. } => Some(attacker_id.as_str()),
    }
}

/// 提取 domain_event 的 target_id (若适用).
fn event_target_id(ev: &DomainEvent) -> Option<&str> {
    match ev {
        DomainEvent::Setup { .. } => None,
        DomainEvent::ArcIncrement { target_id, .. } => Some(target_id.as_str()),
        DomainEvent::RelationshipChange { npc_id, .. } => Some(npc_id.as_str()),
        DomainEvent::StateChange { target_id, .. } => target_id.as_deref(),
        DomainEvent::ItemChange { .. } => None,
        DomainEvent::TimeAdvance { .. } => None,
        DomainEvent::Dialogue { target_id, .. } => target_id.as_deref(),
        DomainEvent::Discovery { .. } => None,
        DomainEvent::Combat { defender_id, .. } => Some(defender_id.as_str()),
    }
}

/// 角色 subagent 调度入口 (M2.4 — LLM 桩).
///
/// 组合 `build_subagent_context` + `filter_events_for_role` + (桩) LLM 调用.
///
/// 当前 LLM 调用为桩: visible_response 返回固定模板,
/// private_intent 返回 "(stub: no LLM yet)", suggested_events 为空.
pub fn dispatch_role_subagent(
    req: &RoleDispatchRequest,
) -> Result<RoleDispatchResult, RealmsError> {
    if req.npc_id.is_empty() {
        return Err(RealmsError::invalid("npc_id is empty"));
    }
    if req.npc_persona.trim().is_empty() {
        return Err(RealmsError::invalid("npc_persona is empty"));
    }

    // 1-4. 拼装 context (stub: 不实际调用 LLM, 仅验证可用)
    let _context = build_subagent_context(req);

    // 桩: 返回固定模板
    Ok(RoleDispatchResult {
        npc_id: req.npc_id.clone(),
        visible_response: format!("（{}: 桩回应 — LLM 未接入）", req.npc_id),
        private_intent: format!("（{}: 桩内心独白）", req.npc_id),
        suggested_events: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::domain_event::{ActorType, TargetType};

    // ========== 测试数据辅助 ==========

    fn make_dialogue(actor: &str, target: &str, summary: &str) -> DomainEvent {
        DomainEvent::Dialogue {
            cycle_id: "c".into(),
            actor_id: actor.into(),
            actor_type: ActorType::Npc,
            target_id: Some(target.into()),
            target_type: TargetType::Npc,
            location_id: None,
            line: "...".into(),
            world_time: None,
            importance: 0.3,
            summary: summary.into(),
        }
    }

    fn make_discovery(npc: &str) -> DomainEvent {
        DomainEvent::Discovery {
            cycle_id: "c".into(),
            discoverer_id: npc.into(),
            location_id: "loc".into(),
            location_name: "某处".into(),
            world_time: None,
            importance: 0.3,
            summary: format!("{npc} 发现了什么"),
        }
    }

    fn make_state_change(actor: &str, target: Option<&str>) -> DomainEvent {
        DomainEvent::StateChange {
            cycle_id: "c".into(),
            actor_id: actor.into(),
            actor_type: ActorType::Npc,
            target_id: target.map(|s| s.to_string()),
            target_type: TargetType::Npc,
            field: "hp".into(),
            old_value: None,
            new_value: "80".into(),
            world_time: None,
            importance: 0.4,
            summary: "HP 变化".into(),
        }
    }

    fn make_setup() -> DomainEvent {
        DomainEvent::Setup {
            cycle_id: "c".into(),
            perspective_id: "p".into(),
            gm_id: "g".into(),
            templates_applied: vec![],
        }
    }

    fn make_time_advance() -> DomainEvent {
        DomainEvent::TimeAdvance {
            cycle_id: "c".into(),
            from_time: "午时".into(),
            to_time: "未时".into(),
            elapsed_minutes: 60,
            importance: 0.2,
            summary: "时间流逝".into(),
        }
    }

    fn lin_persona() -> String {
        "林月如 · 林家堡大小姐".into()
    }

    fn xiaoyao_persona() -> String {
        "李逍遥 · 余杭镇少年".into()
    }

    fn sample_world() -> String {
        "神州浩土".into()
    }

    // ========== M2.4: 独立 context ==========

    #[test]
    fn two_npcs_different_contexts() {
        let events = vec!["林月如走进客栈".to_string(), "李逍遥在柜台".to_string()];

        let req_a = RoleDispatchRequest {
            npc_id: "lin_yueru".into(),
            npc_persona: lin_persona(),
            world_setting: sample_world(),
            visible_events: events.clone(),
            user_message: "你好".into(),
        };
        let req_b = RoleDispatchRequest {
            npc_id: "li_xiaoyao".into(),
            npc_persona: xiaoyao_persona(),
            world_setting: sample_world(),
            visible_events: events,
            user_message: "你好".into(),
        };

        let ctx_a = build_subagent_context(&req_a);
        let ctx_b = build_subagent_context(&req_b);

        // 各自只含自己的 persona
        assert!(ctx_a.contains(&lin_persona()));
        assert!(ctx_b.contains(&xiaoyao_persona()));

        // NPC A 的 context 不含 NPC B 的人设
        assert!(!ctx_a.contains(&xiaoyao_persona()));
        assert!(!ctx_b.contains(&lin_persona()));
    }

    #[test]
    fn no_orchestrator_noise_in_context() {
        let req = RoleDispatchRequest {
            npc_id: "lin_yueru".into(),
            npc_persona: lin_persona(),
            world_setting: sample_world(),
            visible_events: vec!["事件1".into()],
            user_message: "你好".into(),
        };
        let ctx = build_subagent_context(&req);

        // 戒律#6: subagent context 不含 DM orchestrator 内部变量
        let forbidden = ["cycle_id", "gm_id", "prompt_assembler", "orchestrator"];
        for word in forbidden {
            assert!(
                !ctx.contains(word),
                "subagent context 不应含 orchestrator 内部变量 '{word}'"
            );
        }
    }

    #[test]
    fn persona_is_verbatim() {
        let persona = "林月如 · 林家堡大小姐\n\n性格: 刁蛮任性";
        let req = RoleDispatchRequest {
            npc_id: "lin_yueru".into(),
            npc_persona: persona.to_string(),
            world_setting: String::new(),
            visible_events: vec![],
            user_message: "你好".into(),
        };
        let ctx = build_subagent_context(&req);
        // persona 原文不动
        assert!(ctx.contains(persona), "persona 应从 npc_base 原文不动注入");
    }

    #[test]
    fn subagent_context_sections_order() {
        let req = RoleDispatchRequest {
            npc_id: "npc".into(),
            npc_persona: "P".into(),
            world_setting: "W".into(),
            visible_events: vec!["E1".into()],
            user_message: "U".into(),
        };
        let ctx = build_subagent_context(&req);

        // 4 段顺序: 你是 → 世界 → 情境 → 玩家
        let pos_you = ctx.find("# 你是").unwrap();
        let pos_world = ctx.find("# 你所在的世界").unwrap();
        let pos_situation = ctx.find("# 当前情境").unwrap();
        let pos_player = ctx.find("# 玩家的话").unwrap();

        assert!(pos_you < pos_world);
        assert!(pos_world < pos_situation);
        assert!(pos_situation < pos_player);

        // 恰好 3 个 SECTION_SEPARATOR (4 段间)
        assert_eq!(ctx.matches("\n\n---\n\n").count(), 3);
    }

    #[test]
    fn empty_world_setting_uses_placeholder() {
        let req = RoleDispatchRequest {
            npc_id: "npc".into(),
            npc_persona: "P".into(),
            world_setting: String::new(),
            visible_events: vec![],
            user_message: "U".into(),
        };
        let ctx = build_subagent_context(&req);
        assert!(ctx.contains("（未知）"), "空世界观应填占位符");
    }

    // ========== M2.4: 事件可见性过滤 ==========

    #[test]
    fn filter_includes_own_events() {
        let events = vec![
            make_dialogue("lin_yueru", "xiaoyao", "林月如说话"),
            make_discovery("lin_yueru"),
            make_state_change("lin_yueru", None),
        ];
        let visible = filter_events_for_role(&events, "lin_yueru");
        // 3 个事件中林月如都是 actor, 全可见
        assert_eq!(visible.len(), 3);
    }

    #[test]
    fn filter_includes_target_events() {
        let events = vec![make_dialogue("xiaoyao", "lin_yueru", "李逍遥对林月如说话")];
        let visible = filter_events_for_role(&events, "lin_yueru");
        // 林月如是 target, 可见
        assert_eq!(visible.len(), 1);
    }

    #[test]
    fn filter_excludes_unrelated_events() {
        let events = vec![
            make_dialogue("xiaoyao", "zhao_linger", "李逍遥对赵灵儿说话"),
            make_discovery("zhao_linger"),
            make_state_change("zhao_linger", Some("xiaoyao")),
        ];
        let visible = filter_events_for_role(&events, "lin_yueru");
        // 林月如不是 3 个事件中任何一个的 actor 或 target → 全不可见
        assert_eq!(visible.len(), 0);
    }

    #[test]
    fn filter_setup_and_time_advance_are_public() {
        let events = vec![make_setup(), make_time_advance()];
        let visible = filter_events_for_role(&events, "npc_unknown");
        // Setup + TimeAdvance 环境级, 对任意 NPC 可见
        assert_eq!(visible.len(), 2);
    }

    #[test]
    fn filter_mixed_visibility() {
        let events = vec![
            make_setup(),
            make_dialogue("lin_yueru", "xiaoyao", "林→李"),
            make_dialogue("xiaoyao", "zhao_linger", "李→赵"),
            make_discovery("lin_yueru"),
            make_time_advance(),
            make_state_change("zhao_linger", Some("xiaoyao")),
        ];
        let visible = filter_events_for_role(&events, "lin_yueru");
        // 林月如可见: setup(1) + 林→李对话(1) + 林发现(1) + time_advance(1) = 4
        // 不可见: 李→赵对话, 赵 state_change
        assert_eq!(visible.len(), 4);
    }

    // ========== M2.4: dispatch 桩 ==========

    #[test]
    fn dispatch_empty_npc_id_fails() {
        let req = RoleDispatchRequest {
            npc_id: String::new(),
            npc_persona: "P".into(),
            world_setting: "W".into(),
            visible_events: vec![],
            user_message: "U".into(),
        };
        assert!(dispatch_role_subagent(&req).is_err());
    }

    #[test]
    fn dispatch_empty_persona_fails() {
        let req = RoleDispatchRequest {
            npc_id: "npc".into(),
            npc_persona: "   ".into(),
            world_setting: "W".into(),
            visible_events: vec![],
            user_message: "U".into(),
        };
        assert!(dispatch_role_subagent(&req).is_err());
    }

    #[test]
    fn dispatch_ok_returns_stub_result() {
        let req = RoleDispatchRequest {
            npc_id: "lin_yueru".into(),
            npc_persona: lin_persona(),
            world_setting: sample_world(),
            visible_events: vec![],
            user_message: "你好".into(),
        };
        let result = dispatch_role_subagent(&req).unwrap();
        assert_eq!(result.npc_id, "lin_yueru");
        assert!(result.visible_response.contains("桩回应"));
        assert!(result.private_intent.contains("桩内心独白"));
        assert!(result.suggested_events.is_empty());
    }

    // ========== format_visible_events ==========

    #[test]
    fn format_empty_events() {
        let result = format_visible_events(&[]);
        assert!(result.contains("暂无事件"));
    }

    #[test]
    fn format_numbered_events() {
        let events = vec!["A".to_string(), "B".to_string()];
        let result = format_visible_events(&events);
        assert!(result.contains("1. A"));
        assert!(result.contains("2. B"));
    }
}
