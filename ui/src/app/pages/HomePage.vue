<script setup lang="ts">
import { useMutation, useQuery } from "@tanstack/vue-query"
import { ShieldCheck, Zap, Settings2, ArrowUpCircle } from "@lucide/vue"
import { computed, ref, onMounted, onUnmounted, nextTick, watch } from "vue"
import { useSessionStorage } from "@vueuse/core"

import { Button } from "@/components/ui/button"
import { Spinner } from "@/components/ui/spinner"
import { Badge } from "@/components/ui/badge"
import { Switch } from "@/components/ui/switch"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import SectionCard from "@/core/components/SectionCard.vue"
import QueryState from "@/core/components/QueryState.vue"
import { notifySuccess, notifyError } from "@/core/notify"
import { bridge } from "@/core/bridge"

import TargetsPage from "@/features/tricky-store/pages/TargetsPage.vue"
import { rkpDeviceApi } from "@/features/rkp-device/api"

// 1. 硬件 TA 状态查询
const rkpStatus = useQuery({
  queryKey: ["rkp-device.status"],
  queryFn: rkpDeviceApi.status,
  staleTime: 10000,
})

// 2. 调度执行与即时同步状态
const syncing = ref(false)
const syncLogLines = ref<string[]>([])
const logContainer = ref<HTMLElement | null>(null)
const userScrolled = ref(false)
let logPollTimer: ReturnType<typeof setInterval> | null = null

// 3. 用户自定义调度设置 (从 config.toml 读取和保存)
const schedEnabled = ref(true)
const schedDow = ref(4) // 1..7 (4 = 周四)
const schedHour = ref(16) // 0..23 (16 = 16:00)
const savingConfig = ref(false)

// 弹窗状态与临时编辑值
const showSettingsDialog = ref(false)
const tempDow = ref(4)
const tempHour = ref(16)

const dowNames = [
  { val: 1, label: "每周一" },
  { val: 2, label: "每周二" },
  { val: 3, label: "每周三" },
  { val: 4, label: "每周四" },
  { val: 5, label: "每周五" },
  { val: 6, label: "每周六" },
  { val: 7, label: "每周日" },
]

function openSettingsDialog() {
  tempDow.value = schedDow.value
  tempHour.value = schedHour.value
  showSettingsDialog.value = true
}

function cancelDialog() {
  showSettingsDialog.value = false
}

async function saveDialog() {
  schedDow.value = tempDow.value
  schedHour.value = tempHour.value
  await saveConfig(true)
  showSettingsDialog.value = false
}

// 复合高效读取：将 config 读取和 log 读取合并为单次 shell 查询，减少子进程开销
async function initData() {
  try {
    const res = await bridge().exec(
      "cat /data/adb/quack-store/config.toml 2>/dev/null; echo '===QUACK_SPLIT==='; tail -n 60 /data/adb/quack-store/sync.log 2>/dev/null"
    )
    if (res && res.stdout) {
      const parts = res.stdout.split("===QUACK_SPLIT===")
      const tomlText = parts[0] || ""
      const logText = parts[1] || ""

      const enabledMatch = tomlText.match(/enabled\s*=\s*(true|false)/i)
      if (enabledMatch && enabledMatch[1]) schedEnabled.value = enabledMatch[1].toLowerCase() === "true"

      const dowMatch = tomlText.match(/day_of_week\s*=\s*(\d+)/)
      if (dowMatch && dowMatch[1]) schedDow.value = parseInt(dowMatch[1], 10)

      const hourMatch = tomlText.match(/hour\s*=\s*(\d+)/)
      if (hourMatch && hourMatch[1]) schedHour.value = parseInt(hourMatch[1], 10)

      const lines = logText
        .trim()
        .split("\n")
        .filter((l) => l.trim().length > 0)
      if (lines.length > 0) {
        syncLogLines.value = lines.reverse()
      } else {
        syncLogLines.value = ["[暂无调度日志] 守护进程等待下一次周期触发..."]
      }
    }
  } catch {
    // ignore
  }
}

