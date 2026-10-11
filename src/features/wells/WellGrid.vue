<script setup lang="ts">
import { computed, ref } from 'vue'
import { useElementSize } from '@vueuse/core'

const props = defineProps<{
  rows: number
  columns: number
  wells: string[]
  interactive?: boolean
}>()
const emit = defineEmits<{ toggle: [row: number, column: number] }>()
const rowNumbers = computed(() => Array.from({ length: props.rows }, (_, index) => index + 1))
const columnNumbers = computed(() => Array.from({ length: props.columns }, (_, index) => index + 1))
const wellPositions = computed(() => new Set(props.wells))
const viewport = ref<HTMLElement | null>(null)
const { width, height } = useElementSize(viewport)
const cellSize = computed(() => Math.max(42, Math.floor(Math.min(
  (width.value - 62) / props.columns,
  (height.value - 59) / props.rows,
))))
const gridWidth = computed(() => `${props.columns * cellSize.value + 28}px`)

function hasWell(row: number, column: number) {
  return wellPositions.value.has(`${row}:${column}`)
}

function moveFocus(event: KeyboardEvent, row: number, column: number) {
  const offsets: Record<string, [number, number]> = {
    ArrowUp: [-1, 0], ArrowDown: [1, 0], ArrowLeft: [0, -1], ArrowRight: [0, 1],
  }
  const offset = offsets[event.key]
  if (!offset) return
  const nextRow = row + offset[0]
  const nextColumn = column + offset[1]
  if (nextRow < 1 || nextRow > props.rows || nextColumn < 1 || nextColumn > props.columns) return
  event.preventDefault()
  viewport.value?.querySelector<HTMLButtonElement>(`[data-row="${nextRow}"][data-column="${nextColumn}"]`)?.focus()
}
</script>

<template>
  <div ref="viewport" class="min-h-0 min-w-0 flex-1 overflow-auto rounded-lg border bg-card p-4">
    <div class="mx-auto w-max">
      <div class="mb-1.5 flex pl-7 text-[11px] tabular-nums text-muted-foreground" :style="{ width: gridWidth }">
        <span v-for="column in columnNumbers" :key="column" class="text-center" :style="{ width: `${cellSize}px` }">{{ column }}</span>
      </div>
      <div v-for="row in rowNumbers" :key="row" class="flex" :style="{ width: gridWidth }">
        <span class="flex w-7 shrink-0 items-center justify-center text-[11px] tabular-nums text-muted-foreground">{{ row }}</span>
        <div class="grid" :style="{ gridTemplateColumns: `repeat(${columns}, ${cellSize}px)` }">
          <button
            v-for="column in columnNumbers"
            :key="column"
            type="button"
            class="relative -mr-px -mb-px min-w-0 border bg-background motion-safe:transition-colors focus-visible:z-10 focus-visible:outline-2 focus-visible:outline-ring"
            :style="{ width: `${cellSize}px`, height: `${cellSize}px` }"
            :data-row="row"
            :data-column="column"
            :class="interactive ? 'cursor-pointer hover:bg-muted active:bg-accent' : 'cursor-default'"
            :disabled="!interactive"
            :aria-pressed="interactive ? hasWell(row, column) : undefined"
            :aria-label="`Блок ${row}, ${column}${hasWell(row, column) ? ', скважина' : ''}`"
            @click="emit('toggle', row, column)"
            @keydown="moveFocus($event, row, column)"
          >
            <span v-if="hasWell(row, column)" class="absolute inset-0 flex items-center justify-center" aria-hidden="true">
              <span class="size-3 rounded-full border-[3px] border-foreground bg-card" />
            </span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
