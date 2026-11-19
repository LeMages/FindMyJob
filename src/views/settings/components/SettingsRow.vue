<script setup lang="ts">
/**
 * Ligne de réglage : libellé et aide à gauche, contrôle à droite, contenu
 * facultatif en dessous. Libellé et contrôle s'empilent quand la ligne est
 * trop étroite (requête de conteneur).
 */

defineProps<{
  label: string
  help?: string
}>()
</script>

<template>
  <div class="px-5 py-4 border-b border-neutral-100 last:border-b-0 @container">
    <div class="flex flex-col items-start gap-3 @[560px]:flex-row @[560px]:items-center @[560px]:gap-6">
      <div class="flex-1 min-w-0">
        <div class="text-[13.5px] font-medium text-neutral-950">{{ label }}</div>
        <div v-if="help" class="text-[12.5px] text-neutral-500 mt-0.5 leading-[1.45] max-w-[440px]">{{ help }}</div>
      </div>
      <div v-if="$slots.default" class="flex flex-wrap items-center gap-2 flex-none">
        <slot />
      </div>
    </div>
    <slot name="below" />
  </div>
</template>