async function saveConfig(notify = true) {
  savingConfig.value = true
  try {
    const tomlContent = `[scheduler]
enabled = ${schedEnabled.value}
day_of_week = ${schedDow.value}
hour = ${schedHour.value}

[compat]
inject_legacy_rsa = true

[network]
server_strategy = "auto"
`
    const b64 = btoa(tomlContent)
    await bridge().exec(`mkdir -p /data/adb/quack-store && echo "${b64}" | base64 -d > /data/adb/quack-store/config.toml`)
    if (notify) {
      const item = dowNames.find((d) => d.val === schedDow.value)
      const label = item ? item.label : `每周${schedDow.value}`
      notifySuccess(`定时时间已修改为: ${label} ${schedHour.value.toString().padStart(2, "0")}:00`)
    }
  } catch (err: any) {
    notifyError(err)
  } finally {
    savingConfig.value = false
  }
}

async function onToggleScheduler(val: boolean) {
  schedEnabled.value = val
  if (!val) {
    showSettingsDialog.value = false
  }
  await saveConfig(false)
  if (val) {
    notifySuccess("周期自动化同步已开启")
  } else {
    notifySuccess("周期自动化同步已暂停")
  }
}

// 监听日志容器滚动
function onLogScroll(e: Event) {
  const el = e.target as HTMLElement
  if (!el) return
  if (el.scrollTop > 15) {
    userScrolled.value = true
  } else {
    userScrolled.value = false
  }
}

function scrollToLatest() {
  if (logContainer.value) {
    logContainer.value.scrollTo({ top: 0, behavior: "smooth" })
    userScrolled.value = false
  }
}

async function refreshLog() {
  try {
    const res = await bridge().exec("tail -n 60 /data/adb/quack-store/sync.log 2>/dev/null")
    if (res && res.stdout && res.stdout.trim()) {
      const lines = res.stdout
        .trim()
        .split("\n")
        .filter((l) => l.trim().length > 0)
      syncLogLines.value = lines.reverse()

      if (!userScrolled.value && logContainer.value) {
        await nextTick()
        logContainer.value.scrollTop = 0
      }
    }
  } catch {
    // ignore
  }
}

onMounted(() => {
  void initData()
  updateContainerWidth()
  window.addEventListener("resize", updateContainerWidth)
})

onUnmounted(() => {
  if (logPollTimer) clearInterval(logPollTimer)
  window.removeEventListener("resize", updateContainerWidth)
})

// 4. 立即触发硬件签发并同步
const runSyncMutation = useMutation({
  mutationFn: async () => {
    syncing.value = true
    userScrolled.value = false
    if (logPollTimer) clearInterval(logPollTimer)
    logPollTimer = setInterval(() => {
      void refreshLog()
    }, 1000)

    const res = await bridge().exec("sh /data/adb/modules/quack-store/sync_job.sh 2>&1")
    if (res.errno !== 0) {
      throw new Error(res.stdout || res.stderr || `执行失败 (代码 ${res.errno})`)
    }
    return res.stdout
  },
  onSuccess: () => {
    syncing.value = false
    if (logPollTimer) {
      clearInterval(logPollTimer)
      logPollTimer = null
    }
    notifySuccess("RKP 硬件签发并同步成功！已自动写入活跃 Keystore 模块。")
    void refreshLog()
  },
  onError: (err: any) => {
    syncing.value = false
    if (logPollTimer) {
      clearInterval(logPollTimer)
      logPollTimer = null
    }
    notifyError(err)
    void refreshLog()
  },
})

function triggerSync() {
  runSyncMutation.mutate()
}

const currentScheduleLabel = computed(() => {
  const item = dowNames.find((d) => d.val === schedDow.value)
  const dStr = item ? item.label : `每周${schedDow.value}`
  const hStr = `${schedHour.value.toString().padStart(2, "0")}:00`
  return `${dStr} ${hStr}`
})

