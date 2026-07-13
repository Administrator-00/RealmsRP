//! Cycle queries (M4.4) — 周目 CRUD

use chrono::Utc;
use rusqlite::params;

use crate::db::pool::DbPool;
use crate::error::{RealmsError, Result};

/// Cycle 聚合结构体 (M4.4)
#[derive(Debug, Clone)]
pub struct Cycle {
    /// 周目 id (主键)
    pub cycle_id: String,
    /// 所属 world id
    pub world_id: String,
    /// 视角 id (perspective)
    pub perspective_id: String,
    /// GM 模板 id
    pub gm_id: String,
    /// 周目名称 (用户自定义)
    pub cycle_name: String,
    /// 周目描述
    pub cycle_description: Option<String>,
    /// 游戏内当前时间
    pub current_world_time: Option<String>,
    /// 游戏内当前位置
    pub current_location_id: Option<String>,
    /// 累计事件数
    pub total_events: i64,
    /// 累计游玩秒数
    pub total_playtime_seconds: i64,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
    /// 上次游玩时间
    pub last_played_at: Option<String>,
}

/// 创建周目
pub fn create_cycle(
    pool: &DbPool,
    cycle_id: &str,
    world_id: &str,
    perspective_id: &str,
    gm_id: &str,
    cycle_name: &str,
) -> Result<Cycle> {
    let conn = pool
        .get()
        .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;

    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO cycles (cycle_id, world_id, perspective_id, gm_id, cycle_name, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![cycle_id, world_id, perspective_id, gm_id, cycle_name, now, now],
    )
    .map_err(RealmsError::Database)?;

    load_cycle(pool, cycle_id)
}

