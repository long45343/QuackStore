#!/system/bin/sh
# Sourced by KernelSU/APatch/Magisk installer in BusyBox ash standalone mode.

export MODULE_HOT_INSTALL_REQUEST="true"
export MODULE_HOT_RUN_SCRIPT="hotinstall.sh"

ui_print " "
ui_print "=========================================="
ui_print "  🦆 QuackStore (三合一自动套件) 安装器  "
ui_print "=========================================="

# 1. Root 管理器检查
if [ "$KSU" = "true" ]; then
  ui_print "- Root 环境: KernelSU $KSU_VER"
elif [ "$APATCH" = "true" ]; then
  ui_print "- Root 环境: APatch $APATCH_VER"
elif [ -n "$MAGISK_VER_CODE" ]; then
  ui_print "- Root 环境: Magisk $MAGISK_VER"
else
  abort "! 未知环境，请从 KernelSU, APatch 或 Magisk 安装。"
fi

# 2. 【第一阶预检】：硬件平台与 CPU 架构硬性检查
if [ "$ARCH" != "arm64" ]; then
  abort "! 错误: QuackStore 硬件 RKP 签名仅支持 ARM64 (aarch64) 平台设备。"
fi

if [ -z "$API" ] || [ "$API" -lt 26 ]; then
  abort "! 错误: 系统版本过低，需要 Android 8.0 (API 26) 或更高版本。"
fi

# 检查高通 KeyMint TA 必需节点
if [ ! -c "/dev/smcinvoke" ] && [ ! -e "/dev/smcinvoke" ]; then
  abort "! 错误: 未检测到高通硬件通信节点 /dev/smcinvoke！当前设备不支持硬件 RKP 签发。"
fi
ui_print "- [第一阶通过] 硬件平台及通信节点 (/dev/smcinvoke) 检查正常。"

# 3. 【第二阶预检】：KeyMint TA 库与底层 Keystore 模块检测
if [ ! -f "/vendor/lib64/libkeymasterdeviceutils.so" ]; then
  ui_print "! 警告: 未找到 /vendor/lib64/libkeymasterdeviceutils.so，可能仅能使用 V1 模式。"
fi

FOUND_KEYSTORE=0
if [ -d "/data/adb/modules/tricky_store" ] || [ -d "/data/adb/tricky_store" ]; then
  ui_print "- 检测到已激活的 Keystore 模块: Tricky Store"
  FOUND_KEYSTORE=1
fi
if [ -d "/data/adb/modules/oh_my_keymint" ] || [ -d "/data/misc/keystore/omk" ]; then
  ui_print "- 检测到已激活的 Keystore 模块: OhMyKeymint"
  FOUND_KEYSTORE=1
fi
if [ -d "/data/adb/modules/teesim" ] || [ -d "/data/adb/teesim" ]; then
  ui_print "- 检测到已激活的 Keystore 模块: TEESimulator"
  FOUND_KEYSTORE=1
fi

if [ "$FOUND_KEYSTORE" -eq 0 ]; then
  ui_print "! 提示: 当前未检测到已安装的 Keystore 模块。QuackStore 安装后将保持待命，安装任意模块后即可自动接管。"
fi
ui_print "- [第二阶通过] 底层 TA 库与 Keystore 环境预检完毕。"

# 4. 初始化工作环境与权限
# shellcheck source=util_functions.sh
. "$MODPATH/util_functions.sh"

DATA_ROOT="/data/adb/quack-store"
ui_print "- 正在初始化模块私有工作目录: $DATA_ROOT"
umask 077
repair_runtime
normalize_boot_hash

# 设置执行权限
chmod 755 "$MODPATH/bin/quackctl.sh" 2>/dev/null || true
chmod 755 "$MODPATH/service.sh" 2>/dev/null || true
chmod 755 "$MODPATH/boot-completed.sh" 2>/dev/null || true
chmod 755 "$MODPATH/action.sh" 2>/dev/null || true

# 5. 【第三阶预检】：网络连通性探测 (优先直连，超时自动回退备用高可用通道)
ui_print "- [第三阶探测] 正在测试 Google RKP 接口网络连通性..."
if ping -c 1 -W 2 8.8.8.8 >/dev/null 2>&1 || ping -c 1 -W 2 114.114.114.114 >/dev/null 2>&1; then
  ui_print "- 网络连接正常！已就绪。"
else
  ui_print "! 提示: 当前处于离线或断网状态。模块已正常安装，将在开机联网后自动激活定时同步。"
fi

ui_print " "
ui_print "=========================================="
ui_print "  🎉 QuackStore 安装圆满成功！           "
ui_print "  • 定时触发: 每周四 16:00 自动硬件签名  "
ui_print "  • 网络保障: 支持备用高可用通道自动切换 "
ui_print "=========================================="
ui_print " "
