//! GM Dispatcher (M7) — 完整 dispatch 管线: context → LLM → parse → audit
//!
//! 串接 M2.4-M2.6 的三重防护 + LLM 调用 + 输出解析 + 审计.
//!
//! 管线:
//! 1. build_subagent_context (M2.4 独立 context)
//! 2. LlmProvider::chat (调 LLM)
//! 3. parse_subagent_output (提取 visible/intent/events)
//! 4. audit_for_crosstalk (审计)
//! 5. 返回 FullDispatchResult

use super::gm_router::{build_subagent_context, RoleDispatchRequest, RoleToolWhitelist};
use crate::error::{RealmsError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// ============================================================================
// LLM Provider 抽象
// ============================================================================

/// LLM 调用抽象 (支持 mock / real HTTP / AIRP engine 等多种后端).
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// 发送 system_prompt + user_message, 返回原始 LLM 输出.
    async fn chat(&self, system_prompt: &str, user_message: &str) -> Result<String>;
}

/// Mock LLM provider (测试用).
///
/// 返回预设的 canned response, 用于验证管线逻辑而不依赖真实 LLM.
pub struct MockLlmProvider {
    /// 预设的 LLM 回复 (模拟 LLM 输出)
    pub canned_response: String,
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    async fn chat(&self, _system: &str, _user: &str) -> Result<String> {
        Ok(self.canned_response.clone())
    }
}

/// HTTP LLM provider — 通过 AIRP Engine `/v1/chat/completions` 调用真实 LLM.
///
/// 将 system_prompt 嵌入内联角色卡 JSON, 通过 SSE 流接收回复.
pub struct HttpLlmProvider {
    /// AIRP engine URL (如 http://127.0.0.1:8000)
    pub engine_url: String,
    /// reqwest HTTP 客户端
    http_client: reqwest::Client,
}

