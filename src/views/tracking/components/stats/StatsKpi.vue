<script setup lang="ts">
/**
 * Chiffre clé de l'onglet Statistiques : libellé, valeur, précision.
 */

defineProps<{
  label: string
  value: string
  sub: string
}>()
</script>

<template>
  <div class="bg-white border border-neutral-200 rounded-xl px-[18px] py-4 min-w-0">
    <div class="text-[12px] text-neutral-500">{{ label }}</div>
    <div class="text-[28px] font-semibold text-neutral-950 mt-1 tracking-[-.01em]">{{ value }}</div>
    <div class="text-[11.5px] text-neutral-500 font-mono mt-0.5">{{ sub }}</div>
  </div>
</template>
