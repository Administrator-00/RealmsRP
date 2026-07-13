//! 应用配置管理 (G0)
//!
//! 配置持久化到 `data/config.json`，启动时自动加载。
//! WebUI 设置页通过 `/api/config` 端点读写。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::{RealmsError, Result};

/// LLM 提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// 提供商: "OpenAI" 或 "Anthropic"
    pub provider: String,
    /// API endpoint URL
    pub endpoint: String,
    /// API key
    pub api_key: String,
    /// 模型名称
    pub model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "OpenAI".into(),
            endpoint: "https://api.deepseek.com/v1/chat/completions".into(),
            api_key: String::new(),
            model: "deepseek-chat".into(),
        }
    }
}

/// 应用配置根
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub llm: LlmConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
        }
    }
}

impl AppConfig {
    /// 从 `{data_dir}/config.json` 加载配置；如果文件不存在则创建默认配置并写入。
    pub fn load(data_dir: &Path) -> Result<Self> {
        let path = config_path(data_dir);
        if path.exists() {
            let content = std::fs::read_to_string(&path).map_err(RealmsError::Io)?;
            let config: AppConfig =
                serde_json::from_str(&content).map_err(|e| RealmsError::Config(e.to_string()))?;
            Ok(config)
        } else {
            let config = AppConfig::default();
            config.save(data_dir)?;
            Ok(config)
        }
    }

    /// 保存配置到 `{data_dir}/config.json`。
    pub fn save(&self, data_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(data_dir)?;
        let path = config_path(data_dir);
        let json = serde_json::to_string_pretty(self).map_err(RealmsError::Serde)?;
        std::fs::write(&path, json).map_err(RealmsError::Io)?;
        Ok(())
    }

    /// 脱敏版 llm 配置（给 WebUI 看，隐藏 api_key 中间部分）
    pub fn masked_llm(&self) -> LlmConfig {
        let masked_key = mask_api_key(&self.llm.api_key);
        LlmConfig {
            api_key: masked_key,
            ..self.llm.clone()
        }
    }

    /// 用给定 llm 配置部分更新，合并后保存。
    pub fn update_llm(&mut self, llm: LlmConfig, data_dir: &Path) -> Result<()> {
        // 验证
        if llm.provider != "OpenAI" && llm.provider != "Anthropic" {
            return Err(RealmsError::invalid(
                "provider must be 'OpenAI' or 'Anthropic'",
            ));
        }
        if llm.endpoint.is_empty() {
            return Err(RealmsError::invalid("endpoint must not be empty"));
        }
        // 如果传过来的 api_key 是脱敏后的，保留原来的
        if llm.api_key.starts_with("sk-") && llm.api_key.contains("***") {
            // 脱敏键不覆盖
        } else {
            self.llm.api_key = llm.api_key;
        }
        self.llm.provider = llm.provider;
        self.llm.endpoint = llm.endpoint;
        self.llm.model = llm.model;
        self.save(data_dir)?;
        Ok(())
    }
}

/// 配置文件的完整路径
fn config_path(data_dir: &Path) -> PathBuf {
    data_dir.join("config.json")
}

/// 脱敏 api_key，保留首尾各 3 字符，中间替换为 `***`
pub fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        return String::new();
    }
    if key.len() <= 8 {
        return format!("{}***", &key[..key.len().min(3)]);
    }
    let start = &key[..3];
    let end = &key[key.len() - 3..];
    format!("{start}***{end}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_matches_expected() {
        let c = AppConfig::default();
        assert_eq!(c.llm.provider, "OpenAI");
        assert_eq!(
            c.llm.endpoint,
            "https://api.deepseek.com/v1/chat/completions"
        );
        assert_eq!(c.llm.api_key, "");
        assert_eq!(c.llm.model, "deepseek-chat");
    }

    #[test]
    fn load_creates_default_when_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let config = AppConfig::load(dir.path()).expect("load");
        assert_eq!(config.llm.provider, "OpenAI");
        assert!(dir.path().join("config.json").exists());
    }

    #[test]
    fn save_and_reload_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut config = AppConfig::load(dir.path()).expect("load");
        config.llm.api_key = "sk-test-secret".into();
        config.llm.model = "claude-sonnet".into();
        config.save(dir.path()).expect("save");

        let reloaded = AppConfig::load(dir.path()).expect("reload");
        assert_eq!(reloaded.llm.api_key, "sk-test-secret");
        assert_eq!(reloaded.llm.model, "claude-sonnet");
    }

    #[test]
    fn masked_key_shows_asterisks() {
        let m = mask_api_key("sk-abc123xyz789");
        assert!(m.contains("***"));
        assert!(m.starts_with("sk-"));
    }

    #[test]
    fn masked_key_empty() {
        assert_eq!(mask_api_key(""), "");
    }

    #[test]
    fn masked_key_short() {
        let m = mask_api_key("sk-abc");
        assert!(m.contains("***"));
    }

    #[test]
    fn masked_llm_does_not_leak_full_key() {
        let config = AppConfig {
            llm: LlmConfig {
                provider: "OpenAI".into(),
                endpoint: "https://api.test.com/v1".into(),
                api_key: "sk-very-secret-key-that-should-not-leak".into(),
                model: "gpt-4".into(),
            },
        };
        let masked = config.masked_llm();
        assert!(!masked.api_key.contains("very-secret"));
        assert!(masked.api_key.contains("***"));
    }

    #[test]
    fn update_llm_rejects_invalid_provider() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut config = AppConfig::load(dir.path()).expect("load");
        let bad = LlmConfig {
            provider: "DeepSeek".into(),
            endpoint: "https://api.example.com".into(),
            api_key: "sk-123".into(),
            model: "v3".into(),
        };
        let err = config.update_llm(bad, dir.path()).unwrap_err();
        assert!(err.to_string().contains("provider"));
    }

    #[test]
    fn update_llm_preserves_key_when_masked() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut config = AppConfig::load(dir.path()).expect("load");
        config.llm.api_key = "sk-real-secret-key".into();
        // 传入脱敏版 api_key — 应保留原值
        let masked_req = LlmConfig {
            provider: "Anthropic".into(),
            endpoint: "https://api.anthropic.com".into(),
            api_key: "sk-***key".into(),
            model: "claude-sonnet".into(),
        };
        config.update_llm(masked_req, dir.path()).expect("update with masked key");
        assert_eq!(config.llm.api_key, "sk-real-secret-key");
        assert_eq!(config.llm.provider, "Anthropic");
    }

    #[test]
    fn update_llm_rejects_empty_endpoint() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut config = AppConfig::load(dir.path()).expect("load");
        let bad = LlmConfig {
            provider: "OpenAI".into(),
            endpoint: "".into(),
            api_key: "sk-123".into(),
            model: "gpt-4".into(),
        };
        let err = config.update_llm(bad, dir.path()).unwrap_err();
        assert!(err.to_string().contains("endpoint"));
    }
}
