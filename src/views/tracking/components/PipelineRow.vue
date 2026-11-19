<script setup lang="ts">
/**
 * Ligne du tableau des candidatures : entreprise (avatar + nom), poste,
 * statut, date d'envoi (« — » tant qu'elle n'est pas envoyée), puis
 * inactivité, ou date de clôture pour une candidature clôturée.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des modules internes
import { statusColor, statusLabel } from '@/composables/candidature_display'
import { dateLocale } from '@/i18n'
import type { CandidatureView } from '@/types'

const { t } = useI18n()

const props = defineProps<{
  candidature: CandidatureView
  /** Ligne du groupe « Clôturées » : couleurs atténuées, date de clôture. */
  closed: boolean
}>()

defineEmits<{ open: [] }>()

/** Palette des couleurs d'avatar, assignées de façon stable par candidature. */
const AVATAR_COLORS = [
  'bg-blue-100 text-blue-700',
  'bg-emerald-100 text-emerald-700',
  'bg-amber-100 text-amber-700',
  'bg-rose-100 text-rose-700',
  'bg-purple-100 text-purple-700',
  'bg-cyan-100 text-cyan-700',
]

/** Couleur propre à chaque issue d'une candidature clôturée. */
const CLOSED_COLORS: Record<string, string> = {
  refus: 'bg-rose-100 text-rose-700',
  abandon: 'bg-neutral-200 text-neutral-600',
  ghosting: 'bg-neutral-100 text-neutral-500',
  offre_refusee: 'bg-amber-100 text-amber-700',
}

const c = computed(() => props.candidature)

/** Initiales de l'entreprise (deux lettres), vide si le nom est absent. */
const initials = computed(() =>
  (c.value.entreprise ?? '').split(' ').map((w) => w[0] ?? '').join('').slice(0, 2).toUpperCase(),
)

const pillColor = computed(() => (props.closed ? CLOSED_COLORS[c.value.status] : undefined) ?? statusColor(c.value.status))

/** Jours écoulés depuis le dernier changement de statut. */
const inactivityDays = computed(() =>
  Math.floor((Date.now() - new Date(c.value.derniere_maj_statut).getTime()) / (1000 * 60 * 60 * 24)),
)

/** Vert jusqu'à 7 jours, orange jusqu'à 15, rouge au-delà. */
const inactivityColor = computed(() => {
  if (inactivityDays.value <= 7) return 'text-green-600'
  if (inactivityDays.value <= 15) return 'text-orange-500'
  return 'text-red-600'
})

function longDate(iso: string): string {
  return new Date(iso).toLocaleDateString(dateLocale(), { day: 'numeric', month: 'long', year: 'numeric' })
}
</script>

<template>
  <tr class="border-b border-neutral-100 cursor-pointer hover:bg-surface-3" @click="$emit('open')">
    <td class="py-2.5 px-4">
      <div class="flex items-center gap-[10px]">
        <div class="w-[26px] h-[26px] rounded-7 flex-none flex items-center justify-center font-semibold text-[11px]" :class="AVATAR_COLORS[c.id % AVATAR_COLORS.length]">{{ initials }}</div>
        <span class="font-semibold" :class="closed ? 'text-neutral-600' : ''">{{ c.entreprise ?? '—' }}</span>
      </div>
    </td>
    <td class="py-2.5 px-4" :class="closed ? 'text-neutral-500' : ''">{{ c.poste }}</td>
    <td class="py-2.5 px-4">
      <span class="inline-block font-medium text-[12px] rounded-5 px-[9px] py-1 whitespace-nowrap" :class="pillColor">{{ statusLabel(c.status) }}</span>
    </td>
    <td class="hidden @[620px]:table-cell py-2.5 px-4 font-mono text-neutral-500 whitespace-nowrap">
      <span v-if="!c.date_envoi" class="text-neutral-300">—</span>
      <template v-else>{{ longDate(c.date_envoi) }}</template>
    </td>
    <td v-if="closed" class="py-2.5 px-4 font-mono text-neutral-400 whitespace-nowrap">{{ longDate(c.derniere_maj_statut) }}</td>
    <td v-else class="py-2.5 px-4 font-mono whitespace-nowrap" :class="inactivityColor">{{ t('tracking.inactivity_days', { n: inactivityDays }) }}</td>
  </tr>
</template>
