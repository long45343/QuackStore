#!/system/bin/sh
# ==============================================================================
# QuackStore: 每周四 16:00 定时调度守护进程
# ==============================================================================

MODDIR="${0%/*}"
VARDIR="/data/adb/quack-store"
CONFIG_FILE="$VARDIR/config.toml"
PID_FILE="$VARDIR/daemon.pid"
LAST_RUN_FILE="$VARDIR/last_run"
LOG_FILE="$VARDIR/sync.log"
SYNC_SCRIPT="$MODDIR/sync_job.sh"

mkdir -p "$VARDIR"
echo $$ > "$PID_FILE"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [daemon] $*" >> "$LOG_FILE"
}

log "QuackStore 定时调度守护进程启动 (PID: $$)"

CHECK_INTERVAL=300 # 常规检查间隔: 5 分钟
RETRY_DELAY=900    # 失败重试间隔: 15 分钟

while true; do
    CURRENT_DOW=$(date +%u)      # 1=Mon, 2=Tue, 3=Wed, 4=Thu, 5=Fri, 6=Sat, 7=Sun
    CURRENT_HOUR=$(date +%H)     # 00..23 (24小时制)
    TODAY_DATE=$(date +%Y-%m-%d) # 当前日期

    # 动态从 config.toml 读取用户设置 (默认每周四 16:00 启用)
    SCHED_ENABLED=true
    TARGET_DOW=4
    TARGET_HOUR=16

    if [ -f "$CONFIG_FILE" ]; then
        CFG_ENABLED=$(grep -E '^\s*enabled\s*=' "$CONFIG_FILE" 2>/dev/null | tail -n 1 | cut -d'=' -f2 | tr -d ' "')
        [ "$CFG_ENABLED" = "false" ] && SCHED_ENABLED=false

        CFG_DOW=$(grep -E '^\s*day_of_week\s*=' "$CONFIG_FILE" 2>/dev/null | tail -n 1 | cut -d'=' -f2 | tr -d ' "')
        [ -n "$CFG_DOW" ] && TARGET_DOW="$CFG_DOW"

        CFG_HOUR=$(grep -E '^\s*hour\s*=' "$CONFIG_FILE" 2>/dev/null | tail -n 1 | cut -d'=' -f2 | tr -d ' "')
        [ -n "$CFG_HOUR" ] && TARGET_HOUR="$CFG_HOUR"
    fi

    LAST_RUN=""
    if [ -f "$LAST_RUN_FILE" ]; then
        LAST_RUN=$(cat "$LAST_RUN_FILE" 2>/dev/null | tr -d ' \r\n')
    fi

    # 条件：调度开启、到达设定周几且到达设定时间
    if [ "$SCHED_ENABLED" = "true" ] && [ "$CURRENT_DOW" -eq "$TARGET_DOW" ] && [ "$CURRENT_HOUR" -ge "$TARGET_HOUR" ]; then
        if [ "$LAST_RUN" != "$TODAY_DATE" ]; then
            log "触发定时任务: 到达预定时间 (周${TARGET_DOW} ${TARGET_HOUR}:00，今日尚未同步)，开始执行..."
            
            sh "$SYNC_SCRIPT"
            EXIT_CODE=$?

            if [ "$EXIT_CODE" -eq 0 ]; then
                log "今日任务执行成功完成！等待下一周期触发。"
                sleep "$CHECK_INTERVAL"
            else
                log "任务执行失败 (退出码: $EXIT_CODE)，将在 15 分钟后重试..."
                sleep "$RETRY_DELAY"
            fi
            continue
        fi
    fi

    sleep "$CHECK_INTERVAL"
done
