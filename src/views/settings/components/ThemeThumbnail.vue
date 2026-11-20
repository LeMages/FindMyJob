<script setup lang="ts">
/**
 * Miniature d'un thème (barre latérale, titre, carte). Les couleurs sont
 * fixes : la miniature montre le thème qu'elle représente, quel que soit le
 * thème actif.
 */

// Importation des modules internes
import type { ThemeSetting } from '@/types'

defineProps<{
  theme: ThemeSetting
  selected: boolean
}>()

/** Fond de la miniature et couleurs de ses trois blocs (barre, titre, carte). */
const PALETTES: Record<ThemeSetting, { bg: string; parts: [string, string, string]; card: string }> = {
  clair: { bg: '#f7f6f3', parts: ['#ebe9e3', '#1a1a18', '#fff'], card: '1px solid #e3e1db' },
  sombre: { bg: '#1d1d1b', parts: ['#262624', '#f0f0ed', '#2a2a27'], card: '1px solid #3a3a37' },
  systeme: { bg: 'linear-gradient(135deg,#f7f6f3 50%,#1d1d1b 50%)', parts: ['#ebe9e3', '#8a8a85', 'rgba(127,127,127,.25)'], card: 'none' },
}
</script>

<template>
  <span
    class="block relative overflow-hidden w-[84px] h-[54px] rounded-9 border"
    :class="selected ? 'border-neutral-950 shadow-[0_0_0_1px_var(--color-neutral-950)]' : 'border-neutral-200'"
    :style="{ background: PALETTES[theme].bg }"
  >
    <span class="absolute rounded-[3px] left-0 top-0 bottom-0 w-[22px]" :style="{ background: PALETTES[theme].parts[0] }" />
    <span class="absolute rounded-[3px] left-[30px] top-[10px] w-10 h-[6px]" :style="{ background: PALETTES[theme].parts[1] }" />
    <span class="absolute rounded-[3px] left-[30px] top-[22px] w-[46px] h-[22px]" :style="{ background: PALETTES[theme].parts[2], border: PALETTES[theme].card }" />
  </span>
</template>
