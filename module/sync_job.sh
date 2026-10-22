#!/system/bin/sh
# ==============================================================================
# QuackStore: 动态自适应多后端 + 2100年 AOSP RSA 占位兼容 + 备用高可用通道回退
# ==============================================================================

MODDIR="${0%/*}"
VARDIR="/data/adb/quack-store"
LOG_FILE="$VARDIR/sync.log"
LAST_RUN_FILE="$VARDIR/last_run"
TEMP_KEYBOX="$VARDIR/temp_keybox.xml"
QUACKCTL="$MODDIR/bin/quackctl.sh"
[ -x "$QUACKCTL" ] || QUACKCTL="/data/adb/modules/quack-store/bin/quackctl.sh"
RSA_BLOCK_FILE="$MODDIR/aosp_rsa_block.xml"

mkdir -p "$VARDIR"
touch "$LOG_FILE"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

log "========================================================"
log "开始执行 RKP 签发与自动同步任务..."

# 1. 检查基础环境
if [ ! -x "$QUACKCTL" ]; then
    log "错误: 未找到 QuackStore 控制脚本: $QUACKCTL"
    exit 1
fi

# 2. 检查设备端 RKP 硬件前置条件
log "检查硬件 TA 与 RKP 状态..."
STATUS_RES=$("$QUACKCTL" rkp-device status 2>&1)
if ! echo "$STATUS_RES" | grep -q '"prerequisites_met":true'; then
    log "错误: 设备前置条件未满足: $STATUS_RES"
    exit 1
fi

# 3. 尝试签名签发并导出到临时文件 (优先直连，失败无缝回退备用通道)
log "正在调用硬件 TA 签名并向 Google 签发 RKP 证书 (protocol=auto)..."
rm -f "$TEMP_KEYBOX"

PROVISION_RES=$("$QUACKCTL" rkp-device provision --acknowledge-limitations --protocol auto --output "$TEMP_KEYBOX" 2>&1)
PROVISION_STATUS=$?

if [ "$PROVISION_STATUS" -ne 0 ] || [ ! -s "$TEMP_KEYBOX" ]; then
    log "提示: 直连未响应，正在通过备用高可用通道重试签发..."
    PROVISION_RES=$("$QUACKCTL" rkp-device provision --acknowledge-limitations --protocol auto --server-url "https://deviceapi.micloud.xiaomi.net/proxyGoogle/v1" --output "$TEMP_KEYBOX" 2>&1)
    PROVISION_STATUS=$?
fi

if [ "$PROVISION_STATUS" -ne 0 ] || [ ! -s "$TEMP_KEYBOX" ]; then
    log "错误: RKP 签发未成功 (exit code: $PROVISION_STATUS)"
    log "签发详情: $PROVISION_RES"
    rm -f "$TEMP_KEYBOX"
    exit 2
fi

# 4. 校验临时文件内容格式
if ! grep -q "<Keybox" "$TEMP_KEYBOX" || ! grep -q "</Keybox>" "$TEMP_KEYBOX"; then
    log "错误: 生成文件非合法 Keybox XML 结构"
    rm -f "$TEMP_KEYBOX"
    exit 3
fi

KEYBOX_SIZE=$(wc -c < "$TEMP_KEYBOX" | tr -d ' ')
log "RKP Keybox 签发成功！原始大小: ${KEYBOX_SIZE} 字节"

# 4.1 核心兼容处理: 检测是否缺少 RSA 算法节点 (针对尚未发布 b12557c 的正式版 OMK)
if ! grep -q 'algorithm="rsa"' "$TEMP_KEYBOX"; then
    log "检测到 Keybox 仅含 ECDSA，缺少 RSA 节点。准备补齐 AOSP RSA 占位块..."
    if [ -f "$RSA_BLOCK_FILE" ]; then
        MERGED_KEYBOX="${TEMP_KEYBOX}.merged"
        awk -v rfile="$RSA_BLOCK_FILE" '
            /<\/Keybox>/ {
                while ((getline l < rfile) > 0) print l
                close(rfile)
            }
            { print }
        ' "$TEMP_KEYBOX" > "$MERGED_KEYBOX"

        if grep -q 'algorithm="rsa"' "$MERGED_KEYBOX" && grep -q 'algorithm="ecdsa"' "$MERGED_KEYBOX"; then
            mv -f "$MERGED_KEYBOX" "$TEMP_KEYBOX"
            NEW_SIZE=$(wc -c < "$TEMP_KEYBOX" | tr -d ' ')
            log "已成功注入 AOSP RSA 占位块！合成 Keybox 大小: ${NEW_SIZE} 字节 (双算法齐全)"
        else
            rm -f "$MERGED_KEYBOX"
            log "警告: RSA 占位块注入失败，保持原纯 ECDSA 继续"
        fi
    else
        log "警告: 未找到 $RSA_BLOCK_FILE，跳过 RSA 注入"
    fi
fi

# 5. 自动安装到当前激活的 Keystore 模块 (Tricky Store / OhMyKeymint / TEESimulator)
log "正在同步安装到当前活跃的 Keystore 模块..."
INSTALL_RES=$("$QUACKCTL" tricky-store keybox install "$TEMP_KEYBOX" 2>&1)
INSTALL_STATUS=$?

TARGET_PATH=""
BACKEND_NAME=""

if [ "$INSTALL_STATUS" -eq 0 ]; then
    # 从返回的 JSON 中解析 target_path 和 backend
    TARGET_PATH=$(echo "$INSTALL_RES" | grep -o '"target_path":"[^"]*"' | head -n 1 | cut -d'"' -f4)
    BACKEND_NAME=$(echo "$INSTALL_RES" | grep -o '"backend":"[^"]*"' | head -n 1 | cut -d'"' -f4)
    log "QuackStore 安装完成，识别后端: [${BACKEND_NAME:-未知}]"
fi

# 6. 验证或备用兜底同步
if [ -n "$TARGET_PATH" ] && [ -s "$TARGET_PATH" ]; then
    chmod 644 "$TARGET_PATH" 2>/dev/null
    FINAL_SIZE=$(wc -c < "$TARGET_PATH" | tr -d ' ')
    log "成功确认目标文件: $TARGET_PATH (${FINAL_SIZE} 字节)"
else
    log "警告: 未能从 install 响应确认路径，尝试检测已有目录进行兜底..."
    FALLBACK_DONE=0
    # 依次探测 OhMyKeymint / Tricky Store / TEESimulator
    for CANDIDATE in "/data/misc/keystore/omk/keybox.xml" "/data/adb/tricky_store/keybox.xml" "/data/adb/teesim/keybox.xml"; do
        CANDIDATE_DIR=$(dirname "$CANDIDATE")
        if [ -d "$CANDIDATE_DIR" ]; then
            [ -f "$CANDIDATE" ] && cp -f "$CANDIDATE" "${CANDIDATE}.bak"
            cp -f "$TEMP_KEYBOX" "$CANDIDATE"
            chmod 644 "$CANDIDATE" 2>/dev/null
            log "已兜底同步写入至: $CANDIDATE"
            FALLBACK_DONE=1
            break
        fi
    done
    if [ "$FALLBACK_DONE" -eq 0 ]; then
        log "错误: 未检测到任何已知的 Keystore 模块存储目录！"
        log "========================================================"
        exit 4
    fi
fi

# 7. 清理并记录完成状态
rm -f "$TEMP_KEYBOX"
date '+%Y-%m-%d' > "$LAST_RUN_FILE"
log "任务圆满完成！已更新上次运行记录。"
log "========================================================"
exit 0
