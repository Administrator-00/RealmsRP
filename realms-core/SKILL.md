# Realms RP 平台 Skill Guide

> 下次 session 自动加载。**硬性约束 + 常用命令 + 数据/端口分配。**

## 必读

- `/home/jhwh/src/Realms/3.md` (40KB) - 最终融合产品设计 (**权威**)
- `/home/jhwh/src/Realms/README.md` (13KB) - 项目入口
- `/home/jhwh/src/Realms/VIBE_CODING.md` (执行手册)
- `/home/jhwh/src/Realms/STATUS.md` (进度跟踪)
- `/home/jhwh/src/Realms/1.md` 2.md (按需查,3.md 优先)

## 硬性约束

1. **不修改** `../AIRP/` `../AIRP-MCP-Server/` `../AIRP-State-Protocol/` `../tavern2agent/`
2. 只在 `realms-core/` 写代码
3. 戒律#6 不可破坏(`subagent_context_has_no_orchestrator_noise`)
4. 不引入新重型依赖(只用: serde/tokio/rusqlite/thiserror/tracing/chrono/async-trait)
5. 每个 `pub fn` 配 doc comment + ≥1 测试
6. 错误用 `Result<T, RealmsError>`, 不用 `panic` / `unwrap` / `unsafe`
7. 单文件 < 500 行,`pub fn` < 50 行(超出就拆)
8. 跨平台路径用 `Path`/`PathBuf`,不用 `&str` 拼路径

## 目录布局

```
/home/jhwh/src/Realms/
├── AIRP/                       # 上游: engine + webui + protocol
├── AIRP-MCP-Server/            # 上游: RP 数据后端 (38 工具)
├── AIRP-State-Protocol/        # 上游: widget 协议
├── tavern2agent/               # 上游: subagent 模板
├── realms-core/                # ★ 本仓 (胶水层, ~6000 行)
│   ├── engine/{orchestrator,reducers,events,cycle}
│   ├── db/{migrations,queries}
│   ├── webui/{widgets,components,pages,utils}
│   ├── worlds/                 # 仙剑示例
│   ├── gms/                    # 全局 GM 模板
│   └── perspectives/           # 用户视角
├── docs/adr/                   # 架构决策记录
├── 1.md 2.md 3.md README.md VIBE_CODING.md
└── STATUS.md
```

## 端口分配

| 服务 | 端口 | 备注 |
|---|---|---|
| AIRP engine | 8000 | `cd AIRP && ./target/debug/airp-core daemon --port 8000` |
| MCP-Server | 3001 | `./target/debug/airp-mcp serve --bind 127.0.0.1:3001 --data-dir ./data` |
| AIRP webui | **9001** | `node serve.js` (实际 9001,VIBE_CODING 写 5173 错误) |
| Tauri 桌面 (可选) | 1420 | |
| realms-core 自有服务 (后期) | 8001 | |

## 数据目录

| 数据 | 路径 | 格式 |
|---|---|---|
| SQLite 长期事实库 | `realms-core/data/memory.db` | WAL + FTS5 + JSON1 |
| World/Perspective/GM | `realms-core/{worlds,perspectives,gms}/` | 文件系统 |
| 短期 chat | AIRP-MCP-Server `data/chat.jsonl` | JSONL |
| 中期 volume | AIRP-MCP-Server `data/volume/*.md` | Markdown |
| 仙剑示例 | `realms-core/worlds/xiatian_qixia_1/` | |

**重要**:`realms-core/data/` 跟 `AIRP-MCP-Server/data/` 错开,避免污染。

## 常用命令

```bash
# 测试
cd realms-core && cargo test
cd realms-core && cargo test test_name -- --nocapture
cd realms-core && cargo clippy --all-targets -- -D warnings
cd realms-core && cargo fmt

# 启动 AIRP engine
cd AIRP && cargo build -p airp-core && ./target/debug/airp-core daemon --port 8000

# 启动 MCP-Server
cd AIRP-MCP-Server && cargo build -p airp-mcp-server && ./target/debug/airp-mcp serve --bind 127.0.0.1:3001 --data-dir ./data

# 启动 webui
cd AIRP/webui && node serve.js

# 验证
curl -s http://localhost:8000/health
curl -s http://localhost:3001/health
```

## 实施路径

M0 → M1 → M2 → M3 → M4 → M5 → M6 → M7 → M8 → M9 → M10 → M11

**完整 MVP 预计 18-24 天 (单人)**。详见 VIBE_CODING.md §5。

## 决策树速查

| 情况 | 动作 |
|---|---|
| cargo build 失败 | cargo clean → cargo update → 看错误 |
| cargo test 失败 | cargo test test_name --nocapture → 找 root cause |
| 上游编译失败 | 不修,在 realms-core 加 wrapper |
| 卡住 30 分钟 | 重读 3.md 相关章节 → 问用户 |
| 想引入新依赖 | 先评估能否用现有实现,必要时报用户 |
| 想改上游 | **禁止**,在 realms-core 加 wrapper |
| 不知道下一步 | 看 STATUS.md "下次 session 起点" |
