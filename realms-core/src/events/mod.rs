//! 领域事件模块
//!
//! DomainEvent 是 Realms 事件溯源体系的核心类型.
//! 详见 3.md §2.2 domain_events 表.

pub mod domain_event;

pub use domain_event::{ActorType, DomainEvent, EventType, TargetType};
