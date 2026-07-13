# Realms 项目状态

> 最后更新: 2026-07-13, Session #2 (M0 完成)

## 当前阶段

**M0 完成** ✅ 基础环境已验证,准备进入 M1

## 已完成

- [x] Session #1 (2026-07-13)
  - [x] 架构设计 (1.md / 2.md / 3.md / README.md / VIBE_CODING.md)
  - [x] 技术栈选型 (5 复用 + 1 胶水层)
  - [x] 工具链验证 (rustc 1.89 / node 26 / sqlite 3.53 / git 2.55)
  - [x] 4 个上游项目克隆 (gh-proxy.com 镜像)
  - [x] realms-core 目录骨架
  - [x] SKILL.md + .gitignore
- [x] Session #2 (2026-07-13) **— M0 全部完成**
  - [x] M0.1 AIRP engine 编译 (35.91s debug) + daemon 启动 (PID 16807, :8000)
  - [x] M0.2 MCP-Server 编译 (19.01s debug) + serve 启动 (PID 20644, :3001)
  - [x] M0.3 webui 启动 (PID 16989, **:9001** — VIBE_CODING 写错)
  - [x] M0.4 端到端连通测试通过
    - engine /health → ok
    - engine /v1/chat/completions → 200 SSE (无 LLM key 时空流,正常)
    - MCP /health → "AIRP MCP Server"
    - MCP /mcp/v1 initialize → 成功 (返回 serverInfo + 38 tools 描述)
  - [x] M0.5 仙剑示例数据预置 (7 个 .md 文件)
    - worlds/xiatian_qixia_1/setting.md (1864 bytes)
    - worlds/xiatian_qixia_1/npcs/{li_xiaoyao,lin_yueru,zhao_linger,npc_innkeeper}.md
    - worlds/xiatian_qixia_1/gms/style_linyueru.md (仙剑专属)
    - gms/default.md (跨世界通用)
  - [x] ADR 0001: webui 实际端口 9001
  - [x] ADR 0002: M0 用 debug build
  - [x] SKILL.md 端口表修正

## 进行中

- 无 (M0 收尾,待 M1 启动)

## 待开始

- M1: 角色卡编译 (tavern2agent)
  - M1.1 安装 tavern2agent skill
  - M1.2 准备 4 个 ST 格式角色卡 (V2 JSON)
  - M1.3 编译角色卡 → .pi/agents/*.md
  - M1.4 验证模板可加载
- M2: DM 编排器 (11 个子任务,~3 天)
- ... (M3-M11)

## 决策记录

- 见 `docs/adr/`
- **ADR-0001**: webui 实际端口 9001 (VIBE_CODING 5173 错)
- **ADR-0002**: M0 用 debug build
- **ADR-0003** (隐式): 网络不可达 GitHub,使用 gh-proxy.com 镜像 (Session #1)
- **ADR-0004** (隐式): 1.md / 2.md 不在"必读"之列,仅按需查 (3.md 是唯一权威)

## 已知问题

- 无 API key 配 LLM provider — 这是用户责任,需要时给
  - Settings: `POST /v1/settings` 可热重载,见 VIBE_CODING.md §3.1

## 当前运行中的服务

| 服务 | PID | 端口 | 状态 |
|---|---|---|---|
| AIRP engine (debug) | 16807 | 8000 | ✅ |
| MCP-Server (debug) | 20644 | 3001 | ✅ |
| AIRP webui | 16989 | 9001 | ✅ |

## 下次 session 起点

**M1.1**: 安装 tavern2agent skill (读 `tavern2agent/README.md` 找安装方式)
