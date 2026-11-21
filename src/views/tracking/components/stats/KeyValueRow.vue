<script setup lang="ts">
/**
 * Ligne « libellé … valeur » des cartes Durées et Relances.
 */

defineProps<{
  label: string
  value: string
}>()
</script>

<template>
  <div class="flex justify-between gap-3 py-[7px] border-b border-neutral-100 last:border-b-0 text-[12.5px] text-neutral-600">
    <span>{{ label }}</span>
    <b class="text-neutral-950 font-semibold whitespace-nowrap">{{ value }}</b>
  </div>
</template>
