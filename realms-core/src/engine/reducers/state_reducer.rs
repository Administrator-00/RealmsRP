//! State reducer (M2.7) — 处理 DomainEvent::StateChange
//!
//! 负责:
//! - 将 StateChange 事件写入 character_states 表
//! - 字段: hp / mp / location_id / status_flag / arc_phase / mood
//! - 对不存在的行执行 UPSERT (INSERT 默认值 → UPDATE 指定字段)
//! - 产出 RFC6902 PatchOp 供 WebUI 增量更新
//!
//! 约束:
//! - 在给定 `rusqlite::Transaction` 内执行 (由 M2.11 event_reducer 管理事务边界)
//! - 不开启独立事务 (避免嵌套)

use rusqlite::{params, Transaction};

use super::PatchOp;
use crate::error::{RealmsError, Result};
use crate::events::domain_event::{DomainEvent, TargetType};

/// 处理 StateChange 事件, 写入 character_states 并返回 patch.
///
/// # 支持的字段
///
/// | field       | SQL 列         | 值类型 |
/// |-------------|----------------|--------|
/// | `hp`        | hp             | i64    |
/// | `mp`        | mp             | i64    |
/// | `location_id` | location_id  | text   |
/// | `status_flag` | status_flags | JSON merge |
/// | `arc_phase` | arc_phase      | text   |
/// | `mood`      | perceived_mood | text   |
///
/// # 错误
///
/// - event 不是 StateChange → `RealmsError::Event`
/// - 不支持的 field → `RealmsError::InvalidInput`
/// - target_type=Npc 但 target_id=None → `RealmsError::InvalidInput`
/// - hp/mp new_value 非数字 → `RealmsError::InvalidInput`
pub fn apply_state_change(tx: &Transaction<'_>, event: &DomainEvent) -> Result<Vec<PatchOp>> {
    let (cycle_id, actor_id, target_id, target_type, field, _old_value, new_value, _world_time) =
        match event {
            DomainEvent::StateChange {
                cycle_id,
                actor_id,
                target_id,
                target_type,
                field,
                old_value,
                new_value,
                world_time,
                ..
            } => (
                cycle_id.as_str(),
                actor_id.as_str(),
                target_id.as_deref(),
                *target_type,
                field.as_str(),
                old_value.as_deref(),
                new_value.as_str(),
                world_time.as_deref(),
            ),
            _ => {
                return Err(RealmsError::Event(format!(
                    "expected StateChange, got {:?}",
                    event.event_type().as_str()
                )))
            }
        };

    // 确定受影响 NPC: target_id (如果是 Npc) 或 actor_id
    let affected_npc = resolve_affected_npc(actor_id, target_id, target_type)?;

    // 确保行存在 (INSERT 默认值)
    ensure_row_exists(tx, cycle_id, affected_npc)?;

    // 根据 field 执行 UPDATE
    let patch = match field {
        "hp" | "mp" => {
            let val: i64 = new_value
                .parse()
                .map_err(|_| RealmsError::invalid(format!("{field} not a number: {new_value}")))?;
            tx.execute(
                &format!(
                    "UPDATE character_states SET {field} = ?3, updated_at = datetime('now') WHERE cycle_id = ?1 AND npc_id = ?2"
                ),
                params![cycle_id, affected_npc, val],
            )?;
            PatchOp::replace(path(cycle_id, affected_npc, field), serde_json::json!(val))
        }
        "location_id" | "arc_phase" | "mood" => {
            let col = match field {
                "location_id" => "location_id",
                "arc_phase" => "arc_phase",
                "mood" => "perceived_mood",
                _ => unreachable!(),
            };
            tx.execute(
                &format!(
                    "UPDATE character_states SET {col} = ?3, updated_at = datetime('now') WHERE cycle_id = ?1 AND npc_id = ?2"
                ),
                params![cycle_id, affected_npc, new_value],
            )?;
            PatchOp::replace(
                path(cycle_id, affected_npc, field),
                serde_json::json!(new_value),
            )
        }
        "status_flag" => {
            // JSON merge: 读取现有 flags, 合并 new_value, 写回
            let current: String = tx
                .query_row(
                    "SELECT status_flags FROM character_states WHERE cycle_id = ?1 AND npc_id = ?2",
                    params![cycle_id, affected_npc],
                    |r| r.get(0),
                )
                .unwrap_or_else(|_| "{}".into());

            let merged = merge_json_flags(&current, new_value);
            tx.execute(
                "UPDATE character_states SET status_flags = ?3, updated_at = datetime('now') WHERE cycle_id = ?1 AND npc_id = ?2",
                params![cycle_id, affected_npc, &merged],
            )?;
            PatchOp::replace(
                path(cycle_id, affected_npc, "status_flags"),
                serde_json::json!(merged),
            )
        }
        other => {
            return Err(RealmsError::invalid(format!(
                "unsupported state field: {other}"
            )))
        }
    };

    Ok(vec![patch])
}

