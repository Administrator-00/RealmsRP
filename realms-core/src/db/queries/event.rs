//! Event queries (M4.6)

use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::db::pool::DbPool;
use crate::error::{RealmsError, Result};

/// domain_events 行投影
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRow {
    /// 事件 id
    pub id: i64,
    /// 周目 id
    pub cycle_id: String,
    /// 事件类型
    pub event_type: String,
    /// 行为者 id
    pub actor_id: Option<String>,
    /// 行为者类型
    pub actor_type: Option<String>,
    /// 目标 id
    pub target_id: Option<String>,
    /// 目标类型
    pub target_type: Option<String>,
    /// 重要性
    pub importance: f64,
    /// 摘要
    pub summary: String,
    /// 游戏内时间
    pub world_time: Option<String>,
    /// 是否里程碑事件
    pub is_milestone: bool,
    /// 创建时间
    pub created_at: String,
}

/// 按时间倒序列出某 cycle 的事件 (分页).
pub fn list_events(
    pool: &DbPool,
    cycle_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<EventRow>> {
    let conn = pool
        .get()
        .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;
    let mut stmt = conn
        .prepare(
            "SELECT id, cycle_id, event_type, actor_id, actor_type,
                    target_id, target_type, importance, summary, world_time,
                    is_milestone, created_at
             FROM domain_events WHERE cycle_id=?1
             ORDER BY id DESC LIMIT ?2 OFFSET ?3",
        )
        .map_err(RealmsError::Database)?;

    let rows = stmt
        .query_map(params![cycle_id, limit, offset], |r| {
            Ok(EventRow {
                id: r.get(0)?,
                cycle_id: r.get(1)?,
                event_type: r.get(2)?,
                actor_id: r.get(3)?,
                actor_type: r.get(4)?,
                target_id: r.get(5)?,
                target_type: r.get(6)?,
                importance: r.get(7)?,
                summary: r.get(8)?,
                world_time: r.get(9)?,
                is_milestone: r.get::<_, i64>(10)? != 0,
                created_at: r.get(11)?,
            })
        })
        .map_err(RealmsError::Database)?;

    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(RealmsError::Database)
}

/// 事件计数.
pub fn count_events(pool: &DbPool, cycle_id: &str) -> Result<i64> {
    let conn = pool
        .get()
        .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;
    conn.query_row(
        "SELECT COUNT(*) FROM domain_events WHERE cycle_id=?1",
        params![cycle_id],
        |r| r.get(0),
    )
    .map_err(RealmsError::Database)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::pool::init_memory_pool;

    #[test]
    fn list_empty_returns_empty() {
        let pool = init_memory_pool().expect("pool");
        let list = list_events(&pool, "c1", 10, 0).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn count_zero_for_empty() {
        let pool = init_memory_pool().expect("pool");
        assert_eq!(count_events(&pool, "c1").unwrap(), 0);
    }
}
