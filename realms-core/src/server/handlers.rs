//! API 端点处理 (G2)
//!
//! 所有 HTTP handler。设计：薄层，大部分逻辑委托给 engine/cycle/db 模块。

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::db::queries::{
    character_state, cycle, event, relationship,
};
use crate::engine::cycle::{
    cycle_manager::NewCycleContext,
    gm_loader,
    perspective,
    world_loader,
};
use crate::engine::orchestrator::gm_dispatcher::{self, LlmProvider};
use crate::server::config::{AppConfig, LlmConfig};
use crate::server::state::SharedState;
use crate::error::RealmsError;

// ============================================================================
// 请求/响应体
// ============================================================================

/// POST /api/cycles 请求体
#[derive(Debug, Deserialize)]
pub struct CreateCycleReq {
    pub world_id: String,
    pub perspective_id: String,
    pub gm_id: String,
    pub cycle_name: String,
    /// 可选，OC 关系模板选择: npc_id → template_id
    #[serde(default)]
    pub oc_selections: std::collections::HashMap<String, Option<String>>,
}

/// POST /api/cycles/:id/turn 请求体
#[derive(Debug, Deserialize)]
pub struct TurnReq {
    pub user_message: String,
}

// ============================================================================
// 工具函数
// ============================================================================

/// JSON 错误响应包装
fn err_resp(code: StatusCode, msg: String) -> (StatusCode, Json<serde_json::Value>) {
    (
        code,
        Json(serde_json::json!({"error": msg})),
    )
}

/// 将 RealmsError 映射为 HTTP 状态码
fn error_status(err: &RealmsError) -> StatusCode {
    match err {
        RealmsError::NotFound(_) => StatusCode::NOT_FOUND,
        RealmsError::InvalidInput(_) | RealmsError::Event(_) => StatusCode::BAD_REQUEST,
        RealmsError::Cycle(_) => StatusCode::CONFLICT,
        RealmsError::Config(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// ============================================================================
// 配置端点 (G0)
// ============================================================================

/// GET /api/config — 返回当前配置 (api_key 脱敏)
pub async fn get_config(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let config = state.config.read().unwrap();
    let masked = config.masked_llm();
    Json(serde_json::json!({
        "llm": {
            "provider": masked.provider,
            "endpoint": masked.endpoint,
            "api_key": masked.api_key,
            "model": masked.model,
        }
    }))
}

/// PUT /api/config — 更新配置并保存到 config.json
pub async fn put_config(
    State(state): State<SharedState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let llm_obj = body
        .get("llm")
        .ok_or_else(|| err_resp(StatusCode::BAD_REQUEST, "missing 'llm' field".into()))?;

    let provider = llm_obj["provider"].as_str().unwrap_or("OpenAI").to_string();
    let endpoint = llm_obj["endpoint"].as_str().unwrap_or("").to_string();
    let api_key = llm_obj["api_key"].as_str().unwrap_or("").to_string();
    let model = llm_obj["model"].as_str().unwrap_or("").to_string();

    let llm = LlmConfig {
        provider,
        endpoint,
        api_key,
        model,
    };

    // 加写锁，直接更新内存中的 config 并持久化
    let mut config = state.config.write().unwrap();
    config
        .update_llm(llm, &state.data_dir)
        .map_err(|e| err_resp(error_status(&e), e.to_string()))?;

    let masked = config.masked_llm();
    Ok(Json(serde_json::json!({
        "llm": {
            "provider": masked.provider,
            "endpoint": masked.endpoint,
            "api_key": masked.api_key,
            "model": masked.model,
        }
    })))
}

/// POST /api/config/test — 测试 LLM 连接
pub async fn test_config(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let (endpoint, api_key, model) = {
        let config = state.config.read().unwrap();
        if config.llm.api_key.is_empty() {
            return Ok(Json(serde_json::json!({
                "status": "unconfigured",
                "message": "API key is empty"
            })));
        }
        (
            config.llm.endpoint.clone(),
            config.llm.api_key.clone(),
            config.llm.model.clone(),
        )
    };

    let provider = HttpTestProvider {
        endpoint,
        api_key,
        model,
        client: reqwest::Client::new(),
    };

    match provider.chat("Reply with just the word 'ok'.", "ping").await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "ok",
            "message": "Connection successful"
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        }))),
    }
}

