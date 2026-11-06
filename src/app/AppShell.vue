<script setup lang="ts">
import { useStorage } from '@vueuse/core'
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useRoute, useRouter } from 'vue-router'
import { ChevronDown, CircleDot, Database, PanelLeftClose, PanelLeftOpen, Settings2 } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { closeDesktopWindow, confirmDiscard, hasDesktopRuntime, openCpsFile, openProjectFile, saveProjectFile } from '@/lib/tauri'
import { useWellModelStore } from '@/stores/wellModel'

const route = useRoute()
const router = useRouter()
const model = useWellModelStore()
const importError = ref('')
const importing = ref(false)
const isDesktop = hasDesktopRuntime()
let unlistenClose: (() => void) | undefined

async function mayDiscard() {
  return !model.dirty || await confirmDiscard()
}

async function runAction(action: () => Promise<void>) {
  importError.value = ''
  importing.value = true
  try { await action() }
  catch (error) { importError.value = `Операция не выполнена: ${String(error)}` }
  finally { importing.value = false }
}

async function importFile() {
  await runAction(async () => {
    const path = await openCpsFile()
    if (path && await mayDiscard()) {
      await model.importCps(path)
      await router.push('/well-map')
    }
  })
}

async function openFile() {
  await runAction(async () => {
    const path = await openProjectFile()
    if (path && await mayDiscard()) {
      await model.openProject(path)
      await router.push('/well-map')
    }
  })
}

async function saveFile(saveAs = false) {
  await runAction(async () => {
    const path = saveAs || !model.projectPath ? await saveProjectFile() : null
    if (path || (!saveAs && model.projectPath)) await model.saveProject(path)
  })
}

onMounted(async () => {
  if (!isDesktop) return
  try {
    unlistenClose = await getCurrentWindow().onCloseRequested(async (event) => {
      if (model.dirty) {
        event.preventDefault()
        if (await mayDiscard()) await getCurrentWindow().destroy()
      }
    })
    await model.restore()
  } catch (error) { importError.value = `Не удалось восстановить проект: ${String(error)}` }
})
onBeforeUnmount(() => unlistenClose?.())
const isSidebarCollapsed = useStorage('well-forge-sidebar-collapsed', false)
const navigation = [
  { to: '/', label: 'Начальные данные', icon: Database },
  { to: '/well-map', label: 'Карта скважин', icon: CircleDot },
]
</script>

<template>
  <div class="flex h-dvh min-h-[640px] w-full flex-col overflow-hidden bg-background text-foreground">
    <header class="flex h-11 shrink-0 items-center border-b bg-card">
      <div
        class="flex h-full shrink-0 items-center gap-2.5 border-r motion-safe:transition-[width] motion-safe:duration-150"
        :class="isSidebarCollapsed ? 'w-14 justify-center px-0' : 'w-64 px-4'"
      >
        <span class="flex size-7 items-center justify-center rounded-md bg-sidebar-primary text-sidebar-primary-foreground">
          <CircleDot class="size-4" aria-hidden="true" />
        </span>
        <span v-if="!isSidebarCollapsed" class="truncate text-sm font-semibold tracking-tight text-sidebar-foreground">Скважинатор 9000</span>
      </div>
      <Button
        variant="ghost"
        size="icon-sm"
        class="ml-2"
        :aria-label="isSidebarCollapsed ? 'Развернуть боковую панель' : 'Свернуть боковую панель'"
        :aria-expanded="!isSidebarCollapsed"
        aria-controls="app-sidebar"
        :title="isSidebarCollapsed ? 'Развернуть боковую панель' : 'Свернуть боковую панель'"
        @click="isSidebarCollapsed = !isSidebarCollapsed"
      >
        <PanelLeftOpen v-if="isSidebarCollapsed" class="size-4" aria-hidden="true" />
        <PanelLeftClose v-else class="size-4" aria-hidden="true" />
      </Button>
      <span v-if="isDesktop" class="ml-3 max-w-48 truncate text-xs text-muted-foreground" :title="model.projectPath ?? model.projectName">{{ model.projectName }}{{ model.dirty ? ' •' : '' }}</span>
      <nav aria-label="Меню приложения" class="flex items-center gap-1 px-3">
        <DropdownMenu>
          <DropdownMenuTrigger as-child>
            <Button variant="ghost" size="sm" class="gap-1.5" aria-label="Меню Файл">
              Файл
              <ChevronDown class="size-3 text-muted-foreground" aria-hidden="true" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start" class="w-64">
            <DropdownMenuItem :disabled="!isDesktop || importing" @select="importFile">
              {{ importing ? 'Импорт…' : 'Импорт CPS' }}
            </DropdownMenuItem>
            <DropdownMenuItem :disabled="!isDesktop || importing" @select="openFile">Открыть проект</DropdownMenuItem>
            <DropdownMenuItem :disabled="!isDesktop || importing" @select="saveFile(false)">Сохранить</DropdownMenuItem>
            <DropdownMenuItem :disabled="!isDesktop || importing" @select="saveFile(true)">Сохранить как…</DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem :disabled="!isDesktop" @select="closeDesktopWindow">
              Выход
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </nav>
      <RouterLink
        to="/settings"
        class="mr-3 ml-auto flex h-7 items-center gap-2 rounded-md px-2.5 text-sm text-muted-foreground motion-safe:transition-colors hover:bg-muted hover:text-foreground focus-visible:outline-2 focus-visible:outline-ring"
        active-class="bg-muted text-foreground"
        :aria-current="route.path === '/settings' ? 'page' : undefined"
      >
        <Settings2 class="size-4" aria-hidden="true" />
        <span>Параметры</span>
      </RouterLink>
    </header>

    <div class="flex min-h-0 flex-1">
      <aside
        id="app-sidebar"
        class="shrink-0 border-r border-sidebar-border bg-sidebar motion-safe:transition-[width] motion-safe:duration-150"
        :class="isSidebarCollapsed ? 'w-14' : 'w-64'"
      >
        <nav aria-label="Разделы приложения" class="space-y-1 p-2">
          <RouterLink
            v-for="item in navigation"
            :key="item.to"
            :to="item.to"
            class="flex min-h-9 items-center rounded-md text-sm text-sidebar-foreground motion-safe:transition-colors hover:bg-sidebar-accent focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-sidebar-ring"
            :class="isSidebarCollapsed ? 'justify-center px-0' : 'gap-2.5 px-2.5'"
            active-class="bg-sidebar-accent font-medium text-sidebar-accent-foreground"
            :aria-current="route.path === item.to ? 'page' : undefined"
            :aria-label="isSidebarCollapsed ? item.label : undefined"
            :title="isSidebarCollapsed ? item.label : undefined"
          >
            <component :is="item.icon" class="size-4 shrink-0" aria-hidden="true" />
            <span v-if="!isSidebarCollapsed" class="truncate">{{ item.label }}</span>
          </RouterLink>
        </nav>
      </aside>

      <main class="min-h-0 min-w-0 flex-1 overflow-auto p-5">
        <div v-if="importError" class="mb-3 rounded-md border border-destructive bg-card px-3 py-2 text-sm text-destructive" role="alert">{{ importError }}</div>
        <div v-if="model.warning" class="mb-3 rounded-md border bg-card px-3 py-2 text-sm text-muted-foreground" role="status">{{ model.warning }}</div>
        <RouterView />
      </main>
    </div>
  </div>
</template>
