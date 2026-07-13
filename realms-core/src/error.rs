//! Realms 统一错误类型
//!
//! 所有 pub fn 都应返回 `Result<T, RealmsError>`, 不用 `panic` / `unwrap` / `String` 错误.
//!
//! 错误分类:
//! - `NotFound`: 资源不存在 (cycle/world/npc/...)
//! - `InvalidInput`: 用户/外部输入不符合 schema
//! - `Database`: SQLite 错误 (rusqlite 自动 From)
//! - `Pool`: 连接池错误 (r2d2 自动 From)
//! - `Io`: 文件系统错误 (std::io::Error 自动 From)
//! - `Serde`: JSON 序列化错误 (serde_json 自动 From)
//! - `Internal`: 内部状态错误 (用 `internal()` 构造)
//! - `Config`: 配置错误 (缺 key, 格式错等)
//! - `Cycle`: 周目状态错误 (重复创建, 状态冲突)
//! - `Event`: 事件验证错误 (validate 失败)

use thiserror::Error;

/// Realms 统一错误类型
#[derive(Debug, Error)]
pub enum RealmsError {
    /// 资源不存在
    #[error("not found: {0}")]
    NotFound(String),

    /// 无效输入
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// SQLite 错误
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// 连接池错误
    #[error("pool error: {0}")]
    Pool(#[from] r2d2::Error),

    /// IO 错误
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 序列化错误
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// 内部错误
    #[error("internal: {0}")]
    Internal(String),

    /// 配置错误
    #[error("config: {0}")]
    Config(String),

    /// 周目状态错误
    #[error("cycle: {0}")]
    Cycle(String),

    /// 事件验证错误
    #[error("event validation: {0}")]
    Event(String),
}

impl RealmsError {
    /// 构造内部错误 (用 panic message 风格, 但不 panic)
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// 构造 NotFound 错误 (便捷构造)
    pub fn not_found(resource: &str, id: &str) -> Self {
        Self::NotFound(format!("{}: {}", resource, id))
    }

    /// 构造 InvalidInput 错误 (便捷构造)
    pub fn invalid(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }
}

/// Realms Result 类型别名
pub type Result<T> = std::result::Result<T, RealmsError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_helper() {
        let e = RealmsError::not_found("cycle", "cycle_001");
        assert!(e.to_string().contains("cycle_001"));
        assert!(e.to_string().contains("not found"));
    }

    #[test]
    fn invalid_helper() {
        let e = RealmsError::invalid("bad input");
        assert!(e.to_string().contains("invalid input"));
    }

    #[test]
    fn internal_helper() {
        let e = RealmsError::internal("bug");
        assert!(e.to_string().contains("internal: bug"));
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let err: RealmsError = io_err.into();
        assert!(matches!(err, RealmsError::Io(_)));
    }

    #[test]
    fn from_serde_error() {
        let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let err: RealmsError = json_err.into();
        assert!(matches!(err, RealmsError::Serde(_)));
    }
}
