<script setup lang="ts">
import { computed } from 'vue'
import WellGrid from '@/features/wells/WellGrid.vue'
import { useWellModelStore } from '@/stores/wellModel'

const model = useWellModelStore()
const visibleWells = computed(() => model.calculatedWells ?? [])
</script>

<template>
  <div class="flex h-full min-h-[440px] flex-col gap-4">
    <div class="flex shrink-0 flex-wrap items-start justify-between gap-3">
      <div>
        <h2 class="text-lg font-semibold tracking-tight">Карта скважин</h2>
        <p class="mt-1 text-sm text-muted-foreground">
          {{ model.importedGrid ? model.projectName : 'Импортируйте файл CPS через меню «Файл».' }}
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
      v-if="model.importedGrid"
      :key="model.projectName + ':' + model.importedGrid.rows + ':' + model.importedGrid.columns"
      :rows="model.importedGrid.rows"
      :columns="model.importedGrid.columns"
      :values="model.importedGrid.values"
      :x-min="model.importedGrid.xMin"
      :x-max="model.importedGrid.xMax"
      :y-min="model.importedGrid.yMin"
      :y-max="model.importedGrid.yMax"
      :wells="visibleWells"
    />
  </div>
</template>
