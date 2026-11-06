import { invoke, isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { open, save, confirm } from '@tauri-apps/plugin-dialog'

export function hasDesktopRuntime(): boolean {
  return isTauri()
}

export function greetFromRust(name: string): Promise<string> {
  return invoke<string>('greet', { name })
}

export function closeDesktopWindow(): Promise<void> {
  return getCurrentWindow().close()
}

export function openCpsFile(): Promise<string | null> {
  return open({ multiple: false, filters: [{ name: 'CPS', extensions: ['cps'] }] })
}

export function openProjectFile(): Promise<string | null> {
  return open({ multiple: false, filters: [{ name: 'Well Forge', extensions: ['wellforge'] }] })
}

export function saveProjectFile(): Promise<string | null> {
  return save({ defaultPath: 'project.wellforge', filters: [{ name: 'Well Forge', extensions: ['wellforge'] }] })
}

export function confirmDiscard(): Promise<boolean> {
  return confirm('В проекте есть несохранённые изменения. Продолжить без сохранения?', { title: 'Несохранённые изменения', kind: 'warning' })
}
