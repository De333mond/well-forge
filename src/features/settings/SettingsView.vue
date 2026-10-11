<script setup lang="ts">
import ThemeSwitcher from '@/components/ThemeSwitcher.vue'
</script>

<template>
  <div class="space-y-5">
    <div>
      <h1 class="text-lg font-semibold tracking-tight">Параметры приложения</h1>
      <p class="mt-1 text-sm text-muted-foreground">Настройте внешний вид интерфейса.</p>
    </div>

    <section aria-labelledby="appearance-heading" class="max-w-2xl rounded-lg border bg-card">
      <div class="border-b px-4 py-3">
        <h2 id="appearance-heading" class="text-sm font-semibold">Внешний вид</h2>
      </div>
      <div class="flex flex-wrap items-center justify-between gap-4 px-4 py-3">
        <div>
          <p class="text-sm font-medium">Тема оформления</p>
          <p class="mt-0.5 text-xs text-muted-foreground">Системная тема следует настройкам операционной системы.</p>
        </div>
        <ThemeSwitcher />
      </div>
    </section>
  </div>
</template>