impl HttpLlmProvider {
    /// 创建新的 HTTP provider.
    pub fn new(engine_url: impl Into<String>) -> Self {
        Self {
            engine_url: engine_url.into(),
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for HttpLlmProvider {
    async fn chat(&self, system_prompt: &str, user_message: &str) -> Result<String> {
        // 将 system_prompt 嵌入内联角色卡
        let card = serde_json::json!({
            "name": "subagent",
            "description": system_prompt,
            "personality": "",
            "first_mes": "",
            "scenario": "",
            "mes_example": ""
        });

        let body = serde_json::json!({
            "user_profile": {"name": "GM", "variables": {}},
            "message": user_message,
            "character_card_id": card.to_string()
        });

        let resp = self
            .http_client
            .post(format!("{}/v1/chat/completions", self.engine_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| RealmsError::internal(format!("HTTP request failed: {e}")))?;

        let text = resp
            .text()
            .await
            .map_err(|e| RealmsError::internal(format!("read response: {e}")))?;

        Ok(parse_sse_response(&text))
    }
}

/// 解析 AIRP Engine SSE 响应 (event: message / data: {...}),
/// 提取所有 body_chunk 文本并拼接.
fn parse_sse_response(sse_text: &str) -> String {
    let mut result = String::new();
    for line in sse_text.lines() {
        if let Some(data) = line.strip_prefix("data: ") {
            if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) {
                if chunk.get("type").and_then(|v| v.as_str()) == Some("body_chunk") {
                    if let Some(text) = chunk.get("text").and_then(|v| v.as_str()) {
                        result.push_str(text);
                    }
                }
            }
        }
    }
    result
}

// ============================================================================
// 完整 Dispatch 数据结构
// ============================================================================

/// subagent 建议的领域事件 (不含 cycle_id, 由 GM 补充)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SuggestedEvent {
    /// 事件类型 (dialogue / state_change / ...)
    pub event_type: String,
    /// 行为者 id
    pub actor_id: String,
    /// 事件摘要
    pub summary: String,
    /// 重要性 [0.0, 1.0]
    pub importance: f64,
}

/// 完整 dispatch 结果 (替代 M2.4 的 stub RoleDispatchResult)
#[derive(Debug, Clone, PartialEq)]
pub struct FullDispatchResult {
    /// NPC id
    pub npc_id: String,
    /// NPC 公开说/做的事 (叙事文本)
    pub visible_response: String,
    /// NPC 内心独白 (不透给其他角色)
    pub private_intent: String,
    /// 建议的领域事件 (由 GM 审核后决定是否写入)
    pub suggested_events: Vec<SuggestedEvent>,
    /// 审计警告 (crosstalk 等)
    pub audit_warnings: Vec<String>,
}

// ============================================================================
// 管线核心
// ============================================================================

/// 完整 dispatch: context → LLM → parse → audit.
///
/// # 参数
/// - `req`: 角色调度请求 (npc_id + persona + world + events + user_msg)
/// - `whitelist`: 工具白名单 (M2.6, 用于后续扩展)
/// - `llm`: LLM provider (mock / real)
/// - `other_npc_ids`: 同场景其他 NPC id 列表 (用于审计 crosstalk)
pub async fn full_dispatch(
    req: &RoleDispatchRequest,
    whitelist: &RoleToolWhitelist,
    llm: &dyn LlmProvider,
    other_npc_ids: &[String],
) -> Result<FullDispatchResult> {
    if req.npc_id.is_empty() {
        return Err(RealmsError::invalid("npc_id is empty"));
    }
    if req.npc_persona.trim().is_empty() {
        return Err(RealmsError::invalid("npc_persona is empty"));
    }
    if !whitelist.allows(super::gm_router::RoleCapability::SuggestEvent) {
        return Err(RealmsError::invalid(format!(
            "NPC {} lacks SuggestEvent capability",
            req.npc_id
        )));
    }

    // 1. 构建 subagent context (M2.4)
    let system_prompt = build_subagent_context(req);

    // 2. 调 LLM
    let raw_response = llm.chat(&system_prompt, &req.user_message).await?;

    // 3. 解析 LLM 输出
    let (visible, intent, events) = parse_subagent_output(&raw_response)?;

    // 4. 审计 crosstalk
    let warnings = audit_for_crosstalk(&intent, other_npc_ids);

    Ok(FullDispatchResult {
        npc_id: req.npc_id.clone(),
        visible_response: visible,
        private_intent: intent,
        suggested_events: events,
        audit_warnings: warnings,
    })
}

// ============================================================================
// 输出解析
// ============================================================================

/// 解析 subagent LLM 输出, 提取 [VISIBLE] / [INTENT] / [EVENTS] 三段.
///
/// 格式:
/// ```text
/// [VISIBLE]
/// 公开说的话或做的事...
///
/// [INTENT]
/// 内心独白...
///
/// [EVENTS]
/// [{"type":"dialogue","actor":"npc","summary":"...","importance":0.3}]
/// ```
pub fn parse_subagent_output(raw: &str) -> Result<(String, String, Vec<SuggestedEvent>)> {
    let visible = extract_section(raw, "VISIBLE").unwrap_or_else(|| raw.trim().to_string());
    let intent = extract_section(raw, "INTENT").unwrap_or_default();
    let events = extract_section(raw, "EVENTS")
        .map(|json_str| parse_suggested_events(&json_str).unwrap_or_default())
        .unwrap_or_default();

    Ok((visible, intent, events))
}

/// 提取标记段 (如 `[VISIBLE]\n...\n\n[INTENT]` 中 [VISIBLE] 的内容).
fn extract_section(raw: &str, marker: &str) -> Option<String> {
    let start_tag = format!("[{}]", marker);
    let start = raw.find(&start_tag)? + start_tag.len();
    let remainder = raw[start..].trim_start();

    // 找到下一个 `[` 标记 (即下一段开头) 作为结束
    let end = remainder.find("\n[").unwrap_or(remainder.len());
    let content = remainder[..end].trim().to_string();

    if content.is_empty() {
        None
    } else {
        Some(content)
    }
}

/// 解析 [EVENTS] 段的 JSON 数组.
fn parse_suggested_events(json_str: &str) -> Result<Vec<SuggestedEvent>> {
    let trimmed = json_str.trim();
    if trimmed.is_empty() || trimmed == "[]" {
        return Ok(vec![]);
    }
    serde_json::from_str(trimmed)
        .map_err(|e| RealmsError::invalid(format!("invalid EVENTS JSON: {e}")))
}

// ============================================================================
// 审计 (M7.4)
// ============================================================================

/// 审计 subagent 输出的 private_intent 是否串台.
///
/// 检查规则:
/// - private_intent 不应提及同场景其他 NPC 的名字 (可能串台)
/// - private_intent 不应包含 DM orchestrator 内部变量
pub fn audit_for_crosstalk(private_intent: &str, other_npc_ids: &[String]) -> Vec<String> {
    let mut warnings = Vec::new();

    let intent_lower = private_intent.to_lowercase();

    // 检查是否提到其他 NPC
    for npc_id in other_npc_ids {
        if intent_lower.contains(&npc_id.to_lowercase()) {
            warnings.push(format!(
                "crosstalk: private_intent mentions other NPC '{}'",
                npc_id
            ));
        }
    }

    // 检查是否含 orchestrator 内部变量
    let forbidden = [
        "cycle_id",
        "gm_id",
        "prompt_assembler",
        "orchestrator",
        "subagent",
    ];
    for word in forbidden {
        if intent_lower.contains(word) {
            warnings.push(format!(
                "crosstalk: private_intent contains orchestrator variable '{}'",
                word
            ));
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::super::gm_router::RoleCapability;
    use super::*;

    fn make_req() -> RoleDispatchRequest {
        RoleDispatchRequest {
            npc_id: "lin_yueru".into(),
            npc_persona: "林月如 persona".into(),
            world_setting: "神州浩土".into(),
            visible_events: vec!["事件1".into()],
            user_message: "你好".into(),
        }
    }

    fn mock_llm(response: &str) -> MockLlmProvider {
        MockLlmProvider {
            canned_response: response.to_string(),
        }
    }

    // ===== M7.1-M7.2: 完整 dispatch =====

    #[tokio::test]
    async fn full_dispatch_with_mock_llm() {
        let req = make_req();
        let whitelist = RoleToolWhitelist::npc_default();
        let llm =
            mock_llm("[VISIBLE]\n你好，我是林月如。\n\n[INTENT]\n这人看着面生。\n\n[EVENTS]\n[]");

        let result = full_dispatch(&req, &whitelist, &llm, &[])
            .await
            .expect("dispatch");
        assert_eq!(result.visible_response, "你好，我是林月如。");
        assert_eq!(result.private_intent, "这人看着面生。");
        assert!(result.suggested_events.is_empty());
        assert!(result.audit_warnings.is_empty());
    }

    #[tokio::test]
    async fn dispatch_with_suggested_events() {
        let req = make_req();
        let whitelist = RoleToolWhitelist::npc_default();
        let llm = mock_llm(
            "[VISIBLE]\n喂！\n\n[INTENT]\n又见面了。\n\n[EVENTS]\n[{\"event_type\":\"dialogue\",\"actor_id\":\"lin_yueru\",\"summary\":\"林月如打招呼\",\"importance\":0.3}]",
        );

        let result = full_dispatch(&req, &whitelist, &llm, &[])
            .await
            .expect("dispatch");
        assert_eq!(result.suggested_events.len(), 1);
        assert_eq!(result.suggested_events[0].event_type, "dialogue");
        assert_eq!(result.suggested_events[0].importance, 0.3);
    }

    #[tokio::test]
    async fn dispatch_without_suggest_event_capability_fails() {
        let req = make_req();
        let whitelist = RoleToolWhitelist {
            capabilities: vec![RoleCapability::LookupSelf],
        };
        let llm = mock_llm("[VISIBLE]\nhi\n\n[INTENT]\n...");
        assert!(full_dispatch(&req, &whitelist, &llm, &[]).await.is_err());
    }

    // ===== M7.4: 审计 =====

    #[test]
    fn audit_detects_crosstalk() {
        let intent = "li_xiaoyao 似乎有心事，我得留意他。";
        let warnings = audit_for_crosstalk(intent, &["li_xiaoyao".into()]);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("li_xiaoyao"));
    }

    #[test]
    fn audit_detects_orchestrator_noise() {
        let intent = "我应该用 cycle_id=c1 查询";
        let warnings = audit_for_crosstalk(intent, &[]);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("cycle_id"));
    }

    #[test]
    fn audit_clean_intent_no_warnings() {
        let intent = "这人看起来是个江湖人，小心为上。";
        let warnings = audit_for_crosstalk(intent, &["li_xiaoyao".into()]);
        assert!(warnings.is_empty());
    }

    // ===== parse_subagent_output =====

    #[test]
    fn parse_three_sections() {
        let raw = "[VISIBLE]\n你好\n\n[INTENT]\n秘密\n\n[EVENTS]\n[]";
        let (v, i, e) = parse_subagent_output(raw).expect("parse");
        assert_eq!(v, "你好");
        assert_eq!(i, "秘密");
        assert!(e.is_empty());
    }

    #[test]
    fn parse_missing_intent_ok() {
        let raw = "[VISIBLE]\n你好\n\n[EVENTS]\n[]";
        let (v, i, _) = parse_subagent_output(raw).expect("parse");
        assert_eq!(v, "你好");
        assert!(i.is_empty());
    }

    #[test]
    fn parse_no_markers_uses_all_as_visible() {
        let raw = "你好，我是林月如。";
        let (v, i, e) = parse_subagent_output(raw).expect("parse");
        assert_eq!(v, "你好，我是林月如。");
        assert!(i.is_empty());
        assert!(e.is_empty());
    }

    // ===== M7.5: 真实 LLM 集成测试 (需运行中的 AIRP engine) =====

    #[tokio::test]
    #[ignore = "需要运行中的 AIRP engine + 已配置 LLM provider"]
    async fn integration_real_llm_subagent_dispatch() {
        let provider = HttpLlmProvider::new("http://127.0.0.1:8000");
        let whitelist = RoleToolWhitelist::npc_default();

        let req = RoleDispatchRequest {
            npc_id: "npc_innkeeper".into(),
            npc_persona: "你是余杭镇盛帆客栈的老板。你精明圆滑，嘴上刻薄但心地善良。\\n\\n说话风格：语气圆滑，常以'客官'称呼他人。".into(),
            world_setting: "神州浩土，仙魔纷争的时代。".into(),
            visible_events: vec!["一位背剑的年轻人走进客栈".into()],
            user_message: "（你看到一位背剑的年轻人走进客栈）".into(),
        };

        let result = full_dispatch(&req, &whitelist, &provider, &[])
            .await
            .expect("real LLM dispatch");

        println!("=== visible_response ===\n{}", result.visible_response);
        println!("=== private_intent ===\n{}", result.private_intent);
        println!("=== suggested_events ===\n{:?}", result.suggested_events);
        println!("=== audit_warnings ===\n{:?}", result.audit_warnings);

        assert!(!result.visible_response.is_empty(), "LLM 应返回可见回复");
        assert!(result.visible_response.len() > 5, "回复过短");
    }
}
