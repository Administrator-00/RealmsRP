//! M10.2 arc 跳变检测 + M10.7 端到端 gameplay demo
//!
//! 模拟完整一轮游戏: 创建 world → 创建 cycle → 多轮对话事件 →
//! 验证 state / relationship / arc / memory 全部正确落盘.

use std::collections::HashMap;

use realms_core::db::pool::init_memory_pool;
use realms_core::engine::cycle::oc_initializer;
use realms_core::engine::reducers::process_event;
use realms_core::events::domain_event::{ActorType, DomainEvent, TargetType};

#[test]
fn arc_jump_detection() {
    // 验证 arc_reducer 边界: clamp / 里程碑 / 无跳变
    let pool = init_memory_pool().expect("pool");
    let conn = pool.get().unwrap();
    conn.execute_batch(&format!(
        "{}\n{}",
        realms_core::engine::reducers::CHARACTER_STATES_DDL,
        realms_core::engine::reducers::DOMAIN_EVENTS_DDL,
    ))
    .unwrap();

    // 信任从 0 → 55 → 应停在 "朋友" (≥50), 不该跳变到 "生死之交"
    let ev1 = DomainEvent::ArcIncrement {
        cycle_id: "arc_test".into(),
        actor_id: "gm".into(),
        actor_type: ActorType::System,
        target_id: "npc".into(),
        target_type: TargetType::Npc,
        arc_dimension: "trust_user".into(),
        delta: 55.0,
        world_time: None,
        importance: 0.5,
        summary: "信任 +55".into(),
    };
    process_event(&pool, &ev1).expect("ev1");

    let phase: String = conn
        .query_row(
            "SELECT arc_phase FROM character_states WHERE cycle_id='arc_test' AND npc_id='npc'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(phase, "朋友", "55 → 应停在 '朋友' 而非 '生死之交'");

    let arcs_json: String = conn
        .query_row(
            "SELECT arcs FROM character_states WHERE cycle_id='arc_test' AND npc_id='npc'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let arcs: serde_json::Value = serde_json::from_str(&arcs_json).unwrap();
    assert_eq!(arcs["trust_user"], 55.0);

    // 信任下降 -60 → 应 clamp 到 -5... 不对, clamp 到 0
    let ev2 = DomainEvent::ArcIncrement {
        cycle_id: "arc_test".into(),
        actor_id: "gm".into(),
        actor_type: ActorType::System,
        target_id: "npc".into(),
        target_type: TargetType::Npc,
        arc_dimension: "trust_user".into(),
        delta: -60.0,
        world_time: None,
        importance: 0.5,
        summary: "信任 -60".into(),
    };
    process_event(&pool, &ev2).expect("ev2");

    let arcs_json2: String = conn
        .query_row(
            "SELECT arcs FROM character_states WHERE cycle_id='arc_test' AND npc_id='npc'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let arcs2: serde_json::Value = serde_json::from_str(&arcs_json2).unwrap();
    assert_eq!(
        arcs2["trust_user"], 0.0,
        "55 - 60 = -5, clamp 到 0 (不跳变到负数)"
    );

    let phase2: String = conn
        .query_row(
            "SELECT arc_phase FROM character_states WHERE cycle_id='arc_test' AND npc_id='npc'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(phase2, "陌生人", "回到 0 → '陌生人'");
}

#[test]
fn end_to_end_gameplay_demo() {
    // 完整 demo: 仙剑世界, 创建 OC+cycle, 3 轮对话, 验证全状态
    let pool = init_memory_pool().expect("pool");
    let conn = pool.get().unwrap();
    conn.execute_batch(&format!(
        "{}\n{}\n{}\n{}",
        realms_core::engine::reducers::CHARACTER_STATES_DDL,
        realms_core::engine::reducers::RELATIONSHIP_DDL,
        realms_core::engine::reducers::MEMORY_DDL,
        realms_core::engine::reducers::DOMAIN_EVENTS_DDL,
    ))
    .unwrap();

    // ── 创建周目 ──
    conn.execute(
        "INSERT INTO cycles (cycle_id, world_id, perspective_id, gm_id, cycle_name)
         VALUES ('demo_c1', 'xiatian_qixia_1', 'pov_oc_shixiong', 'style_linyueru', 'demo周目')",
        [],
    )
    .unwrap();

    // ── OC 关系模板初始化 ──
    let mut selections = HashMap::new();
    selections.insert("lin_yueru".to_string(), "tpl_childhood_friend".to_string());
    selections.insert("li_xiaoyao".to_string(), "tpl_stranger".to_string());
    oc_initializer::initialize_cycle_relationships(&pool, "demo_c1", &selections).expect("oc init");

    // ── 第 1 轮: 对话 ──
    let e1 = DomainEvent::Dialogue {
        cycle_id: "demo_c1".into(),
        actor_id: "lin_yueru".into(),
        actor_type: ActorType::Npc,
        target_id: Some("user".into()),
        target_type: TargetType::User,
        location_id: Some("yuhang_town".into()),
        line: "师兄！你怎么在这里？".into(),
        world_time: Some("景天元年 三月初一 辰时".into()),
        importance: 0.5,
        summary: "林月如见到师兄".into(),
    };
    process_event(&pool, &e1).expect("e1");

    // ── 第 2 轮: 好感变化 ──
    let e2 = DomainEvent::RelationshipChange {
        cycle_id: "demo_c1".into(),
        npc_id: "lin_yueru".into(),
        field: "affinity".into(),
        old_value: Some("60".into()),
        new_value: "75".into(),
        delta: Some(15.0),
        world_time: Some("景天元年 三月初一 巳时".into()),
        importance: 0.6,
        summary: "林月如好感 +15 (共同御敌)".into(),
    };
    process_event(&pool, &e2).expect("e2");

    // ── 第 3 轮: 弧光变化 + 状态变化 ──
    let e3 = DomainEvent::ArcIncrement {
        cycle_id: "demo_c1".into(),
        actor_id: "gm".into(),
        actor_type: ActorType::System,
        target_id: "lin_yueru".into(),
        target_type: TargetType::Npc,
        arc_dimension: "trust_user".into(),
        delta: 30.0,
        world_time: Some("景天元年 三月初一 午时".into()),
        importance: 0.7,
        summary: "林月如信任 +30".into(),
    };
    process_event(&pool, &e3).expect("e3");

    let e4 = DomainEvent::StateChange {
        cycle_id: "demo_c1".into(),
        actor_id: "lin_yueru".into(),
        actor_type: ActorType::Npc,
        target_id: Some("lin_yueru".into()),
        target_type: TargetType::Npc,
        field: "hp".into(),
        old_value: Some("100".into()),
        new_value: "65".into(),
        world_time: Some("景天元年 三月初一 午时".into()),
        importance: 0.5,
        summary: "林月如受伤 HP-35".into(),
    };
    process_event(&pool, &e4).expect("e4");

    // ============ 验证 ============

    // 1. 关系: affinity 从 60(青梅竹马) → 75
    let aff: i64 = conn
        .query_row(
            "SELECT affinity FROM npc_user_relationships WHERE cycle_id='demo_c1' AND npc_id='lin_yueru'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(aff, 75);

    // 2. 关系历史: 应有 2 条 (初始模板 + e2)
    let hist_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM relationship_history WHERE cycle_id='demo_c1'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    // 初始模板不写 history, 只有 e2 写 1 条
    assert_eq!(hist_count, 1);

    // 3. 弧光: trust_user 应 = 30 (从 0 开始)
    let arcs_json: String = conn
        .query_row(
            "SELECT arcs FROM character_states WHERE cycle_id='demo_c1' AND npc_id='lin_yueru'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let arcs: serde_json::Value = serde_json::from_str(&arcs_json).unwrap();
    assert_eq!(arcs["trust_user"], 30.0);

    // 4. arc_phase: 30 ≥ 20 → "初识"
    let phase: String = conn
        .query_row(
            "SELECT arc_phase FROM character_states WHERE cycle_id='demo_c1' AND npc_id='lin_yueru'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(phase, "初识");

    // 5. HP: 65
    let hp: i64 = conn
        .query_row(
            "SELECT hp FROM character_states WHERE cycle_id='demo_c1' AND npc_id='lin_yueru'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(hp, 65);

    // 6. Domain events: 4 条
    let ev_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM domain_events WHERE cycle_id='demo_c1'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(ev_count, 4);

    // 7. Episodic memories: 至少 3 条 (e1:2个, e2:1个, e3:1个, e4:1个)
    // e1: lin_yueru+user, e2: lin_yueru, e3: lin_yueru, e4: lin_yueru
    let epi_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM episodic_memories WHERE cycle_id='demo_c1'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(epi_count >= 5, "应有至少 5 条 episodic (2+1+1+1)");

    // 8. Emotional: e1 是 Dialogue, 应有 1 条
    let emo_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM emotional_memories WHERE cycle_id='demo_c1'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(emo_count, 1);

    println!("\n═══════════ E2E Demo 验证通过 ═══════════");
    println!("  周目: demo周目 (仙剑奇侠传一)");
    println!("  事件: 4 条 (对话→好感→信任→受伤)");
    println!("  关系: 林月如 affinity=75 (青梅竹马 +15)");
    println!("  弧光: trust_user=30 → 初识");
    println!("  状态: HP=65/100");
    println!("  记忆: episodic={}, emotional={}", epi_count, emo_count);
    println!("══════════════════════════════════════════\n");
}
