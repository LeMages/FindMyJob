<script setup lang="ts">
/**
 * Liste des sessions de la barre latérale (pastille de statut, nom, compteur
 * des candidatures non clôturées, croix de suppression au survol) suivie de « Nouvelle session ». Utilisée sous
 * l'entrée Candidatures (barre dépliée) et dans le panneau survolé du rail
 * (barre repliée, sans croix de suppression).
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des composables
import { SESSION_STATUS_DOT_CLASS } from '@/composables/session_status_dot'

// Importation des modules internes
import { state, allSessions, selectSession, addSession, openDeleteSessionPopup } from '@/stores/app'

defineProps<{
  /** `true` dans le panneau du rail replié : pas de croix de suppression. */
  compact?: boolean
  /** `true` si l'écran courant appartient aux candidatures (session surlignée). */
  inApplications: boolean
}>()

const { t } = useI18n()
</script>

<template>
  <div
    v-for="s in allSessions"
    :key="s.id"
    class="group flex items-center gap-[9px] py-[6px] px-2 rounded-7 text-[13px] cursor-pointer select-none"
    :class="inApplications && s.id === state.sessionId
      ? 'bg-surface-1 text-neutral-950 font-medium shadow-[0_0_0_1px_var(--color-neutral-200)]'
      : 'text-neutral-600 hover:bg-neutral-200 hover:text-neutral-950'"
    @click="selectSession(s.id)"
  >
    <span
      class="w-[6px] h-[6px] rounded-sm flex-none"
      :class="SESSION_STATUS_DOT_CLASS[s.statut]"
      :title="t(`session.status.${s.statut}`)"
      data-testid="session-status-dot"
    />
    <span class="flex-1 min-w-0 whitespace-nowrap overflow-hidden text-ellipsis">{{ s.nom }}</span>
    <span class="font-mono text-[10.5px] text-neutral-500 flex-none" :class="!compact && 'group-hover:hidden'">{{ s.count }}</span>
    <button
      v-if="!compact"
      class="hidden group-hover:block border-0 bg-transparent text-neutral-400 hover:text-neutral-950 cursor-pointer text-[14px] leading-none px-0.5 flex-none"
      :title="t('common.delete')"
      @click.stop="openDeleteSessionPopup(s.id, s.nom)"
    >×</button>
  </div>
  <div
    class="flex items-center gap-2 py-[6px] px-2 rounded-7 text-[12.5px] text-neutral-500 cursor-pointer mt-0.5 select-none hover:bg-neutral-200 hover:text-neutral-950"
    @click="addSession"
  >
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" /></svg>
    <span>{{ t('sidebar.new_session') }}</span>
  </div>
</template>
