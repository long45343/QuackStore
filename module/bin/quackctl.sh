#!/system/bin/sh
set -eu
umask 077

# WebUI hosts run commands through the system shell; boot scripts already run in BusyBox
# ash standalone mode (KernelSU module guide). Use the manager's BusyBox either way.
if [ -z "${QUACK_STORE_BUSYBOX_REEXEC:-}" ]; then
  for busybox in /data/adb/ksu/bin/busybox /data/adb/ap/bin/busybox /data/adb/magisk/busybox; do
    if [ -x "$busybox" ]; then
      export QUACK_STORE_BUSYBOX_REEXEC=1
      export ASH_STANDALONE=1
      exec "$busybox" sh "$0" "$@"
    fi
  done
fi

SCRIPT_DIR="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
MODULE_ROOT="${QUACK_STORE_ROOT:-$(CDPATH='' cd -- "$SCRIPT_DIR/.." && pwd)}"
case "$MODULE_ROOT" in
  /data/adb/modules/*|/data/adb/modules_update/*)
    DEFAULT_DATA_ROOT="/data/adb/quack-store"
    ;;
  *)
    DEFAULT_DATA_ROOT="$MODULE_ROOT"
    ;;
esac
DATA_ROOT="${QUACK_STORE_DATA_ROOT:-$DEFAULT_DATA_ROOT}"
SEARCHED_CANDIDATES=""
# Callers that parse output (the WebUI, boot scripts) never attach a terminal.
WANTS_JSON=1
[ -t 1 ] && WANTS_JSON=0

# 原生拦截兼容: 如果请求的是 sync-now，直接执行 sync_job.sh 并返回合法的 JSON Envelope
if [ "${1:-}" = "sync-now" ]; then
  if sh "$MODULE_ROOT/sync_job.sh" >/dev/null 2>&1; then
    printf '{"ok":true,"api":1,"command":"sync-now","data":{"status":"success"},"error":null,"ts":%s}\n' "$(date +%s)"
    exit 0
  else
    printf '{"ok":false,"api":1,"command":"sync-now","data":null,"error":{"code":"sync_failed","message":"sync_job.sh execution failed"},"ts":%s}\n' "$(date +%s)"
    exit 1
  fi
fi

for candidate in \
  "$MODULE_ROOT/bin/quackd" \
  "$MODULE_ROOT/target/aarch64-linux-android/release/quackd" \
  "$MODULE_ROOT/target/aarch64-linux-android/debug/quackd" \
  "$MODULE_ROOT/target/release/quackd" \
  "$MODULE_ROOT/target/debug/quackd" \
  "$MODULE_ROOT"/target/*-linux-android/release/quackd \
  "$MODULE_ROOT"/target/*-linux-android/debug/quackd
do
  if [ -n "$SEARCHED_CANDIDATES" ]; then
    SEARCHED_CANDIDATES="$SEARCHED_CANDIDATES, "
  fi
  SEARCHED_CANDIDATES="${SEARCHED_CANDIDATES}${candidate}"

  if [ -f "$candidate" ] && [ ! -x "$candidate" ]; then
    chmod 700 "$candidate" 2>/dev/null || true
  fi

  if [ -x "$candidate" ]; then
    export QUACK_STORE_ROOT="$MODULE_ROOT"
    export QUACK_STORE_DATA_ROOT="$DATA_ROOT"
    if "$candidate" "$@"; then
      exit 0
    else
      status=$?
    fi
    if [ "$status" -eq 126 ] || [ "$status" -eq 127 ]; then
      continue
    fi

    exit "$status"
  fi
done

if [ "$WANTS_JSON" -eq 1 ]; then
  printf '%s\n' "{\"ok\":false,\"api\":1,\"command\":\"bootstrap.wrapper\",\"data\":null,\"error\":{\"code\":\"binary_not_found\",\"message\":\"QuackStore backend binary is missing or not executable.\",\"details\":{\"module_root\":\"$MODULE_ROOT\",\"searched\":\"$SEARCHED_CANDIDATES\"}}}"
else
  echo "QuackStore backend binary is missing or not executable under $MODULE_ROOT" >&2
  echo "searched: $SEARCHED_CANDIDATES" >&2
fi

exit 127