// ==============================================================================
// 5. 极致跟手拖拽引擎 (Interactive Drag Physics) 与 懒加载控制
// ==============================================================================
const activeTabIndex = useSessionStorage<number>("quack-store/active-tab-index", 0)
const mainViewport = ref<HTMLElement | null>(null)
const viewportWidth = ref(360)

// 核心优化：按需懒加载 TargetsPage，首屏避免同时并发扫描几百个应用
const hasVisitedTargets = ref(activeTabIndex.value === 1)
watch(activeTabIndex, (val) => {
  if (val === 1) {
    hasVisitedTargets.value = true
  }
})

const isDragging = ref(false)
const dragDeltaPx = ref(0)
const isLockedHorizontal = ref(false)
const isLockedVertical = ref(false)

let touchStartX = 0
let touchStartY = 0
let touchStartTime = 0

function updateContainerWidth() {
  if (mainViewport.value) {
    viewportWidth.value = mainViewport.value.clientWidth || 360
  }
}

function onTouchStart(e: TouchEvent) {
  if (e.touches.length === 1 && e.touches[0]) {
    touchStartX = e.touches[0].clientX
    touchStartY = e.touches[0].clientY
    touchStartTime = Date.now()
    updateContainerWidth()
    isDragging.value = false
    dragDeltaPx.value = 0
    isLockedHorizontal.value = false
    isLockedVertical.value = false
  }
}

function onTouchMove(e: TouchEvent) {
  if (e.touches.length !== 1 || !e.touches[0] || isLockedVertical.value) return

  const curX = e.touches[0].clientX
  const curY = e.touches[0].clientY
  const dx = curX - touchStartX
  const dy = curY - touchStartY

  if (!isLockedHorizontal.value && !isLockedVertical.value) {
    if (Math.abs(dy) > 7 && Math.abs(dy) > Math.abs(dx)) {
      isLockedVertical.value = true
      return
    }
    if (Math.abs(dx) > 7 && Math.abs(dx) > Math.abs(dy)) {
      isLockedHorizontal.value = true
    }
  }

  if (isLockedHorizontal.value) {
    isDragging.value = true
    let effective = dx
    if ((activeTabIndex.value === 0 && dx > 0) || (activeTabIndex.value === 1 && dx < 0)) {
      effective = dx * 0.25
    }
    dragDeltaPx.value = effective
  }
}

function onTouchEnd() {
  if (!isDragging.value) {
    isLockedHorizontal.value = false
    isLockedVertical.value = false
    return
  }

  const duration = Math.max(1, Date.now() - touchStartTime)
  const distance = dragDeltaPx.value
  const velocity = Math.abs(distance) / duration
  const w = viewportWidth.value || 360
  const distanceThreshold = w * 0.15

  const isFling = duration < 350 && Math.abs(distance) > 25 && velocity > 0.25

  let targetIndex = activeTabIndex.value

  if (activeTabIndex.value === 0) {
    if (distance < 0 && (isFling || distance < -distanceThreshold)) {
      targetIndex = 1
    }
  } else if (activeTabIndex.value === 1) {
    if (distance > 0 && (isFling || distance > distanceThreshold)) {
      targetIndex = 0
    }
  }

  activeTabIndex.value = targetIndex
  isDragging.value = false
  dragDeltaPx.value = 0
  isLockedHorizontal.value = false
  isLockedVertical.value = false
}

const trackStyle = computed(() => {
  const baseOffset = activeTabIndex.value * -50
  let currentOffset = baseOffset

  if (isDragging.value && viewportWidth.value > 0) {
    const deltaPercent = (dragDeltaPx.value / (viewportWidth.value * 2)) * 100
    currentOffset = baseOffset + deltaPercent
  }

  return {
    transform: `translate3d(${currentOffset}%, 0, 0)`,
    transition: isDragging.value ? "none" : "transform 0.28s cubic-bezier(0.25, 1, 0.5, 1)",
  }
})

