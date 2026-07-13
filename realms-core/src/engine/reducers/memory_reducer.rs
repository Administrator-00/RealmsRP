//! Memory reducer (M2.10) — 3 层记忆写入
//!
//! 负责:
//! - episodic_memories: 每个事件为 actor 记录情景记忆
//! - emotional_memories: Dialogue 事件额外记录情感记忆
//! - semantic_memories: 暂为桩 (M4 完成后扩展)

use rusqlite::{params, Transaction};

use super::PatchOp;
use crate::error::{RealmsError, Result};
use crate::events::domain_event::{DomainEvent, EventType};

/// 自动记录事件到 3 层记忆.
///
/// - **episodic**: actor + target (若 Npc) 各 1 条
/// - **emotional**: 仅 Dialogue 类型, 记录交互双方的情感印记
/// - **semantic**: 当前占位 (未实现)
pub fn record_memories(tx: &Transaction<'_>, event: &DomainEvent) -> Result<Vec<PatchOp>> {
    if event.cycle_id().is_empty() {
        return Err(RealmsError::invalid("cycle_id is empty"));
    }

    let cycle_id = event.cycle_id();
    let summary = event.summary();
    let mut patches = Vec::new();

    // === episodic: actor 记一条 ===
    let actor = event_actor_id(event);
    if let Some(a) = actor {
        if !a.is_empty() {
            tx.execute(
                "INSERT INTO episodic_memories (cycle_id, character_id, event_summary) VALUES (?1,?2,?3)",
                params![cycle_id, a, summary],
            )?;
            patches.push(PatchOp::add(
                format!("/memories/episodic/{cycle_id}/{a}"),
                serde_json::json!(summary),
            ));
        }
    }

    // === episodic: target (若 Npc) 也记一条 ===
    let target = event_target_id(event);
    if let Some(t) = target {
        if !t.is_empty() && Some(t) != actor {
            tx.execute(
                "INSERT INTO episodic_memories (cycle_id, character_id, event_summary) VALUES (?1,?2,?3)",
                params![cycle_id, t, summary],
            )?;
            patches.push(PatchOp::add(
                format!("/memories/episodic/{cycle_id}/{t}"),
                serde_json::json!(summary),
            ));
        }
    }

    // === emotional: Dialogue 记录交互情感 ===
    if matches!(event.event_type(), EventType::Dialogue) {
        let emotion_type = default_emotion_for_dialogue(event);
        if let Some(a) = actor {
            let tgt = target.unwrap_or("unknown");
            tx.execute(
                "INSERT INTO emotional_memories (cycle_id, character_id, target_id, emotion_type, trigger_summary) VALUES (?1,?2,?3,?4,?5)",
                params![cycle_id, a, tgt, emotion_type, summary],
            )?;
            patches.push(PatchOp::add(
                format!("/memories/emotional/{cycle_id}/{a}"),
                serde_json::json!({"target": tgt, "emotion": emotion_type}),
            ));
        }
    }

    Ok(patches)
}

/// 从事件的 importance 推断情感类型 (stub: 后续 LLM 可覆盖).
fn default_emotion_for_dialogue(_event: &DomainEvent) -> &'static str {
    // M2 阶段使用固定默认值; M7 接入 LLM 后可由 subagent 输出真实情感标签
    "neutral"
}

/// 提取 actor_id (复用 gm_router 逻辑).
fn event_actor_id(ev: &DomainEvent) -> Option<&str> {
    match ev {
        DomainEvent::Setup { .. } | DomainEvent::TimeAdvance { .. } => None,
        DomainEvent::ArcIncrement { actor_id, .. } => Some(actor_id.as_str()),
        DomainEvent::RelationshipChange { npc_id, .. } => Some(npc_id.as_str()),
        DomainEvent::StateChange { actor_id, .. } => Some(actor_id.as_str()),
        DomainEvent::ItemChange { owner_id, .. } => Some(owner_id.as_str()),
        DomainEvent::Dialogue { actor_id, .. } => Some(actor_id.as_str()),
        DomainEvent::Discovery { discoverer_id, .. } => Some(discoverer_id.as_str()),
        DomainEvent::Combat { attacker_id, .. } => Some(attacker_id.as_str()),
    }
}

