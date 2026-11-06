import { computed, ref, shallowRef } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface Grid {
  rows: number; columns: number; values: (number | null)[]
  xMin: number; xMax: number; yMin: number; yMax: number
}
export type ModelSetting = 'wellCount' | 'scenarios' | 'epsilon' | 'gamma'
interface Project {
  grid: Grid | null; wellCount: number; scenarios: number; epsilon: number; gamma: number
  calculatedWells: string[] | null
}
interface Snapshot { project: Project; path: string | null; dirty: boolean; warning: string | null }

export const useWellModelStore = defineStore('wellModel', () => {
  const importedGrid = shallowRef<Grid | null>(null)
  const wellCount = ref(5)
  const scenarios = ref(100)
  const epsilon = ref(0.2)
  const gamma = ref(0.5)
  const calculatedWells = ref<string[] | null>(null)
  const projectPath = ref<string | null>(null)
  const dirty = ref(false)
  const warning = ref<string | null>(null)
  const blockCount = computed(() => importedGrid.value?.values.filter((value) => value !== null).length ?? 0)
  const projectName = computed(() => projectPath.value?.split(/[\\/]/).pop() ?? 'Новый проект')

  function apply(snapshot: Snapshot) {
    importedGrid.value = snapshot.project.grid
    wellCount.value = snapshot.project.wellCount
    scenarios.value = snapshot.project.scenarios
    epsilon.value = snapshot.project.epsilon
    gamma.value = snapshot.project.gamma
    calculatedWells.value = snapshot.project.calculatedWells
    projectPath.value = snapshot.path
    dirty.value = snapshot.dirty
    warning.value = snapshot.warning
  }
  async function restore() { apply(await invoke<Snapshot>('restore_project')) }
  async function importCps(path: string) { apply(await invoke<Snapshot>('import_cps', { path })) }
  async function openProject(path: string) { apply(await invoke<Snapshot>('open_project', { path })) }
  async function saveProject(path: string | null) { apply(await invoke<Snapshot>('save_project', { path })) }
  async function setSetting(key: ModelSetting, value: number) { apply(await invoke<Snapshot>('update_setting', { key, value })) }

  return { importedGrid, wellCount, scenarios, epsilon, gamma, calculatedWells, projectPath, projectName, dirty, warning, blockCount, restore, importCps, openProject, saveProject, setSetting }
})
