<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Minus, Plus } from '@lucide/vue'
import { useElementSize } from '@vueuse/core'
import { Button } from '@/components/ui/button'

const props = defineProps<{
  rows: number
  columns: number
  values: readonly (number | null)[]
  xMin: number
  xMax: number
  yMin: number
  yMax: number
  wells: string[]
}>()

const canvas = ref<HTMLCanvasElement | null>(null)
const viewport = ref<HTMLElement | null>(null)
const { width: viewportWidth, height: viewportHeight } = useElementSize(viewport)
const maxZoom = 24
const zoom = ref(3)
const scrollLeft = ref(0)
const scrollTop = ref(0)
const isPanning = ref(false)
let panStart: { x: number; y: number; left: number; top: number; pointerId: number } | null = null
const selected = ref<{ row: number; column: number } | null>(null)
const limits = computed(() => {
  let min = Infinity
  let max = -Infinity
  for (const value of props.values) {
    if (value !== null && Number.isFinite(value)) {
      min = Math.min(min, value)
      max = Math.max(max, value)
    }
  }
  return Number.isFinite(min) ? { min, max } : null
})
const selectedValue = computed(() => {
  if (!selected.value) return null
  return props.values[selected.value.row * props.columns + selected.value.column] ?? null
})
const formatter = new Intl.NumberFormat('ru-RU', { maximumFractionDigits: 2 })
const scaleSteps = 5
const coordinateFormatter = new Intl.NumberFormat('ru-RU', { useGrouping: false, maximumFractionDigits: 2 })
const xStep = computed(() => props.columns > 1 ? (props.xMax - props.xMin) / (props.columns - 1) : 0)
const yStep = computed(() => props.rows > 1 ? (props.yMax - props.yMin) / (props.rows - 1) : 0)

function axisTicks(min: number, max: number, step: number, origin: number, extent: number, inset: number) {
  if (step === 0) {
    return origin >= inset && origin <= extent - 40 ? [{ position: origin, label: coordinateFormatter.format(min) }] : []
  }
  const rawInterval = step * 100 / zoom.value
  const magnitude = 10 ** Math.floor(Math.log10(rawInterval))
  const normalized = rawInterval / magnitude
  const interval = (normalized <= 1 ? 1 : normalized <= 2 ? 2 : normalized <= 5 ? 5 : 10) * magnitude
  const first = Math.ceil(min / interval) * interval
  const visibleMin = min + (inset - origin) * step / zoom.value
  const visibleMax = min + (extent - 40 - origin) * step / zoom.value
  const start = Math.max(first, Math.ceil(visibleMin / interval) * interval)
  const end = Math.min(max, visibleMax)
  const ticks: { position: number; label: string }[] = []
  for (let value = start; value <= end + interval * 1e-9; value += interval) {
    ticks.push({ position: origin + (value - min) / step * zoom.value, label: coordinateFormatter.format(value) })
  }
  return ticks
}
const xTicks = computed(() => axisTicks(props.xMin, props.xMax, xStep.value, viewportWidth.value / 2 - scrollLeft.value + zoom.value / 2, viewportWidth.value, 100))
const yTicks = computed(() => axisTicks(props.yMin, props.yMax, yStep.value, viewportHeight.value / 2 - scrollTop.value + zoom.value / 2, viewportHeight.value, 35))

function updateScroll() {
  if (!viewport.value) return
  scrollLeft.value = viewport.value.scrollLeft
  scrollTop.value = viewport.value.scrollTop
}

function scaleLabel(index: number) {
  const range = limits.value
  if (!range) return ''
  if (range.min === range.max) return formatter.format(range.min)
  const step = (range.max - range.min) / scaleSteps
  const lower = range.min + step * index
  const upper = index === scaleSteps - 1 ? range.max : range.min + step * (index + 1)
  return `${formatter.format(lower)} – ${formatter.format(upper)}`
}