const indicatorStyle = computed(() => {
  const basePercent = activeTabIndex.value * 100
  let currentPercent = basePercent

  if (isDragging.value && viewportWidth.value > 0) {
    const deltaPercent = -(dragDeltaPx.value / viewportWidth.value) * 100
    currentPercent = basePercent + deltaPercent
  }

  const clamped = Math.max(0, Math.min(100, currentPercent))

  return {
    transform: `translate3d(${clamped}%, 0, 0)`,
    transition: isDragging.value ? "none" : "transform 0.28s cubic-bezier(0.25, 1, 0.5, 1)",
  }
})

function selectTab(index: number) {
  activeTabIndex.value = index
  isDragging.value = false
  dragDeltaPx.value = 0
}
</script>

<template>
  <div
    class="flex flex-col gap-4 p-3 md:p-6 touch-pan-y min-h-[85vh] select-none"
    @touchstart="onTouchStart"
    @touchmove="onTouchMove"
    @touchend="onTouchEnd"
    @touchcancel="onTouchEnd"
  >
    <!-- 顶部原生级平移指示器 Tab 栏 (吸顶位置精确加上状态栏避让高度 var(--inset-top)，严禁重叠穿插) -->
    <div
      class="sticky z-30 bg-background/95 backdrop-blur-md -mx-3 md:-mx-6 px-3 md:px-6 pt-1 pb-2 border-b border-border/20"
      style="top: calc(3.5rem + var(--inset-top));"
    >
      <div class="relative w-full rounded-lg bg-muted/60 p-1 border">
        <div class="grid grid-cols-2 h-10 w-full relative z-10">
          <button
            type="button"
            class="flex items-center justify-center gap-2 text-sm font-medium transition-colors"
            :class="activeTabIndex === 0 ? 'text-foreground font-semibold' : 'text-muted-foreground'"
            @click="selectTab(0)"
          >
            <Zap class="size-4 text-amber-500" />
            <span>签发与自动化</span>
          </button>
          <button
            type="button"
            class="flex items-center justify-center gap-2 text-sm font-medium transition-colors"
            :class="activeTabIndex === 1 ? 'text-foreground font-semibold' : 'text-muted-foreground'"
            @click="selectTab(1)"
          >
            <ShieldCheck class="size-4 text-emerald-500" />
            <span>作用范围与秘钥管理</span>
          </button>
        </div>

        <div
          class="absolute top-1 bottom-1 left-1 w-[calc(50%-4px)] pointer-events-none rounded-md bg-background shadow-xs will-change-transform flex flex-col justify-end items-center"
          :style="indicatorStyle"
        >
          <div
            class="h-0.5 w-16 rounded-full transition-colors duration-200"
            :class="activeTabIndex === 1 ? 'bg-emerald-500' : 'bg-amber-500'"
          />
        </div>
      </div>
    </div>

    <!-- 视口包裹层与跟手物理轨道 (横向剪裁，垂直放行，双面板独立高度自适应) -->
    <div ref="mainViewport" class="w-full overflow-x-clip overflow-y-visible">
      <div
        class="flex w-[200%] will-change-transform items-start"
        :style="trackStyle"
      >
        <!-- 面板 1: 自动签名与调度 (占 50% 轨道宽，独立深度隔离) -->
        <div
          class="w-1/2 shrink-0 px-1 flex flex-col gap-4"
          :class="{ 'h-0 overflow-hidden invisible': !isDragging && activeTabIndex !== 0 }"
        >
          <!-- 硬件 TA 概况卡片 -->
          <SectionCard title="硬件 TA 概况">
            <QueryState
              :loading="rkpStatus.isPending.value"
              :error="rkpStatus.error.value"
              @retry="rkpStatus.refetch()"
            >
              <div class="flex flex-col gap-3">
                <div class="flex items-center justify-between text-sm">
                  <div class="flex items-center gap-2">
                    <span class="font-medium text-foreground">
                      {{ rkpStatus.data.value?.soc_detection.model || "高通骁龙芯片平台" }}
                    </span>
                    <span class="text-xs font-mono text-muted-foreground">
                      {{ rkpStatus.data.value?.device_path || "/dev/smcinvoke" }}
                    </span>
                  </div>
                  <Badge :variant="rkpStatus.data.value?.prerequisites_met ? 'default' : 'destructive'" class="text-xs">
                    {{ rkpStatus.data.value?.prerequisites_met ? "硬件已就绪" : "未满足条件" }}
                  </Badge>
                </div>

                <div class="grid grid-cols-3 gap-2 pt-2 border-t text-center">
                  <div class="rounded-md border bg-muted/20 p-2 flex flex-col items-center gap-1">
                    <span class="text-xs font-medium text-muted-foreground">CSR V2</span>
                    <Badge
                      :variant="rkpStatus.data.value?.v2_available ? 'outline' : 'secondary'"
                      :class="rkpStatus.data.value?.v2_available ? 'text-emerald-600 border-emerald-300 dark:text-emerald-400' : 'text-zinc-400'"
                      class="text-[11px] px-2 py-0 h-5"
                    >
                      {{ rkpStatus.data.value?.v2_available ? "支持" : "不可用" }}
                    </Badge>
                  </div>
                  <div class="rounded-md border bg-muted/20 p-2 flex flex-col items-center gap-1">
                    <span class="text-xs font-medium text-muted-foreground">V1 AIDL</span>
                    <Badge
                      :variant="rkpStatus.data.value?.v1_available ? 'outline' : 'secondary'"
                      :class="rkpStatus.data.value?.v1_available ? 'text-emerald-600 border-emerald-300 dark:text-emerald-400' : 'text-zinc-400'"
                      class="text-[11px] px-2 py-0 h-5"
                    >
                      {{ rkpStatus.data.value?.v1_available ? "支持" : "不可用" }}
                    </Badge>
                  </div>
                  <div class="rounded-md border bg-muted/20 p-2 flex flex-col items-center gap-1">
                    <span class="text-xs font-medium text-muted-foreground">V1 Native</span>
                    <Badge
                      :variant="rkpStatus.data.value?.v1_native_available ? 'outline' : 'secondary'"
                      :class="rkpStatus.data.value?.v1_native_available ? 'text-emerald-600 border-emerald-300 dark:text-emerald-400' : 'text-zinc-400'"
                      class="text-[11px] px-2 py-0 h-5"
                    >
                      {{ rkpStatus.data.value?.v1_native_available ? "支持" : "不可用" }}
                    </Badge>
                  </div>
                </div>
              </div>
            </QueryState>
          </SectionCard>

          <!-- 周期自动化与操作卡片 -->
          <SectionCard title="周期自动化同步">
            <div class="flex flex-col gap-4">
              <div class="rounded-lg border bg-card p-4 flex flex-col gap-3 text-sm">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <span class="font-medium text-base">周期自动化同步</span>
                    <Badge :variant="schedEnabled ? 'default' : 'secondary'" class="text-xs">
                      {{ schedEnabled ? "运行中" : "已暂停" }}
                    </Badge>
                  </div>
                  <Switch
                    :model-value="schedEnabled"
                    @update:model-value="onToggleScheduler"
                  />
                </div>

                <div v-if="schedEnabled" class="flex items-center justify-between text-muted-foreground pt-2 border-t">
                  <span>定时规则</span>
                  <div class="flex items-center gap-1.5">
                    <span class="font-mono text-foreground font-medium">
                      {{ currentScheduleLabel }} (自动唤醒)
                    </span>
                    <Button variant="ghost" size="sm" class="h-7 px-2 text-xs" @click="openSettingsDialog">
                      <Settings2 class="size-3.5 mr-1" />
                      修改时间
                    </Button>
                  </div>
                </div>
                <div v-else class="text-xs text-muted-foreground pt-2 border-t">
                  定时调度已停用，系统不会在后台自动唤醒执行签名与同步。
                </div>
              </div>

              <!-- 即时手动触发按钮 -->
              <Button
                size="lg"
                class="w-full h-12 text-base font-semibold shadow-sm"
                :disabled="syncing || !rkpStatus.data.value?.prerequisites_met"
                @click="triggerSync"
              >
                <Spinner v-if="syncing" class="mr-2" />
                <Zap v-else class="mr-2 size-5" />
                {{ syncing ? "正在进行硬件签名与同步..." : "立即触发硬件签发并同步" }}
              </Button>
            </div>
          </SectionCard>

          <!-- 调度实时日志卡片 -->
          <SectionCard title="调度与执行日志">
            <template #action>
              <Button
                v-if="userScrolled"
                size="icon-sm"
                class="size-7 rounded-full bg-white text-zinc-950 shadow-sm hover:bg-zinc-100 dark:bg-zinc-100 dark:text-zinc-950 border border-zinc-200"
                title="回到最新"
                @click="scrollToLatest"
              >
                <ArrowUpCircle class="size-4" />
              </Button>
            </template>

            <div
              ref="logContainer"
              class="rounded-lg bg-zinc-950 p-3 font-mono text-xs text-zinc-300 overflow-y-auto max-h-64 leading-relaxed border border-zinc-800 space-y-1 relative scroll-smooth"
              @scroll="onLogScroll"
            >
              <div
                v-for="(line, idx) in syncLogLines"
                :key="idx"
                :class="[
                  'break-words',
                  idx === 0 ? 'text-emerald-400 font-semibold' : 'text-zinc-300',
                ]"
              >
                {{ line }}
              </div>
            </div>
          </SectionCard>
        </div>

        <!-- 面板 2: 目标与密钥管理 (按需懒加载挂载，彻底消除首屏卡顿) -->
        <div
          class="w-1/2 shrink-0 px-1"
          :class="{ 'h-0 overflow-hidden invisible': !isDragging && activeTabIndex !== 1 }"
        >
          <TargetsPage v-if="hasVisitedTargets" :is-active="activeTabIndex === 1" />
          <div v-else class="flex items-center justify-center py-20 text-muted-foreground text-sm">
            <Spinner class="size-5 mr-2" />
            正在就绪目标管理组件...
          </div>
        </div>
      </div>
    </div>

    <!-- 修改时间模态弹窗 (Modal Dialog) -->
    <Dialog v-model:open="showSettingsDialog">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>修改定时同步时间</DialogTitle>
          <DialogDescription>
            设置后台常驻守护进程自动唤醒并执行硬件签名与同步的触发时间。
          </DialogDescription>
        </DialogHeader>

        <div class="grid grid-cols-2 gap-3 py-3">
          <div class="flex flex-col gap-1.5">
            <label class="text-xs text-muted-foreground font-medium">运行星期</label>
            <select
              v-model="tempDow"
              class="h-10 px-3 rounded-md border bg-background text-sm font-medium focus:outline-none focus:ring-2 focus:ring-primary"
            >
              <option v-for="d in dowNames" :key="d.val" :value="d.val">
                {{ d.label }}
              </option>
            </select>
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-xs text-muted-foreground font-medium">触发时刻</label>
            <select
              v-model="tempHour"
              class="h-10 px-3 rounded-md border bg-background text-sm font-mono font-medium focus:outline-none focus:ring-2 focus:ring-primary"
            >
              <option v-for="h in 24" :key="h - 1" :value="h - 1">
                {{ (h - 1).toString().padStart(2, "0") }}:00
              </option>
            </select>
          </div>
        </div>

        <DialogFooter class="flex flex-row justify-end gap-2 pt-2">
          <Button variant="outline" @click="cancelDialog">取消</Button>
          <Button :disabled="savingConfig" @click="saveDialog">
            <Spinner v-if="savingConfig" class="mr-2" />
            保存设置
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
