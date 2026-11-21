<script setup lang="ts">
/**
 * Ligne d'une expérience, d'une formation ou d'un projet : titre, pastilles
 * et période alignée à droite, sous-titre, description, lien, et boutons
 * Modifier / Supprimer pendant l'édition de la carte.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'

// Importation des composants
import ContextIndicator from './ContextIndicator.vue'
import PendingBadge from './PendingBadge.vue'
import RowEditButtons from './RowEditButtons.vue'

// Importation des composables
import { parseDescriptionList } from '@/composables/parseDescriptionList'

const props = defineProps<{
  title: string
  /** Période affichée à droite du titre. */
  period: string
  badge?: string | null
  /** Ligne secondaire (entreprise et lieu, école…). */
  sub?: string | null
  /** Ligne en police mono (stack d'un projet). */
  mono?: string | null
  /** Description saisie : lignes « - » = éléments de liste (voir `parseDescriptionList`). */
  description?: string | null
  url?: string | null
  /** Contexte détaillé : `undefined` masque l'icône, sinon grise ou ambre selon sa présence. */
  contexte?: string | null
  pendingKind?: 'new' | 'edited'
  editing: boolean
  last: boolean
}>()

const emit = defineEmits<{
  edit: []
  remove: []
  'open-url': [url: string]
}>()

/** Intro et éléments de la description, toujours affichés avec des puces (quel que soit le format de session). */
const descriptionList = computed(() => parseDescriptionList(props.description))
const urlLabel = computed(() => props.url?.replace(/^https?:\/\//, '') ?? '')
</script>

<template>
  <div class="flex gap-[14px] py-[14px]" :class="[last ? 'pb-0.5' : 'border-b border-neutral-100', pendingKind && 'opacity-[0.72]']">
    <div class="flex-1 min-w-0">
      <div class="flex items-baseline gap-3">
        <div class="flex items-center gap-2 flex-wrap flex-1 min-w-0">
          <div class="text-[14px] font-semibold">{{ title }}</div>
          <span v-if="badge" class="inline-block font-mono text-[10.5px] border border-neutral-300 rounded-5 px-[7px] py-0.5 text-neutral-700">{{ badge }}</span>
          <ContextIndicator v-if="contexte !== undefined" :provided="!!contexte" />
          <PendingBadge v-if="pendingKind" :kind="pendingKind" />
        </div>
        <div class="font-mono text-[11.5px] text-neutral-400 whitespace-nowrap flex-none">{{ period }}</div>
      </div>
      <div v-if="sub" class="text-[13px] text-neutral-500 mt-px">{{ sub }}</div>
      <div v-if="mono" class="font-mono text-[11px] text-neutral-600 mt-[3px]">{{ mono }}</div>
      <div
        v-if="descriptionList.intro.length"
        class="text-[13px] text-neutral-700 mt-[7px] leading-[1.55] whitespace-pre-line"
      >{{ descriptionList.intro.join('\n') }}</div>
      <ul
        v-if="descriptionList.items.length"
        class="text-[13px] text-neutral-700 mt-[7px] leading-[1.6] pl-[1.15em] list-disc mb-0 [&>li+li]:mt-[5px]"
      >
        <li v-for="(item, ii) in descriptionList.items" :key="ii" class="whitespace-pre-line">{{ item.join('\n') }}</li>
      </ul>
      <button
        v-if="url"
        class="text-[11.5px] text-blue-500 bg-transparent border-none p-0 cursor-pointer inline-flex items-center gap-1 mt-[6px] text-left break-all hover:underline"
        type="button"
        @click="emit('open-url', url)"
      >
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" class="flex-none"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" /><polyline points="15 3 21 3 21 9" /><line x1="10" y1="14" x2="21" y2="3" /></svg>
        {{ urlLabel }}
      </button>
    </div>
    <RowEditButtons v-if="editing" @edit="emit('edit')" @remove="emit('remove')" />
  </div>
</template>
