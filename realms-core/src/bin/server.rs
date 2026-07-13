//! Realms API Server 入口 (G2)
//!
//! 启动 axum HTTP server, 自动从 data/config.json 加载配置。

use std::path::PathBuf;
use std::sync::Arc;

use realms_core::db::pool::init_pool;
use realms_core::server::config::AppConfig;
use realms_core::server::state::AppState;
use realms_core::server::serve;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    // 路径
    let data_dir = PathBuf::from("./data");
    let worlds_dir = PathBuf::from("./worlds");
    let gms_dir = PathBuf::from("./gms");
    let perspectives_dir = PathBuf::from("./perspectives");

    // 初始化数据库
    let pool = init_pool(&data_dir).expect("Failed to init DB");

    // 加载配置
    let config = AppConfig::load(&data_dir).expect("Failed to load config");

    let state = Arc::new(AppState::new(
        pool,
        data_dir,
        worlds_dir,
        gms_dir,
        perspectives_dir,
        config,
    ));

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".into())
        .parse()
        .unwrap_or(3000);

    serve(state, port).await;
}
