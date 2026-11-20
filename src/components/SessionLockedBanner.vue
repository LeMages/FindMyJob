<script setup lang="ts">
/**
 * Bandeau discret affiché en haut d'une session en pause ou terminée (page
 * Candidatures et fiche d'une candidature) : « lecture seule », avec un lien
 * vers l'onglet Paramètres où se change le statut. N'affiche rien pour une
 * session active.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composables
import { SESSION_STATUS_DOT_CLASS } from '@/composables/session_status_dot'

// Importation des modules internes
import { state, openSessionSettings } from '@/stores/app'
import type { SessionStatut } from '@/types'

const props = defineProps<{
  /** Statut de la session affichée. */
  statut: SessionStatut
}>()

const { t } = useI18n()

/** Le lien est inutile quand l'onglet Paramètres est déjà ouvert. */
const showLink = computed(() => !(state.screen === 'tracking' && state.trackingTab === 'parametres'))

const message = computed(() => (props.statut === 'pause' ? t('session.locked.banner.pause') : t('session.locked.banner.termine')))
</script>

<template>
  <div
    v-if="statut !== 'active'"
    class="flex flex-wrap items-center gap-x-3 gap-y-1 px-[14px] py-2 rounded-lg border border-neutral-200 bg-neutral-50 text-[12.5px] text-neutral-600"
    role="status"
  >
    <span class="w-[6px] h-[6px] rounded-sm flex-none" :class="SESSION_STATUS_DOT_CLASS[statut]" />
    <span class="flex-1 min-w-0">{{ message }}</span>
    <button
      v-if="showLink"
      type="button"
      class="bg-transparent border-none p-0 text-[12.5px] font-medium text-neutral-950 underline underline-offset-2 cursor-pointer font-sans"
      @click="openSessionSettings"
    >{{ t('session.locked.change_status') }}</button>
  </div>
</template>
