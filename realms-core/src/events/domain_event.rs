//! DomainEvent (M2.2 实现)
//!
//! 9 个 variant 枚举: Setup / ArcIncrement / RelationshipChange / StateChange /
//! ItemChange / TimeAdvance / Dialogue / Discovery / Combat.
//!
//! 详见 3.md §2.2 domain_events 表.

use serde::{Deserialize, Serialize};

/// 事件类型分类 (与 SQL `event_type` 列对应)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// 周期初始化 (开 cycle 时的批量事件)
    Setup,
    /// 弧光增量
    ArcIncrement,
    /// 关系变更
    RelationshipChange,
    /// 状态变更 (HP/MP/位置/flag)
    StateChange,
    /// 物品变更
    ItemChange,
    /// 时间推进
    TimeAdvance,
    /// 对话
    Dialogue,
    /// 探索/发现
    Discovery,
    /// 战斗
    Combat,
}

impl EventType {
    /// 字符串形式 (用于 SQL `event_type` 列)
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Setup => "setup",
            Self::ArcIncrement => "arc_increment",
            Self::RelationshipChange => "relationship_change",
            Self::StateChange => "state_change",
            Self::ItemChange => "item_change",
            Self::TimeAdvance => "time_advance",
            Self::Dialogue => "dialogue",
            Self::Discovery => "discovery",
            Self::Combat => "combat",
        }
    }

    /// 从字符串解析 (用于读 SQL 行时)
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "setup" => Self::Setup,
            "arc_increment" => Self::ArcIncrement,
            "relationship_change" => Self::RelationshipChange,
            "state_change" => Self::StateChange,
            "item_change" => Self::ItemChange,
            "time_advance" => Self::TimeAdvance,
            "dialogue" => Self::Dialogue,
            "discovery" => Self::Discovery,
            "combat" => Self::Combat,
            _ => return None,
        })
    }
}

/// 行为者类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    /// NPC
    Npc,
    /// 用户 (玩家)
    User,
    /// 系统 (DM 引擎, reducer 等)
    System,
}

/// 目标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetType {
    /// NPC
    Npc,
    /// 用户
    User,
    /// 物品
    Item,
    /// 地点
    Location,
    /// 无目标
    None,
}

/// 领域事件 (事件溯源单一真相源)
///
/// 9 个 variant 覆盖所有可能的状态变更事件.
/// 每个 variant 对应一个 reducer 处理.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainEvent {
    /// 周期初始化 (开 cycle 时由 oc_initializer 产生)
    Setup {
        /// 周目 id
        cycle_id: String,
        /// 视角 id (perspective)
        perspective_id: String,
        /// GM 模板 id
        gm_id: String,
        /// 已应用的关系模板列表
        templates_applied: Vec<AppliedTemplate>,
    },

    /// 弧光增量
    ArcIncrement {
        /// 周目 id
        cycle_id: String,
        /// 行为者 id
        actor_id: String,
        /// 行为者类型
        actor_type: ActorType,
        /// 目标 id
        target_id: String,
        /// 目标类型
        target_type: TargetType,
        /// 弧光维度 (如 trust_user / affection_user / courage)
        arc_dimension: String,
        /// 变化量 (可负)
        delta: f64,
        /// 游戏内时间
        world_time: Option<String>,
        /// 重要性 [0.0, 1.0]
        importance: f64,
        /// 人类可读摘要
        summary: String,
    },

    /// 关系变更 (NPC↔User)
    RelationshipChange {
        /// 周目 id
        cycle_id: String,
        /// NPC id
        npc_id: String,
        /// 变更字段 (affinity / trust / intimacy / respect / type)
        field: String,
        /// 旧值 (首次设置时为 None)
        old_value: Option<String>,
        /// 新值
        new_value: String,
        /// 数值变化量 (类型变更时可能为 None)
        delta: Option<f64>,
        /// 游戏内时间
        world_time: Option<String>,
        /// 重要性 [0.0, 1.0]
        importance: f64,
        /// 人类可读摘要
        summary: String,
    },

    /// 状态变更 (HP/MP/位置/flag)
    StateChange {
        /// 周目 id
        cycle_id: String,
        /// 行为者 id
        actor_id: String,
        /// 行为者类型
        actor_type: ActorType,
        /// 目标 id (环境/无目标时为 None)
        target_id: Option<String>,
        /// 目标类型
        target_type: TargetType,
        /// 变更字段 (hp / mp / location_id / status_flag / arc_phase / mood)
        field: String,
        /// 旧值 (首次设置时为 None)
        old_value: Option<String>,
        /// 新值
        new_value: String,
        /// 游戏内时间
        world_time: Option<String>,
        /// 重要性 [0.0, 1.0]
        importance: f64,
        /// 人类可读摘要
        summary: String,
    },

    /// 物品变更
    ItemChange {
        /// 周目 id
        cycle_id: String,
        /// 持有者 id (npc_id 或 user_perspective_id)
        owner_id: String,
        /// 物品名
        item_name: String,
        /// 变更类型 (acquire / lose / use / equip)
        change_type: String,
        /// 数量变化量 (可负, 表示减少)
        quantity_delta: i64,
        /// 游戏内时间
        world_time: Option<String>,
        /// 重要性 [0.0, 1.0]
        importance: f64,
        /// 人类可读摘要
        summary: String,
    },

    /// 时间推进
    TimeAdvance {
        /// 周目 id
        cycle_id: String,
        /// 起始游戏内时间
        from_time: String,
        /// 目标游戏内时间
        to_time: String,
        /// 经过分钟数
        elapsed_minutes: i64,
        /// 重要性 [0.0, 1.0]
        importance: f64,
        /// 人类可读摘要
        summary: String,
    },

    /// 对话
    Dialogue {
        /// 周目 id
        cycle_id: String,
        /// 说话者 id
        actor_id: String,
        /// 说话者类型
        actor_type: ActorType,
        /// 对话目标 id (独白时为 None)
        target_id: Option<String>,
        /// 目标类型
        target_type: TargetType,
        /// 所在地点 id
        location_id: Option<String>,
        /// 台词内容
        line: String,
        /// 游戏内时间
        world_time: Option<String>,
        /// 重要性 [0.0, 1.0]
        importance: f64,
        /// 人类可读摘要
        summary: String,
    },

    /// 探索/发现
    Discovery {
        /// 周目 id
        cycle_id: String,
        /// 发现者 id
        discoverer_id: String,
        /// 地点 id
        location_id: String,
        /// 地点名称
        location_name: String,
        /// 游戏内时间
        world_time: Option<String>,
        /// 重要性 [0.0, 1.0]
        importance: f64,
        /// 人类可读摘要
        summary: String,
    },

    /// 战斗
    Combat {
        /// 周目 id
        cycle_id: String,
        /// 攻击者 id
        attacker_id: String,
        /// 防守者 id
        defender_id: String,
        /// 结果 (win / lose / flee / draw)
        outcome: String,
        /// 造成伤害
        damage_dealt: i64,
        /// 游戏内时间
        world_time: Option<String>,
        /// 重要性 [0.0, 1.0]
        importance: f64,
        /// 人类可读摘要
        summary: String,
    },
}

