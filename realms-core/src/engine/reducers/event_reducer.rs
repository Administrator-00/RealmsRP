//! Event reducer 主入口 (M2.11) — 串接所有子 reducer + domain_events 写入
//!
//! `process_event` 在单个事务中:
//! 1. 写 domain_events 表
//! 2. 按事件类型分发到对应子 reducer
//! 3. 自动调用 memory_reducer 写记忆层
//! 4. 提交并返回聚合 PatchOp

use rusqlite::params;

use super::{arc_reducer, memory_reducer, relationship_reducer, state_reducer, PatchOp};
use crate::db::pool::DbPool;
use crate::error::{RealmsError, Result};
use crate::events::domain_event::{DomainEvent, EventType};

/// 处理单个 domain_event: 写 event 行 → 分发 reducer → 记忆 → 返回 patches.
///
/// 在单事务内执行, 任何一步失败则回滚.
pub fn process_event(pool: &DbPool, event: &DomainEvent) -> Result<Vec<PatchOp>> {
    let mut conn = pool
        .get()
        .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;
    let tx = conn.transaction()?;

    // 1. 写 domain_events 表
    let cycle_id = event.cycle_id();
    let event_type = event.event_type().as_str();
    let (actor_id, target_id) = extract_ids(event);
    let importance = event.importance();
    let summary = event.summary();

    tx.execute(
        "INSERT INTO domain_events (cycle_id, event_type, actor_id, target_id, importance, summary) VALUES (?1,?2,?3,?4,?5,?6)",
        params![cycle_id, event_type, actor_id, target_id, importance, summary],
    )?;

    let mut all_patches = vec![PatchOp::add(
        format!("/events/{cycle_id}"),
        serde_json::json!({"type": event_type, "summary": summary}),
    )];

    // 2. 按事件类型分发子 reducer
    match event.event_type() {
        EventType::Setup => {
            // Setup 事件不触发数据变更, 仅记录
        }
        EventType::StateChange => {
            all_patches.extend(state_reducer::apply_state_change(&tx, event)?);
        }
        EventType::RelationshipChange => {
            all_patches.extend(relationship_reducer::apply_relationship_change(&tx, event)?);
        }
        EventType::ArcIncrement => {
            all_patches.extend(arc_reducer::apply_arc_increment(&tx, event)?);
        }
        EventType::ItemChange
        | EventType::TimeAdvance
        | EventType::Dialogue
        | EventType::Discovery
        | EventType::Combat => {
            // 这些类型在 M2 阶段不触发专用 reducer (M7 后扩展)
        }
    }

    // 3. 自动记录记忆 (所有事件都记 episodic)
    all_patches.extend(memory_reducer::record_memories(&tx, event)?);

    tx.commit()?;
    Ok(all_patches)
}

