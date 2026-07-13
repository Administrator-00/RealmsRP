//! Relationship reducer (M2.8) — 处理 DomainEvent::RelationshipChange
//!
//! 负责:
//! - 将 RelationshipChange 写入 npc_user_relationships (当前值)
//! - 追加 relationship_history (变更记录)
//! - 产出 RFC6902 PatchOp
//!
//! 支持的 field: affinity / trust / intimacy / respect / type

use rusqlite::{params, Transaction};

use super::PatchOp;
use crate::error::{RealmsError, Result};
use crate::events::domain_event::DomainEvent;

/// 处理 RelationshipChange, 更新 npc_user_relationships 并追加 history.
pub fn apply_relationship_change(
    tx: &Transaction<'_>,
    event: &DomainEvent,
) -> Result<Vec<PatchOp>> {
    let (cycle_id, npc_id, field, old_value, new_value, delta, world_time) = match event {
        DomainEvent::RelationshipChange {
            cycle_id,
            npc_id,
            field,
            old_value,
            new_value,
            delta,
            world_time,
            ..
        } => (
            cycle_id.as_str(),
            npc_id.as_str(),
            field.as_str(),
            old_value.as_deref(),
            new_value.as_str(),
            *delta,
            world_time.as_deref(),
        ),
        _ => {
            return Err(RealmsError::Event(format!(
                "expected RelationshipChange, got {:?}",
                event.event_type().as_str()
            )))
        }
    };

    // 确保 npc_user_relationships 行存在
    tx.execute(
        "INSERT OR IGNORE INTO npc_user_relationships (cycle_id, npc_id) VALUES (?1, ?2)",
        params![cycle_id, npc_id],
    )?;

    let (col, val_i64) = match field {
        "affinity" => ("affinity", parse_or_error(new_value, "affinity")?),
        "trust" => ("trust", parse_or_error(new_value, "trust")?),
        "intimacy" => ("intimacy", parse_or_error(new_value, "intimacy")?),
        "respect" => ("respect", parse_or_error(new_value, "respect")?),
        "type" => {
            // current_type 是文本字段
            tx.execute(
                "UPDATE npc_user_relationships SET current_type = ?3, updated_at = datetime('now') WHERE cycle_id = ?1 AND npc_id = ?2",
                params![cycle_id, npc_id, new_value],
            )?;
            // history 记录
            insert_history(
                tx, cycle_id, npc_id, "type", old_value, new_value, delta, world_time,
            )?;
            return Ok(vec![PatchOp::replace(
                relation_path(cycle_id, npc_id, "current_type"),
                serde_json::json!(new_value),
            )]);
        }
        other => {
            return Err(RealmsError::invalid(format!(
                "unsupported relationship field: {other}"
            )))
        }
    };

    // 数值字段更新 (affinity/trust/intimacy/respect)
    tx.execute(
        &format!("UPDATE npc_user_relationships SET {col} = ?3, updated_at = datetime('now') WHERE cycle_id = ?1 AND npc_id = ?2"),
        params![cycle_id, npc_id, val_i64],
    )?;

    insert_history(
        tx, cycle_id, npc_id, field, old_value, new_value, delta, world_time,
    )?;

    Ok(vec![PatchOp::replace(
        relation_path(cycle_id, npc_id, field),
        serde_json::json!(val_i64),
    )])
}

/// 解析数值字段 (i64), 失败时返回错误.
fn parse_or_error(val: &str, field: &str) -> Result<i64> {
    val.parse()
        .map_err(|_| RealmsError::invalid(format!("{field} not a number: {val}")))
}

/// 插入 relationship_history.
#[allow(clippy::too_many_arguments)]
fn insert_history(
    tx: &Transaction<'_>,
    cycle_id: &str,
    npc_id: &str,
    field: &str,
    old_value: Option<&str>,
    new_value: &str,
    delta: Option<f64>,
    world_time: Option<&str>,
) -> Result<()> {
    let delta_int: Option<i64> = delta.map(|d| d as i64);
    tx.execute(
        "INSERT INTO relationship_history (cycle_id, npc_id, field, old_value, new_value, delta, world_time) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![cycle_id, npc_id, field, old_value, new_value, delta_int, world_time],
    )?;
    Ok(())
}

