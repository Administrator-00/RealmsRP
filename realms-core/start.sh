#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WEBUI_DIR="$SCRIPT_DIR/webui"
API_PORT="${PORT:-3000}"
WEBUI_PORT="${WEBUI_PORT:-5173}"

echo "=== Realms RP 平台 启动 ==="

# 1. Realms API Server
echo "[1/2] Realms API Server (port $API_PORT)..."
cd "$SCRIPT_DIR"
if [ ! -f target/debug/realms-server ]; then
  echo "  首次编译 API Server..."
  cargo build --bin realms-server 2>&1 | tail -3
fi
nohup cargo run --bin realms-server > /tmp/realms-api.log 2>&1 &
API_PID=$!
sleep 3

# 等待 API server 就绪
echo -n "  等待服务就绪..."
for i in $(seq 1 15); do
  if curl -s http://localhost:$API_PORT/api/health > /dev/null 2>&1; then
    echo " OK"
    break
  fi
  sleep 1
  echo -n "."
done
echo "  OK (PID $API_PID)"

# 2. WebUI
echo "[2/2] WebUI (port $WEBUI_PORT)..."
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
echo "  停止: kill $API_PID $WEBUI_PID"
echo "=============================================="

trap "kill $API_PID $WEBUI_PID 2>/dev/null; exit" INT TERM
wait
