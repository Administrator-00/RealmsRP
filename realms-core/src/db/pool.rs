//! SQLite 连接池
//!
//! 封装 r2d2 + rusqlite, 提供 `init_pool()` 初始化数据库 + 跑 migrations.
//!
//! 设计要点:
//! - WAL 模式 (并发读写)
//! - 外键开启 (FOREIGN KEY 强约束)
//! - 内存模式 (测试用) + 文件模式 (生产用)

use std::path::Path;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;

use crate::error::{RealmsError, Result};

/// Realms SQLite 连接池类型
pub type DbPool = Pool<SqliteConnectionManager>;

/// 初始化数据库 (内存模式, 用于测试)
///
/// 跑 migrations + 设置 PRAGMA (WAL / foreign_keys).
pub fn init_memory_pool() -> Result<DbPool> {
    let manager = SqliteConnectionManager::memory().with_init(init_connection);
    let pool = Pool::builder()
        .max_size(4)
        .build(manager)
        .map_err(|e| RealmsError::internal(format!("build memory pool: {e}")))?;
    run_migrations(&pool)?;
    Ok(pool)
}

/// 初始化数据库 (文件模式, 用于生产)
///
/// # 参数
///
/// - `data_dir`: 数据目录 (会自动创建), memory.db 放在此目录下
///
/// # 错误
///
/// - 数据目录创建失败 → `RealmsError::Io`
/// - migrations 失败 → `RealmsError::Database` / `RealmsError::Internal`
pub fn init_pool(data_dir: &Path) -> Result<DbPool> {
    std::fs::create_dir_all(data_dir)?;
    let db_path = data_dir.join("memory.db");
    let manager = SqliteConnectionManager::file(&db_path).with_init(init_connection);
    let pool = Pool::builder()
        .max_size(8)
        .build(manager)
        .map_err(|e| RealmsError::internal(format!("build file pool: {e}")))?;
    run_migrations(&pool)?;
    Ok(pool)
}

/// 初始化单个连接 (PRAGMA 设置)
fn init_connection(conn: &mut Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;
         PRAGMA synchronous = NORMAL;",
    )
}

/// 跑 migrations (M4.3 实现)
fn run_migrations(pool: &DbPool) -> Result<()> {
    crate::db::migration::run(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_pool_init() {
        let pool = init_memory_pool().expect("init memory pool");
        let conn = pool.get().expect("get conn");
        // 验证 PRAGMA
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();
        assert_eq!(journal_mode.to_lowercase(), "memory");

        let foreign_keys: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
            .unwrap();
        assert_eq!(foreign_keys, 1, "foreign_keys must be ON");
    }

    #[test]
    fn memory_pool_concurrent_access() {
        // 验证连接池在多线程下能并发获取连接 (max_size = 4, 起 4 线程)
        use std::thread;
        let pool = init_memory_pool().expect("init");
        let pool = std::sync::Arc::new(pool);
        let mut handles = vec![];
        for _ in 0..4 {
            let pool = pool.clone();
            handles.push(thread::spawn(move || {
                let conn = pool.get().expect("get conn");
                // 固定查询, 避免与循环变量耦合产生误判
                conn.query_row::<i64, _, _>("SELECT 1", [], |r| r.get(0))
                    .unwrap()
            }));
        }
        for h in handles {
            // 每个线程都应拿到连接并查询到 1
            assert_eq!(h.join().unwrap(), 1);
        }
    }

    #[test]
    fn file_pool_creates_dir_and_db() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pool = init_pool(dir.path()).expect("init file pool");
        let conn = pool.get().expect("get conn");
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();
        // WAL 模式在文件型数据库上
        assert_eq!(journal_mode.to_lowercase(), "wal");
        assert!(dir.path().join("memory.db").exists());
    }
}
