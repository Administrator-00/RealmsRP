//! HTTP server 模块 (G2)
//!
//! 用 axum 构建 HTTP API server, 连接 WebUI 和 engine。

pub mod config;
pub mod handlers;
pub mod state;

use axum::{
    routing::{get, post},
    Router,
};
use state::SharedState;
use tower_http::cors::CorsLayer;

/// 构建 axum Router, 注册所有 API 端点
pub fn build_router(state: SharedState) -> Router {
    use handlers::*;

    Router::new()
        // 配置
        .route("/api/config", get(get_config).put(put_config))
        .route("/api/config/test", post(test_config))
        // World
        .route("/api/worlds", get(list_worlds))
        .route("/api/worlds/:id", get(get_world))
        // GM
        .route("/api/gms", get(list_gms))
        // Perspectives
        .route("/api/perspectives", get(list_perspectives).post(create_perspective))
        // Cycles
        .route("/api/cycles", get(list_cycles).post(create_cycle))
        .route(
            "/api/cycles/:id",
            get(get_cycle).delete(delete_cycle),
        )
        .route("/api/cycles/:id/turn", post(process_turn))
        // 健康检查
        .route("/api/health", get(health))
        // CORS (允许 WebUI localhost:5173 访问)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// 启动 HTTP server
pub async fn serve(state: SharedState, port: u16) {
    let app = build_router(state);
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect(&format!("bind to {addr}"));
    println!("Realms API server on http://localhost:{port}");
    axum::serve(listener, app).await.expect("server");
}