/// 提取 actor_id / target_id (用于 domain_events 表的冗余列).
fn extract_ids(event: &DomainEvent) -> (Option<String>, Option<String>) {
    match event {
        DomainEvent::Setup { .. } | DomainEvent::TimeAdvance { .. } => (None, None),
        DomainEvent::ArcIncrement {
            actor_id,
            target_id,
            ..
        } => (Some(actor_id.clone()), Some(target_id.clone())),
        DomainEvent::RelationshipChange { npc_id, .. } => {
            (Some(npc_id.clone()), Some(npc_id.clone()))
        }
        DomainEvent::StateChange {
            actor_id,
            target_id,
            ..
        } => (Some(actor_id.clone()), target_id.clone()),
        DomainEvent::ItemChange { owner_id, .. } => (Some(owner_id.clone()), None),
        DomainEvent::Dialogue {
            actor_id,
            target_id,
            ..
        } => (Some(actor_id.clone()), target_id.clone()),
        DomainEvent::Discovery { discoverer_id, .. } => (Some(discoverer_id.clone()), None),
        DomainEvent::Combat {
            attacker_id,
            defender_id,
            ..
        } => (Some(attacker_id.clone()), Some(defender_id.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::pool::init_memory_pool;
    use crate::events::domain_event::{ActorType, TargetType};

    fn setup_pool() -> DbPool {
        let pool = init_memory_pool().expect("pool");
        let conn = pool.get().expect("conn");
        conn.execute_batch(&format!(
            "{}\n{}\n{}\n{}",
            crate::engine::reducers::CHARACTER_STATES_DDL,
            crate::engine::reducers::RELATIONSHIP_DDL,
            crate::engine::reducers::MEMORY_DDL,
            crate::engine::reducers::DOMAIN_EVENTS_DDL,
        ))
        .expect("schema");
        pool
    }

    // ===== 集成测试 =====

    #[test]
    fn process_state_change_writes_event_and_state() {
        let pool = setup_pool();
        let ev = DomainEvent::StateChange {
            cycle_id: "c1".into(),
            actor_id: "lin_yueru".into(),
            actor_type: ActorType::Npc,
            target_id: Some("lin_yueru".into()),
            target_type: TargetType::Npc,
            field: "hp".into(),
            old_value: None,
            new_value: "75".into(),
            world_time: None,
            importance: 0.4,
            summary: "林月如受伤 HP-25".into(),
        };
        let patches = process_event(&pool, &ev).expect("process");
        assert!(!patches.is_empty(), "应产出 patches");

        // 验证 domain_events
        let conn = pool.get().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM domain_events WHERE cycle_id='c1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "应写 1 条 domain_event");

        // 验证 character_states
        let hp: i64 = conn
            .query_row(
                "SELECT hp FROM character_states WHERE cycle_id='c1' AND npc_id='lin_yueru'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(hp, 75);
    }

    #[test]
    fn process_relationship_change_writes_both_tables() {
        let pool = setup_pool();
        let ev = DomainEvent::RelationshipChange {
            cycle_id: "c1".into(),
            npc_id: "npc".into(),
            field: "affinity".into(),
            old_value: None,
            new_value: "50".into(),
            delta: Some(50.0),
            world_time: None,
            importance: 0.5,
            summary: "好感 +50".into(),
        };
        let patches = process_event(&pool, &ev).expect("process");
        assert!(patches.len() >= 2, "应有 event patch + affinity patch");

        let conn = pool.get().unwrap();
        let aff: i64 = conn
            .query_row(
                "SELECT affinity FROM npc_user_relationships WHERE cycle_id='c1' AND npc_id='npc'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(aff, 50);

        let hist: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM relationship_history WHERE cycle_id='c1' AND npc_id='npc'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(hist, 1, "应有 1 条 history");
    }

    #[test]
    fn process_arc_increment_updates_arcs_and_phase() {
        let pool = setup_pool();
        let ev = DomainEvent::ArcIncrement {
            cycle_id: "c1".into(),
            actor_id: "gm".into(),
            actor_type: ActorType::System,
            target_id: "npc".into(),
            target_type: TargetType::Npc,
            arc_dimension: "trust_user".into(),
            delta: 55.0,
            world_time: None,
            importance: 0.6,
            summary: "信任 +55".into(),
        };
        let patches = process_event(&pool, &ev).expect("process");
        assert!(patches.len() >= 2, "应有 event + arcs/phase patches");

        let conn = pool.get().unwrap();
        let arcs_json: String = conn
            .query_row(
                "SELECT arcs FROM character_states WHERE cycle_id='c1' AND npc_id='npc'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let arcs: serde_json::Value = serde_json::from_str(&arcs_json).unwrap();
        assert_eq!(arcs["trust_user"], 55.0);

        let phase: String = conn
            .query_row(
                "SELECT arc_phase FROM character_states WHERE cycle_id='c1' AND npc_id='npc'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(phase, "朋友", "55 ≥ 50 → 朋友");
    }

    #[test]
    fn process_dialogue_writes_event_and_memory_only() {
        let pool = setup_pool();
        let ev = DomainEvent::Dialogue {
            cycle_id: "c1".into(),
            actor_id: "a".into(),
            actor_type: ActorType::Npc,
            target_id: Some("b".into()),
            target_type: TargetType::Npc,
            location_id: None,
            line: "你好".into(),
            world_time: None,
            importance: 0.2,
            summary: "a 对 b 打招呼".into(),
        };
        let patches = process_event(&pool, &ev).expect("process");
        // 应有: event patch + 2 episodic patches + 1 emotional patch
        assert!(patches.len() >= 3);

        let conn = pool.get().unwrap();
        let epi_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM episodic_memories WHERE cycle_id='c1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(epi_count, 2, "a 和 b 各 1 条 episodic");

        let emo_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM emotional_memories WHERE cycle_id='c1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(emo_count, 1, "Dialogue 应有 1 条 emotional");
    }

    #[test]
    fn transaction_rollback_on_error() {
        let pool = setup_pool();
        // 无效 arc_dimension 会导致 arc_reducer 成功但无 arc_phase (这不会报错)
        // 改用空 cycle_id
        let ev = DomainEvent::Dialogue {
            cycle_id: "".into(),
            actor_id: "a".into(),
            actor_type: ActorType::Npc,
            target_id: None,
            target_type: TargetType::None,
            location_id: None,
            line: "".into(),
            world_time: None,
            importance: 0.1,
            summary: "bad".into(),
        };
        let result = process_event(&pool, &ev);
        assert!(result.is_err());

        // 验证没有写入
        let conn = pool.get().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM domain_events WHERE cycle_id=''",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "事务应回滚, 无脏数据");
    }

    #[test]
    fn demo_memory_inspection() {
        let pool = setup_pool();

        // 模拟一连串事件: 客栈老板招呼 → 林月如受伤 → 好感变化
        let events: Vec<DomainEvent> = vec![
            DomainEvent::Dialogue {
                cycle_id: "c1".into(),
                actor_id: "npc_innkeeper".into(),
                actor_type: ActorType::Npc,
                target_id: Some("user".into()),
                target_type: TargetType::User,
                location_id: Some("yuhang_town".into()),
                line: "客官里面请！".into(),
                world_time: Some("午时".into()),
                importance: 0.3,
                summary: "客栈老板招呼客人".into(),
            },
            DomainEvent::StateChange {
                cycle_id: "c1".into(),
                actor_id: "lin_yueru".into(),
                actor_type: ActorType::Npc,
                target_id: Some("lin_yueru".into()),
                target_type: TargetType::Npc,
                field: "hp".into(),
                old_value: None,
                new_value: "60".into(),
                world_time: Some("未时".into()),
                importance: 0.7,
                summary: "林月如战斗中受伤 HP→60".into(),
            },
            DomainEvent::RelationshipChange {
                cycle_id: "c1".into(),
                npc_id: "lin_yueru".into(),
                field: "affinity".into(),
                old_value: None,
                new_value: "55".into(),
                delta: Some(55.0),
                world_time: Some("未时".into()),
                importance: 0.6,
                summary: "林月如好感 +55".into(),
            },
        ];

        for ev in &events {
            process_event(&pool, ev).expect("process");
        }

        let conn = pool.get().unwrap();

        println!("\n========== 三层记忆内容 ==========\n");

        // L1: episodic_memories
        println!("【L1 情景记忆 episodic_memories】");
        let mut stmt = conn
            .prepare("SELECT id, character_id, content, created_at FROM episodic_memories WHERE cycle_id='c1' ORDER BY id")
            .unwrap();
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .unwrap();
        for row in rows {
            let (id, cid, content, ts) = row.unwrap();
            println!("  [{id}] {cid:>16} | {content} | {ts}");
        }

        // L2: semantic_memories
        println!("\n【L2 语义记忆 semantic_memories】");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM semantic_memories WHERE cycle_id='c1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        println!("  (stub: 共 {count} 条, 语义层写入待 M10 扩展)");

        // L3: emotional_memories
        println!("\n【L3 情感记忆 emotional_memories】");
        let mut stmt = conn
            .prepare("SELECT id, character_id, target_id, emotion_type, intensity, context FROM emotional_memories WHERE cycle_id='c1' ORDER BY id")
            .unwrap();
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, f64>(4)?,
                    r.get::<_, String>(5)?,
                ))
            })
            .unwrap();
        for row in rows {
            let (id, cid, tid, etype, inten, ctx) = row.unwrap();
            println!("  [{id}] {cid:>16} → {tid:>8} | {etype:>8} (intensity={inten}) | {ctx}");
        }

        // 总计
        let epi: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM episodic_memories WHERE cycle_id='c1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let emo: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM emotional_memories WHERE cycle_id='c1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        println!("\n  合计: episodic={epi}, emotional={emo}\n");

        assert!(epi >= 2, "应有至少 2 条 episodic (actor+target)");
        assert!(emo >= 1, "Dialogue 应有 emotional");
    }
}
