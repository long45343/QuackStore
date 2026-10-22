#!/system/bin/sh
MODDIR="${0%/*}"

echo "=============================================="
echo "    🦆 QuackStore 快速操作与状态检测         "
echo "=============================================="
echo ""
echo "正在立即触发一次硬件 RKP 签名、2100年 RSA 注入与自适应同步..."
echo "请稍候..."
echo ""

sh "$MODDIR/sync_job.sh"
RET=$?

echo ""
if [ $RET -eq 0 ]; then
  echo ">>> [成功] RKP 硬件签名并同步成功！"
else
  echo ">>> [失败] 执行失败，退出码: $RET"
fi

echo ""
echo "---------- 调度服务最近日志 ----------"
tail -n 20 /data/adb/quack-store/sync.log 2>/dev/null || echo "(暂无日志)"
echo "--------------------------------------"
