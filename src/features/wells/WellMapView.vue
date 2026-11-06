<script setup lang="ts">
import { computed } from 'vue'
import WellGrid from '@/features/wells/WellGrid.vue'
import { useWellModelStore } from '@/stores/wellModel'

const model = useWellModelStore()
const visibleWells = computed(() => model.calculatedWells ?? [])
// Demonstrates the map until block data can be imported.
const demoSize = 200
const demoValues = Array.from({ length: demoSize * demoSize }, (_, index) => {
  const x = (index % demoSize - 100) / 78
  const y = (Math.floor(index / demoSize) - 100) / 65
  const distance = x * x + y * y
  return distance <= 1 ? Math.round(1000 * (1 - distance)) : null
})
</script>

<template>
  <div class="flex h-full min-h-[440px] flex-col gap-4">
    <div class="flex shrink-0 flex-wrap items-start justify-between gap-3">
      <div>
        <h2 class="text-lg font-semibold tracking-tight">Карта скважин</h2>
        <p class="mt-1 text-sm text-muted-foreground">
          Демонстрационная сетка 200 × 200. Данные будут заменены после импорта.
        </p>
      </div>
      <div class="rounded-md border bg-card px-3 py-1.5 text-xs tabular-nums text-muted-foreground">
        Скважины: <span class="font-semibold text-foreground">{{ visibleWells.length }} / {{ model.wellCount }}</span>
      </div>
    </div>

    <div v-if="model.calculatedWells === null" class="shrink-0 rounded-md border bg-muted px-3 py-2 text-xs text-muted-foreground" role="status">
      Результат расчёта пока отсутствует. Формула штрафа для модели ещё не задана в постановке задачи.
    </div>

    <WellGrid
      :rows="demoSize"
      :columns="demoSize"
      :values="demoValues"
      :wells="visibleWells"
    />
  </div>
</template>
