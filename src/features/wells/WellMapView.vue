<script setup lang="ts">
import { computed } from 'vue'
import { Button } from '@/components/ui/button'
import WellGrid from '@/features/wells/WellGrid.vue'
import { hasDesktopRuntime } from '@/lib/tauri'
import { algorithmLabels, useWellModelStore } from '@/stores/wellModel'

const model = useWellModelStore()
const visibleWells = computed(() => model.calculatedWells ?? [])
const canCalculate = computed(() => hasDesktopRuntime() && model.blockCount > model.wellCount)
const penaltyFormat = new Intl.NumberFormat('ru-RU', { maximumSignificantDigits: 8 })
const timeFormat = new Intl.NumberFormat('ru-RU', { maximumFractionDigits: 3, style: 'unit', unit: 'second' })
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
      <div class="flex items-center gap-3">
        <span class="text-xs tabular-nums text-muted-foreground">
          Скважины: <span class="font-semibold text-foreground">{{ visibleWells.length }} / {{ model.wellCount }}</span>
        </span>
        <Button v-if="model.calculating" size="sm" variant="outline" :disabled="model.cancelling" @click="model.cancelCalculation">
          Отменить
        </Button>
        <Button v-else size="sm" :disabled="!canCalculate" @click="model.calculate">
          Рассчитать
        </Button>
      </div>
    </div>

    <p v-if="model.calculationError" class="shrink-0 text-sm text-destructive" role="alert">{{ model.calculationError }}</p>
    <p v-if="model.calculating" class="shrink-0 text-sm text-muted-foreground" role="status">
      {{ model.cancelling ? 'Отменяем расчёт…' : 'Выполняется расчёт…' }}
    </p>
    <div v-else-if="model.calculation" class="shrink-0 rounded-md border bg-card px-3 py-2">
      <dl class="flex flex-wrap gap-x-6 gap-y-2 text-xs tabular-nums">
        <div>
          <dt class="text-muted-foreground">Наихудший штраф</dt>
          <dd class="mt-0.5 font-medium">{{ penaltyFormat.format(model.calculation.worstPenalty) }}</dd>
        </div>
        <div>
          <dt class="text-muted-foreground">Штраф при номинальных запасах</dt>
          <dd class="mt-0.5 font-medium">{{ penaltyFormat.format(model.calculation.nominalPenalty) }}</dd>
        </div>
        <div>
          <dt class="text-muted-foreground">Время расчёта</dt>
          <dd class="mt-0.5 font-medium">{{ timeFormat.format(model.calculation.elapsedMs / 1000) }}</dd>
        </div>
        <div>
          <dt class="text-muted-foreground">Алгоритм</dt>
          <dd class="mt-0.5 font-medium">{{ algorithmLabels[model.calculation.algorithm] }}</dd>
        </div>
        <div v-if="model.calculation.tailPenalty !== undefined">
          <dt class="text-muted-foreground">Среднее взвешенное расстояние в худших {{ model.tailPercent }}% сценариев</dt>
          <dd class="mt-0.5 font-medium">{{ penaltyFormat.format(model.calculation.tailPenalty) }}</dd>
        </div>
      </dl>
      <p v-if="!model.calculation.optimal" class="mt-2 text-xs text-muted-foreground">
        Найдена допустимая схема. Оптимум поиска не подтверждён.
        <template v-if="model.calculation.algorithm === 'local_search'">Блоки закреплены за ближайшими скважинами.</template>
      </p>
    </div>
    <div v-else-if="model.importedGrid && model.calculatedWells === null" class="shrink-0 text-xs text-muted-foreground" role="status">
      {{ canCalculate ? 'Алгоритм расчёта — ' + algorithmLabels[model.algorithm].toLowerCase() + '.' : 'Для расчёта нужно больше блоков с положительными запасами, чем скважин.' }}
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
      :well-by-cell="model.calculation?.wellByCell"
    />
  </div>
</template>
