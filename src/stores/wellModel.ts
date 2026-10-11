import { computed, ref } from 'vue'
import { defineStore } from 'pinia'

export type ModelSetting = 'rows' | 'columns' | 'wellCount' | 'scenarios' | 'epsilon' | 'gamma'

export const useWellModelStore = defineStore('wellModel', () => {
  const rows = ref(8)
  const columns = ref(10)
  const wellCount = ref(5)
  const scenarios = ref(100)
  const epsilon = ref(0.2)
  const gamma = ref(0.5)
  const reserves = ref<Record<string, number>>({})
  const initialWells = ref<string[]>([])
  // The solver can populate this when the penalty formula is specified.
  const calculatedWells = ref<string[] | null>(null)
  const blockCount = computed(() => rows.value * columns.value)

  const key = (row: number, column: number) => `${row}:${column}`

  function setSetting(setting: ModelSetting, rawValue: number) {
    const value = Number.isFinite(rawValue) ? rawValue : 0
    if (setting === 'rows' || setting === 'columns') {
      const otherSize = setting === 'rows' ? columns.value : rows.value
      const next = Math.min(30, Math.max(otherSize === 1 ? 2 : 1, Math.trunc(value)))
      if (setting === 'rows') rows.value = next
      else columns.value = next
      wellCount.value = Math.min(wellCount.value, blockCount.value - 1)
      initialWells.value = initialWells.value.filter((position) => {
        const [row, column] = position.split(':').map(Number)
        return row !== undefined && column !== undefined && row <= rows.value && column <= columns.value
      }).slice(0, wellCount.value)
    } else if (setting === 'wellCount') {
      wellCount.value = Math.min(blockCount.value - 1, Math.max(1, Math.trunc(value)))
      initialWells.value = initialWells.value.slice(0, wellCount.value)
    } else if (setting === 'scenarios') {
      scenarios.value = Math.max(1, Math.trunc(value))
    } else if (setting === 'epsilon') {
      epsilon.value = Math.min(1, Math.max(0, value))
    } else {
      gamma.value = Math.min(1, Math.max(0, value))
    }
    calculatedWells.value = null
  }

  function getReserve(row: number, column: number) {
    return reserves.value[key(row, column)] ?? 100
  }

  function setReserve(row: number, column: number, rawValue: number) {
    if (!Number.isFinite(rawValue) || rawValue < 0) return
    reserves.value[key(row, column)] = rawValue
    calculatedWells.value = null
  }

  function toggleInitialWell(row: number, column: number) {
    const position = key(row, column)
    if (initialWells.value.includes(position)) {
      initialWells.value = initialWells.value.filter((well) => well !== position)
    } else if (initialWells.value.length < wellCount.value) {
      initialWells.value = [...initialWells.value, position]
    } else {
      return false
    }
    calculatedWells.value = null
    return true
  }

  return {
    rows, columns, blockCount, wellCount, scenarios, epsilon, gamma,
    initialWells, calculatedWells, setSetting, getReserve, setReserve, toggleInitialWell,
  }
})
