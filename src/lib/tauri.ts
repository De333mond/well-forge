import { invoke, isTauri } from '@tauri-apps/api/core'

export function hasDesktopRuntime(): boolean {
  return isTauri()
}

export function greetFromRust(name: string): Promise<string> {
  return invoke<string>('greet', { name })
}
