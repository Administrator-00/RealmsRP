# ADR 0003: M1.3 手写 .pi/agents/*.md 而非 pi 嵌套编译

## Context

VIBE_CODING.md §5 M1.3 步骤:
> 对每个角色卡,启动 pi 让它编译
> 输出: realms-core/.pi/agents/{name}.md

tavern2agent 是 pi skill (SKILL.md 描述),不能独立命令行运行。它的"编译"流程是:
1. 用户提供 ST 角色卡
2. 启动 pi session
3. 告诉 pi: "转换这张卡"
4. pi 加载 tavern2agent skill,跑 scripts/extract_card.py → 形成 IR → 写 .pi/agents/*.md

但**我们当前正运行在 pi 中**。要"启动 pi 让它编译"意味着嵌套 pi 调用。

## Options

| 方案 | 优势 | 劣势 |
|---|---|---|
| **A. 手写 .pi/agents/*.md** | 直接、不嵌套、产物可控 | 不走 tavern2agent 标准流程,跳过 IR/Runtime Plan |
| B. 嵌套启动 pi 子进程 | 走完整 tavern2agent 流程 | 嵌套 session 复杂、需模型 API key、产物体积大(完整 pi project 非单个 .md) |
| C. 只跑 extract_card.py,不写 .pi/agents/ | 保留 ST 数据原样,运行时用 V2 JSON 直接 | 3.md §11.3 ⑬ 明确要求 .pi/agents/*.md 作为 subagent 模板 |

## Decision

**采用 A: 手写 .pi/agents/*.md**。

理由:
1. **产物对齐 3.md §11.3 ⑬**:该章节明确说"角色 subagent 隔离 = tavern2agent 模板 (.pi/agents/lin-yueru.md 等)" — 我们需要的是 subagent 模板,不是完整 pi project
2. **避免嵌套**: 我们是 pi, 启动 pi 是反模式
3. **可控性**: 手写模板可以精确对齐 3.md 的 5 重防护 (tools 白名单、输出格式、禁止写状态)
4. **M1 验收是"4 个模板可被加载"** — 手写模板更容易验证
5. **未来可升级**: M11 仙剑完整 demo 时可加完整 tavern2agent 流程,补充完整 pi project

## Consequences

- ✅ M1 4 个 subagent 模板已就位
- ✅ 模板含 frontmatter (name/description/tools/model) + 完整 system prompt
- ✅ tools 限定 read/grep/find/ls (只读,符合 5 重防护 #3)
- ⚠️ 未走 tavern2agent 完整 IR 流程,可能错过一些"卡审计"洞察 (如世界书/TH scripts/regex scripts)
- 📌 仙剑 4 NPC 性格数据已分两处存储:
  - `worlds/xiatian_qixia_1/npcs/*.md` (MCP-Server 数据源, 性格不可变)
  - `realms-core/.pi/agents/*.md` (pi subagent 模板)
  - 两处内容同源,**更新时需同步**

## Status

**Accepted** (2026-07-13, Session #2)

## 未来改进

- M11 时可考虑用 `tavern2agent/scripts/extract_card.py` 重新生成 IR,做交叉验证
- 加 `sync_check.sh` 脚本定期校验两处数据一致性
