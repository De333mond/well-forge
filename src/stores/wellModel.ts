import { computed, ref, shallowRef } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { useStorage } from '@vueuse/core'

export interface Grid {
  rows: number; columns: number; values: (number | null)[]
  xMin: number; xMax: number; yMin: number; yMax: number
}
export type ModelSetting = 'wellCount' | 'scenarios' | 'epsilon' | 'gamma' | 'tailPercent'
export const algorithmLabels = {
  scenarios: 'Сценарный',
  interval: 'Интервальный',
  exhaustive: 'Полный перебор',
  local_search: 'Локальный поиск',
}
export type CalculationAlgorithm = keyof typeof algorithmLabels

export function isCalculationAlgorithm(value: unknown): value is CalculationAlgorithm {
  return typeof value === 'string' && Object.hasOwn(algorithmLabels, value)
}
interface Calculation {
  algorithm: CalculationAlgorithm
  worstPenalty: number
  nominalPenalty: number
  elapsedMs: number
  optimal: boolean
  tailPenalty?: number
  wellByCell?: number[]
}
interface Project {
  grid: Grid | null; wellCount: number; scenarios: number; epsilon: number; gamma: number; tailPercent: number
  calculatedWells: string[] | null
  calculation?: Calculation
}
interface Snapshot { project: Project; path: string | null; dirty: boolean; warning: string | null }

export const useWellModelStore = defineStore('wellModel', () => {
  const importedGrid = shallowRef<Grid | null>(null)
  const wellCount = ref(5)
  const scenarios = ref(100)
  const epsilon = ref(0.2)
  const gamma = ref(0.5)
  const tailPercent = ref(10)
  const calculatedWells = ref<string[] | null>(null)
  const calculation = shallowRef<Calculation | null>(null)
  const algorithm = useStorage<CalculationAlgorithm>('well-forge-calculation-algorithm', 'interval')
  if (!isCalculationAlgorithm(algorithm.value)) algorithm.value = 'interval'
  const calculating = ref(false)
  const cancelling = ref(false)
  const calculationError = ref('')
  const projectPath = ref<string | null>(null)
  const dirty = ref(false)
  const warning = ref<string | null>(null)
  const blockCount = computed(() => importedGrid.value?.values.filter((value) => value !== null && value > 0).length ?? 0)
  const projectName = computed(() => projectPath.value?.split(/[\\/]/).pop() ?? 'Новый проект')

  function apply(snapshot: Snapshot) {
    importedGrid.value = snapshot.project.grid
    wellCount.value = snapshot.project.wellCount
    scenarios.value = snapshot.project.scenarios
    epsilon.value = snapshot.project.epsilon
    gamma.value = snapshot.project.gamma
    tailPercent.value = snapshot.project.tailPercent
    calculatedWells.value = snapshot.project.calculatedWells
    calculation.value = snapshot.project.calculation ?? null
    projectPath.value = snapshot.path
    dirty.value = snapshot.dirty
    warning.value = snapshot.warning
    calculationError.value = ''
  }
  async function restore() { apply(await invoke<Snapshot>('restore_project')) }
  async function importCps(path: string) { apply(await invoke<Snapshot>('import_cps', { path })) }
  async function openProject(path: string) { apply(await invoke<Snapshot>('open_project', { path })) }
  async function saveProject(path: string | null) { apply(await invoke<Snapshot>('save_project', { path })) }
  async function setSetting(key: ModelSetting, value: number) { apply(await invoke<Snapshot>('update_setting', { key, value })) }

  async function calculate() {
    if (calculating.value) return
    calculating.value = true
    calculationError.value = ''
    try {
      apply(await invoke<Snapshot>('calculate_wells', { algorithm: algorithm.value }))
    } catch (error) {
      calculationError.value = String(error)
    } finally {
      calculating.value = false
      cancelling.value = false
    }
  }

  async function cancelCalculation() {
    cancelling.value = true
    try {
      await invoke('cancel_calculation')
    } catch (error) {
      cancelling.value = false
      calculationError.value = String(error)
    }
  }

  return {
    importedGrid, wellCount, scenarios, epsilon, gamma, tailPercent, calculatedWells, calculation, algorithm,
    calculating, cancelling, calculationError, projectPath, projectName, dirty, warning, blockCount,
    restore, importCps, openProject, saveProject, setSetting, calculate, cancelCalculation,
  }
})
