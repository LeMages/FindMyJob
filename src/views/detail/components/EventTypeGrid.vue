<script setup lang="ts">
/**
 * Choix du type d'événement (étape 1 de « Ajouter un événement ») : une
 * grille de deux colonnes par catégorie, chaque événement avec son icône, son
 * libellé et une description d'une ligne ; les issues ont une icône ambre.
 * Propre à `DetailView`.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des composants
import TimelineEventIcon from './TimelineEventIcon.vue'

// Importation des modules internes
import { eventActionLabel } from '@/composables/candidature_display'
import type { AvailableEvent } from '@/types'
import type { EventGroup } from './event_categories'

defineProps<{ groups: EventGroup[] }>()

defineEmits<{ select: [event: AvailableEvent] }>()

const { t, te } = useI18n()

/** Description d'une ligne d'un événement, vide si le type est inconnu du frontend. */
const description = (eventType: string) => {
  const key = `event.description.${eventType}`
  return te(key) ? t(key) : ''
}
</script>

<template>
  <div v-for="(group, i) in groups" :key="group.category">
    <div class="text-[11px] font-semibold text-neutral-500 font-mono tracking-[.05em] uppercase mb-2" :class="i > 0 ? 'mt-4' : ''">
      {{ t(`event.category.${group.category}`) }}
    </div>
    <div class="grid grid-cols-1 min-[560px]:grid-cols-2 gap-2">
      <button
        v-for="ev in group.items"
        :key="ev.event_type"
        type="button"
        class="flex items-center gap-3 px-[14px] py-3 border border-neutral-200 rounded-10 bg-white cursor-pointer font-sans text-inherit text-left hover:border-neutral-400 hover:bg-neutral-50"
        @click="$emit('select', ev)"
      >
        <span
          class="w-8 h-8 rounded-full flex items-center justify-center flex-none [&_svg]:w-[15px] [&_svg]:h-[15px]"
          :class="group.category === 'outcome' ? 'bg-amber-50 text-amber-700' : 'bg-neutral-100 text-neutral-700'"
        >
          <TimelineEventIcon :event-type="ev.event_type" />
        </span>
        <span class="min-w-0">
          <span class="block text-[13.5px] font-semibold text-neutral-950">{{ eventActionLabel(ev.event_type, ev.label) }}</span>
          <span v-if="description(ev.event_type)" class="block text-[11.5px] text-neutral-500 mt-px">{{ description(ev.event_type) }}</span>
        </span>
      </button>
    </div>
  </div>
</template>
