<script setup lang="ts">
import ThemeSwitcher from '@/components/ThemeSwitcher.vue'
import { ChevronDown } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { algorithmLabels, isCalculationAlgorithm, useWellModelStore } from '@/stores/wellModel'

const model = useWellModelStore()
const descriptions = {
  scenarios: 'Поиск по выборке сценариев запасов.',
  interval: 'Поиск по наихудшему сочетанию запасов.',
  exhaustive: 'Проверка всех комбинаций мест скважин по наихудшему штрафу.',
  local_search: 'Улучшение нескольких стартовых схем переносом скважин.',
}

function selectAlgorithm(value: unknown) {
  if (isCalculationAlgorithm(value)) model.algorithm = value
}
</script>

<template>
  <div class="space-y-5">
    <div>
      <h1 class="text-lg font-semibold tracking-tight">Параметры приложения</h1>
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

    <section aria-labelledby="calculation-heading" class="max-w-2xl rounded-lg border bg-card">
      <div class="border-b px-4 py-3">
        <h2 id="calculation-heading" class="text-sm font-semibold">Расчёт</h2>
      </div>
      <div class="flex flex-wrap items-center justify-between gap-4 px-4 py-3">
        <div>
          <p class="text-sm font-medium">Алгоритм расчёта</p>
          <p class="mt-0.5 text-xs text-muted-foreground">
            {{ descriptions[model.algorithm] }}
          </p>
        </div>
        <DropdownMenu>
          <DropdownMenuTrigger as-child>
            <Button variant="outline" size="sm" aria-label="Выбрать алгоритм расчёта" class="min-w-36 justify-between gap-2">
              {{ algorithmLabels[model.algorithm] }}
              <ChevronDown class="size-3 text-muted-foreground" aria-hidden="true" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" class="w-40">
            <DropdownMenuRadioGroup :model-value="model.algorithm" @update:model-value="selectAlgorithm">
              <DropdownMenuRadioItem v-for="(label, algorithm) in algorithmLabels" :key="algorithm" :value="algorithm">
                {{ label }}
              </DropdownMenuRadioItem>
            </DropdownMenuRadioGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </section>
  </div>
</template>
