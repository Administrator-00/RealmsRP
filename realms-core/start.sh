#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REALMS_DIR="$(dirname "$SCRIPT_DIR")"
AIRP_DIR="$REALMS_DIR/AIRP"
AIRP_MCP_DIR="$REALMS_DIR/AIRP-MCP-Server"
AIRP_SP_DIR="$REALMS_DIR/AIRP-State-Protocol"
TAVERN_DIR="$REALMS_DIR/tavern2agent"
WEBUI_DIR="$SCRIPT_DIR/webui"
ENGINE_PORT=8000
API_PORT=3000
WEBUI_PORT="${WEBUI_PORT:-5173}"

echo "=== Realms RP 平台 启动 ==="

# 0. 克隆上游依赖 (如果不存在)
clone_if_missing() {
  local dir="$1" url="$2" name="$3"
  if [ ! -d "$dir" ]; then
    echo "[0] 克隆上游依赖: $name..."
    git clone "$url" "$dir" 2>&1 | tail -1
  fi
}

clone_if_missing "$AIRP_DIR"      "https://github.com/GhostXia/AIRP.git"              "AIRP (engine)"
clone_if_missing "$AIRP_MCP_DIR"  "https://github.com/GhostXia/AIRP-MCP-Server.git"   "AIRP-MCP-Server"
clone_if_missing "$AIRP_SP_DIR"   "https://github.com/GhostXia/AIRP-State-Protocol.git" "AIRP-State-Protocol"
clone_if_missing "$TAVERN_DIR"    "https://github.com/Xerxes-2/tavern2agent.git"      "tavern2agent"

# 1. AIRP Engine (LLM 网关层)
echo "[1/3] AIRP Engine (port $ENGINE_PORT)..."
cd "$AIRP_DIR"
if [ ! -f target/debug/airp-core ]; then
  echo "  首次编译 AIRP Engine..."
  cargo build -p airp-core 2>&1 | tail -3
fi
nohup cargo run -p airp-core -- daemon --port "$ENGINE_PORT" > /tmp/realms-engine.log 2>&1 &
ENGINE_PID=$!

# 等待 engine 就绪
echo -n "  等待服务就绪"
for i in $(seq 1 20); do
  if curl -s http://localhost:$ENGINE_PORT/health > /dev/null 2>&1; then
    echo " OK"
    break
  fi
  sleep 1
  echo -n "."
done
echo "  OK (PID $ENGINE_PID)"

# 2. Realms API Server (胶水层)
echo "[2/3] Realms API Server (port $API_PORT)..."
cd "$SCRIPT_DIR"
if [ ! -f target/debug/realms-server ]; then
  echo "  首次编译 API Server..."
  cargo build --bin realms-server 2>&1 | tail -3
fi
nohup cargo run --bin realms-server > /tmp/realms-api.log 2>&1 &
API_PID=$!
sleep 3
echo "  OK (PID $API_PID)"

# 3. WebUI
echo "[3/3] WebUI (port $WEBUI_PORT)..."
cd "$WEBUI_DIR"
if [ ! -d node_modules ]; then
  echo "  首次安装依赖..."
  npm install --silent 2>&1 | tail -3
fi
nohup npx vite --port "$WEBUI_PORT" --host > /tmp/realms-webui.log 2>&1 &
WEBUI_PID=$!
sleep 2

echo ""
echo "=============================================="
echo "  Realms RP 平台已就绪!"
echo "  打开浏览器: http://localhost:$WEBUI_PORT"
echo ""
echo "  使用流程:"
echo "    → 设置页 填写 LLM API Key → 保存"
echo "    → 选世界 → 选角色 → 选 GM → 开周目 → 玩!"
echo ""
echo "  停止: kill $ENGINE_PID $API_PID $WEBUI_PID"
echo "=============================================="

trap "kill $ENGINE_PID $API_PID $WEBUI_PID 2>/dev/null; exit" INT TERM
wait
