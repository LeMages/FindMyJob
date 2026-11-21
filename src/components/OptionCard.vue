<script setup lang="ts">
/**
 * Carte cliquable d'un choix (suite d'une analyse impossible, façon de
 * postuler) : icône, titre avec badge facultatif, description et flèche.
 * Mise en avant (`primary`), chargement (`busy`, spinner à la place de
 * l'icône) ou estompée (`dimmed`) pendant qu'une autre option travaille.
 */

// Importation des composants
import UiIcon, { type UiIconName } from './UiIcon.vue'

withDefaults(defineProps<{
  icon: UiIconName
  title: string
  description: string
  /** Badge affiché après le titre (« Conseillé », « Le plus efficace »). */
  badge?: string | null
  primary?: boolean
  busy?: boolean
  dimmed?: boolean
  disabled?: boolean
}>(), { badge: null, primary: false, busy: false, dimmed: false, disabled: false })

defineEmits<{ select: [] }>()
</script>

<template>
  <button
    type="button"
    class="group w-full flex items-center gap-[14px] px-[18px] py-4 border rounded-xl bg-white cursor-pointer font-sans text-inherit text-left hover:border-neutral-400 disabled:cursor-not-allowed"
    :class="[
      primary ? 'border-neutral-950 shadow-[0_0_0_1px_var(--color-neutral-950)]' : 'border-neutral-200',
      dimmed && 'opacity-40',
      disabled && !busy && !dimmed && 'opacity-40',
    ]"
    :disabled="disabled"
    @click="$emit('select')"
  >
    <span class="w-[38px] h-[38px] rounded-10 bg-neutral-100 text-neutral-700 flex items-center justify-center flex-none">
      <UiIcon :name="busy ? 'spinner' : icon" :size="18" />
    </span>
    <span class="flex-1 min-w-0">
      <span class="flex flex-wrap items-center gap-2 text-[14px] font-semibold text-neutral-950">
        {{ title }}
        <span v-if="badge" class="text-[10.5px] font-medium px-[7px] py-px rounded-full bg-neutral-100 text-neutral-700">{{ badge }}</span>
      </span>
      <span class="flex items-center gap-[5px] text-[12.5px] text-neutral-500 mt-0.5">
        <UiIcon v-if="busy" name="spark" />
        {{ description }}
      </span>
    </span>
    <span v-if="!busy" class="flex text-neutral-400 group-hover:text-neutral-950">
      <UiIcon name="arrow" />
    </span>
  </button>
</template>
