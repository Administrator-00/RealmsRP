//! Arc reducer (M2.9) — 处理 DomainEvent::ArcIncrement
//!
//! 负责:
//! - 读取 character_states.arcs JSON, 对指定 arc_dimension 加上 delta
//! - 按里程碑判定 arc_phase (仅对 trust_user / affection_user / hostility_user)
//! - 产出 RFC6902 PatchOp

use rusqlite::{params, Transaction};

use super::PatchOp;
use crate::error::{RealmsError, Result};
use crate::events::domain_event::{DomainEvent, TargetType};

/// 会触发 arc_phase 更新的关系维度.
const RELATION_DIMENSIONS: &[&str] = &["trust_user", "affection_user", "hostility_user"];

/// arc_dimension → milestones 映射 (3.md §2.2 arc_dimensions).
/// 每个 (threshold, label), threshold 从低到高.
const ARC_MILESTONES: &[(&str, &[(i64, &str)])] = &[
    (
        "trust_user",
        &[
            (0, "陌生人"),
            (20, "初识"),
            (50, "朋友"),
            (80, "生死之交"),
            (100, "以命相托"),
        ],
    ),
    (
        "affection_user",
        &[(0, "无感"), (30, "萌芽"), (60, "暗恋"), (90, "表白")],
    ),
    (
        "hostility_user",
        &[(0, "无"), (40, "不满"), (70, "厌恶"), (100, "死敌")],
    ),
    (
        "courage",
        &[(0, "怯懦"), (30, "平常"), (60, "勇敢"), (100, "无畏")],
    ),
    (
        "maturity",
        &[(0, "天真"), (30, "成长中"), (60, "成熟"), (100, "通透")],
    ),
];

/// 处理 ArcIncrement, 更新 character_states.arcs 并判定 arc_phase.
pub fn apply_arc_increment(tx: &Transaction<'_>, event: &DomainEvent) -> Result<Vec<PatchOp>> {
    let (cycle_id, target_id, target_type, arc_dimension, delta) = match event {
        DomainEvent::ArcIncrement {
            cycle_id,
            target_id,
            target_type,
            arc_dimension,
            delta,
            ..
        } => (
            cycle_id.as_str(),
            target_id.as_str(),
            *target_type,
            arc_dimension.as_str(),
            *delta,
        ),
        _ => {
            return Err(RealmsError::Event(format!(
                "expected ArcIncrement, got {:?}",
                event.event_type().as_str()
            )))
        }
    };

    // 确定受影响 NPC
    let affected_npc = match target_type {
        TargetType::Npc => target_id,
        _ => return Err(RealmsError::invalid("ArcIncrement target_type must be Npc")),
    };

    // 确保行存在
    tx.execute(
        "INSERT OR IGNORE INTO character_states (cycle_id, npc_id) VALUES (?1, ?2)",
        params![cycle_id, affected_npc],
    )?;

    // 读取当前 arcs
    let current_json: String = tx
        .query_row(
            "SELECT arcs FROM character_states WHERE cycle_id = ?1 AND npc_id = ?2",
            params![cycle_id, affected_npc],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "{}".into());

    let mut arcs: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&current_json).unwrap_or_default();

    // 更新目标维度 (clamp [0, 100])
    let old_val: f64 = arcs
        .get(arc_dimension)
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let new_val = (old_val + delta).clamp(0.0, 100.0);
    arcs.insert(arc_dimension.to_string(), serde_json::json!(new_val));

    let new_json = serde_json::to_string(&arcs).unwrap_or_else(|_| "{}".into());

    tx.execute(
        "UPDATE character_states SET arcs = ?3, updated_at = datetime('now') WHERE cycle_id = ?1 AND npc_id = ?2",
        params![cycle_id, affected_npc, &new_json],
    )?;

    let mut patches = vec![PatchOp::replace(
        format!("/character_states/{cycle_id}/{affected_npc}/arcs/{arc_dimension}"),
        serde_json::json!(new_val),
    )];

    // 判定 arc_phase (仅关系维度)
    if RELATION_DIMENSIONS.contains(&arc_dimension) {
        if let Some(phase) = determine_arc_phase(arc_dimension, new_val as i64) {
            tx.execute(
                "UPDATE character_states SET arc_phase = ?3, updated_at = datetime('now') WHERE cycle_id = ?1 AND npc_id = ?2",
                params![cycle_id, affected_npc, phase],
            )?;
            patches.push(PatchOp::replace(
                format!("/character_states/{cycle_id}/{affected_npc}/arc_phase"),
                serde_json::json!(phase),
            ));
        }
    }

    Ok(patches)
}

