<script setup lang="ts">
/**
 * Pastille d'état arrondie (point de couleur + texte) : neutre, succès,
 * avertissement ou erreur.
 */

withDefaults(defineProps<{
  tone?: 'neutral' | 'ok' | 'warn' | 'err'
  /** `false` pour masquer le point de couleur (ex. pastille de chargement). */
  dot?: boolean
}>(), { tone: 'neutral', dot: true })

const TONES = {
  neutral: 'bg-neutral-100 text-neutral-700',
  ok: 'bg-green-50 text-green-700',
  warn: 'bg-amber-50 text-amber-700',
  err: 'bg-red-50 text-red-600',
}
</script>

<template>
  <span class="inline-flex items-center gap-[6px] h-[26px] px-[10px] rounded-full text-[12px] whitespace-nowrap" :class="TONES[tone]">
    <i v-if="dot" class="w-[7px] h-[7px] rounded-full bg-current inline-block" />
    <slot />
  </span>
</template>