// ============================================================================
// World 端点
// ============================================================================

/// GET /api/worlds — 列出所有 world
pub async fn list_worlds(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let ids = world_loader::list_worlds(&state.worlds_dir).map_err(|e| {
        err_resp(error_status(&e), e.to_string())
    })?;

    let mut summaries = Vec::new();
    for id in &ids {
        if let Ok(world) = world_loader::load_world(&state.worlds_dir, id) {
            let preview: String = world.setting.chars().take(120).collect();
            summaries.push(serde_json::json!({
                "world_id": world.world_id,
                "setting_preview": preview,
                "npc_count": world.npc_personas.len(),
                "npc_ids": world.npc_personas.keys().collect::<Vec<_>>(),
            }));
        }
    }
    Ok(Json(serde_json::Value::Array(summaries)))
}

/// GET /api/worlds/:id — 获取 world 详情
pub async fn get_world(
    State(state): State<SharedState>,
    Path(world_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let world = world_loader::load_world(&state.worlds_dir, &world_id).map_err(|e| {
        err_resp(error_status(&e), e.to_string())
    })?;

    Ok(Json(serde_json::json!({
        "world_id": world.world_id,
        "setting": world.setting,
        "npc_personas": world.npc_personas,
    })))
}

// ============================================================================
// GM 端点
// ============================================================================

/// GET /api/gms — 列出所有 GM 模板
pub async fn list_gms(
    State(state): State<SharedState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let world_id = params.get("world_id").map(|s| s.as_str());
    let gms = gm_loader::list_gms(&state.gms_dir, world_id).map_err(|e| {
        err_resp(error_status(&e), e.to_string())
    })?;

    let summaries: Vec<serde_json::Value> = gms
        .into_iter()
        .map(|g| {
            let preview: String = g.content.chars().take(120).collect();
            serde_json::json!({
                "gm_id": g.gm_id,
                "content_preview": preview,
            })
        })
        .collect();

    Ok(Json(serde_json::Value::Array(summaries)))
}

// ============================================================================
// Perspective 端点
// ============================================================================

/// GET /api/perspectives — 列出所有视角
pub async fn list_perspectives(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let perspectives =
        perspective::list_perspectives(&state.perspectives_dir).map_err(|e| {
            err_resp(error_status(&e), e.to_string())
        })?;

    let summaries: Vec<serde_json::Value> = perspectives
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "source": p.source,
                "name": p.name,
                "world_id": p.world_id,
            })
        })
        .collect();

    Ok(Json(serde_json::Value::Array(summaries)))
}

/// POST /api/perspectives — 创建新视角 (OC)
pub async fn create_perspective(
    State(state): State<SharedState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let id = body["id"].as_str().unwrap_or("").to_string();
    let source = body["source"].as_str().unwrap_or("oc").to_string();
    let name = body["name"].as_str().unwrap_or("").to_string();
    let persona_data = body["persona_data"].as_str().unwrap_or("").to_string();
    let world_id = body["world_id"].as_str().map(|s| s.to_string());

    if id.is_empty() || name.is_empty() {
        return Err(err_resp(StatusCode::BAD_REQUEST, "id and name required".into()));
    }

    let p = perspective::Perspective {
        id: id.clone(),
        source,
        name: name.clone(),
        persona_data,
        world_id: world_id.clone(),
    };

    perspective::save_perspective(&state.perspectives_dir, &p).map_err(|e| {
        err_resp(error_status(&e), e.to_string())
    })?;

    Ok(Json(serde_json::json!({
        "id": id,
        "source": "oc",
        "name": name,
        "world_id": world_id,
    })))
}

