//! Cross-cycle comparison queries (M8.1-M8.2)
//!
//! M8.1: 同一 NPC 在不同 cycle 下的关系对比
//! M8.2: 同一 cycle 内所有 NPC 的关系矩阵

use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::db::pool::DbPool;
use crate::error::{RealmsError, Result};

/// 跨周目对比行: 同一 NPC 在某 world 下各 cycle 的关系数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCycleComparison {
    /// 周目 id
    pub cycle_id: String,
    /// 周目名称
    pub cycle_name: String,
    /// NPC id
    pub npc_id: String,
    /// 好感度
    pub affinity: i64,
    /// 信任
    pub trust: i64,
    /// 亲密
    pub intimacy: i64,
    /// 关系类型 (stranger/friend/lover/...)
    pub current_type: Option<String>,
    /// 初始模板
    pub initial_template: Option<String>,
}

/// 查询同一 NPC 在不同 cycle 下的关系 (跨周目对比).
///
/// 用于回答: "林月如在不同的周目里对我的关系变化"
pub fn compare_npc_across_cycles(
    pool: &DbPool,
    world_id: &str,
    npc_id: &str,
) -> Result<Vec<CrossCycleComparison>> {
    let conn = pool
        .get()
        .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;

    let mut stmt = conn
        .prepare(
            "SELECT c.cycle_id, c.cycle_name, r.npc_id,
                    r.affinity, r.trust, r.intimacy,
                    r.current_type, r.initial_template
             FROM npc_user_relationships r
             JOIN cycles c ON r.cycle_id = c.cycle_id
             WHERE r.npc_id = ?1 AND c.world_id = ?2
             ORDER BY c.created_at DESC",
        )
        .map_err(RealmsError::Database)?;

    let rows = stmt
        .query_map(params![npc_id, world_id], |r| {
            Ok(CrossCycleComparison {
                cycle_id: r.get(0)?,
                cycle_name: r.get(1)?,
                npc_id: r.get(2)?,
                affinity: r.get(3)?,
                trust: r.get(4)?,
                intimacy: r.get(5)?,
                current_type: r.get(6)?,
                initial_template: r.get(7)?,
            })
        })
        .map_err(RealmsError::Database)?;

    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(RealmsError::Database)
}

/// 关系矩阵行: 同一 cycle 内某 NPC 的关系数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipMatrixRow {
    /// NPC id
    pub npc_id: String,
    /// 好感度
    pub affinity: i64,
    /// 信任
    pub trust: i64,
    /// 亲密
    pub intimacy: i64,
    /// 关系类型
    pub current_type: Option<String>,
}

/// 查询某 cycle 下所有 NPC 的关系矩阵 (跨 NPC 对比).
///
/// 用于回答: "在当前周目里，所有 NPC 对我的关系一览"
pub fn relationship_matrix(pool: &DbPool, cycle_id: &str) -> Result<Vec<RelationshipMatrixRow>> {
    let conn = pool
        .get()
        .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;

    let mut stmt = conn
        .prepare(
            "SELECT npc_id, affinity, trust, intimacy, current_type
             FROM npc_user_relationships
             WHERE cycle_id = ?1
             ORDER BY ABS(affinity) DESC",
        )
        .map_err(RealmsError::Database)?;

    let rows = stmt
        .query_map(params![cycle_id], |r| {
            Ok(RelationshipMatrixRow {
                npc_id: r.get(0)?,
                affinity: r.get(1)?,
                trust: r.get(2)?,
                intimacy: r.get(3)?,
                current_type: r.get(4)?,
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
    use crate::db::queries::cycle;

    fn setup() -> DbPool {
        init_memory_pool().expect("pool")
    }

    #[test]
    fn cross_cycle_comparison_empty() {
        let pool = setup();
        let result = compare_npc_across_cycles(&pool, "world_x", "npc_x").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn cross_cycle_comparison_two_cycles() {
        let pool = setup();
        // 创建 2 个周目 + 关系数据
        cycle::create_cycle(&pool, "c1", "w1", "p", "g", "周目1").unwrap();
        cycle::create_cycle(&pool, "c2", "w1", "p", "g", "周目2").unwrap();

        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO npc_user_relationships (cycle_id, npc_id, affinity, trust, intimacy, current_type)
             VALUES ('c1', 'lin_yueru', 60, 80, 70, 'close_friend')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO npc_user_relationships (cycle_id, npc_id, affinity, trust, intimacy, current_type)
             VALUES ('c2', 'lin_yueru', 30, 40, 20, 'friend')",
            [],
        ).unwrap();

        let rows = compare_npc_across_cycles(&pool, "w1", "lin_yueru").unwrap();
        assert_eq!(rows.len(), 2);
        // 同秒创建, 顺序不保证; 验证两条都在
        let names: Vec<&str> = rows.iter().map(|r| r.cycle_name.as_str()).collect();
        assert!(names.contains(&"周目1"));
        assert!(names.contains(&"周目2"));
        let affs: Vec<i64> = rows.iter().map(|r| r.affinity).collect();
        assert!(affs.contains(&60));
        assert!(affs.contains(&30));
    }

    #[test]
    fn relationship_matrix_empty() {
        let pool = setup();
        let result = relationship_matrix(&pool, "c1").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn relationship_matrix_sorted_by_affinity() {
        let pool = setup();
        cycle::create_cycle(&pool, "c1", "w1", "p", "g", "test").unwrap();

        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO npc_user_relationships (cycle_id, npc_id, affinity) VALUES ('c1', 'a', 0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO npc_user_relationships (cycle_id, npc_id, affinity) VALUES ('c1', 'b', -80)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO npc_user_relationships (cycle_id, npc_id, affinity) VALUES ('c1', 'c', 70)",
            [],
        ).unwrap();

        let rows = relationship_matrix(&pool, "c1").unwrap();
        assert_eq!(rows.len(), 3);
        // 按 ABS(affinity) DESC: -80(80) → 70(70) → 0(0)
        assert_eq!(rows[0].npc_id, "b"); // |-80| = 80 最大
        assert_eq!(rows[1].npc_id, "c"); // |70| = 70
        assert_eq!(rows[2].npc_id, "a"); // |0| = 0
    }
}