function draw() {
  const element = canvas.value
  const context = element?.getContext('2d')
  if (!element || !context) return
  const ratio = window.devicePixelRatio || 1
  element.width = Math.round(props.columns * zoom.value * ratio)
  element.height = Math.round(props.rows * zoom.value * ratio)
  context.scale(ratio, ratio)
  const styles = getComputedStyle(element)
  const paletteCanvas = document.createElement('canvas')
  paletteCanvas.width = 256
  paletteCanvas.height = 1
  const paletteContext = paletteCanvas.getContext('2d')
  if (!paletteContext) return
  const gradient = paletteContext.createLinearGradient(0.5, 0, 255.5, 0)
  for (let index = 0; index < scaleSteps; index++) {
    gradient.addColorStop(index / (scaleSteps - 1), styles.getPropertyValue(`--map-scale-${index + 1}`).trim())
  }
  paletteContext.fillStyle = gradient
  paletteContext.fillRect(0, 0, 256, 1)
  const pixels = paletteContext.getImageData(0, 0, 256, 1).data
  const colors = Array.from({ length: 256 }, (_, index) => `rgb(${pixels[index * 4]}, ${pixels[index * 4 + 1]}, ${pixels[index * 4 + 2]})`)
  const empty = styles.getPropertyValue('--map-empty').trim()
  const range = limits.value
  for (let row = 0; row < props.rows; row++) {
    for (let column = 0; column < props.columns; column++) {
      const value = props.values[row * props.columns + column]
      const index = value == null || !Number.isFinite(value) || !range ? -1
        : range.min === range.max ? 128 : Math.round(Math.max(0, Math.min(1, (value - range.min) / (range.max - range.min))) * 255)
      context.fillStyle = index < 0 ? empty : colors[index]!
      context.fillRect(column * zoom.value, row * zoom.value, zoom.value, zoom.value)
    }
  }
  context.strokeStyle = styles.getPropertyValue('--map-marker').trim()
  context.lineWidth = Math.max(1, zoom.value / 4)
  for (const position of props.wells) {
    const [row, column] = position.split(':').map(Number)
    if (row === undefined || column === undefined || row < 1 || row > props.rows || column < 1 || column > props.columns) continue
    context.beginPath()
    context.arc((column - 0.5) * zoom.value, (row - 0.5) * zoom.value, Math.max(2, zoom.value / 3), 0, 2 * Math.PI)
    context.stroke()
  }
  if (selected.value) {
    context.strokeStyle = styles.getPropertyValue('--map-selection').trim()
    context.lineWidth = 2
    context.strokeRect(selected.value.column * zoom.value + 1, selected.value.row * zoom.value + 1, Math.max(1, zoom.value - 2), Math.max(1, zoom.value - 2))
  }
}

function startPan(event: PointerEvent) {
  if (event.button !== 2 || !viewport.value) return
  event.preventDefault()
  panStart = { x: event.clientX, y: event.clientY, left: viewport.value.scrollLeft, top: viewport.value.scrollTop, pointerId: event.pointerId }
  isPanning.value = true
  viewport.value.setPointerCapture(event.pointerId)
}

function pan(event: PointerEvent) {
  if (!panStart || !viewport.value || event.pointerId !== panStart.pointerId) return
  viewport.value.scrollLeft = panStart.left - (event.clientX - panStart.x)
  viewport.value.scrollTop = panStart.top - (event.clientY - panStart.y)
}

function endPan(event: PointerEvent) {
  if (!panStart || event.pointerId !== panStart.pointerId) return
  panStart = null
  isPanning.value = false
  if (viewport.value?.hasPointerCapture(event.pointerId)) viewport.value.releasePointerCapture(event.pointerId)
}

function zoomAtPointer(event: WheelEvent) {
  event.preventDefault()
  if (!event.deltaY || !canvas.value || !viewport.value) return
  const nextZoom = Math.max(2, Math.min(maxZoom, zoom.value + (event.deltaY < 0 ? 1 : -1)))
  if (nextZoom === zoom.value) return
  const bounds = canvas.value.getBoundingClientRect()
  const x = (event.clientX - bounds.left) / zoom.value
  const y = (event.clientY - bounds.top) / zoom.value
  zoom.value = nextZoom
  void nextTick(() => {
    if (!canvas.value || !viewport.value) return
    const updated = canvas.value.getBoundingClientRect()
    viewport.value.scrollLeft += updated.left + x * nextZoom - event.clientX
    viewport.value.scrollTop += updated.top + y * nextZoom - event.clientY
  })
}

