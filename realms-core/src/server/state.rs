//! 共享应用状态 (G2)
//!
//! 所有 handler 共享的 AppState，包含 DB 连接池、路径、LLM 配置等。

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::db::pool::DbPool;
use crate::server::config::AppConfig;

/// 所有 handler 共享的状态
pub struct AppState {
    /// SQLite 连接池
    pub pool: DbPool,
    /// 数据目录 (含 config.json + memory.db)
    pub data_dir: PathBuf,
    /// World 数据目录
    pub worlds_dir: PathBuf,
    /// GM 模板目录
    pub gms_dir: PathBuf,
    /// 视角数据目录
    pub perspectives_dir: PathBuf,
    /// 应用配置 (包含 LLM 配置)，读写锁保护
    pub config: RwLock<AppConfig>,
}

/// Arc-wrapped 共享状态
pub type SharedState = Arc<AppState>;

impl AppState {
    /// 创建完整 (含路径) 的状态。config 需通过 `AppConfig::load` 提前加载。
    pub fn new(
        pool: DbPool,
        data_dir: PathBuf,
        worlds_dir: PathBuf,
        gms_dir: PathBuf,
        perspectives_dir: PathBuf,
        config: AppConfig,
    ) -> Self {
        Self {
            pool,
            data_dir,
            worlds_dir,
            gms_dir,
            perspectives_dir,
            config: RwLock::new(config),
        }
    }
}
