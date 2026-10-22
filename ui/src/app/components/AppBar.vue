<script setup lang="ts">
import { ArrowLeft, Settings } from "@lucide/vue"
import { computed } from "vue"
import { useI18n } from "vue-i18n"
import { useRoute, useRouter } from "vue-router"

import { Button } from "@/components/ui/button"

defineProps<{
  title: string
  titleVisible?: boolean
  titleIsHeading?: boolean
}>()

const route = useRoute()
const router = useRouter()
const { t } = useI18n()

const isHome = computed(() => route.name === "home")

function back() {
  if (window.history.state?.back) {
    router.back()
    return
  }
  const parent = route.matched.at(-2)
  void router.replace(parent && parent.path !== route.path ? parent.path : "/")
}
</script>

<template>
  <header class="bg-background pt-safe px-safe sticky top-0 z-40 border-b border-border/40 backdrop-blur">
    <div class="mx-auto flex h-14 max-w-3xl items-center justify-between px-3">
      <!-- 图标与标题文字始终并排同一行 -->
      <div class="flex items-center gap-2.5 min-w-0 flex-1">
        <Button
          v-if="!isHome"
          variant="ghost"
          size="icon"
          class="size-9 shrink-0"
          :aria-label="t('core.nav.back')"
          @click="back"
        >
          <ArrowLeft class="rtl-flip size-5" />
        </Button>
        <img
          v-else
          src="/duck-logo.svg"
          alt=""
          class="size-7 dark:invert shrink-0"
          width="28"
          height="28"
        />
        <h1 class="font-bold text-xl tracking-tight text-foreground truncate select-none">
          {{ title }}
        </h1>
      </div>

      <!-- 右侧设置与扩展插槽 -->
      <div class="flex items-center gap-1 shrink-0">
        <div id="app-bar-actions" class="flex items-center gap-1" />
        <Button
          v-if="isHome"
          variant="ghost"
          size="icon"
          class="size-9"
          :aria-label="t('core.nav.settings')"
          @click="router.push('/settings')"
        >
          <Settings class="size-5" />
        </Button>
      </div>
    </div>
  </header>
</template>