function selectAt(event: MouseEvent) {
  const bounds = canvas.value?.getBoundingClientRect()
  if (!bounds) return
  selected.value = {
    row: Math.min(props.rows - 1, Math.max(0, Math.floor((event.clientY - bounds.top) / zoom.value))),
    column: Math.min(props.columns - 1, Math.max(0, Math.floor((event.clientX - bounds.left) / zoom.value))),
  }
  canvas.value?.focus()
}

function moveSelection(event: KeyboardEvent) {
  const offsets: Record<string, [number, number]> = {
    ArrowUp: [-1, 0], ArrowDown: [1, 0], ArrowLeft: [0, -1], ArrowRight: [0, 1],
  }
  const offset = offsets[event.key]
  if (!offset) return
  event.preventDefault()
  const current = selected.value ?? { row: 0, column: 0 }
  selected.value = {
    row: Math.max(0, Math.min(props.rows - 1, current.row + offset[0])),
    column: Math.max(0, Math.min(props.columns - 1, current.column + offset[1])),
  }
  void nextTick(() => {
    if (!canvas.value || !viewport.value || !selected.value) return
    const bounds = canvas.value.getBoundingClientRect()
    const visible = viewport.value.getBoundingClientRect()
    const left = bounds.left + selected.value.column * zoom.value
    const top = bounds.top + selected.value.row * zoom.value
    if (left < visible.left) viewport.value.scrollLeft -= visible.left - left
    else if (left + zoom.value > visible.right) viewport.value.scrollLeft += left + zoom.value - visible.right
    if (top < visible.top) viewport.value.scrollTop -= visible.top - top
    else if (top + zoom.value > visible.bottom) viewport.value.scrollTop += top + zoom.value - visible.bottom
  })
}

let observer: MutationObserver | undefined
onMounted(() => {
  draw()
  void nextTick(() => {
    if (!viewport.value) return
    zoom.value = Math.max(2, Math.min(maxZoom, Math.floor(Math.min(
      (viewport.value.clientWidth - 120) / props.columns,
      (viewport.value.clientHeight - 60) / props.rows,
    ))))
    void nextTick(() => {
      if (!viewport.value) return
      viewport.value.scrollLeft = viewport.value.scrollWidth / 2 - viewport.value.clientWidth / 2
      viewport.value.scrollTop = viewport.value.scrollHeight / 2 - viewport.value.clientHeight / 2
      updateScroll()
    })
  })
  observer = new MutationObserver(draw)
  observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] })
})
onBeforeUnmount(() => observer?.disconnect())
watch([() => props.values, () => props.rows, () => props.columns, () => props.wells, zoom, selected], draw, { flush: 'post' })
</script>

