<script setup lang="ts">
import { computed } from 'vue'
import { Input } from '@/components/ui/input'
import { useWellModelStore, type ModelSetting } from '@/stores/wellModel'

const model = useWellModelStore()
const rowNumbers = computed(() => Array.from({ length: model.rows }, (_, index) => index + 1))
const columnNumbers = computed(() => Array.from({ length: model.columns }, (_, index) => index + 1))

function updateSetting(key: ModelSetting, event: Event) {
  const input = event.target as HTMLInputElement
  model.setSetting(key, Number(input.value))
  input.value = String(model[key])
}

function updateReserve(row: number, column: number, event: Event) {
  const input = event.target as HTMLInputElement
  model.setReserve(row, column, Number(input.value))
  input.value = String(model.getReserve(row, column))
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col gap-5">
    <div class="shrink-0">
      <h2 class="text-lg font-semibold tracking-tight">Параметры задачи</h2>
      <p class="mt-1 text-sm text-muted-foreground">
        Задайте геометрию залежи, число скважин и номинальные запасы каждого блока.
      </p>
    </div>

    <div class="grid min-h-0 flex-1 grid-cols-[280px_minmax(0,1fr)] gap-4">
      <section aria-labelledby="model-parameters" class="rounded-lg border bg-card p-4">
        <h3 id="model-parameters" class="mb-4 text-sm font-semibold">Параметры модели</h3>
        <div class="grid grid-cols-2 gap-3">
          <label class="space-y-1.5 text-xs font-medium">
            <span>Строки</span>
            <Input type="number" :model-value="model.rows" min="1" max="30" step="1" @change="updateSetting('rows', $event)" />
          </label>
          <label class="space-y-1.5 text-xs font-medium">
            <span>Столбцы</span>
            <Input type="number" :model-value="model.columns" min="1" max="30" step="1" @change="updateSetting('columns', $event)" />
          </label>
          <label class="space-y-1.5 text-xs font-medium">
            <span>Блоки n</span>
            <Input :model-value="model.blockCount" readonly aria-label="Число блоков" />
          </label>
          <label class="space-y-1.5 text-xs font-medium">
            <span>Скважины s</span>
            <Input type="number" :model-value="model.wellCount" min="1" :max="model.blockCount - 1" step="1" @change="updateSetting('wellCount', $event)" />
          </label>
          <label class="space-y-1.5 text-xs font-medium">
            <span>Сценарии N</span>
            <Input type="number" :model-value="model.scenarios" min="1" step="1" @change="updateSetting('scenarios', $event)" />
          </label>
          <span class="self-end pb-1 text-xs text-muted-foreground">Для оценки неопределённости</span>
          <label class="space-y-1.5 text-xs font-medium">
            <span>Параметр ε</span>
            <Input type="number" :model-value="model.epsilon" min="0" max="1" step="0.01" @change="updateSetting('epsilon', $event)" />
          </label>
          <label class="space-y-1.5 text-xs font-medium">
            <span>Вес расстояния γ</span>
            <Input type="number" :model-value="model.gamma" min="0" max="1" step="0.01" @change="updateSetting('gamma', $event)" />
          </label>
        </div>
        <p class="mt-4 border-t pt-3 text-xs leading-5 text-muted-foreground">
          ε и γ задаются от 0 до 1. Расстояния Rᵢⱼ определяются по центрам блоков прямоугольной сетки.
        </p>
      </section>

      <section aria-labelledby="block-reserves" class="flex min-h-[360px] min-w-0 flex-col overflow-hidden rounded-lg border bg-card">
        <div class="shrink-0 border-b px-4 py-3">
          <h3 id="block-reserves" class="text-sm font-semibold">Номинальные запасы Vⱼ</h3>
          <p class="mt-0.5 text-xs text-muted-foreground">Введите значение для каждого блока. Начальное значение — 100.</p>
        </div>
        <div class="min-h-0 flex-1 overflow-auto">
          <table class="w-max min-w-full border-separate border-spacing-0 text-xs">
            <thead>
              <tr>
                <th scope="col" class="sticky left-0 top-0 z-20 w-10 border-b border-r bg-card px-2 py-2 text-muted-foreground">№</th>
                <th v-for="column in columnNumbers" :key="column" scope="col" class="sticky top-0 z-10 min-w-20 border-b border-r bg-card px-2 py-2 font-medium text-muted-foreground">
                  {{ column }}
                </th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="row in rowNumbers" :key="row">
                <th scope="row" class="sticky left-0 z-10 border-b border-r bg-card px-2 py-2 font-medium text-muted-foreground">{{ row }}</th>
                <td v-for="column in columnNumbers" :key="column" class="border-b border-r p-1">
                  <Input
                    type="number"
                    :model-value="model.getReserve(row, column)"
                    min="0"
                    step="any"
                    class="w-20 text-right tabular-nums"
                    :aria-label="`Запасы блока: строка ${row}, столбец ${column}`"
                    @change="updateReserve(row, column, $event)"
                  />
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
    </div>
  </div>
</template>
