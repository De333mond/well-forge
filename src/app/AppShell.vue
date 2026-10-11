<script setup lang="ts">
import { Blocks, House } from '@lucide/vue'
import ThemeSwitcher from '@/components/ThemeSwitcher.vue'
</script>

<template>
  <div class="min-h-screen bg-background">
    <div class="mx-auto flex min-h-screen max-w-7xl">
      <aside class="hidden w-52 shrink-0 flex-col border-r border-sidebar-border bg-sidebar px-3 py-4 sm:flex">
        <div class="mb-8 flex items-center gap-2.5 px-2">
          <span class="flex size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
            <Blocks class="size-4" aria-hidden="true" />
          </span>
          <span class="text-sm font-semibold tracking-tight text-sidebar-foreground">Well Forge</span>
        </div>

        <nav aria-label="Main navigation" class="space-y-1">
          <RouterLink
            to="/"
            class="flex h-9 items-center gap-2.5 rounded-lg bg-sidebar-accent px-3 text-sm font-medium text-sidebar-accent-foreground transition-colors"
            aria-current="page"
          >
            <House class="size-4" aria-hidden="true" />
            Overview
          </RouterLink>
        </nav>
      </aside>

      <div class="min-w-0 flex-1">
        <header class="flex h-14 items-center justify-between border-b bg-card px-5 sm:px-8">
          <div class="flex items-center gap-2.5">
            <span class="flex size-7 items-center justify-center rounded-md bg-primary text-primary-foreground sm:hidden">
              <Blocks class="size-4" aria-hidden="true" />
            </span>
            <span class="text-sm font-medium text-foreground sm:text-muted-foreground">
              <span class="sm:hidden">Well Forge</span>
              <span class="hidden sm:inline">Overview</span>
            </span>
          </div>
          <ThemeSwitcher />
        </header>

        <main class="mx-auto max-w-4xl px-5 py-8 sm:px-8 sm:py-10">
          <RouterView />
        </main>
      </div>
    </div>
  </div>
</template>