/// 提取 target_id.
fn event_target_id(ev: &DomainEvent) -> Option<&str> {
    match ev {
        DomainEvent::Setup { .. }
        | DomainEvent::ItemChange { .. }
        | DomainEvent::TimeAdvance { .. }
        | DomainEvent::Discovery { .. } => None,
        DomainEvent::ArcIncrement { target_id, .. } => Some(target_id.as_str()),
        DomainEvent::RelationshipChange { npc_id, .. } => Some(npc_id.as_str()),
        DomainEvent::StateChange { target_id, .. } => target_id.as_deref(),
        DomainEvent::Dialogue { target_id, .. } => target_id.as_deref(),
        DomainEvent::Combat { defender_id, .. } => Some(defender_id.as_str()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::domain_event::{ActorType, TargetType};

    fn setup_conn() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().expect("open");
        conn.execute_batch(crate::engine::reducers::MEMORY_DDL)
            .expect("ddl");
        conn
    }

    fn make_dialogue(cycle: &str, actor: &str, target: &str) -> DomainEvent {
        DomainEvent::Dialogue {
            cycle_id: cycle.into(),
            actor_id: actor.into(),
            actor_type: ActorType::Npc,
            target_id: Some(target.into()),
            target_type: TargetType::Npc,
            location_id: None,
            line: "...".into(),
            world_time: None,
            importance: 0.4,
            summary: format!("{actor} 对 {target} 说话"),
        }
    }

    #[test]
    fn episodic_for_actor() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        let ev = make_dialogue("c1", "lin_yueru", "xiaoyao");
        let patches = record_memories(&tx, &ev).expect("record");
        tx.commit().expect("commit");

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM episodic_memories WHERE cycle_id='c1' AND character_id='lin_yueru'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "actor 应有 1 条 episodic");
        assert!(!patches.is_empty());
    }

    #[test]
    fn episodic_for_target() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        record_memories(&tx, &make_dialogue("c1", "a", "b")).expect("ok");
        tx.commit().expect("commit");

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM episodic_memories WHERE cycle_id='c1' AND character_id='b'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "target 也应有 episodic");
    }

    #[test]
    fn dialogue_creates_emotional() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        record_memories(&tx, &make_dialogue("c1", "lin_yueru", "xiaoyao")).expect("ok");
        tx.commit().expect("commit");

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM emotional_memories WHERE cycle_id='c1' AND character_id='lin_yueru'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "Dialogue 应产生 emotional 记忆");
    }

    #[test]
    fn non_dialogue_no_emotional() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        let ev = DomainEvent::Discovery {
            cycle_id: "c1".into(),
            discoverer_id: "npc".into(),
            location_id: "loc".into(),
            location_name: "某处".into(),
            world_time: None,
            importance: 0.3,
            summary: "发现".into(),
        };
        record_memories(&tx, &ev).expect("ok");
        tx.commit().expect("commit");

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM emotional_memories WHERE cycle_id='c1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "非 Dialogue 不应产生 emotional");
    }

    #[test]
    fn empty_cycle_id_fails() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        let ev = DomainEvent::Dialogue {
            cycle_id: "".into(),
            actor_id: "a".into(),
            actor_type: ActorType::Npc,
            target_id: Some("b".into()),
            target_type: TargetType::Npc,
            location_id: None,
            line: "".into(),
            world_time: None,
            importance: 0.3,
            summary: "bad".into(),
        };
        assert!(record_memories(&tx, &ev)
            .unwrap_err()
            .to_string()
            .contains("cycle_id"));
    }
}
