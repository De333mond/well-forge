<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import WellGrid from '@/features/wells/WellGrid.vue'
import { useWellModelStore } from '@/stores/wellModel'

const model = useWellModelStore()
const route = useRoute()
const isInitial = computed(() => route.path === '/initial-placement')
const visibleWells = computed(() => isInitial.value ? model.initialWells : model.calculatedWells ?? [])
const message = ref('')

function toggle(row: number, column: number) {
  if (!isInitial.value) return
  message.value = model.toggleInitialWell(row, column)
    ? ''
    : `Можно разместить не более ${model.wellCount} скважин.`
}
</script>

<template>
  <div class="flex h-full min-h-[440px] flex-col gap-4">
    <div class="flex shrink-0 flex-wrap items-start justify-between gap-3">
      <div>
        <h2 class="text-lg font-semibold tracking-tight">{{ isInitial ? 'Начальное расположение' : 'Карта скважин' }}</h2>
        <p class="mt-1 text-sm text-muted-foreground">
          {{ isInitial ? 'Нажмите на блок, чтобы поставить или убрать скважину.' : 'Расположение скважин после расчёта модели.' }}
        </p>
      </div>
      <div class="rounded-md border bg-card px-3 py-1.5 text-xs tabular-nums text-muted-foreground">
        Скважины: <span class="font-semibold text-foreground">{{ visibleWells.length }} / {{ model.wellCount }}</span>
        <span class="mx-2 text-border">|</span>
        Блоки: <span class="font-semibold text-foreground">{{ model.blockCount }}</span>
      </div>
    </div>

    <p v-if="isInitial && message" class="shrink-0 text-xs text-destructive" role="status">{{ message }}</p>
    <div v-if="!isInitial && model.calculatedWells === null" class="shrink-0 rounded-md border bg-muted px-3 py-2 text-xs text-muted-foreground" role="status">
      Результат расчёта пока отсутствует. Формула штрафа для модели ещё не задана в постановке задачи.
    </div>

    <WellGrid
      :rows="model.rows"
      :columns="model.columns"
      :wells="visibleWells"
      :interactive="isInitial"
      @toggle="toggle"
    />
  </div>
</template>