// ============================================================================
// Cycle 端点
// ============================================================================

/// GET /api/cycles — 列出某 world 下的所有周目
pub async fn list_cycles(
    State(state): State<SharedState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let world_id = params.get("world_id").ok_or_else(|| {
        err_resp(StatusCode::BAD_REQUEST, "query param 'world_id' required".into())
    })?;

    let cycles = cycle::list_cycles(&state.pool, world_id).map_err(|e| {
        err_resp(error_status(&e), e.to_string())
    })?;

    let json_cycles: Vec<serde_json::Value> = cycles
        .into_iter()
        .map(|c| {
            serde_json::json!({
                "cycle_id": c.cycle_id,
                "world_id": c.world_id,
                "perspective_id": c.perspective_id,
                "gm_id": c.gm_id,
                "cycle_name": c.cycle_name,
                "cycle_description": c.cycle_description,
                "current_world_time": c.current_world_time,
                "total_events": c.total_events,
                "created_at": c.created_at,
                "updated_at": c.updated_at,
                "last_played_at": c.last_played_at,
            })
        })
        .collect();

    Ok(Json(serde_json::Value::Array(json_cycles)))
}

/// POST /api/cycles — 创建新周目
pub async fn create_cycle(
    State(state): State<SharedState>,
    Json(body): Json<CreateCycleReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // 1. 验证 world 存在
    world_loader::load_world(&state.worlds_dir, &body.world_id).map_err(|e| {
        err_resp(error_status(&e), e.to_string())
    })?;

    // 2. 验证 GM 存在
    gm_loader::load_gm(&state.gms_dir, Some(&body.world_id), &body.gm_id).map_err(|e| {
        err_resp(error_status(&e), e.to_string())
    })?;

    // 3. 验证 perspective 存在
    let persp =
        perspective::load_perspective(&state.perspectives_dir, &body.perspective_id).map_err(
            |e| err_resp(error_status(&e), e.to_string()),
        )?;
    perspective::validate_perspective_for_world(&persp, &body.world_id).map_err(|e| {
        err_resp(error_status(&e), e.to_string())
    })?;

    // 4. 生成 cycle_id
    let cycle_id = format!("cycle_{}", chrono::Utc::now().format("%Y%m%d%H%M%S"));

    let ctx = NewCycleContext::new(
        state.worlds_dir.clone(),
        state.gms_dir.clone(),
        state.pool.clone(),
    );

    // 5. 创建周目 + 初始化 character_states
    let cyc = ctx
        .start_new_cycle(
            &cycle_id,
            &body.world_id,
            &body.perspective_id,
            &body.gm_id,
            &body.cycle_name,
        )
        .map_err(|e| err_resp(error_status(&e), e.to_string()))?;

    // 6. 处理 OC 关系模板 (如果有)
    if !body.oc_selections.is_empty() {
        use crate::engine::cycle::oc_initializer;
        use std::collections::HashMap;
        let selections: HashMap<String, String> = body
            .oc_selections
            .iter()
            .filter_map(|(k, v)| v.as_ref().map(|t| (k.clone(), t.clone())))
            .collect();
        if !selections.is_empty() {
            oc_initializer::initialize_cycle_relationships(
                &state.pool,
                &cycle_id,
                &selections,
            )
            .map_err(|e| err_resp(error_status(&e), e.to_string()))?;
        }
    }

    Ok(Json(serde_json::json!({
        "cycle_id": cyc.cycle.cycle_id,
        "world_id": cyc.cycle.world_id,
        "perspective_id": cyc.cycle.perspective_id,
        "gm_id": cyc.cycle.gm_id,
        "cycle_name": cyc.cycle.cycle_name,
        "created_at": cyc.cycle.created_at,
    })))
}

