<script setup lang="ts">
import { Input } from '@/components/ui/input'
import { useWellModelStore, type ModelSetting } from '@/stores/wellModel'
import { hasDesktopRuntime } from '@/lib/tauri'
import { ref } from 'vue'

const model = useWellModelStore()
const error = ref('')
const updating = ref(false)
const isDesktop = hasDesktopRuntime()

async function updateSetting(key: ModelSetting, event: Event) {
  const input = event.target as HTMLInputElement
  updating.value = true
  try {
    await model.setSetting(key, Number(input.value))
    error.value = ''
  } catch (cause) {
    error.value = `Не удалось изменить параметр: ${String(cause)}`
  } finally {
    input.value = String(model[key])
    updating.value = false
  }
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

    <p v-if="error" role="alert" class="text-sm text-destructive">{{ error }}</p>
    <p v-if="!isDesktop" class="text-sm text-muted-foreground">Для работы с проектом запустите приложение Tauri.</p>
    <div>
      <section aria-labelledby="model-parameters" class="w-full max-w-sm rounded-lg border bg-card p-4">
        <h3 id="model-parameters" class="mb-4 text-sm font-semibold">Параметры модели</h3>
        <fieldset :disabled="!isDesktop || updating" class="grid grid-cols-2 gap-3 disabled:opacity-60">
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
        </fieldset>
        <p class="mt-4 border-t pt-3 text-xs leading-5 text-muted-foreground">
          ε и γ задаются от 0 до 1. Расстояния Rᵢⱼ определяются по центрам блоков прямоугольной сетки.
        </p>
      </section>
    </div>
  </div>
</template>
