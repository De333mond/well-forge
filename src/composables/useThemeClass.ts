import { watch, type ComputedRef } from 'vue'

export function useThemeClass(isDark: ComputedRef<boolean>) {
  watch(isDark, (dark) => document.documentElement.classList.toggle('dark', dark), {
    immediate: true,
  })
}
