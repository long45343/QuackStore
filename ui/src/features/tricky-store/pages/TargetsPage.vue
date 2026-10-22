<script setup lang="ts">
import { useMutation, useQueryClient } from "@tanstack/vue-query"
import { PackageSearch, Search } from "@lucide/vue"
import { onKeyStroke } from "@vueuse/core"
import { computed, ref, useTemplateRef, watch } from "vue"
import { useRoute } from "vue-router"

import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty"
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group"
import QueryState from "@/core/components/QueryState.vue"
import VirtualList from "@/core/components/VirtualList.vue"
import { useRouteModal } from "@/core/modal"
import { notifySuccess } from "@/core/notify"
import { bridge } from "@/core/bridge"

import { trickyStoreApi } from "../api"
import AppRow from "../components/AppRow.vue"
import BackendCard from "../components/BackendCard.vue"
import ModeSheet from "../components/ModeSheet.vue"
import SaveFab from "../components/SaveFab.vue"
import SystemAppsDialog from "../components/SystemAppsDialog.vue"
import TargetsMenu from "../components/TargetsMenu.vue"
import { useTrickyI18n } from "../i18n"
import { STATUS_KEY, useLabels, useStatusQuery } from "../queries"
import { matches, sortedApps } from "../selection"
import { useTrickyStore } from "../store"

const { t } = useTrickyI18n()
const props = withDefaults(
  defineProps<{ isActive?: boolean }>(),
  { isActive: true },
)
const route = useRoute()
const isDirectPage = computed(() => {
  const host = bridge().hostModule()?.id
  const fromKeystore = Boolean(host && ["tricky_store", "oh_my_keymint", "teesim"].includes(host))
  return fromKeystore || (route?.path?.startsWith("/tricky-store") ?? false)
})
const store = useTrickyStore()
const status = useStatusQuery()
const client = useQueryClient()
const modeSheet = useRouteModal("mode")
const systemApps = useRouteModal("system-apps")

watch(status.data, (next) => next && store.load(next), { immediate: true })

const allPackages = computed<readonly string[]>((previous) => {
  const names = new Set(store.status?.packages.map((entry) => entry.package_name))
  for (const name of store.draft?.targets.keys() ?? []) names.add(name)
  const next = [...names].sort()
  return previous?.length === next.length && next.every((name, index) => name === previous[index])
    ? previous
    : next
})
const labels = useLabels(allPackages)

const query = ref("")
const search = useTemplateRef<{ $el: HTMLElement }>("search")
const catalog = computed(() =>
  store.status ? sortedApps(store.status, allPackages.value, labels.value) : [],
)
const listed = computed<typeof catalog.value>((previous) => {
  const next = catalog.value.filter(
    ({ packageName, system }) =>
      !system || store.draft?.systemApps.has(packageName) || store.draft?.targets.has(packageName),
  )
  return previous?.length === next.length && next.every((item, index) => item === previous[index])
    ? previous
    : next
})
const visible = computed(() => listed.value.filter((item) => matches(item, query.value)))
const tunable = computed(
  () => !!store.schema && (store.schema.supports_app_mode || store.schema.supports_per_app_policy),
)

function counted(changed: number, key: "list.selectedCount" | "list.deselectedCount") {
  notifySuccess(changed ? t(key, { count: changed }) : t("list.nothingChanged"))
}

/** With a search active, bulk actions apply to the matching apps only. */
const scope = () => visible.value.map((item) => item.packageName)
const selectAll = () => store.selectMany(scope())
function deselectAll() {
  store.deselectMany(query.value.trim() ? scope() : [...(store.draft?.targets.keys() ?? [])])
}

const save = useMutation({
  mutationFn: async () => {
    await client.cancelQueries({ queryKey: STATUS_KEY })
    const saved = await store.save()
    client.setQueryData(STATUS_KEY, store.status)
    return saved
  },
  meta: { errorTitle: () => t("ta.prompt_save_error") },
  onSuccess: (saved) =>
    notifySuccess(
      t("ta.prompt_saved_target"),
      saved.restart_required ? t("policy.restart") : undefined,
    ),
})

async function refresh() {
  if (store.saving) return
  const next = await status.refetch()
  if (!next.isSuccess || !next.data) return
  store.load(next.data, true)
  void client.invalidateQueries({ queryKey: ["tricky-store", "labels"] })
  window.scrollTo({ top: 0 })
}

const denylist = useMutation({
  mutationFn: trickyStoreApi.denylist,
  onSuccess: (data) => counted(store.selectMany(data.packages), "list.selectedCount"),
})
const autoAdd = useMutation({
  mutationFn: async (enabled: boolean) => {
    await client.cancelQueries({ queryKey: STATUS_KEY })
    const result = await store.saveOnly({ autoAdd: enabled })
    client.setQueryData(STATUS_KEY, store.status)
    return result
  },
  meta: { errorTitle: () => t("ta.prompt_save_error") },
})

function discard() {
  if (store.status) store.load(store.status, true)
  notifySuccess(t("list.discarded"))
}