/// 构建 RFC6902 path: `/relationships/{cycle_id}/{npc_id}/{field}`
fn relation_path(cycle_id: &str, npc_id: &str, field: &str) -> String {
    format!("/relationships/{cycle_id}/{npc_id}/{field}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::domain_event::DomainEvent;

    fn setup_conn() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().expect("open");
        conn.execute_batch(crate::engine::reducers::RELATIONSHIP_DDL)
            .expect("create rel tables");
        conn
    }

    fn make_rel(
        cycle: &str,
        npc: &str,
        field: &str,
        old: Option<&str>,
        new: &str,
        delta: Option<f64>,
    ) -> DomainEvent {
        DomainEvent::RelationshipChange {
            cycle_id: cycle.into(),
            npc_id: npc.into(),
            field: field.into(),
            old_value: old.map(|s| s.to_string()),
            new_value: new.into(),
            delta,
            world_time: Some("午时".into()),
            importance: 0.5,
            summary: format!("{npc}.{field} → {new}"),
        }
    }

    // ===== 基础 =====

    #[test]
    fn affinity_change_upserts_and_returns_patch() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        let ev = make_rel("c1", "lin_yueru", "affinity", None, "60", Some(60.0));

        let patches = apply_relationship_change(&tx, &ev).expect("apply");
        tx.commit().expect("commit");

        let aff: i64 = conn
            .query_row(
                "SELECT affinity FROM npc_user_relationships WHERE cycle_id='c1' AND npc_id='lin_yueru'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(aff, 60);
        assert_eq!(&patches[0].path, "/relationships/c1/lin_yueru/affinity");
        assert_eq!(patches[0].value, Some(serde_json::json!(60)));
    }

    #[test]
    fn trust_change_persists() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        apply_relationship_change(
            &tx,
            &make_rel("c1", "npc_a", "trust", None, "85", Some(85.0)),
        )
        .expect("apply");
        tx.commit().expect("commit");

        let t: i64 = conn
            .query_row(
                "SELECT trust FROM npc_user_relationships WHERE cycle_id='c1' AND npc_id='npc_a'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(t, 85);
    }

    #[test]
    fn type_change_updates_current_type() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        apply_relationship_change(
            &tx,
            &make_rel("c1", "npc", "type", Some("stranger"), "friend", None),
        )
        .expect("apply");
        tx.commit().expect("commit");

        let t: String = conn
            .query_row(
                "SELECT current_type FROM npc_user_relationships WHERE cycle_id='c1' AND npc_id='npc'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(t, "friend");
    }

    // ===== history =====

    #[test]
    fn history_is_appended() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        apply_relationship_change(&tx, &make_rel("c1", "npc", "trust", None, "50", Some(50.0)))
            .expect("step1");
        apply_relationship_change(
            &tx,
            &make_rel("c1", "npc", "trust", Some("50"), "70", Some(20.0)),
        )
        .expect("step2");
        tx.commit().expect("commit");

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM relationship_history WHERE cycle_id='c1' AND npc_id='npc'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2, "两次变更应有两条 history");

        // 验证第一条 history
        let old: String = conn
            .query_row(
                "SELECT old_value FROM relationship_history WHERE cycle_id='c1' AND npc_id='npc' ORDER BY id LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap_or_default();
        assert!(old.is_empty() || old == "null", "首次变更 old_value 为空");

        // 验证第二条 history 的 old/new
        let (old2, new2): (String, String) = conn
            .query_row(
                "SELECT old_value, new_value FROM relationship_history WHERE cycle_id='c1' AND npc_id='npc' ORDER BY id DESC LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(old2, "50");
        assert_eq!(new2, "70");
    }

    #[test]
    fn delta_is_recorded() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        apply_relationship_change(
            &tx,
            &make_rel("c1", "npc", "affinity", None, "30", Some(30.0)),
        )
        .expect("apply");
        tx.commit().expect("commit");

        let delta: i64 = conn
            .query_row(
                "SELECT delta FROM relationship_history WHERE cycle_id='c1' AND npc_id='npc'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(delta, 30);
    }

    // ===== 错误 =====

    #[test]
    fn non_relationship_event_fails() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        let ev = DomainEvent::Setup {
            cycle_id: "c".into(),
            perspective_id: "p".into(),
            gm_id: "g".into(),
            templates_applied: vec![],
        };
        assert!(apply_relationship_change(&tx, &ev)
            .unwrap_err()
            .to_string()
            .contains("RelationshipChange"));
    }

    #[test]
    fn unsupported_field_fails() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        let ev = make_rel("c", "n", "unknown", None, "x", None);
        assert!(apply_relationship_change(&tx, &ev)
            .unwrap_err()
            .to_string()
            .contains("unsupported"));
    }

    #[test]
    fn non_numeric_field_fails() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        let ev = make_rel("c", "n", "affinity", None, "abc", None);
        assert!(apply_relationship_change(&tx, &ev)
            .unwrap_err()
            .to_string()
            .contains("not a number"));
    }
}
