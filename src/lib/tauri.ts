import { invoke, isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { open } from '@tauri-apps/plugin-dialog'
import { readTextFile } from '@tauri-apps/plugin-fs'

export function hasDesktopRuntime(): boolean {
  return isTauri()
}

export function greetFromRust(name: string): Promise<string> {
  return invoke<string>('greet', { name })
}

export function closeDesktopWindow(): Promise<void> {
  return getCurrentWindow().close()
}

export async function openCpsFile(): Promise<{ name: string; content: string } | null> {
  const path = await open({ multiple: false, filters: [{ name: 'CPS', extensions: ['cps'] }] })
  if (!path) return null
  return { name: path.split(/[\\/]/).pop() ?? path, content: await readTextFile(path) }
}
