import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import type { CpsGrid } from '@/features/import/cps'

export type ModelSetting = 'rows' | 'columns' | 'wellCount' | 'scenarios' | 'epsilon' | 'gamma'

export const useWellModelStore = defineStore('wellModel', () => {
  const rows = ref(0)
  const columns = ref(0)
  const importedGrid = ref<CpsGrid | null>(null)
  const importedFileName = ref('')
  const wellCount = ref(5)
  const scenarios = ref(100)
  const epsilon = ref(0.2)
  const gamma = ref(0.5)
  const reserves = ref<Record<string, number>>({})
  // The solver can populate this when the penalty formula is specified.
  const calculatedWells = ref<string[] | null>(null)
  const blockCount = computed(() => importedGrid.value?.values.filter((value) => value !== null).length ?? 0)

  const key = (row: number, column: number) => `${row}:${column}`

  function setImportedGrid(grid: CpsGrid, fileName: string) {
    importedGrid.value = grid
    importedFileName.value = fileName
    rows.value = grid.rows
    columns.value = grid.columns
    wellCount.value = Math.min(wellCount.value, Math.max(1, blockCount.value - 1))
    calculatedWells.value = null
  }

  function setSetting(setting: ModelSetting, rawValue: number) {
    const value = Number.isFinite(rawValue) ? rawValue : 0
    if (setting === 'rows' || setting === 'columns') {
      const otherSize = setting === 'rows' ? columns.value : rows.value
      const next = Math.min(30, Math.max(otherSize === 1 ? 2 : 1, Math.trunc(value)))
      if (setting === 'rows') rows.value = next
      else columns.value = next
      wellCount.value = Math.min(wellCount.value, blockCount.value - 1)
    } else if (setting === 'wellCount') {
      wellCount.value = Math.min(Math.max(1, blockCount.value - 1), Math.max(1, Math.trunc(value)))
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

  return {
    rows, columns, blockCount, wellCount, scenarios, epsilon, gamma, importedGrid, importedFileName, setImportedGrid,
    calculatedWells, setSetting, getReserve, setReserve,
  }
})
