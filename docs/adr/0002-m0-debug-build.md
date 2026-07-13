# ADR 0002: M0 使用 debug build 而非 release build

## Context

VIBE_CODING.md §3.1 / M0.1 指定 `cargo build --release -p airp-core`。
但 release 编译耗时是 debug 的 3-5 倍(预计 5-10 分钟 vs 35 秒),M0 阶段目标是"验证基础环境连通",不要求生产性能。

## Decision

**M0 阶段用 debug build**(`cargo build -p airp-core`,不指定 `--release`)。

**M11 完成或性能成为瓶颈时再补 release build**。

## Consequences

- ✅ M0 编译从 ~5-10 分钟降到 35-60 秒
- ✅ 端到端连通测试不受影响
- ⚠️ 性能略低(开发阶段可接受)
- 📌 仙剑示例数据 + 端到端连通都已验证

## Status

**Accepted** (2026-07-13, Session #2)