<template>
  <section aria-label="Карта блоков" class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden rounded-lg border bg-card">
    <div class="flex shrink-0 flex-wrap items-center justify-between gap-3 border-b px-4 py-2">
      <div class="flex items-center gap-2 text-xs text-muted-foreground">
        <span>Масштаб</span>
        <Button variant="outline" size="icon" class="size-7" :disabled="zoom <= 2" aria-label="Уменьшить масштаб" @click="zoom--"><Minus class="size-3.5" /></Button>
        <span class="w-9 text-center tabular-nums">{{ zoom }} px</span>
        <Button variant="outline" size="icon" class="size-7" :disabled="zoom >= maxZoom" aria-label="Увеличить масштаб" @click="zoom++"><Plus class="size-3.5" /></Button>
      </div>
      <div v-if="limits" class="flex items-center gap-2 text-xs tabular-nums text-muted-foreground" aria-label="Цветовая шкала значений">
        <span>{{ formatter.format(limits.min) }}</span>
        <div class="flex h-2.5 w-32 rounded-sm" style="background: linear-gradient(to right, var(--map-scale-1), var(--map-scale-2), var(--map-scale-3), var(--map-scale-4), var(--map-scale-5))" role="group" aria-label="Диапазоны значений">
          <div v-for="index in scaleSteps" :key="index" class="group relative flex-1">
            <span v-if="index > 1" aria-hidden="true" class="pointer-events-none absolute inset-y-0 left-0 z-10 w-px bg-(--map-scale-divider)" />
            <button
              type="button"
              class="block h-full w-full focus-visible:relative focus-visible:z-10 focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-ring"
              :class="{ 'rounded-l-sm': index === 1, 'rounded-r-sm': index === scaleSteps }"
              :aria-label="`Запасы: ${scaleLabel(index - 1)}`"
            />
            <span role="tooltip" class="pointer-events-none absolute right-0 top-full z-20 mt-2 hidden whitespace-nowrap rounded-md border bg-popover px-2 py-1 text-popover-foreground shadow-sm group-hover:block group-focus-within:block">
              {{ scaleLabel(index - 1) }}
            </span>
          </div>
        </div>
        <span>{{ formatter.format(limits.max) }}</span>
        <span>Запасы</span>
      </div>
      <span v-else class="text-xs text-muted-foreground">Нет данных для шкалы</span>
    </div>
    <div class="relative min-h-0 min-w-0 flex-1 overflow-hidden">
    <div
      ref="viewport"
      class="h-full w-full overflow-auto"
      :class="isPanning ? 'cursor-grabbing' : 'cursor-crosshair'"
      @scroll="updateScroll"
      @pointerdown="startPan"
      @pointermove="pan"
      @pointerup="endPan"
      @pointercancel="endPan"
      @lostpointercapture="endPan"
      @contextmenu.prevent
      @wheel.prevent="zoomAtPointer"
    >
      <div class="relative" :style="{ width: `${columns * zoom + viewportWidth}px`, height: `${rows * zoom + viewportHeight}px` }">
      <canvas
        ref="canvas"
        class="absolute focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring"
        :style="{ width: `${columns * zoom}px`, height: `${rows * zoom}px`, left: `${viewportWidth / 2}px`, top: `${viewportHeight / 2}px`, imageRendering: 'pixelated' }"
        tabindex="0"
        role="img"
        :aria-label="`Карта ${rows} на ${columns} блоков. Выберите блок мышью или стрелками.`"
        @click="selectAt"
        @keydown="moveSelection"
      />
      </div>
    </div>
    <div aria-hidden="true" class="pointer-events-none absolute inset-x-0 top-0 h-7 border-b bg-card text-[10px] tabular-nums text-muted-foreground">
      <span class="absolute left-2 top-1 font-medium">X</span>
      <span v-for="tick in xTicks" :key="tick.label" class="absolute top-1 -translate-x-1/2" :style="{ left: `${tick.position}px` }">{{ tick.label }}</span>
    </div>
    <div aria-hidden="true" class="pointer-events-none absolute bottom-0 left-0 top-7 w-24 border-r bg-card text-[10px] tabular-nums text-muted-foreground">
      <span class="absolute left-2 top-1 font-medium">Y</span>
      <span v-for="tick in yTicks" :key="tick.label" class="absolute right-2 -translate-y-1/2" :style="{ top: `${tick.position - 28}px` }">{{ tick.label }}</span>
    </div>
    </div>
    <div class="min-h-9 shrink-0 border-t px-4 py-2 text-xs tabular-nums text-muted-foreground" aria-live="polite">
      <template v-if="selected">X {{ coordinateFormatter.format(xMin + selected.column * xStep) }}, Y {{ coordinateFormatter.format(yMin + selected.row * yStep) }} · Строка {{ selected.row + 1 }}, столбец {{ selected.column + 1 }} · {{ selectedValue === null ? 'Нет данных' : `Запасы: ${formatter.format(selectedValue)}` }}</template>
      <template v-else>Выберите блок мышью или стрелками. ПКМ — перемещение, колёсико — масштаб.</template>
    </div>
  </section>
</template>
