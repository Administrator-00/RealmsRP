//! Reducer 流水线子模块 (M2.7-M2.10)
//!
//! - `state_reducer`: 处理 HP/MP/位置/状态变化
//! - `relationship_reducer`: 处理 RelationshipChange
//! - `arc_reducer`: 处理 ArcIncrement + 判定 arc_phase
//! - `memory_reducer`: 处理 3 层记忆写入
//!
//! 所有 reducer 共用 [`PatchOp`] 作为 RFC6902 patch 产出类型.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

pub mod arc_reducer;
pub mod relationship_reducer;
pub mod state_reducer;

pub use arc_reducer::apply_arc_increment;
pub use relationship_reducer::apply_relationship_change;
pub use state_reducer::apply_state_change;

/// RFC6902 JSON Patch 操作 — 所有 reducer 的统一产出类型.
///
/// WebUI 通过 AIRP-State-Protocol 消费 patch, 增量更新 widget.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PatchOp {
    /// 操作: "add" | "replace" | "remove"
    pub op: String,
    /// JSON Pointer 路径 (如 `/character_states/c1/lin_yueru/hp`)
    pub path: String,
    /// 新值 (remove 时省略)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<JsonValue>,
}

impl PatchOp {
    /// 构造 replace patch.
    pub fn replace(path: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            op: "replace".into(),
            path: path.into(),
            value: Some(value),
        }
    }

    /// 构造 add patch.
    pub fn add(path: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            op: "add".into(),
            path: path.into(),
            value: Some(value),
        }
    }
}

/// character_states 表 DDL (M2.7 测试用; M4.1 正式迁移会覆盖).
pub const CHARACTER_STATES_DDL: &str = "
CREATE TABLE IF NOT EXISTS character_states (
    cycle_id      TEXT NOT NULL,
    npc_id        TEXT NOT NULL,
    hp            INTEGER DEFAULT 100,
    hp_max        INTEGER DEFAULT 100,
    mp            INTEGER DEFAULT 50,
    mp_max        INTEGER DEFAULT 50,
    location_id   TEXT,
    status_flags  TEXT NOT NULL DEFAULT '{}',
    arcs          TEXT NOT NULL DEFAULT '{}',
    arc_phase     TEXT,
    perceived_mood TEXT NOT NULL DEFAULT 'neutral',
    updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (cycle_id, npc_id)
);";

/// npc_user_relationships + relationship_history 表 DDL (M2.8 测试用).
pub const RELATIONSHIP_DDL: &str = "
CREATE TABLE IF NOT EXISTS npc_user_relationships (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    cycle_id             TEXT NOT NULL,
    npc_id               TEXT NOT NULL,
    affinity             INTEGER DEFAULT 0,
    trust                INTEGER DEFAULT 0,
    intimacy             INTEGER DEFAULT 0,
    respect              INTEGER DEFAULT 0,
    current_type         TEXT,
    initial_template     TEXT,
    initial_setup_event_id INTEGER,
    first_met_at         TEXT,
    last_changed_event_id INTEGER,
    updated_at           TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(cycle_id, npc_id)
);

CREATE TABLE IF NOT EXISTS relationship_history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    cycle_id    TEXT NOT NULL,
    npc_id      TEXT NOT NULL,
    field       TEXT NOT NULL,
    old_value   TEXT,
    new_value   TEXT,
    delta       INTEGER,
    event_id    INTEGER,
    world_time  TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);";
