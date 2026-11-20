<script setup lang="ts">
/**
 * Barre d'onglets de la page Candidatures : tableau des candidatures,
 * planning des entretiens, statistiques et paramètres de la session.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des modules internes
import type { TrackingTab } from '@/types'

const { t } = useI18n()

defineProps<{
  active: TrackingTab
}>()

defineEmits<{
  select: [tab: TrackingTab]
}>()

const TABS: { id: TrackingTab; key: string }[] = [
  { id: 'candidatures', key: 'tracking.tabs.applications' },
  { id: 'planning', key: 'tracking.tabs.planning' },
  { id: 'stats', key: 'tracking.tabs.stats' },
  { id: 'parametres', key: 'tracking.tabs.settings' },
]
</script>

<template>
  <div class="px-5 sm:px-[30px] flex items-center gap-1 border-b border-neutral-200 overflow-x-auto">
    <button
      v-for="tab in TABS"
      :key="tab.id"
      class="px-4 py-2 text-[13px] font-medium border-b-2 -mb-px cursor-pointer font-sans whitespace-nowrap"
      :class="active === tab.id ? 'border-neutral-950 text-neutral-950' : 'border-transparent text-neutral-500 hover:text-neutral-800'"
      @click="$emit('select', tab.id)"
    >
      {{ t(tab.key) }}
    </button>
  </div>
</template>