// Tricky Addon's keyboard shortcuts.
const typing = (event: KeyboardEvent) =>
  event.target instanceof HTMLElement && !!event.target.closest("input, textarea")
function shortcut(key: string, action: () => void) {
  onKeyStroke(key, (event) => {
    if (!event.ctrlKey || !store.status?.active || (key !== "f" && key !== "s" && typing(event)))
      return
    event.preventDefault()
    action()
  })
}
shortcut("a", selectAll)
shortcut("d", deselectAll)
shortcut("f", () => search.value?.$el.querySelector("input")?.focus())
shortcut("s", () => {
  if (!save.isPending.value && !store.saving) save.mutate()
})
onKeyStroke("Escape", (event) => {
  if (typing(event) && query.value) query.value = ""
})
</script>

<template>
  <div
    class="flex flex-col gap-3 px-4 pb-32"
    :style="{
      paddingTop: isDirectPage ? 'calc(var(--inset-top) + 0.875rem)' : '0.5rem',
    }"
  >
    <QueryState
      :loading="status.isPending.value"
      :error="status.error.value"
      @retry="status.refetch()"
    >
      <template v-if="store.status && store.draft">
        <Empty v-if="!store.status.active">
          <EmptyHeader>
            <EmptyMedia variant="icon"><PackageSearch /></EmptyMedia>
            <EmptyTitle>{{ t("backend.none") }}</EmptyTitle>
            <EmptyDescription>{{ t("backend.noneHint") }}</EmptyDescription>
          </EmptyHeader>
        </Empty>

        <template v-else>
          <!-- 独立模式下的状态栏实色遮蔽条：固定在屏幕最顶端，彻底遮挡滚上来的应用列表，绝不露馅 -->
          <div
            v-if="isDirectPage"
            class="fixed top-0 inset-x-0 z-30 bg-background pointer-events-none"
            style="height: var(--inset-top);"
          />

          <BackendCard :status="store.status" />

          <!-- 搜索栏区域：与上方 BackendCard 保持标准自适应间距，吸顶在状态栏下方或 Tab 栏下方 -->
          <div
            class="bg-background sticky z-20 -mx-4 px-4 py-2 flex items-center gap-2 border-b border-border/40"
            :style="{
              top: isDirectPage ? 'var(--inset-top)' : 'calc(6.75rem + var(--inset-top))',
            }"
          >
            <InputGroup ref="search" class="bg-card h-12 rounded-full border-0 flex-1">
              <InputGroupAddon class="ps-4"><Search class="size-5" /></InputGroupAddon>
              <InputGroupInput
                v-model="query"
                type="search"
                :placeholder="t('ta.search_bar_search_placeholder')"
                :title="t('list.shortcuts')"
              />
              <InputGroupAddon
                align="inline-end"
                class="text-muted-foreground pe-4 text-xs tabular-nums"
              >
                {{ t("list.selected", { count: store.draft.targets.size }) }}
              </InputGroupAddon>
            </InputGroup>

            <!-- 操作菜单 (三个点)：移至搜索框右侧，不再外溢到全局 AppBar -->
            <TargetsMenu
              v-if="store.status?.active"
              class="shrink-0"
              :denylist="store.status.root_manager?.kind === 'magisk'"
              :auto-add="store.draft?.autoAdd ?? false"
              :has-policy="!!store.schema?.default_policy.length"
              :saving="store.saving || save.isPending.value"
              @select-all="selectAll"
              @deselect-all="deselectAll"
              @refresh="refresh"
              @denylist="denylist.mutate()"
              @system-apps="systemApps.show()"
              @auto-add="autoAdd.mutate($event)"
            />
          </div>

          <VirtualList
            :items="visible"
            :item-key="(item) => item.packageName"
            :estimate-size="96"
            window-scroll
          >
            <template #default="{ item, index }">
              <AppRow
                :class="{
                  'rounded-t-lg': !store.draft.targets.has(item.packageName) && index === 0,
                  'rounded-b-lg':
                    !store.draft.targets.has(item.packageName) && index === visible.length - 1,
                }"
                :package-name="item.packageName"
                :label="item.label"
                :system="item.system"
                :mode="store.draft.targets.get(item.packageName) ?? null"
                :custom-policy="store.draft.perApp.has(item.packageName)"
                :tunable="tunable"
                @toggle="store.toggle(item.packageName)"
                @tune="modeSheet.show({ pkg: item.packageName })"
              />
            </template>
          </VirtualList>
          <p v-if="!visible.length" class="text-muted-foreground py-8 text-center text-sm">
            {{ t("list.empty") }}
          </p>
        </template>
      </template>
    </QueryState>

    <template v-if="store.status?.active">
      <SaveFab
        :dirty="store.dirty"
        :saving="save.isPending.value || store.saving"
        :visible="props.isActive"
        @save="save.mutate()"
        @discard="discard"
      />
      <ModeSheet :labels="labels" />
      <SystemAppsDialog :labels="labels" />
    </template>
  </div>
</template>