/// GET /api/cycles/:id — 获取周目详情
pub async fn get_cycle(
    State(state): State<SharedState>,
    Path(cycle_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let cyc = cycle::load_cycle(&state.pool, &cycle_id).map_err(|e| {
        err_resp(error_status(&e), e.to_string())
    })?;

    // 加载 character_states
    let world =
        world_loader::load_world(&state.worlds_dir, &cyc.world_id).map_err(|e| {
            err_resp(error_status(&e), e.to_string())
        })?;

    let mut char_states: Vec<serde_json::Value> = Vec::new();
    for npc_id in world.npc_personas.keys() {
        if let Ok(cs) =
            character_state::load_character_state(&state.pool, &cycle_id, npc_id)
        {
            char_states.push(serde_json::json!({
                "npc_id": cs.npc_id,
                "hp": cs.hp,
                "hp_max": cs.hp_max,
                "mp": cs.mp,
                "mp_max": cs.mp_max,
                "location_id": cs.location_id,
                "arc_phase": cs.arc_phase,
                "perceived_mood": cs.perceived_mood,
                "arcs": serde_json::from_str::<serde_json::Value>(&cs.arcs_json).unwrap_or_default(),
                "status_flags": serde_json::from_str::<serde_json::Value>(&cs.status_flags_json).unwrap_or_default(),
            }));
        }
    }

    // 加载 relationships
    let mut rels: Vec<serde_json::Value> = Vec::new();
    for npc_id in world.npc_personas.keys() {
        if let Ok(rel) =
            relationship::load_relationship(&state.pool, &cycle_id, npc_id)
        {
            rels.push(serde_json::json!({
                "npc_id": rel.npc_id,
                "affinity": rel.affinity,
                "trust": rel.trust,
                "intimacy": rel.intimacy,
                "respect": rel.respect,
                "current_type": rel.current_type,
            }));
        }
    }

    // 加载最近 20 个 events
    let events = event::list_events(&state.pool, &cycle_id, 20, 0).map_err(|e| {
        err_resp(error_status(&e), e.to_string())
    })?;
    let json_events: Vec<serde_json::Value> = events
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id,
                "event_type": e.event_type,
                "actor_id": e.actor_id,
                "summary": e.summary,
                "world_time": e.world_time,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "cycle": {
            "cycle_id": cyc.cycle_id,
            "world_id": cyc.world_id,
            "perspective_id": cyc.perspective_id,
            "gm_id": cyc.gm_id,
            "cycle_name": cyc.cycle_name,
            "cycle_description": cyc.cycle_description,
            "current_world_time": cyc.current_world_time,
            "total_events": cyc.total_events,
            "created_at": cyc.created_at,
            "updated_at": cyc.updated_at,
            "last_played_at": cyc.last_played_at,
        },
        "character_states": char_states,
        "relationships": rels,
        "events": json_events,
    })))
}