/// 加载周目
pub fn load_cycle(pool: &DbPool, cycle_id: &str) -> Result<Cycle> {
    let conn = pool
        .get()
        .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;

    conn.query_row(
        "SELECT cycle_id, world_id, perspective_id, gm_id, cycle_name,
                cycle_description, current_world_time, current_location_id,
                total_events, total_playtime_seconds,
                created_at, updated_at, last_played_at
         FROM cycles WHERE cycle_id = ?1",
        params![cycle_id],
        |r| {
            Ok(Cycle {
                cycle_id: r.get(0)?,
                world_id: r.get(1)?,
                perspective_id: r.get(2)?,
                gm_id: r.get(3)?,
                cycle_name: r.get(4)?,
                cycle_description: r.get(5)?,
                current_world_time: r.get(6)?,
                current_location_id: r.get(7)?,
                total_events: r.get(8)?,
                total_playtime_seconds: r.get(9)?,
                created_at: r.get(10)?,
                updated_at: r.get(11)?,
                last_played_at: r.get(12)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => RealmsError::not_found("cycle", cycle_id),
        other => RealmsError::Database(other),
    })
}

/// 列出某 world 下所有周目 (按 updated_at 倒序)
pub fn list_cycles(pool: &DbPool, world_id: &str) -> Result<Vec<Cycle>> {
    let conn = pool
        .get()
        .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;

    let mut stmt = conn
        .prepare(
            "SELECT cycle_id, world_id, perspective_id, gm_id, cycle_name,
                    cycle_description, current_world_time, current_location_id,
                    total_events, total_playtime_seconds,
                    created_at, updated_at, last_played_at
             FROM cycles WHERE world_id = ?1 ORDER BY updated_at DESC",
        )
        .map_err(RealmsError::Database)?;

    let rows = stmt
        .query_map(params![world_id], |r| {
            Ok(Cycle {
                cycle_id: r.get(0)?,
                world_id: r.get(1)?,
                perspective_id: r.get(2)?,
                gm_id: r.get(3)?,
                cycle_name: r.get(4)?,
                cycle_description: r.get(5)?,
                current_world_time: r.get(6)?,
                current_location_id: r.get(7)?,
                total_events: r.get(8)?,
                total_playtime_seconds: r.get(9)?,
                created_at: r.get(10)?,
                updated_at: r.get(11)?,
                last_played_at: r.get(12)?,
            })
        })
        .map_err(RealmsError::Database)?;

    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(RealmsError::Database)
}

/// 更新 last_played_at (切换周目)
pub fn touch_cycle(pool: &DbPool, cycle_id: &str) -> Result<()> {
    let conn = pool
        .get()
        .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "UPDATE cycles SET last_played_at = ?1, updated_at = ?2 WHERE cycle_id = ?3",
        params![now, now, cycle_id],
    )
    .map_err(RealmsError::Database)?;
    Ok(())
}

/// 删除周目 (级联: 所有 cycle_id 关联的表)
pub fn delete_cycle(pool: &DbPool, cycle_id: &str) -> Result<()> {
    let conn = pool
        .get()
        .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;

    // 逐表删除 (SQLite 不强制 foreign key CASCADE, 需手动)
    let tables = [
        "emotional_memories",
        "semantic_memories",
        "episodic_memories",
        "relationship_history",
        "npc_user_relationships",
        "items",
        "locations",
        "domain_events",
        "character_states",
    ];
    for t in tables {
        conn.execute(
            &format!("DELETE FROM {t} WHERE cycle_id = ?1"),
            params![cycle_id],
        )
        .map_err(RealmsError::Database)?;
    }

    conn.execute("DELETE FROM cycles WHERE cycle_id = ?1", params![cycle_id])
        .map_err(RealmsError::Database)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::pool::init_memory_pool;

    fn setup() -> DbPool {
        init_memory_pool().expect("pool")
    }

    #[test]
    fn create_and_load_cycle() {
        let pool = setup();
        let c = create_cycle(
            &pool,
            "c1",
            "xiatian_qixia_1",
            "pov_oc_shixiong",
            "style_linyueru",
            "周目1",
        )
        .expect("create");
        assert_eq!(c.cycle_id, "c1");
        assert_eq!(c.world_id, "xiatian_qixia_1");
        assert_eq!(c.cycle_name, "周目1");

        let loaded = load_cycle(&pool, "c1").expect("load");
        assert_eq!(loaded.gm_id, "style_linyueru");
    }

    #[test]
    fn load_nonexistent_returns_not_found() {
        let pool = setup();
        let err = load_cycle(&pool, "no_such_cycle").unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn list_cycles_by_world() {
        let pool = setup();
        create_cycle(&pool, "a", "world_x", "p", "g", "A").unwrap();
        create_cycle(&pool, "b", "world_x", "p", "g", "B").unwrap();
        create_cycle(&pool, "c", "world_y", "p", "g", "C").unwrap();

        let list = list_cycles(&pool, "world_x").unwrap();
        assert_eq!(list.len(), 2);

        let list_y = list_cycles(&pool, "world_y").unwrap();
        assert_eq!(list_y.len(), 1);
    }

    #[test]
    fn touch_updates_last_played() {
        let pool = setup();
        create_cycle(&pool, "c1", "w", "p", "g", "test").unwrap();
        touch_cycle(&pool, "c1").unwrap();

        let c = load_cycle(&pool, "c1").unwrap();
        assert!(c.last_played_at.is_some());
    }

    #[test]
    fn delete_cycle_cascades() {
        let pool = setup();
        create_cycle(&pool, "c1", "w", "p", "g", "test").unwrap();

        // 插入关联数据
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO character_states (cycle_id, npc_id) VALUES ('c1', 'npc')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO domain_events (cycle_id, event_type, summary) VALUES ('c1', 'setup', 'test')",
            [],
        ).unwrap();

        delete_cycle(&pool, "c1").unwrap();

        // 验证级联删除
        let cs_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM character_states WHERE cycle_id='c1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cs_count, 0, "character_states 应级联删除");

        let ev_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM domain_events WHERE cycle_id='c1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(ev_count, 0, "domain_events 应级联删除");

        // cycle 本身也删除了
        assert!(load_cycle(&pool, "c1").is_err());
    }
}