/// 根据 arc_dimension 和当前值查找里程碑阶段.
///
/// 返回最高的 `threshold ≤ current_value` 对应的 label.
/// 非关系维度 (courage/maturity 等) 返回 None (不更新 arc_phase).
fn determine_arc_phase(dimension: &str, current_value: i64) -> Option<&'static str> {
    let milestones = ARC_MILESTONES.iter().find(|(dim, _)| *dim == dimension)?.1;

    // milestones 按 threshold 升序, 找最后一个 ≤ current_value
    let label = milestones
        .iter()
        .rev()
        .find(|(thresh, _)| *thresh <= current_value)?
        .1;
    Some(label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::domain_event::{ActorType, DomainEvent};

    fn setup_conn() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().expect("open");
        conn.execute_batch(crate::engine::reducers::CHARACTER_STATES_DDL)
            .expect("ddl");
        conn
    }

    fn make_arc(cycle: &str, target: &str, dim: &str, delta: f64) -> DomainEvent {
        DomainEvent::ArcIncrement {
            cycle_id: cycle.into(),
            actor_id: "gm".into(),
            actor_type: ActorType::System,
            target_id: target.into(),
            target_type: TargetType::Npc,
            arc_dimension: dim.into(),
            delta,
            world_time: None,
            importance: 0.5,
            summary: format!("{target}.{dim} += {delta}"),
        }
    }

    // ===== 基础 =====

    #[test]
    fn trust_increment_from_zero() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        let patches = apply_arc_increment(&tx, &make_arc("c1", "lin_yueru", "trust_user", 25.0))
            .expect("apply");
        tx.commit().expect("commit");

        let arcs_json: String = conn
            .query_row(
                "SELECT arcs FROM character_states WHERE cycle_id='c1' AND npc_id='lin_yueru'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let arcs: serde_json::Value = serde_json::from_str(&arcs_json).unwrap();
        assert_eq!(arcs["trust_user"], 25.0);

        // 验证 arc_phase 更新 (25 ≥ 20 → "初识")
        let phase: String = conn
            .query_row(
                "SELECT arc_phase FROM character_states WHERE cycle_id='c1' AND npc_id='lin_yueru'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(phase, "初识");

        // 验证 patches
        assert_eq!(patches.len(), 2, "应产出 arcs + arc_phase 两个 patch");
    }

    #[test]
    fn trust_increment_milestone_transition() {
        let mut conn = setup_conn();
        // 先用 trust=45 初始化 (朋友阈值是 50, 未达到)
        {
            let tx = conn.transaction().expect("tx");
            apply_arc_increment(&tx, &make_arc("c1", "npca", "trust_user", 45.0)).expect("step1");
            tx.commit().expect("commit");
        }
        // 再加 10 → 55, 跨过 50 → "朋友"
        {
            let tx = conn.transaction().expect("tx");
            apply_arc_increment(&tx, &make_arc("c1", "npca", "trust_user", 10.0)).expect("step2");
            tx.commit().expect("commit");
        }
        let phase: String = conn
            .query_row(
                "SELECT arc_phase FROM character_states WHERE cycle_id='c1' AND npc_id='npca'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(phase, "朋友");
    }

    #[test]
    fn affection_user_phase() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        // affection 65 → 70 (跨过 60 "暗恋" 但没跨 90)
        apply_arc_increment(&tx, &make_arc("c1", "npc", "affection_user", 70.0)).expect("apply");
        tx.commit().expect("commit");

        let phase: String = conn
            .query_row(
                "SELECT arc_phase FROM character_states WHERE cycle_id='c1' AND npc_id='npc'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(phase, "暗恋");
    }

    #[test]
    fn negative_delta_decreases() {
        let mut conn = setup_conn();
        {
            let tx = conn.transaction().expect("tx");
            apply_arc_increment(&tx, &make_arc("c1", "npc", "trust_user", 60.0)).expect("set");
            tx.commit().expect("commit");
        }
        {
            let tx = conn.transaction().expect("tx");
            apply_arc_increment(&tx, &make_arc("c1", "npc", "trust_user", -45.0)).expect("neg");
            tx.commit().expect("commit");
        }

        let arcs_json: String = conn
            .query_row(
                "SELECT arcs FROM character_states WHERE cycle_id='c1' AND npc_id='npc'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let arcs: serde_json::Value = serde_json::from_str(&arcs_json).unwrap();
        assert_eq!(arcs["trust_user"], 15.0, "60 - 45 = 15");

        // arc_phase 回到 "陌生人" (15 < 20)
        let phase: String = conn
            .query_row(
                "SELECT arc_phase FROM character_states WHERE cycle_id='c1' AND npc_id='npc'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(phase, "陌生人");
    }

    #[test]
    fn clamp_to_zero() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        apply_arc_increment(&tx, &make_arc("c1", "npc", "trust_user", -999.0)).expect("apply");
        tx.commit().expect("commit");

        let arcs_json: String = conn
            .query_row(
                "SELECT arcs FROM character_states WHERE cycle_id='c1' AND npc_id='npc'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let arcs: serde_json::Value = serde_json::from_str(&arcs_json).unwrap();
        assert_eq!(arcs["trust_user"], 0.0, "clamp 到 0");
    }

    #[test]
    fn clamp_to_100() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        apply_arc_increment(&tx, &make_arc("c1", "npc", "trust_user", 999.0)).expect("apply");
        tx.commit().expect("commit");

        let arcs_json: String = conn
            .query_row(
                "SELECT arcs FROM character_states WHERE cycle_id='c1' AND npc_id='npc'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let arcs: serde_json::Value = serde_json::from_str(&arcs_json).unwrap();
        assert_eq!(arcs["trust_user"], 100.0, "clamp 到 100");
    }

    #[test]
    fn courage_dimension_no_arc_phase() {
        // courage 不是关系维度, 不应更新 arc_phase
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        let patches =
            apply_arc_increment(&tx, &make_arc("c1", "npc", "courage", 50.0)).expect("apply");
        tx.commit().expect("commit");

        // 只有一个 patch (arcs), 没有 arc_phase patch
        assert_eq!(patches.len(), 1, "非关系维度不应产出 arc_phase patch");

        let phase: Option<String> = conn
            .query_row(
                "SELECT arc_phase FROM character_states WHERE cycle_id='c1' AND npc_id='npc'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(phase.is_none(), "非关系维度不设 arc_phase");
    }

    // ===== 错误 =====

    #[test]
    fn wrong_event_type_fails() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        let ev = DomainEvent::Setup {
            cycle_id: "c".into(),
            perspective_id: "p".into(),
            gm_id: "g".into(),
            templates_applied: vec![],
        };
        assert!(apply_arc_increment(&tx, &ev)
            .unwrap_err()
            .to_string()
            .contains("ArcIncrement"));
    }

    // ===== determine_arc_phase 纯函数 =====

    #[test]
    fn determine_phase_matches_milestone() {
        assert_eq!(determine_arc_phase("trust_user", 0), Some("陌生人"));
        assert_eq!(determine_arc_phase("trust_user", 19), Some("陌生人"));
        assert_eq!(determine_arc_phase("trust_user", 20), Some("初识"));
        assert_eq!(determine_arc_phase("trust_user", 55), Some("朋友"));
        assert_eq!(determine_arc_phase("trust_user", 100), Some("以命相托"));
    }

    #[test]
    fn unknown_dimension_returns_none() {
        assert_eq!(determine_arc_phase("nonexistent", 50), None);
    }
}
