//! Relationship queries (M4.7)

use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::db::pool::DbPool;
use crate::error::{RealmsError, Result};

/// npc_user_relationships 行投影
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipRow {
    /// 周目 id
    pub cycle_id: String,
    /// NPC id
    pub npc_id: String,
    /// 好感度 (-100..100)
    pub affinity: i64,
    /// 信任 (0..100)
    pub trust: i64,
    /// 亲密 (0..100)
    pub intimacy: i64,
    /// 尊敬 (0..100)
    pub respect: i64,
    /// 关系类型
    pub current_type: Option<String>,
    /// 初始模板 id
    pub initial_template: Option<String>,
    /// 首次相遇时间
    pub first_met_at: Option<String>,
}

/// 加载单个 NPC 关系.
pub fn load_relationship(pool: &DbPool, cycle_id: &str, npc_id: &str) -> Result<RelationshipRow> {
    let conn = pool
        .get()
        .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;
    conn.query_row(
        "SELECT cycle_id, npc_id, affinity, trust, intimacy, respect,
                current_type, initial_template, first_met_at
         FROM npc_user_relationships WHERE cycle_id=?1 AND npc_id=?2",
        params![cycle_id, npc_id],
        |r| {
            Ok(RelationshipRow {
                cycle_id: r.get(0)?,
                npc_id: r.get(1)?,
                affinity: r.get(2)?,
                trust: r.get(3)?,
                intimacy: r.get(4)?,
                respect: r.get(5)?,
                current_type: r.get(6)?,
                initial_template: r.get(7)?,
                first_met_at: r.get(8)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            RealmsError::not_found("relationship", &format!("{cycle_id}/{npc_id}"))
        }
        other => RealmsError::Database(other),
    })
}

/// 列出某 cycle 下所有 NPC 关系 (按 affinity 绝对值降序).
pub fn list_relationships(pool: &DbPool, cycle_id: &str) -> Result<Vec<RelationshipRow>> {
    let conn = pool
        .get()
        .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;
    let mut stmt = conn
        .prepare(
            "SELECT cycle_id, npc_id, affinity, trust, intimacy, respect,
                    current_type, initial_template, first_met_at
             FROM npc_user_relationships WHERE cycle_id=?1
             ORDER BY ABS(affinity) DESC",
        )
        .map_err(RealmsError::Database)?;

    let rows = stmt
        .query_map(params![cycle_id], |r| {
            Ok(RelationshipRow {
                cycle_id: r.get(0)?,
                npc_id: r.get(1)?,
                affinity: r.get(2)?,
                trust: r.get(3)?,
                intimacy: r.get(4)?,
                respect: r.get(5)?,
                current_type: r.get(6)?,
                initial_template: r.get(7)?,
                first_met_at: r.get(8)?,
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
        let err = load_relationship(&pool, "c1", "npc_x").unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn list_empty_cycle() {
        let pool = init_memory_pool().expect("pool");
        let list = list_relationships(&pool, "c1").unwrap();
        assert!(list.is_empty());
    }
}
