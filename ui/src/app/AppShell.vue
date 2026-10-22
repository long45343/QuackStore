<script setup lang="ts">
import "vue-sonner/style.css"

import { ConfigProvider } from "reka-ui"
import { computed } from "vue"
import { useI18n } from "vue-i18n"
import { useRoute } from "vue-router"

import { Toaster } from "@/components/ui/sonner"
import ErrorDetailsDialog from "@/core/components/ErrorDetailsDialog.vue"
import { isRtl } from "@/core/i18n"
import { useDarkTheme } from "@/core/theme"
import { bridge } from "@/core/bridge"

import AppBar from "./components/AppBar.vue"

const dark = useDarkTheme()
const route = useRoute()
const { t, locale } = useI18n()
// reka-ui lays its components out left to right unless told otherwise.
const dir = computed(() => (isRtl(locale.value) ? "rtl" : "ltr"))

const isKeystoreHost = computed(() => {
  const hostId = bridge().hostModule()?.id
  const isKeystoreModule = Boolean(hostId && ["tricky_store", "oh_my_keymint", "teesim"].includes(hostId))
  const isDirectTrickyStore = route.path.startsWith("/tricky-store")
  return isKeystoreModule || isDirectTrickyStore
})

const title = computed(() => {
  const key = [...route.matched].reverse().find((record) => record.meta.titleKey)?.meta.titleKey
  return key ? t(key) : t("core.app.name")
})
</script>

<template>
  <ConfigProvider :dir="dir">
    <div class="min-h-dvh">
      <AppBar v-if="!isKeystoreHost" :title="title" />
      <main class="px-safe pb-safe mx-auto max-w-3xl">
        <RouterView v-slot="{ Component, route: current }">
          <component :is="Component" :key="current.matched[0]?.path" />
        </RouterView>
      </main>
      <Toaster
        :theme="dark ? 'dark' : 'light'"
        position="top-center"
        rich-colors
        close-button
        :offset="{ top: 'calc(var(--inset-top) + 16px)' }"
        :mobile-offset="{ top: 'calc(var(--inset-top) + 8px)' }"
        :toast-options="{
          classes: {
            content: 'min-w-0 flex-1',
            description: 'whitespace-pre-line',
            actionButton: 'shrink-0',
          },
        }"
      />
      <ErrorDetailsDialog />
    </div>
  </ConfigProvider>
</template>
