//! M11.10 仙剑 3 个完整 Cycle Demo
//!
//! Cycle 1: 李逍遥（内置NPC）+ 林月如文风 → 走主线
//! Cycle 2: OC 师兄 + 古龙武侠 → 戏说江湖
//! Cycle 3: OC 仙女散人 + 诙谐吐槽 → 轻松视角

use std::collections::HashMap;

use realms_core::db::pool::init_memory_pool;
use realms_core::engine::cycle::oc_initializer;
use realms_core::engine::reducers::process_event;
use realms_core::events::domain_event::{ActorType, DomainEvent, TargetType};

fn setup_demo_db() -> r2d2::Pool<r2d2_sqlite::SqliteConnectionManager> {
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
    // 插入 cycle 行
    conn.execute("INSERT INTO cycles (cycle_id, world_id, perspective_id, gm_id, cycle_name) VALUES ('c1', 'xiatian_qixia_1', 'pov_npc_li_xiaoyao', 'style_linyueru', 'demo-cycle-1')", []).unwrap();
    conn.execute("INSERT INTO cycles (cycle_id, world_id, perspective_id, gm_id, cycle_name) VALUES ('c2', 'xiatian_qixia_1', 'pov_oc_shixiong', 'style_gulong', 'demo-cycle-2')", []).unwrap();
    conn.execute("INSERT INTO cycles (cycle_id, world_id, perspective_id, gm_id, cycle_name) VALUES ('c3', 'xiatian_qixia_1', 'pov_oc_xianv', 'style_humor', 'demo-cycle-3')", []).unwrap();
    pool
}

