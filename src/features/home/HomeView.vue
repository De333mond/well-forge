<script setup lang="ts">
import GreetingCard from '@/features/greeting/GreetingCard.vue'
</script>

<template>
  <div class="space-y-7">
    <div class="space-y-2">
      <p class="text-xs font-semibold uppercase tracking-[0.12em] text-primary">Workspace</p>
      <h1 class="text-2xl font-semibold tracking-tight text-foreground">Welcome to Well Forge</h1>
      <p class="max-w-xl text-sm leading-6 text-muted-foreground">
        A clear starting point for your desktop app.
      </p>
    </div>

    <GreetingCard />
  </div>
</template>
