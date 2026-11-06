<script setup lang="ts">
import { Input } from '@/components/ui/input'
import { useWellModelStore, type ModelSetting } from '@/stores/wellModel'

const model = useWellModelStore()

function updateSetting(key: ModelSetting, event: Event) {
  const input = event.target as HTMLInputElement
  model.setSetting(key, Number(input.value))
  input.value = String(model[key])
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col gap-5">
    <div class="shrink-0">
      <h2 class="text-lg font-semibold tracking-tight">Параметры задачи</h2>
      <p class="mt-1 text-sm text-muted-foreground">
        Задайте параметры расчёта и число скважин.
      </p>
    </div>

    <div>
      <section aria-labelledby="model-parameters" class="w-full max-w-sm rounded-lg border bg-card p-4">
        <h3 id="model-parameters" class="mb-4 text-sm font-semibold">Параметры модели</h3>
        <div class="grid grid-cols-2 gap-3">
          <label class="space-y-1.5 text-xs font-medium">
            <span>Скважины s</span>
            <Input type="number" :model-value="model.wellCount" min="1" :max="Math.max(1, model.blockCount - 1)" step="1" @change="updateSetting('wellCount', $event)" />
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
    </div>
  </div>
</template>
