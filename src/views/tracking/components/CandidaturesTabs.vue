<script setup lang="ts">
/**
 * Barre d'onglets de la page Candidatures : bascule entre le pipeline de
 * candidatures, le planning des entretiens et l'onglet Paramètres.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

defineProps<{
  active: 'candidatures' | 'planning' | 'parametres'
}>()

defineEmits<{
  select: [tab: 'candidatures' | 'planning' | 'parametres']
}>()
</script>

<template>
  <div class="px-[30px] flex items-center gap-1 border-b border-neutral-200">
    <button
      class="px-4 py-2 text-[13px] font-medium border-b-2 -mb-px cursor-pointer font-sans"
      :class="active === 'candidatures' ? 'border-neutral-950 text-neutral-950' : 'border-transparent text-neutral-500 hover:text-neutral-800'"
      @click="$emit('select', 'candidatures')"
    >
      {{ t('tracking.tabs.applications') }}
    </button>
    <button
      class="px-4 py-2 text-[13px] font-medium border-b-2 -mb-px cursor-pointer font-sans"
      :class="active === 'planning' ? 'border-neutral-950 text-neutral-950' : 'border-transparent text-neutral-500 hover:text-neutral-800'"
      @click="$emit('select', 'planning')"
    >
      {{ t('tracking.tabs.planning') }}
    </button>
    <button
      class="px-4 py-2 text-[13px] font-medium border-b-2 -mb-px cursor-pointer font-sans"
      :class="active === 'parametres' ? 'border-neutral-950 text-neutral-950' : 'border-transparent text-neutral-500 hover:text-neutral-800'"
      @click="$emit('select', 'parametres')"
    >
      {{ t('tracking.tabs.settings') }}
    </button>
  </div>
</template>