#[test]
fn cycle1_li_xiaoyao_main_story() {
    let pool = setup_demo_db();
    println!("\n═══════════ Cycle 1: 李逍遥 · 林月如文风 · 主线 ═══════════");

    // 初始关系：对林月如是默认（NPC 视角无 OC 模板，手动设）
    let conn = pool.get().unwrap();
    conn.execute("INSERT INTO npc_user_relationships (cycle_id, npc_id, affinity, trust, intimacy, current_type) VALUES ('c1','lin_yueru',50,60,40,'friend')", []).unwrap();
    conn.execute("INSERT INTO npc_user_relationships (cycle_id, npc_id, affinity, trust, intimacy, current_type) VALUES ('c1','zhao_linger',30,40,20,'stranger')", []).unwrap();

    // 第 1 轮：李逍遥在客栈遇到赵灵儿
    let e1 = DomainEvent::Dialogue {
        cycle_id: "c1".into(),
        actor_id: "li_xiaoyao".into(),
        actor_type: ActorType::Npc,
        target_id: Some("zhao_linger".into()),
        target_type: TargetType::Npc,
        location_id: Some("shengfan_inn".into()),
        line: "你就是那个在仙灵岛上的姑娘？".into(),
        world_time: Some("景天元年 三月初一 辰时".into()),
        importance: 0.8,
        summary: "李逍遥初见赵灵儿".into(),
    };
    process_event(&pool, &e1).unwrap();

    // 第 2 轮：林月如出现，三人结伴
    let e2 = DomainEvent::Dialogue {
        cycle_id: "c1".into(),
        actor_id: "lin_yueru".into(),
        actor_type: ActorType::Npc,
        target_id: Some("li_xiaoyao".into()),
        target_type: TargetType::Npc,
        location_id: Some("suzhou_market".into()),
        line: "喂！臭蛋，你们要去哪儿？带上我！".into(),
        world_time: Some("景天元年 三月初二 午时".into()),
        importance: 0.6,
        summary: "林月如加入队伍".into(),
    };
    process_event(&pool, &e2).unwrap();

    // 第 3 轮：战斗中林月如受伤，好感变化
    let e3 = DomainEvent::StateChange {
        cycle_id: "c1".into(),
        actor_id: "lin_yueru".into(),
        actor_type: ActorType::Npc,
        target_id: Some("lin_yueru".into()),
        target_type: TargetType::Npc,
        field: "hp".into(),
        old_value: Some("100".into()),
        new_value: "50".into(),
        world_time: Some("景天元年 三月初二 未时".into()),
        importance: 0.7,
        summary: "林月如战斗中受伤".into(),
    };
    process_event(&pool, &e3).unwrap();

    let e4 = DomainEvent::RelationshipChange {
        cycle_id: "c1".into(),
        npc_id: "lin_yueru".into(),
        field: "affinity".into(),
        old_value: Some("50".into()),
        new_value: "70".into(),
        delta: Some(20.0),
        world_time: Some("景天元年 三月初二 未时".into()),
        importance: 0.7,
        summary: "李逍遥护林月如，好感+20".into(),
    };
    process_event(&pool, &e4).unwrap();

    // 验证
    let aff: i64 = conn.query_row("SELECT affinity FROM npc_user_relationships WHERE cycle_id='c1' AND npc_id='lin_yueru'", [], |r| r.get(0)).unwrap();
    assert_eq!(aff, 70);
    let ev_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM domain_events WHERE cycle_id='c1'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(ev_count, 4);
    let hp: i64 = conn
        .query_row(
            "SELECT hp FROM character_states WHERE cycle_id='c1' AND npc_id='lin_yueru'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(hp, 50);
    let epi: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM episodic_memories WHERE cycle_id='c1'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(epi >= 4);

    println!("  事件: 4 条 | 林月如 affinity=70 | HP=50 | 记忆: {epi} 条");
    println!("══════════════════════════════════════════\n");
}

#[test]
fn cycle2_oc_shixiong_gulong_style() {
    let pool = setup_demo_db();
    println!("\n═══════════ Cycle 2: OC 师兄 · 古龙武侠 · 戏说江湖 ═══════════");

    // OC 模板：对林月如→青梅竹马，对李逍遥→萍水相逢
    let mut sel = HashMap::new();
    sel.insert("lin_yueru".to_string(), "tpl_childhood_friend".to_string());
    sel.insert("li_xiaoyao".to_string(), "tpl_stranger".to_string());
    oc_initializer::initialize_cycle_relationships(&pool, "c2", &sel).unwrap();

    let conn = pool.get().unwrap();

    // 师兄在余杭镇客栈遇到李逍遥
    let e1 = DomainEvent::Dialogue {
        cycle_id: "c2".into(),
        actor_id: "li_xiaoyao".into(),
        actor_type: ActorType::Npc,
        target_id: Some("user".into()),
        target_type: TargetType::User,
        location_id: Some("shengfan_inn".into()),
        line: "阁下背剑独行，想必也是江湖中人。不如喝一杯？".into(),
        world_time: Some("景天元年 三月初一 申时".into()),
        importance: 0.4,
        summary: "李逍遥邀师兄喝酒".into(),
    };
    process_event(&pool, &e1).unwrap();

    // 师兄发现林月如也在此地
    let e2 = DomainEvent::Dialogue {
        cycle_id: "c2".into(),
        actor_id: "lin_yueru".into(),
        actor_type: ActorType::Npc,
        target_id: Some("user".into()),
        target_type: TargetType::User,
        location_id: Some("shengfan_inn".into()),
        line: "师兄？你怎么也下山了？师父知道吗？".into(),
        world_time: Some("景天元年 三月初一 酉时".into()),
        importance: 0.5,
        summary: "林月如认出师兄".into(),
    };
    process_event(&pool, &e2).unwrap();

    // 好感因重逢而提升
    let e3 = DomainEvent::RelationshipChange {
        cycle_id: "c2".into(),
        npc_id: "lin_yueru".into(),
        field: "affinity".into(),
        old_value: Some("60".into()),
        new_value: "80".into(),
        delta: Some(20.0),
        world_time: Some("景天元年 三月初一 酉时".into()),
        importance: 0.6,
        summary: "青梅竹马重逢，好感+20".into(),
    };
    process_event(&pool, &e3).unwrap();

    // Arc: trust_user
    let e4 = DomainEvent::ArcIncrement {
        cycle_id: "c2".into(),
        actor_id: "gm".into(),
        actor_type: ActorType::System,
        target_id: "lin_yueru".into(),
        target_type: TargetType::Npc,
        arc_dimension: "trust_user".into(),
        delta: 45.0,
        world_time: Some("景天元年 三月初一 戌时".into()),
        importance: 0.5,
        summary: "师兄证明了自己的实力".into(),
    };
    process_event(&pool, &e4).unwrap();

    // 验证
    let aff: i64 = conn.query_row("SELECT affinity FROM npc_user_relationships WHERE cycle_id='c2' AND npc_id='lin_yueru'", [], |r| r.get(0)).unwrap();
    assert_eq!(aff, 80);
    let phase: String = conn
        .query_row(
            "SELECT arc_phase FROM character_states WHERE cycle_id='c2' AND npc_id='lin_yueru'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(phase, "初识"); // 45 ≥ 20 → 初识
    let epi: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM episodic_memories WHERE cycle_id='c2'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(epi >= 4);

    println!("  关系: 林月如 affinity=80 | arc: trust=45→初识 | 记忆: {epi} 条");
    println!("══════════════════════════════════════════\n");
}

#[test]
fn cycle3_oc_xianv_humor_style() {
    let pool = setup_demo_db();
    println!("\n═══════════ Cycle 3: OC 仙女散人 · 诙谐吐槽 · 轻松视角 ═══════════");

    let mut sel = HashMap::new();
    sel.insert("lin_yueru".to_string(), "tpl_stranger".to_string());
    sel.insert("li_xiaoyao".to_string(), "tpl_stranger".to_string());
    sel.insert(
        "zhao_linger".to_string(),
        "tpl_love_at_first_sight".to_string(),
    );
    oc_initializer::initialize_cycle_relationships(&pool, "c3", &sel).unwrap();

    let conn = pool.get().unwrap();

    // 仙女散人在苏州集市乱逛
    let e1 = DomainEvent::Dialogue {
        cycle_id: "c3".into(),
        actor_id: "lin_yueru".into(),
        actor_type: ActorType::Npc,
        target_id: Some("user".into()),
        target_type: TargetType::User,
        location_id: Some("suzhou_market".into()),
        line: "这位姐姐，你这衣服好好看！在哪买的？".into(),
        world_time: Some("景天元年 三月初三 巳时".into()),
        importance: 0.3,
        summary: "林月如被仙女散人的衣服吸引".into(),
    };
    process_event(&pool, &e1).unwrap();

    // 遇到妖怪
    let e2 = DomainEvent::StateChange {
        cycle_id: "c3".into(),
        actor_id: "lin_yueru".into(),
        actor_type: ActorType::Npc,
        target_id: Some("lin_yueru".into()),
        target_type: TargetType::Npc,
        field: "mood".into(),
        old_value: None,
        new_value: "happy".into(),
        world_time: Some("景天元年 三月初三 午时".into()),
        importance: 0.4,
        summary: "聊天很开心".into(),
    };
    process_event(&pool, &e2).unwrap();

    // 对赵灵儿一见钟情
    let e3 = DomainEvent::RelationshipChange {
        cycle_id: "c3".into(),
        npc_id: "zhao_linger".into(),
        field: "affinity".into(),
        old_value: Some("70".into()),
        new_value: "85".into(),
        delta: Some(15.0),
        world_time: Some("景天元年 三月初三 未时".into()),
        importance: 0.8,
        summary: "被赵灵儿的温柔打动".into(),
    };
    process_event(&pool, &e3).unwrap();

    // 验证
    let zl_aff: i64 = conn.query_row("SELECT affinity FROM npc_user_relationships WHERE cycle_id='c3' AND npc_id='zhao_linger'", [], |r| r.get(0)).unwrap();
    assert_eq!(zl_aff, 85); // 70(一见钟情) + 15 = 85
    let ev_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM domain_events WHERE cycle_id='c3'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(ev_count, 3);
    let emo: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM emotional_memories WHERE cycle_id='c3'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(emo, 1); // 只有 e1 是 Dialogue

    println!("  赵灵儿 affinity=85 (一见钟情+15) | 事件: 3 | emotional: 1");
    println!("══════════════════════════════════════════\n");
}