/// OC 关系模板应用记录 (Setup 事件的子结构)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedTemplate {
    /// 被设置关系的 NPC id
    pub npc_id: String,
    /// 模板 id (如 tpl_childhood_friend)
    pub template_id: String,
    /// 初始好感度
    pub affinity: i32,
    /// 初始信任
    pub trust: i32,
    /// 初始亲密
    pub intimacy: i32,
    /// 初始关系类型
    pub current_type: String,
}

impl DomainEvent {
    /// 事件类型 (与 SQL `event_type` 列对应)
    pub fn event_type(&self) -> EventType {
        match self {
            Self::Setup { .. } => EventType::Setup,
            Self::ArcIncrement { .. } => EventType::ArcIncrement,
            Self::RelationshipChange { .. } => EventType::RelationshipChange,
            Self::StateChange { .. } => EventType::StateChange,
            Self::ItemChange { .. } => EventType::ItemChange,
            Self::TimeAdvance { .. } => EventType::TimeAdvance,
            Self::Dialogue { .. } => EventType::Dialogue,
            Self::Discovery { .. } => EventType::Discovery,
            Self::Combat { .. } => EventType::Combat,
        }
    }

    /// cycle_id (所有 variant 都有)
    pub fn cycle_id(&self) -> &str {
        match self {
            Self::Setup { cycle_id, .. }
            | Self::ArcIncrement { cycle_id, .. }
            | Self::RelationshipChange { cycle_id, .. }
            | Self::StateChange { cycle_id, .. }
            | Self::ItemChange { cycle_id, .. }
            | Self::TimeAdvance { cycle_id, .. }
            | Self::Dialogue { cycle_id, .. }
            | Self::Discovery { cycle_id, .. }
            | Self::Combat { cycle_id, .. } => cycle_id,
        }
    }

    /// 重要性 [0.0, 1.0]
    pub fn importance(&self) -> f64 {
        match self {
            Self::Setup { .. } => 0.8,
            Self::ArcIncrement { importance, .. }
            | Self::RelationshipChange { importance, .. }
            | Self::StateChange { importance, .. }
            | Self::ItemChange { importance, .. }
            | Self::TimeAdvance { importance, .. }
            | Self::Dialogue { importance, .. }
            | Self::Discovery { importance, .. }
            | Self::Combat { importance, .. } => *importance,
        }
    }

