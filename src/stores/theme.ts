import { usePreferredDark, useStorage } from '@vueuse/core'
import { defineStore } from 'pinia'
import { computed } from 'vue'
import { useThemeClass } from '@/composables/useThemeClass'

export type ThemeMode = 'light' | 'dark' | 'system'

export const useThemeStore = defineStore('theme', () => {
  const mode = useStorage<ThemeMode>('well-forge-theme', 'system')
  if (mode.value !== 'light' && mode.value !== 'dark' && mode.value !== 'system') {
    mode.value = 'system'
  }

  const systemIsDark = usePreferredDark()
  const isDark = computed(() => mode.value === 'dark' || (mode.value === 'system' && systemIsDark.value))

  useThemeClass(isDark)

  function setMode(value: ThemeMode) {
    mode.value = value
  }

  return { mode, isDark, setMode }
})
