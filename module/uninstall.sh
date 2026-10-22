#!/system/bin/sh
# Runs when root manager uninstalls QuackStore.

MODDIR=${0%/*}
MODULE_ID="quack-store"
DATA_ROOT="/data/adb/$MODULE_ID"

# 终止常驻守护进程
if [ -f "$DATA_ROOT/var/daemon.pid" ]; then
  PID=$(cat "$DATA_ROOT/var/daemon.pid" 2>/dev/null | tr -d ' \r\n')
  [ -n "$PID" ] && kill -9 "$PID" 2>/dev/null || true
fi

"$MODDIR/bin/quackctl.sh" tricky-store entry remove >/dev/null 2>&1 || true

rm -f /storage/emulated/0/stop-tspa-auto-target
rm -f /data/adb/disable_prop_handler /data/adb/boot_hash
rm -rf "$DATA_ROOT"
