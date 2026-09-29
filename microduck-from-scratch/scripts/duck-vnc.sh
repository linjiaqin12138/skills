#!/usr/bin/env bash
# duck-vnc.sh — 容器内一键起"浏览器远程桌面 + MuJoCo 原生 viewer"。
#
# 链路：Xvfb（虚拟显示器 :99）→ x11vnc（把 :99 导出成 VNC :5900）
#       → websockify（VNC over WebSocket + noVNC 网页 :6080）
#       → duck_body.py --viewer（GLFW 渲染进 :99，llvmpipe 软件 GL）
# 宿主机浏览器打开 http://localhost:6080/vnc.html 即可看到并操作仿真。
#
# 用法：bash scripts/duck-vnc.sh [--port 7801 ...]（参数原样透传给 duck_body.py）
# 退出（Ctrl+C 或 viewer 关窗）时 trap 清掉整串子进程。
set -u

DISPLAY_NUM=99
SCREEN=1280x800x24        # 【代理假设】够看清 viewer 双面板，noVNC 自适应缩放
VNC_PORT=5900
WEB_PORT=6080
LOG_DIR=/tmp/duck-vnc
mkdir -p "$LOG_DIR"

PIDS=()
cleanup() {
    [ ${#PIDS[@]} -gt 0 ] && kill "${PIDS[@]}" 2>/dev/null
    wait 2>/dev/null
}
trap cleanup EXIT INT TERM

# 每一环启动后等一小会儿确认进程还活着，死了就吐日志报出是哪一环挂的。
start() {  # start <名字> <日志> <命令...>
    local name=$1 log=$2; shift 2
    "$@" >"$LOG_DIR/$log" 2>&1 &
    local pid=$!
    PIDS+=($pid)
    sleep 1
    if ! kill -0 "$pid" 2>/dev/null; then
        echo "duck-vnc: $name 启动失败，日志：" >&2
        tail -20 "$LOG_DIR/$log" >&2
        exit 1
    fi
    echo "duck-vnc: $name 已启动 (pid $pid，日志 $LOG_DIR/$log)"
}

# 屏幕上有旧 Xvfb 锁文件时 Xvfb 会拒起；先清掉（本脚本独占 :99）。
rm -f "/tmp/.X${DISPLAY_NUM}-lock" "/tmp/.X11-unix/X${DISPLAY_NUM}"

start Xvfb xvfb.log Xvfb ":${DISPLAY_NUM}" -screen 0 "$SCREEN"
export DISPLAY=":${DISPLAY_NUM}"
# 软件渲染走 llvmpipe，不碰容器里不存在的 GPU。
export LIBGL_ALWAYS_SOFTWARE=1

start x11vnc x11vnc.log x11vnc -display "$DISPLAY" -forever -nopw -shared \
    -rfbport "$VNC_PORT"
start websockify websockify.log websockify --web /usr/share/novnc \
    "$WEB_PORT" "localhost:$VNC_PORT"

echo "duck-vnc: 浏览器打开 http://localhost:${WEB_PORT}/vnc.html"
echo "duck-vnc: 物理节拍仍由 daemon 驱动（miniduckd --sim 127.0.0.1:<port>）"

# sim 跑前台：stderr 直接可见，它退出（关 viewer 窗）时 trap 把整串一起收。
# 不能 exec：exec 会替换掉 shell，EXIT trap 就不生效了。
python3 /work/sim/duck_body.py --viewer "$@"
