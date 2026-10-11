<script setup lang="ts">
import { ChevronDown, Monitor, Moon, Sun } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useThemeStore } from '@/stores/theme'

const theme = useThemeStore()

function selectMode(value: unknown) {
  if (value === 'light' || value === 'dark' || value === 'system') {
    theme.setMode(value)
  }
}
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button variant="outline" size="sm" aria-label="Choose theme" class="gap-2">
        <Sun v-if="theme.mode === 'light'" class="size-4" aria-hidden="true" />
        <Moon v-else-if="theme.mode === 'dark'" class="size-4" aria-hidden="true" />
        <Monitor v-else class="size-4" aria-hidden="true" />
        <span class="hidden capitalize sm:inline">{{ theme.mode }}</span>
        <ChevronDown class="size-3 text-muted-foreground" aria-hidden="true" />
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent align="end" class="w-40">
      <DropdownMenuLabel>Appearance</DropdownMenuLabel>
      <DropdownMenuSeparator />
      <DropdownMenuRadioGroup :model-value="theme.mode" @update:model-value="selectMode">
        <DropdownMenuRadioItem value="light">
          <Sun class="size-4" aria-hidden="true" />
          Light
        </DropdownMenuRadioItem>
        <DropdownMenuRadioItem value="dark">
          <Moon class="size-4" aria-hidden="true" />
          Dark
        </DropdownMenuRadioItem>
        <DropdownMenuRadioItem value="system">
          <Monitor class="size-4" aria-hidden="true" />
          System
        </DropdownMenuRadioItem>
      </DropdownMenuRadioGroup>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
