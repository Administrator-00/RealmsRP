# ADR-0005: gm_router 用独立 context builder + event filter 两层解耦而非单一函数

- 状态: Accepted
- 日期: 2026-07-13

## Context

VIBE_CODING.md M2.4 的 `dispatch_role_subagent` 签名将 6 步全部耦合在一个 async 函数里:

```rust
pub async fn dispatch_role_subagent(req: RoleDispatchRequest) -> Result<RoleDispatchResult> {
    // 1. 读 npc_base.persona
    // 2. 读 world.setting
    // 3. 过滤 visible_context
    // 4. 拼装 subagent context
    // 5. 调 LLM
    // 6. 解析输出
}
```

但 M2.4 阶段不接入 LLM (M2 stub), 且第 1-2 步需要 M4-M5 的数据层
(npc_base / world_loader) 还未落地; 第 5-6 步需要 LLM provider 集成 (M7).

如果强行用这个单一函数, 要么得大量 mock 外部依赖, 要么只能写空洞的桩测试.

## Options

**选项 A**: 直接在 `dispatch_role_subagent` 里写桩 (所有外部依赖都 hardcode 占位值).
- 优点: 与 VIBE 字面对齐
- 缺点: 测试弱 (测不了 context 隔离的真实逻辑);
  dispatch 函数膨胀 (6 个关注点堆在一起)

**选项 B**: 拆成两层纯函数 + 一个薄桩入口.
- `build_subagent_context(&RoleDispatchRequest) -> String` — 拼装 context
- `filter_events_for_role(&[DomainEvent], npc_id) -> Vec<&DomainEvent>` — 事件可见性过滤
- `dispatch_role_subagent(&RoleDispatchRequest) -> Result<RoleDispatchResult>` — 组合上述 + LLM 桩

- 优点: 核心逻辑 (context 隔离 / 事件过滤) 成为可独立单测的纯函数,
  与数据层 (M4-M5) 和 LLM (M7) 解耦; 后续 M4-M5 只需填充 `RoleDispatchRequest` 字段即可.
- 缺点: 多一个函数参数 (events 需从外部传入), 与 VIBE 的单函数签名有偏差

## Decision

采用 **选项 B**:
- `build_subagent_context` + `filter_events_for_role` 作为纯逻辑 (M2.4 核心产出)
- `dispatch_role_subagent` 作为薄桩入口 (验证参数, 调用 1+2, 返回 stub 结果)
- 14 单测全部围绕前两个纯函数

`filter_events_for_role` 的可视规则:
- NPC 是 actor 或 target → 可见
- Setup / TimeAdvance → 环境级公开 (所有 NPC 可见)
- 其他 → 不可见

后续 M2.5 (PublicPolicy) 在此基础上叠加第二层过滤 (`can_see_secrets` 等).

## Consequences

- (+) 14 个单测可验证 "林月如 context 不含李逍遥人设 / 戒律#6 无 orchestrator noise",
  无需任何外部依赖 (DB / LLM / 文件系统)
- (+) M2.5 的 PublicPolicy 可以写成 `filter_events_for_role` 的包装层, 不修改 M2.4 的代码
- (+) 后续接入 LLM 时, `dispatch_role_subagent` 的桩体直接替换为真实 LLM 调用
- (−) `RoleDispatchRequest` 持有 `visible_events: Vec<String>` (摘要文本) 而非 `Vec<DomainEvent>`;
  上游调用方需要先把筛选后的事件转成摘要再填入请求 — 这是故意的: subagent context 里放
  原始 Rust 结构体序列化不如直接放可读摘要, 且避免了 subagent context 被解读为"结构体泄露".
  (后续如需保留原始事件引用, 可以加 `ref_events: Vec<&DomainEvent>` 伴随字段.)

## Status

Accepted, 由 M2.4 实现落地. 后续 M4-M5 数据层上线后需验证 `RoleDispatchRequest`
的字段可由 `Cycle` + `world_loader` + `gm_loader` 组装.

Refs: VIBE_CODING.md M2.4 / 3.md §7 (5 重防护)