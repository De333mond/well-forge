import { invoke, isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'

export function hasDesktopRuntime(): boolean {
  return isTauri()
}

export function greetFromRust(name: string): Promise<string> {
  return invoke<string>('greet', { name })
}

export function closeDesktopWindow(): Promise<void> {
  return getCurrentWindow().close()
}
