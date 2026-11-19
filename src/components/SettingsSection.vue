<script setup lang="ts">
/**
 * Section des Paramètres : titre, phrase d'explication, action facultative
 * dans l'en-tête, et panneau regroupant les lignes de réglage.
 */

defineProps<{
  title: string
  description?: string
}>()
</script>

<template>
  <section class="mb-[30px]">
    <div class="flex flex-wrap items-end justify-between gap-x-4 gap-y-2 mb-[10px]">
      <div class="min-w-0">
        <h2 class="text-[15px] font-semibold text-neutral-950 m-0">{{ title }}</h2>
        <p v-if="description" class="text-[12.5px] text-neutral-500 mt-[3px] mb-0">{{ description }}</p>
      </div>
      <slot name="action" />
    </div>
    <div class="border border-neutral-200 rounded-xl bg-surface-1">
      <slot />
    </div>
  </section>
</template>