    /// 验证必填字段 (防止 reducer 拿到坏数据)
    ///
    /// # 错误
    ///
    /// - `cycle_id` 空 → `RealmsError::Event`
    /// - `importance` 不在 [0, 1] → `RealmsError::Event`
    /// - `summary` 空 → `RealmsError::Event`
    pub fn validate(&self) -> crate::error::Result<()> {
        use crate::error::RealmsError;

        if self.cycle_id().is_empty() {
            return Err(RealmsError::Event("cycle_id is empty".into()));
        }
        let imp = self.importance();
        if !(0.0..=1.0).contains(&imp) {
            return Err(RealmsError::Event(format!(
                "importance {} out of [0, 1]",
                imp
            )));
        }
        let summary = self.summary();
        if summary.is_empty() {
            return Err(RealmsError::Event("summary is empty".into()));
        }
        Ok(())
    }

    /// summary 字段 (所有 variant 都有, 但 setup 例外)
    pub fn summary(&self) -> &str {
        match self {
            Self::Setup { .. } => "cycle setup",
            Self::ArcIncrement { summary, .. }
            | Self::RelationshipChange { summary, .. }
            | Self::StateChange { summary, .. }
            | Self::ItemChange { summary, .. }
            | Self::TimeAdvance { summary, .. }
            | Self::Dialogue { summary, .. }
            | Self::Discovery { summary, .. }
            | Self::Combat { summary, .. } => summary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_round_trip() {
        for et in [
            EventType::Setup,
            EventType::ArcIncrement,
            EventType::RelationshipChange,
            EventType::StateChange,
            EventType::ItemChange,
            EventType::TimeAdvance,
            EventType::Dialogue,
            EventType::Discovery,
            EventType::Combat,
        ] {
            assert_eq!(EventType::parse(et.as_str()), Some(et));
        }
    }

    #[test]
    fn event_type_parse_unknown() {
        assert_eq!(EventType::parse("nonsense"), None);
    }

    #[test]
    fn setup_event_validate_ok() {
        let ev = DomainEvent::Setup {
            cycle_id: "c1".into(),
            perspective_id: "p1".into(),
            gm_id: "default".into(),
            templates_applied: vec![],
        };
        assert!(ev.validate().is_ok());
    }

    #[test]
    fn empty_cycle_id_fails() {
        let ev = DomainEvent::Setup {
            cycle_id: "".into(),
            perspective_id: "p1".into(),
            gm_id: "default".into(),
            templates_applied: vec![],
        };
        assert!(ev.validate().is_err());
    }

    #[test]
    fn importance_out_of_range_fails() {
        let ev = DomainEvent::Dialogue {
            cycle_id: "c1".into(),
            actor_id: "u".into(),
            actor_type: ActorType::User,
            target_id: Some("npc1".into()),
            target_type: TargetType::Npc,
            location_id: None,
            line: "hello".into(),
            world_time: None,
            importance: 1.5,
            summary: "test".into(),
        };
        assert!(ev.validate().is_err());
    }

    #[test]
    fn empty_summary_fails() {
        let ev = DomainEvent::Dialogue {
            cycle_id: "c1".into(),
            actor_id: "u".into(),
            actor_type: ActorType::User,
            target_id: None,
            target_type: TargetType::None,
            location_id: None,
            line: "hello".into(),
            world_time: None,
            importance: 0.5,
            summary: "".into(),
        };
        assert!(ev.validate().is_err());
    }

    #[test]
    fn arc_increment_serialize_deserialize() {
        let ev = DomainEvent::ArcIncrement {
            cycle_id: "c1".into(),
            actor_id: "u".into(),
            actor_type: ActorType::User,
            target_id: "lin_yueru".into(),
            target_type: TargetType::Npc,
            arc_dimension: "trust_user".into(),
            delta: 5.0,
            world_time: Some("景天元年 三月初一".into()),
            importance: 0.6,
            summary: "林月如对 user 信任 +5".into(),
        };
        let json = serde_json::to_string(&ev).unwrap();
        let back: DomainEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev.event_type(), back.event_type());
        assert_eq!(ev.cycle_id(), back.cycle_id());
    }

    #[test]
    fn all_variants_have_event_type() {
        // 给每个 variant 一个最小事件, 验证 event_type() 返回正确类型
        let events = vec![
            (
                DomainEvent::Setup {
                    cycle_id: "c".into(),
                    perspective_id: "p".into(),
                    gm_id: "g".into(),
                    templates_applied: vec![],
                },
                EventType::Setup,
            ),
            (
                DomainEvent::ArcIncrement {
                    cycle_id: "c".into(),
                    actor_id: "a".into(),
                    actor_type: ActorType::System,
                    target_id: "t".into(),
                    target_type: TargetType::Npc,
                    arc_dimension: "courage".into(),
                    delta: 1.0,
                    world_time: None,
                    importance: 0.5,
                    summary: "s".into(),
                },
                EventType::ArcIncrement,
            ),
        ];
        for (ev, et) in events {
            assert_eq!(ev.event_type(), et);
            assert!(ev.validate().is_ok());
        }
    }
}