/// DELETE /api/cycles/:id — 删除周目
pub async fn delete_cycle(
    State(state): State<SharedState>,
    Path(cycle_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    cycle::delete_cycle(&state.pool, &cycle_id).map_err(|e| {
        err_resp(error_status(&e), e.to_string())
    })?;

    Ok(Json(serde_json::json!({"deleted": cycle_id})))
}

/// POST /api/cycles/:id/turn — 处理一轮玩家输入 (SSE 流)
pub async fn process_turn(
    State(state): State<SharedState>,
    Path(cycle_id): Path<String>,
    Json(body): Json<TurnReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // 1. 加载周目上下文
    let ctx = NewCycleContext::new(
        state.worlds_dir.clone(),
        state.gms_dir.clone(),
        state.pool.clone(),
    );
    let cycle_ctx = ctx.load_cycle_context(&cycle_id).map_err(|e| {
        err_resp(error_status(&e), e.to_string())
    })?;

    // 2. 加载 perspective
    let persp = perspective::load_perspective(
        &state.perspectives_dir,
        &cycle_ctx.cycle.perspective_id,
    )
    .map_err(|e| err_resp(error_status(&e), e.to_string()))?;

    // 3. 组装 system_prompt
    let system_prompt = format!(
        "# 角色\n{}\n\n---\n\n# 世界观\n{}\n\n---\n\n# 文风指引\n{}",
        persp.persona_data, cycle_ctx.world.setting, cycle_ctx.gm.content
    );

    // 4. 检查是否有 api_key
    {
        let config = state.config.read().unwrap();
        if config.llm.api_key.is_empty() {
            return Err(err_resp(
                StatusCode::BAD_REQUEST,
                "LLM API key not configured. Visit settings page.".into(),
            ));
        }
    }

    // 5. 调用 LLM
    let (endpoint, api_key, model) = {
        let config = state.config.read().unwrap();
        (
            config.llm.endpoint.clone(),
            config.llm.api_key.clone(),
            config.llm.model.clone(),
        )
    };
    let provider = HttpTestProvider {
        endpoint,
        api_key,
        model,
        client: reqwest::Client::new(),
    };

    let llm_output = provider
        .chat(&system_prompt, &body.user_message)
        .await
        .map_err(|e| err_resp(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 6. 解析 LLM 输出
    let (visible_response, private_intent, _suggestions) =
        gm_dispatcher::parse_subagent_output(&llm_output)
            .map_err(|e| err_resp(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 7. 返回结果
    Ok(Json(serde_json::json!({
        "narrative": visible_response,
        "private_intent": private_intent,
    })))
}

// ============================================================================
// 系统端点
// ============================================================================

/// GET /api/health — 健康检查
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": crate::VERSION,
    }))
}

// ============================================================================
// HttpLlmProvider for turn processing
// ============================================================================

/// 简单的 HTTP LLM Provider (内联，不依赖 AIRP engine)
struct HttpTestProvider {
    endpoint: String,
    api_key: String,
    model: String,
    client: reqwest::Client,
}

#[async_trait::async_trait]
impl LlmProvider for HttpTestProvider {
    async fn chat(&self, system_prompt: &str, user_message: &str) -> crate::error::Result<String> {
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_message}
            ],
            "stream": false
        });

        let resp = self
            .client
            .post(&self.endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| RealmsError::internal(format!("HTTP error: {e}")))?;

        let resp_json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| RealmsError::internal(format!("JSON parse: {e}")))?;

        let content = resp_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        if content.is_empty() {
            // Anthropic 格式
            let anthro_content =
                resp_json["content"][0]["text"].as_str().unwrap_or("");
            Ok(anthro_content.to_string())
        } else {
            Ok(content)
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::pool::init_memory_pool;
    use std::sync::Arc;
    use std::sync::RwLock;
    use std::path::PathBuf;

    fn make_state(
        pool: crate::db::pool::DbPool,
        worlds_dir: PathBuf,
    ) -> SharedState {
        let gms_dir = tempfile::tempdir().expect("gms_dir");
        let perspectives_dir = tempfile::tempdir().expect("perspectives_dir");
        let data_dir = tempfile::tempdir().expect("data_dir");
        let config = AppConfig::load(data_dir.path()).expect("config");

        Arc::new(crate::server::state::AppState {
            pool,
            data_dir: data_dir.path().to_path_buf(),
            worlds_dir,
            gms_dir: gms_dir.path().to_path_buf(),
            perspectives_dir: perspectives_dir.path().to_path_buf(),
            config: RwLock::new(config),
        })
    }

    fn setup_world(dir: &std::path::Path, id: &str) {
        let w = dir.join(id);
        std::fs::create_dir_all(w.join("npcs")).unwrap();
        std::fs::write(w.join("setting.md"), "# Test World\nA test world setting.").unwrap();
        std::fs::write(w.join("npcs").join("npc_a.md"), "NPC A").unwrap();
        std::fs::write(w.join("npcs").join("npc_b.md"), "NPC B").unwrap();
    }

    #[tokio::test]
    async fn health_endpoint() {
        let result = health().await;
        let body = result.0;
        assert_eq!(body["status"], "ok");
        assert!(body.get("version").is_some());
    }

    #[tokio::test]
    async fn list_worlds_returns_worlds() {
        let worlds_dir = tempfile::tempdir().expect("worlds");
        setup_world(worlds_dir.path(), "test_world");
        let pool = init_memory_pool().expect("pool");
        let state = make_state(pool, worlds_dir.path().to_path_buf());

        let result = list_worlds(State(state.clone()))
            .await
            .expect("list_worlds");
        let worlds = result.0;
        let arr = worlds.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["world_id"], "test_world");
    }

    #[tokio::test]
    async fn get_world_detail() {
        let worlds_dir = tempfile::tempdir().expect("worlds");
        setup_world(worlds_dir.path(), "test_world");
        let pool = init_memory_pool().expect("pool");
        let state = make_state(pool, worlds_dir.path().to_path_buf());

        let result = get_world(State(state.clone()), Path("test_world".into()))
            .await
            .expect("get_world");
        let body = result.0;
        assert_eq!(body["world_id"], "test_world");
        assert!(body["setting"].as_str().unwrap().contains("Test World"));
    }

    #[tokio::test]
    async fn get_world_not_found() {
        let worlds_dir = tempfile::tempdir().expect("worlds");
        let pool = init_memory_pool().expect("pool");
        let state = make_state(pool, worlds_dir.path().to_path_buf());

        let result = get_world(State(state), Path("nonexistent".into())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_gms_returns_templates() {
        let worlds_dir = tempfile::tempdir().expect("worlds");
        let pool = init_memory_pool().expect("pool");
        let gms_dir = tempfile::tempdir().expect("gms_dir");
        let perspectives_dir = tempfile::tempdir().expect("perspectives_dir");
        let data_dir = tempfile::tempdir().expect("data_dir");
        let config = AppConfig::load(data_dir.path()).expect("config");

        let worlds_path = worlds_dir.path().to_path_buf();
        let gms_path = gms_dir.path().to_path_buf();
        let perspectives_path = perspectives_dir.path().to_path_buf();
        let data_path = data_dir.path().to_path_buf();

        // 创建 GM 文件
        std::fs::write(gms_path.join("test_gm.md"), "# Test GM\nStyle guide.").unwrap();

        let state: SharedState = Arc::new(crate::server::state::AppState {
            pool,
            data_dir: data_path,
            worlds_dir: worlds_path,
            gms_dir: gms_path,
            perspectives_dir: perspectives_path,
            config: RwLock::new(config),
        });

        // Keep tempdirs alive
        let _worlds = worlds_dir;
        let _gms = gms_dir;
        let _persp = perspectives_dir;
        let _data = data_dir;

        let result = list_gms(
            State(state),
            Query(std::collections::HashMap::new()),
        )
        .await
        .expect("list_gms");
        let gms = result.0;
        let arr = gms.as_array().unwrap();
        assert!(!arr.is_empty());
        assert!(arr.iter().any(|g| g["gm_id"] == "test_gm"));
    }

    #[tokio::test]
    async fn create_and_get_cycle() {
        let worlds_dir = tempfile::tempdir().expect("worlds");
        setup_world(worlds_dir.path(), "test_world");
        let pool = init_memory_pool().expect("pool");

        // 手动创建 state 保持 tempdir 存活
        let gms_dir = tempfile::tempdir().expect("gms_dir");
        let perspectives_dir = tempfile::tempdir().expect("perspectives_dir");
        let data_dir = tempfile::tempdir().expect("data_dir");
        let config = AppConfig::load(data_dir.path()).expect("config");

        let worlds_path = worlds_dir.path().to_path_buf();
        let gms_path = gms_dir.path().to_path_buf();
        let perspectives_path = perspectives_dir.path().to_path_buf();
        let data_path = data_dir.path().to_path_buf();

        // 创建 GM
        std::fs::write(gms_path.join("test_gm.md"), "# Test GM").unwrap();

        // 创建 perspective
        let p = perspective::Perspective {
            id: "pov_test".into(),
            source: "npc".into(),
            name: "Test NPC".into(),
            persona_data: "Test persona".into(),
            world_id: Some("test_world".into()),
        };
        perspective::save_perspective(&perspectives_path, &p).expect("save perspective");

        let state: SharedState = Arc::new(crate::server::state::AppState {
            pool,
            data_dir: data_path,
            worlds_dir: worlds_path,
            gms_dir: gms_path,
            perspectives_dir: perspectives_path,
            config: RwLock::new(config),
        });

        // Keep tempdirs alive until end of test
        let _worlds = worlds_dir;
        let _gms = gms_dir;
        let _persp = perspectives_dir;
        let _data = data_dir;

        let req = CreateCycleReq {
            world_id: "test_world".into(),
            perspective_id: "pov_test".into(),
            gm_id: "test_gm".into(),
            cycle_name: "Test Cycle".into(),
            oc_selections: Default::default(),
        };

        // 创建 cycle
        let result = create_cycle(State(state.clone()), Json(req))
            .await
            .expect("create_cycle");
        let cyc = result.0;
        assert_eq!(cyc["cycle_name"], "Test Cycle");
        let cycle_id = cyc["cycle_id"].as_str().unwrap().to_string();

        // 获取详情
        let detail = get_cycle(State(state.clone()), Path(cycle_id.clone()))
            .await
            .expect("get_cycle");
        let body = detail.0;
        assert_eq!(body["cycle"]["cycle_name"], "Test Cycle");
        assert_eq!(body["character_states"].as_array().unwrap().len(), 2);
        // relationships are 0 by default (no OC templates set)
        assert_eq!(body["relationships"].as_array().unwrap().len(), 0);

        // 列出
        let mut params = std::collections::HashMap::new();
        params.insert("world_id".into(), "test_world".into());
        let list = list_cycles(State(state.clone()), Query(params))
            .await
            .expect("list_cycles");
        assert_eq!(list.0.as_array().unwrap().len(), 1);

        // 删除
        let del = delete_cycle(State(state.clone()), Path(cycle_id.clone()))
            .await
            .expect("delete_cycle");
        assert_eq!(del.0["deleted"], cycle_id.clone());

        // 验证已删除
        let detail2 = get_cycle(State(state), Path(cycle_id.clone())).await;
        assert!(detail2.is_err());
    }

    #[tokio::test]
    async fn create_cycle_requires_valid_world() {
        let worlds_dir = tempfile::tempdir().expect("worlds");
        let pool = init_memory_pool().expect("pool");
        let state = make_state(pool, worlds_dir.path().to_path_buf());

        let req = CreateCycleReq {
            world_id: "nonexistent".into(),
            perspective_id: "pov_test".into(),
            gm_id: "test_gm".into(),
            cycle_name: "Bad Cycle".into(),
            oc_selections: Default::default(),
        };

        let result = create_cycle(State(state), Json(req)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_config_returns_masked_key() {
        let worlds_dir = tempfile::tempdir().expect("worlds");
        let pool = init_memory_pool().expect("pool");
        let state = make_state(pool, worlds_dir.path().to_path_buf());

        let result = get_config(State(state)).await;
        let body = result.0;
        assert!(body.get("llm").is_some());
        assert_eq!(body["llm"]["provider"], "OpenAI");
    }

    #[tokio::test]
    async fn process_turn_requires_api_key() {
        let worlds_dir = tempfile::tempdir().expect("worlds");
        let pool = init_memory_pool().expect("pool");
        let state = make_state(pool, worlds_dir.path().to_path_buf());

        // API key should be empty by default
        assert!(state.config.read().unwrap().llm.api_key.is_empty());

        let req = TurnReq {
            user_message: "hello".into(),
        };

        let result = process_turn(State(state), Path("any_cycle".into()), Json(req)).await;
        assert!(result.is_err());
    }
}