/// 解析受影响的 NPC id: target_id (Npc) 或 actor_id.
fn resolve_affected_npc<'a>(
    actor_id: &'a str,
    target_id: Option<&'a str>,
    target_type: TargetType,
) -> Result<&'a str> {
    match target_type {
        TargetType::Npc => {
            target_id.ok_or_else(|| RealmsError::invalid("target_type=Npc but target_id is None"))
        }
        _ => Ok(actor_id),
    }
}

/// 确保 character_states 中存在 (cycle, npc) 行 (不存在则 INSERT 默认值).
fn ensure_row_exists(tx: &Transaction<'_>, cycle_id: &str, npc_id: &str) -> Result<()> {
    tx.execute(
        "INSERT OR IGNORE INTO character_states (cycle_id, npc_id) VALUES (?1, ?2)",
        params![cycle_id, npc_id],
    )?;
    Ok(())
}

/// 合并 JSON flags: 读取 current (JSON 字符串), 将 new_flags (逗号分隔 key=value)
/// 合并进去, 返回新 JSON 字符串.
///
/// Example: current={"poisoned":true}, new_flags="cursed=true,poisoned=false"
/// → {"poisoned":false,"cursed":true}
fn merge_json_flags(current: &str, new_flags: &str) -> String {
    let mut map: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(current).unwrap_or_default();

    for pair in new_flags.split(',') {
        let (k, v) = match pair.split_once('=') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue,
        };
        if k.is_empty() {
            continue;
        }
        // 尝试解析为 bool / int / string
        let val = match v {
            "true" => serde_json::Value::Bool(true),
            "false" => serde_json::Value::Bool(false),
            other => {
                if let Ok(n) = other.parse::<i64>() {
                    serde_json::Value::Number(n.into())
                } else {
                    serde_json::Value::String(other.to_string())
                }
            }
        };
        map.insert(k.to_string(), val);
    }

    serde_json::to_string(&map).unwrap_or_else(|_| "{}".into())
}

