# ADR-0004: prompt_assembler 用独立 PromptContext 而非 Cycle / GmPrompt 复合类型

- 状态: Accepted
- 日期: 2026-07-13

## Context

3.md §1.4 给出的 `build_system_prompt` 签名依赖 `Cycle` / `GmPrompt` 复合类型:

```rust
pub fn build_system_prompt(cycle: &Cycle, gm: &GmPrompt) -> String
```

但 `Cycle` (周目聚合: world + perspective + 状态 + 弧光 + 记忆) 要到 M4-M5 才定稿,
`GmPrompt` / `Gm` 也要等 GM 数据层 (M5) 落地. 在 M2.3 实现核心拼接逻辑时,
如果直接依赖这两个类型, 要么得先写一版占位 `Cycle` / `GmPrompt` (后面再加字段时破坏兼容),
要么得在 M2.3 一次性拉通 M4-M5 的数据模型 — 违反 VIBE_CODING §1.2 的顺序性原则.

## Options

### 选项 A: 直接用 3.md 的 `(cycle: &Cycle, gm: &GmPrompt)` 签名
- 优点: 与 3.md 字面对齐
- 缺点: 必须提前定 `Cycle` / `GmPrompt`, 与 M4-M5 顺序冲突

### 选项 B: 用独立 `PromptContext` (7 段 `String` 字段)
- 优点: prompt_assembler 与数据层解耦, 可在 M2.3 独立单测; M4-M5 再
  提供 `Cycle -> PromptContext` 适配器即可
- 缺点: 多一次中间状态拷贝 (但与 prompt 拼接本身 O(n) 字符串相比可忽略)

### 选项 C: 7 个独立参数 `build_system_prompt(persona, world, gm, state, arc_phase, arc, mem, rules)`
- 优点: 零类型依赖
- 缺点: 8 参函数难看, 易传错位, 扩展字段困难

## Decision

采用 **选项 B**: 引入 [`PromptContext`], 包含 7 个 `String` 字段
(perspective_persona / world_setting / gm_content / current_state /
arc_phase `Option<String>` / arc_context / relevant_memories / world_rules),
外加一个 `arc_phase` 备注. `build_system_prompt(&ctx) -> String`.

## Consequences

- (+) prompt_assembler 的 31 个单测在 M2.3 完全自洽, 无需 DB / world loader.
- (+) M4-M5 落地 `Cycle` 后, 只需在 cycle_manager 实现 `impl From<&Cycle> for PromptContext`
  (或一个 `assemble_prompt_context(&pool, &cycle) -> PromptContext` 适配函数),
  即可无缝接入, 不修改 prompt_assembler 任何代码.
- (+) `gm_content` 为空时填占位符的逻辑收口在 `gm_section` 私有函数,
  上层调用方无需关心.
- (−) `PromptContext` 是多一层间接, 编排器入口要手动组装 ctx.
- (−) `arc_phase` 被单列字段 (3.md §1.4 原本把它含在 `arc_context` 内),
  改为显式字段会让弧光阶段更醒目, 但与 3.md §1.4 字面略偏 —— 这点偏差是良性的,
  已在 prompt_assembler 模块文档记录, 不需要回改 3.md.

## Status

Accepted, 由 M2.3 实现落地. 后续若 M4-M5 发现 `PromptContext` 字段不足以表达状态投影,
再以 ADR-NNNN 增补字段 (而非删字段, 保持向后兼容).

Refs: 3.md §1.4 / VIBE_CODING.md M2.3