//! Character state queries (M4.5)

use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::db::pool::DbPool;
use crate::error::{RealmsError, Result};

/// character_states 行投影
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStateRow {
    /// 周目 id
    pub cycle_id: String,
    /// NPC id
    pub npc_id: String,
    /// arcs JSON (各维度当前值)
    pub arcs_json: String,
    /// 当前 HP
    pub hp: i64,
    /// 最大 HP
    pub hp_max: i64,
    /// 当前 MP
    pub mp: i64,
    /// 最大 MP
    pub mp_max: i64,
    /// 当前位置 id
    pub location_id: Option<String>,
    /// status_flags JSON
    pub status_flags_json: String,
    /// 当前弧光阶段
    pub arc_phase: Option<String>,
    /// 情绪
    pub perceived_mood: String,
    /// 最近事件 id
    pub last_event_id: Option<i64>,
}

/// 加载单个 NPC 状态.
pub fn load_character_state(
    pool: &DbPool,
    cycle_id: &str,
    npc_id: &str,
) -> Result<CharacterStateRow> {
    let conn = pool
        .get()
        .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;
    conn.query_row(
        "SELECT cycle_id, npc_id, arcs, hp, hp_max, mp, mp_max,
                location_id, status_flags, arc_phase, perceived_mood, last_event_id
         FROM character_states WHERE cycle_id=?1 AND npc_id=?2",
        params![cycle_id, npc_id],
        |r| {
            Ok(CharacterStateRow {
                cycle_id: r.get(0)?,
                npc_id: r.get(1)?,
                arcs_json: r.get(2)?,
                hp: r.get(3)?,
                hp_max: r.get(4)?,
                mp: r.get(5)?,
                mp_max: r.get(6)?,
                location_id: r.get(7)?,
                status_flags_json: r.get(8)?,
                arc_phase: r.get(9)?,
                perceived_mood: r.get(10)?,
                last_event_id: r.get(11)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            RealmsError::not_found("character_state", &format!("{cycle_id}/{npc_id}"))
        }
        other => RealmsError::Database(other),
    })
}

/// 列出某 cycle 下所有 NPC 状态.
pub fn list_character_states(pool: &DbPool, cycle_id: &str) -> Result<Vec<CharacterStateRow>> {
    let conn = pool
        .get()
        .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;
    let mut stmt = conn
        .prepare(
            "SELECT cycle_id, npc_id, arcs, hp, hp_max, mp, mp_max,
                    location_id, status_flags, arc_phase, perceived_mood, last_event_id
             FROM character_states WHERE cycle_id=?1 ORDER BY npc_id",
        )
        .map_err(RealmsError::Database)?;

    let rows = stmt
        .query_map(params![cycle_id], |r| {
            Ok(CharacterStateRow {
                cycle_id: r.get(0)?,
                npc_id: r.get(1)?,
                arcs_json: r.get(2)?,
                hp: r.get(3)?,
                hp_max: r.get(4)?,
                mp: r.get(5)?,
                mp_max: r.get(6)?,
                location_id: r.get(7)?,
                status_flags_json: r.get(8)?,
                arc_phase: r.get(9)?,
                perceived_mood: r.get(10)?,
                last_event_id: r.get(11)?,
            })
        })
        .map_err(RealmsError::Database)?;

    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(RealmsError::Database)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::pool::init_memory_pool;

    #[test]
    fn load_nonexistent_returns_not_found() {
        let pool = init_memory_pool().expect("pool");
        let err = load_character_state(&pool, "c1", "npc_x").unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn list_empty_cycle() {
        let pool = init_memory_pool().expect("pool");
        let list = list_character_states(&pool, "c1").unwrap();
        assert!(list.is_empty());
    }
}