/// 构建 RFC6902 path: `/character_states/{cycle_id}/{npc_id}/{field}`
fn path(cycle_id: &str, npc_id: &str, field: &str) -> String {
    format!("/character_states/{cycle_id}/{npc_id}/{field}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::domain_event::ActorType;

    fn setup_conn() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().expect("open in-memory");
        conn.execute_batch(crate::engine::reducers::CHARACTER_STATES_DDL)
            .expect("create table");
        conn
    }

    fn make_ev(
        cycle: &str,
        actor: &str,
        target: Option<&str>,
        field: &str,
        val: &str,
    ) -> DomainEvent {
        DomainEvent::StateChange {
            cycle_id: cycle.into(),
            actor_id: actor.into(),
            actor_type: ActorType::Npc,
            target_id: target.map(|s| s.to_string()),
            target_type: if target.is_some() {
                TargetType::Npc
            } else {
                TargetType::User
            },
            field: field.into(),
            old_value: None,
            new_value: val.into(),
            world_time: None,
            importance: 0.3,
            summary: "state change".into(),
        }
    }

    // ===== 基础 =====

    #[test]
    fn hp_change_updates_row() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("begin tx");
        let ev = make_ev("c1", "lin_yueru", Some("lin_yueru"), "hp", "80");
        let patches = apply_state_change(&tx, &ev).expect("apply");
        tx.commit().expect("commit");

        let hp: i64 = conn
            .query_row(
                "SELECT hp FROM character_states WHERE cycle_id='c1' AND npc_id='lin_yueru'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(hp, 80);
        assert_eq!(&patches[0].path, "/character_states/c1/lin_yueru/hp");
        assert_eq!(patches[0].value, Some(serde_json::json!(80)));
    }

    #[test]
    fn mp_change_updates_row() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("begin tx");
        let ev = make_ev("c1", "lin_yueru", Some("lin_yueru"), "mp", "35");
        apply_state_change(&tx, &ev).expect("apply");
        tx.commit().expect("commit");

        let mp: i64 = conn
            .query_row(
                "SELECT mp FROM character_states WHERE cycle_id='c1' AND npc_id='lin_yueru'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(mp, 35);
    }

    #[test]
    fn location_change_upserts() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("begin tx");
        let ev = make_ev("c1", "npc_x", Some("npc_x"), "location_id", "yuhang");
        apply_state_change(&tx, &ev).expect("apply");
        tx.commit().expect("commit");

        let loc: String = conn
            .query_row(
                "SELECT location_id FROM character_states WHERE cycle_id='c1' AND npc_id='npc_x'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(loc, "yuhang");

        let hp: i64 = conn
            .query_row(
                "SELECT hp FROM character_states WHERE cycle_id='c1' AND npc_id='npc_x'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(hp, 100, "默认 HP=100");
    }

    #[test]
    fn mood_change_to_perceived_mood() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("begin tx");
        let ev = make_ev("c1", "lin_yueru", Some("lin_yueru"), "mood", "happy");
        let patches = apply_state_change(&tx, &ev).expect("apply");
        tx.commit().expect("commit");

        let mood: String = conn.query_row(
            "SELECT perceived_mood FROM character_states WHERE cycle_id='c1' AND npc_id='lin_yueru'",
            [], |r| r.get(0)).unwrap();
        assert_eq!(mood, "happy");
        assert!(patches[0].path.contains("mood"));
    }

    #[test]
    fn arc_phase_change() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("begin tx");
        let ev = make_ev("c1", "lin_yueru", Some("lin_yueru"), "arc_phase", "lover");
        apply_state_change(&tx, &ev).expect("apply");
        tx.commit().expect("commit");

        let phase: String = conn
            .query_row(
                "SELECT arc_phase FROM character_states WHERE cycle_id='c1' AND npc_id='lin_yueru'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(phase, "lover");
    }

    #[test]
    fn status_flag_merge_two_steps() {
        let mut conn = setup_conn();
        {
            let tx = conn.transaction().expect("tx1");
            apply_state_change(
                &tx,
                &make_ev("c1", "n", Some("n"), "status_flag", "poisoned=true"),
            )
            .expect("step1");
            tx.commit().expect("commit1");
        }
        {
            let tx = conn.transaction().expect("tx2");
            apply_state_change(
                &tx,
                &make_ev(
                    "c1",
                    "n",
                    Some("n"),
                    "status_flag",
                    "cursed=true,poisoned=false",
                ),
            )
            .expect("step2");
            tx.commit().expect("commit2");
        }
        let flags: String = conn
            .query_row(
                "SELECT status_flags FROM character_states WHERE cycle_id='c1' AND npc_id='n'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let v: serde_json::Value = serde_json::from_str(&flags).unwrap();
        assert_eq!(v["poisoned"], false);
        assert_eq!(v["cursed"], true);
    }

    // ===== 错误 =====

    #[test]
    fn non_state_change_fails() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        let ev = DomainEvent::Setup {
            cycle_id: "c".into(),
            perspective_id: "p".into(),
            gm_id: "g".into(),
            templates_applied: vec![],
        };
        assert!(apply_state_change(&tx, &ev)
            .unwrap_err()
            .to_string()
            .contains("StateChange"));
    }

    #[test]
    fn unsupported_field_fails() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        let ev = make_ev("c", "n", Some("n"), "unknown", "x");
        assert!(apply_state_change(&tx, &ev)
            .unwrap_err()
            .to_string()
            .contains("unsupported"));
    }

    #[test]
    fn hp_non_numeric_fails() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        let ev = make_ev("c", "n", Some("n"), "hp", "abc");
        assert!(apply_state_change(&tx, &ev)
            .unwrap_err()
            .to_string()
            .contains("not a number"));
    }

    #[test]
    fn target_npc_without_id_fails() {
        let mut conn = setup_conn();
        let tx = conn.transaction().expect("tx");
        let ev = DomainEvent::StateChange {
            cycle_id: "c".into(),
            actor_id: "dm".into(),
            actor_type: ActorType::System,
            target_id: None,
            target_type: TargetType::Npc,
            field: "hp".into(),
            old_value: None,
            new_value: "50".into(),
            world_time: None,
            importance: 0.3,
            summary: "bad".into(),
        };
        assert!(apply_state_change(&tx, &ev)
            .unwrap_err()
            .to_string()
            .contains("target_id"));
    }

    // ===== merge_json_flags =====

    #[test]
    fn merge_flags_empty_current() {
        let v: serde_json::Value =
            serde_json::from_str(&merge_json_flags("{}", "poisoned=true")).unwrap();
        assert_eq!(v["poisoned"], true);
    }

    #[test]
    fn merge_flags_overwrite() {
        let v: serde_json::Value = serde_json::from_str(&merge_json_flags(
            r#"{"poisoned":true}"#,
            "poisoned=false,cursed=true",
        ))
        .unwrap();
        assert_eq!(v["poisoned"], false);
        assert_eq!(v["cursed"], true);
    }

    #[test]
    fn merge_flags_numeric() {
        let v: serde_json::Value =
            serde_json::from_str(&merge_json_flags("{}", "stack=3")).unwrap();
        assert_eq!(v["stack"], 3);
    }

    #[test]
    fn merge_flags_empty_pairs_ignored() {
        let v: serde_json::Value = serde_json::from_str(&merge_json_flags("{}", ", =,")).unwrap();
        assert!(v.as_object().unwrap().is_empty());
    }
}
