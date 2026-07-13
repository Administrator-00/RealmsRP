//! Migration runner — 按版本号顺序执行 SQL migration.
//!
//! 使用 `include_str!` 嵌入 migration SQL, 通过 `meta` 表追踪当前版本.
//! `run()` 仅应用版本号高于当前版本的 migration, 保证幂等.

use rusqlite::params;

use crate::db::pool::DbPool;
use crate::error::{RealmsError, Result};

/// 内嵌 migration 列表 (version_number, sql).
/// 新增 migration 时在此数组追加 (按 version 升序).
const MIGRATIONS: &[(&str, &str)] = &[("3", include_str!("migrations/0001_init.sql"))];

/// 执行所有未应用的 migration.
///
/// 第一次调用时 `meta` 表可能不存在, 会先创建.
/// 后续调用仅执行新 migration (幂等).
pub fn run(pool: &DbPool) -> Result<()> {
    let conn = pool
        .get()
        .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;

    // 确保 meta 表存在
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT);
         INSERT OR IGNORE INTO meta(key, value) VALUES ('schema_version', '0');",
    )
    .map_err(RealmsError::Database)?;

    // 读取当前 schema_version
    let current: String = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "0".into());
    let current_ver: i32 = current.parse().unwrap_or(0);

    for (ver_str, sql) in MIGRATIONS {
        let ver: i32 = ver_str.parse().unwrap_or(0);
        if ver > current_ver {
            conn.execute_batch(sql).map_err(RealmsError::Database)?;
            conn.execute(
                "UPDATE meta SET value = ?1 WHERE key = 'schema_version'",
                params![ver_str],
            )
            .map_err(RealmsError::Database)?;
        }
    }

    Ok(())
}

/// 读取当前 schema version (测试/诊断用).
pub fn current_version(pool: &DbPool) -> Result<String> {
    let conn = pool
        .get()
        .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;
    let v: String = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "0".into());
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::pool::init_memory_pool;

    #[test]
    fn run_creates_schema_and_sets_version() {
        let pool = init_memory_pool().expect("pool");
        run(&pool).expect("run");

        let v = current_version(&pool).expect("version");
        assert_eq!(v, "3", "应为 schema_version=3");

        let conn = pool.get().unwrap();
        // 验证核心表存在
        let tables = [
            "cycles",
            "character_states",
            "npc_user_relationships",
            "domain_events",
            "arc_dimensions",
        ];
        for t in tables {
            let count: i64 = conn
                .query_row(
                    &format!(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{t}'"
                    ),
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert!(count > 0, "表 {t} 应存在");
        }
    }

    #[test]
    fn run_is_idempotent() {
        let pool = init_memory_pool().expect("pool");
        run(&pool).expect("first");
        // 第二次 run 不应 panic (幂等)
        run(&pool).expect("second");
        assert_eq!(current_version(&pool).unwrap(), "3");
    }

    #[test]
    fn arc_dimensions_are_prepopulated() {
        let pool = init_memory_pool().expect("pool");
        run(&pool).expect("run");

        let conn = pool.get().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM arc_dimensions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 5, "应有 5 条预置弧光维度");
    }
}
