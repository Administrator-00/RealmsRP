//! M10.1 串台检测 — 多 NPC 隔离验证
//!
//! 验证 5 重防护的核心承诺: 林月如不知道李逍遥的秘密, 反之亦然.
//!
//! 测试场景: 4 个 NPC 同时处理同一事件流, 验证:
//! 1. 各自 context 仅含自己的 persona
//! 2. private_intent 不含其他 NPC id
//! 3. context 不含 orchestrator 内部变量
//! 4. event filter 正确隔离不可见事件

use realms_core::engine::orchestrator::gm_router::{
    build_subagent_context, filter_events_for_role, RoleDispatchRequest,
};
use realms_core::events::domain_event::{ActorType, DomainEvent, TargetType};

#[test]
fn four_npcs_context_isolation() {
    let personas = vec![
        ("lin_yueru", "林月如 · 林家堡大小姐。性格：敢爱敢恨。"),
        ("li_xiaoyao", "李逍遥 · 余杭镇少年。性格：油嘴滑舌。"),
        ("zhao_linger", "赵灵儿 · 女娲族后裔。性格：温柔善良。"),
        ("npc_innkeeper", "盛帆客栈老板。性格：精明圆滑。"),
    ];

    let world = "神州浩土，仙魔纷争。";
    let events = vec![
        "林月如走进余杭镇客栈".to_string(),
        "李逍遥在客栈柜台帮忙".to_string(),
    ];

    let contexts: Vec<_> = personas
        .iter()
        .map(|(id, persona)| {
            let req = RoleDispatchRequest {
                npc_id: id.to_string(),
                npc_persona: persona.to_string(),
                world_setting: world.to_string(),
                visible_events: events.clone(),
                user_message: "你好".to_string(),
            };
            (id, build_subagent_context(&req))
        })
        .collect();

    // 验证: 每个 context 只含自己的 persona
    for (id, ctx) in &contexts {
        for (other_id, other_persona) in &personas {
            if **id == *other_id {
                assert!(
                    ctx.contains(other_persona),
                    "{id} 的 context 应含自己的 persona"
                );
            } else {
                assert!(
                    !ctx.contains(other_persona),
                    "{id} 的 context 不应含 {other_id} 的 persona (串台!)"
                );
            }
        }
    }

    // 验证: 无 orchestrator 内部变量
    let forbidden = ["cycle_id", "gm_id", "prompt_assembler", "orchestrator"];
    for (id, ctx) in &contexts {
        for word in &forbidden {
            assert!(
                !ctx.contains(word),
                "{id} 的 context 含 orchestrator 变量 '{word}' (戒律#6 违反!)"
            );
        }
    }
}

#[test]
fn event_filter_isolates_secrets() {
    // 场景: 林月如暗中对李逍遥说秘密话, 赵灵儿不应看到
    let secret_dialogue = DomainEvent::Dialogue {
        cycle_id: "c1".into(),
        actor_id: "lin_yueru".into(),
        actor_type: ActorType::Npc,
        target_id: Some("li_xiaoyao".into()),
        target_type: TargetType::Npc,
        location_id: None,
        line: "李逍遥，我有个秘密告诉你...".into(),
        world_time: None,
        importance: 0.9,
        summary: "林月如对李逍遥说秘密".into(),
    };

    let public_event = DomainEvent::Dialogue {
        cycle_id: "c1".into(),
        actor_id: "npc_innkeeper".into(),
        actor_type: ActorType::Npc,
        target_id: Some("lin_yueru".into()),
        target_type: TargetType::Npc,
        location_id: None,
        line: "客官里面请！".into(),
        world_time: None,
        importance: 0.3,
        summary: "客栈老板招呼林月如".into(),
    };

    let events = vec![secret_dialogue, public_event];

    // 赵灵儿: 不是秘密对话的 actor/target, 不应对其可见
    let linger_visible = filter_events_for_role(&events, "zhao_linger");
    assert_eq!(
        linger_visible.len(),
        0,
        "赵灵儿不应看到林月如和李逍遥的秘密对话"
    );

    // 林月如: 是 actor, 应该可见
    let lin_visible = filter_events_for_role(&events, "lin_yueru");
    assert_eq!(
        lin_visible.len(),
        2,
        "林月如应看到自己的对话 + 客栈老板的话"
    );

    // 李逍遥: 是 target, 应对其可见
    let xiao_visible = filter_events_for_role(&events, "li_xiaoyao");
    assert_eq!(xiao_visible.len(), 1, "李逍遥应看到林月如对自己说的话");
}

#[test]
fn audit_detects_crosstalk_in_mock_dispatch() {
    use realms_core::engine::orchestrator::gm_dispatcher::audit_for_crosstalk;

    // 模拟场景: 林月如的 private_intent 意外提到李逍遥的秘密
    let clean_intent = "这人看起来挺有趣，不知道有什么本事。";
    let leaky_intent = "li_xiaoyao 的身世原来是女娲后裔？难以置信。";

    let other_npcs = ["li_xiaoyao".into(), "zhao_linger".into()];

    // clean intent → 无告警
    let warnings = audit_for_crosstalk(clean_intent, &other_npcs);
    assert!(warnings.is_empty(), "干净的 private_intent 不应触发告警");

    // leaky intent → 告警
    let warnings = audit_for_crosstalk(leaky_intent, &other_npcs);
    assert!(!warnings.is_empty(), "泄露的 private_intent 应触发告警");
    assert!(warnings[0].contains("li_xiaoyao"));
}

#[test]
fn mock_full_dispatch_no_crosstalk() {
    use realms_core::engine::orchestrator::gm_dispatcher::{full_dispatch, MockLlmProvider};
    use realms_core::engine::orchestrator::gm_router::RoleToolWhitelist;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    // 4 个 NPC, 各自独立 dispatch
    let npcs = [
        ("lin_yueru", "林月如 · 林家堡大小姐"),
        ("li_xiaoyao", "李逍遥 · 余杭镇少年"),
        ("zhao_linger", "赵灵儿 · 女娲族后裔"),
        ("npc_innkeeper", "盛帆客栈老板"),
    ];

    let results: Vec<_> = npcs
        .iter()
        .map(|(id, persona)| {
            let req = RoleDispatchRequest {
                npc_id: id.to_string(),
                npc_persona: persona.to_string(),
                world_setting: "神州浩土".into(),
                visible_events: vec!["一位背剑的年轻人走进客栈".into()],
                user_message: "你好".into(),
            };
            let whitelist = RoleToolWhitelist::npc_default();
            let llm = MockLlmProvider {
                canned_response: format!(
                    "[VISIBLE]\n{id} 的回复\n\n[INTENT]\n{id} 的内心独白，不含其他 NPC 信息"
                ),
            };
            let other_ids: Vec<String> = npcs
                .iter()
                .filter(|(oid, _)| oid != id)
                .map(|(oid, _)| oid.to_string())
                .collect();

            rt.block_on(full_dispatch(&req, &whitelist, &llm, &other_ids))
                .expect("dispatch")
        })
        .collect();

    // 验证: 每个 NPC 的 private_intent 不含其他 NPC id
    for (i, result) in results.iter().enumerate() {
        let (my_id, _) = npcs[i];
        for (j, (other_id, _)) in npcs.iter().enumerate() {
            if i != j {
                assert!(
                    !result.private_intent.contains(other_id),
                    "{} 的 private_intent 含 {} 的信息 (串台!)",
                    my_id,
                    other_id
                );
            }
        }
        assert!(
            result.audit_warnings.is_empty(),
            "{} 的 audit 应无告警",
            my_id
        );
    }
}
