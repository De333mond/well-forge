<script setup lang="ts">
import { ArrowRight, Terminal } from '@lucide/vue'
import { ref } from 'vue'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { greetFromRust, hasDesktopRuntime } from '@/lib/tauri'

const name = ref('')
const message = ref('')
const isBusy = ref(false)
const isDesktop = hasDesktopRuntime()

async function submit() {
  const value = name.value.trim()
  if (!value || isBusy.value) return

  if (!isDesktop) {
    message.value = 'Open the desktop app to try the Rust command.'
    return
  }

  isBusy.value = true
  try {
    message.value = await greetFromRust(value)
  } catch {
    message.value = 'The greeting could not be loaded.'
  } finally {
    isBusy.value = false
  }
}
</script>

<template>
  <Card>
    <CardHeader>
      <div class="mb-2 flex size-9 items-center justify-center rounded-lg bg-secondary text-secondary-foreground">
        <Terminal class="size-4" aria-hidden="true" />
      </div>
      <CardTitle>Say hello</CardTitle>
      <CardDescription>A small example of the desktop connection.</CardDescription>
    </CardHeader>
    <CardContent class="space-y-3">
      <form class="flex flex-col gap-2 sm:flex-row" @submit.prevent="submit">
        <Input v-model="name" aria-label="Your name" placeholder="Your name" required />
        <Button type="submit" :disabled="isBusy" class="sm:shrink-0">
          Send greeting
          <ArrowRight class="size-4" aria-hidden="true" />
        </Button>
      </form>
      <p class="min-h-5 text-sm text-muted-foreground" role="status" aria-live="polite">
        {{ message || 'Enter your name to test the connection.' }}
      </p>
    </CardContent>
  </Card>
</template>
