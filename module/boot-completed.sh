#!/system/bin/sh
# Runs once boot has completed and `pm` works. Nothing here may block or fail the boot.

MODPATH=${0%/*}
QUACKCTL="$MODPATH/bin/quackctl.sh"
TSPA_DIR="/data/adb/modules/tsupport-advance"
TSPA_FLAG="/storage/emulated/0/stop-tspa-auto-target"

# Late pass for props the system sets after post-fs-data.
sh "$MODPATH/prop.sh"

"$QUACKCTL" tricky-store entry apply >/dev/null 2>&1 || true
"$QUACKCTL" tricky-store auto-apply >/dev/null 2>&1 || true
"$QUACKCTL" describe >/dev/null 2>&1 || true

# 启动 QuackStore 原生 Rust 调度守护进程 (quackd daemon)
if [ -f "/data/adb/quack-store/var/daemon.pid" ]; then
  OLD_PID=$(cat "/data/adb/quack-store/var/daemon.pid" 2>/dev/null | tr -d ' \r\n')
  if [ -n "$OLD_PID" ] && [ -d "/proc/$OLD_PID" ]; then
    kill -9 "$OLD_PID" 2>/dev/null || true
  fi
fi
nohup "$QUACKCTL" daemon >/dev/null 2>&1 &

# TSupport-Advance 兼容标志
tries=0
until [ -d /storage/emulated/0/Android ] || [ "$tries" -ge 90 ]; do
  sleep 2
  tries=$((tries + 1))
done
if [ -d "$TSPA_DIR" ]; then
  touch "$TSPA_FLAG" 2>/dev/null || true
else
  rm -f "$TSPA_FLAG"
fi
