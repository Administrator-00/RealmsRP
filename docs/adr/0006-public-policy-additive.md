# ADR-0006: PublicPolicy 叠加式过滤 (扩展而非替换 M2.4 基础规则)

- 状态: Accepted
- 日期: 2026-07-13

## Context

VIBE_CODING.md M2.5 的 hiddenPublicPolicy 定义比较模糊:

```rust
pub struct PublicPolicy {
    pub can_see_secrets: bool,
    pub can_see_other_intent: bool,
    pub can_see_dm_internal: bool,
}
pub fn filter_events_for_role(events: &[DomainEvent], policy: &PublicPolicy) -> Vec<DomainEvent>
```

但 M2.4 已经实现了 `filter_events_for_role(events, npc_id)` 基于 actor/target 的基础过滤。
M2.5 如果替换 M2.4 的函数, 默认三 false 会导致 NPC 连自己的事件也看不到 (过于严格).

## Options

**选项 A**: 替换式 — M2.5 的 `filter_events_for_role` 完全替代 M2.4, 默认三 false → 看不到任何事件.
- 缺点: 需要调用方手动聚合 npc_id + policy 信息, 破坏 M2.4 的 simplicity

**选项 B**: 叠加式 — M2.4 的基础规则 (actor/target/environment)  不变, M2.5 在「非基础可见」
的事件上打开额外通道.
- 优点: M2.4 行为不变 (无回归); policy 是 "放行额外事件" 的许可, 不是 "限制已有可见"
- 缺点: policy 不能用来「隐藏 NPC 不该看到的环境级事件」 (如 Setup 始终可见)

**选项 C**: 两层分离 — `filter_events_for_role(events, npc_id)` 做基础过滤,
  `apply_policy_filter(result, policy)` 做二次过滤.
- 优点: 两个函数正交, 可组合
- 缺点: 调用方需传两次 filter, 略繁琐

## Decision

采用 **选项 B** (叠加式), 并在函数命名上显式区分:

- `filter_events_for_role(events, npc_id)` — M2.4 基础 (返回 actor/target/environment 事件)
- `filter_events_with_policy(events, npc_id, policy)` — M2.5 叠加 (基础 + policy 放行)

叠加逻辑:
1. 基础可见 (M2.4) → 直接通过 (不受 policy 限制)
2. 非基础可见 + can_see_dm_internal & event.is_system_actor() → 放行
3. 非基础可见 + can_see_secrets & event.importance() > 0.8 → 放行
4. 非基础可见 + can_see_other_intent & event is Dialogue → 放行
5. 其余 → 不通过

环境级事件 (Setup/TimeAdvance) 在步骤 1 被 M2.4 放行, 不受 policy 影响.
M2.4 中已对 actor/target 做了最严格限制, 在此之后 policy 只做加法.

## Consequences

- (+) 9 个 M2.5 测试全部在 `filter_events_with_policy` 上, 不影响 14 个 M2.4 测试
- (+) `filter_events_for_role` 可独立使用 (轻量级, 无 policy 概念的场景)
- (−) NPC 始终能看到环境级事件 (Setup/TimeAdvance) — 这符合设计预期:
  这些是 cycle 的公共事实, 不应被隐藏; 真正的 "秘密信息" 应在事件级别标记
  (未来通过向 DomainEvent 加 `is_secret: bool` 字段实现更精准的秘密控制)

## Status

Accepted. 若后续发现需要「隐藏环境级事件」的场景, 可通过 ADR-NNNN 将
Setup/TimeAdvance 也纳入 policy 控制范围 (目前不需要).

Refs: 3.md §7 (5 重防护, hiddenPublicPolicy) / VIBE_CODING.md M2.5