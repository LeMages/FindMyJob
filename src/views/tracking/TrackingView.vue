<script setup lang="ts">
/**
 * Vue « Suivi » : pipeline des candidatures de la session courante sous
 * forme de tableau, avec statistiques et corbeille.
 */

// Importation des modules Vue / framework
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import type { CandidatureView } from '@/types'
import {
  currentSession,
  navigate,
  openCard,
  openTrashPopup,
  closeTrashPopup,
  addSession,
} from '@/stores/app'
import { backendErrorMessage } from '@/composables/backend_error_message'
import { dateLocale } from '@/i18n'
import { statusLabel, statusColor, TRASHED_STATUSES } from '@/composables/candidature_display'
import { useDocumentsStatus } from '@/composables/useDocumentsStatus'
import TrashPopup from '@/views/tracking/components/TrashPopup.vue'
import CandidaturesTabs from '@/views/tracking/components/CandidaturesTabs.vue'
import PlanningPanel from '@/views/tracking/components/PlanningPanel.vue'
import SessionParametersForm from '@/views/tracking/components/SessionParametersForm.vue'

const { t } = useI18n()

const candidatures = ref<CandidatureView[]>([])
const loading = ref(true)
const loadError = ref<string | null>(null)
const activeTab = ref<'candidatures' | 'planning' | 'parametres'>('candidatures')

const sessionId = computed(() => currentSession.value?.id ?? null)

const sessionIsAccepted = computed(() =>
  candidatures.value.some(c => c.status === 'accepte')
)

const { documentsBlocking, missingDocumentsMessage, loadDocumentsStatus } = useDocumentsStatus()

// Les candidatures aux statuts terminaux négatifs ne figurent plus dans la
// liste principale : elles sont reléguées à la corbeille (voir
// `trashedCandidatures`).
const visibleCandidatures = computed(() =>
  candidatures.value.filter(c => !TRASHED_STATUSES.includes(c.status))
)

const trashedCandidatures = computed(() =>
  candidatures.value.filter(c => TRASHED_STATUSES.includes(c.status))
)

/**
 * Date affichée dans la colonne « Candidaté » : date de création de la fiche.
 *
 * @param c - Candidature à afficher.
 * @returns Date formatée en français (ex. « 12 juin 2026 »).
 */
function candidatedDate(c: CandidatureView): string {
  const iso = c.created_at
  return new Date(iso).toLocaleDateString(dateLocale(), { day: 'numeric', month: 'long', year: 'numeric' })
}

/**
 * Nombre de jours écoulés depuis le dernier changement de statut.
 *
 * @param c - Candidature à mesurer.
 * @returns Nombre de jours d'inactivité, arrondi à l'entier inférieur.
 */
function inactivityDays(c: CandidatureView): number {
  const lastChange = new Date(c.derniere_maj_statut).getTime()
  return Math.floor((Date.now() - lastChange) / (1000 * 60 * 60 * 24))
}

/**
 * Couleur du texte d'inactivité selon le nombre de jours écoulés :
 * vert jusqu'à 7 jours, orange de 8 à 15 jours, rouge au-delà.
 *
 * @param days - Nombre de jours d'inactivité.
 * @returns Classe Tailwind de couleur de texte.
 */
function inactivityColor(days: number): string {
  if (days <= 7) return 'text-green-600'
  if (days <= 15) return 'text-orange-500'
  return 'text-red-600'
}

/**
 * Initiales d'un nom d'entreprise, affichées dans l'avatar de la colonne
 * « Entreprise » (même logique que les initiales de contact en page détail).
 *
 * @param nom - Nom de l'entreprise, `null` si non renseigné.
 * @returns Deux lettres majuscules, ou chaîne vide si le nom est absent.
 */
function initials(nom: string | null): string {
  return (nom ?? '').split(' ').map(w => w[0] ?? '').join('').slice(0, 2).toUpperCase()
}

/** Palette des couleurs d'avatar, assignées de façon stable par candidature. */
const AVATAR_COLORS = [
  'bg-blue-100 text-blue-700',
  'bg-emerald-100 text-emerald-700',
  'bg-amber-100 text-amber-700',
  'bg-rose-100 text-rose-700',
  'bg-purple-100 text-purple-700',
  'bg-cyan-100 text-cyan-700',
]

/**
 * Couleur d'avatar d'une candidature, stable pour une même candidature
 * (dérivée de son id) mais variée d'une candidature à l'autre.
 *
 * @param id - Identifiant de la candidature.
 * @returns Classes Tailwind de fond et de texte à appliquer à l'avatar.
 */
function avatarColor(id: number): string {
  return AVATAR_COLORS[id % AVATAR_COLORS.length]
}

/**
 * Ouvre le détail d'une candidature depuis une ligne du pipeline.
 *
 * @param c - Candidature dont la ligne a été cliquée.
 */
function openRow(c: CandidatureView) {
  openCard({
    id: c.id,
    company: c.entreprise ?? '—',
    role: c.poste,
    location: c.localisation ?? '',
    date: candidatedDate(c),
    meta: '',
  })
}

/** Charge les candidatures de la session courante depuis le backend. */
async function loadCandidatures() {
  const sid = currentSession.value?.id
  if (!sid) {
    candidatures.value = []
    loading.value = false
    return
  }
  loading.value = true
  loadError.value = null
  try {
    candidatures.value = await invoke<CandidatureView[]>('get_candidatures', {
      sessionId: sid,
    })
  } catch (e: unknown) {
    loadError.value = backendErrorMessage(e, t('error.fallback.load_applications'))
  } finally {
    loading.value = false
  }
}

onMounted(loadCandidatures)
watch(sessionId, loadCandidatures)
onMounted(loadDocumentsStatus)
watch(sessionId, loadDocumentsStatus)
</script>

<template>
  <!-- Aucune session -->
  <div v-if="!currentSession" class="flex items-center justify-center h-full">
    <button
      class="px-6 py-3 text-[14px] h-9 rounded-lg inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap font-medium border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
      @click="addSession"
    >
      {{ t('tracking.create_first_session') }}
    </button>
  </div>

  <!-- Session sélectionnée -->
  <template v-else>
    <!-- Header -->
    <div class="px-[30px] pt-[26px] pb-4">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-2xl font-semibold leading-tight">{{ currentSession.nom }}</h1>
          <div class="text-[13px] text-neutral-500 mt-[5px] font-mono">
            {{ t('tracking.summary', { n: visibleCandidatures.length, date: new Date().toLocaleDateString(dateLocale(), { day: 'numeric', month: 'long' }) }, visibleCandidatures.length) }}
          </div>
        </div>
        <div class="flex items-center gap-[10px]">
          <button
            class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
            @click="openTrashPopup"
          >
            <svg width="15" height="16" viewBox="0 0 15 16" fill="none">
              <rect x="2" y="5" width="11" height="10" rx="1.5" stroke="currentColor" stroke-width="1.4" />
              <line x1="1" y1="5" x2="14" y2="5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
              <line x1="5.5" y1="2" x2="9.5" y2="2" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
              <line x1="5.5" y1="8" x2="5.5" y2="12" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
              <line x1="9.5" y1="8" x2="9.5" y2="12" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
            </svg>
            {{ t('tracking.trash') }}
          </button>
          <span
            class="inline-block"
            :title="documentsBlocking ? t('tracking.documents_blocking_hint', { message: missingDocumentsMessage }) : undefined"
          >
            <button
              class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
              :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': sessionIsAccepted || documentsBlocking }"
              :disabled="sessionIsAccepted || documentsBlocking"
              @click="navigate('new')"
            >
              {{ t('tracking.new_application') }}
            </button>
          </span>
        </div>
      </div>
    </div>

    <!-- Onglets -->
    <CandidaturesTabs :active="activeTab" @select="tab => (activeTab = tab)" />

    <!-- Onglet Candidatures -->
    <template v-if="activeTab === 'candidatures'">
      <!-- Loading -->
      <div v-if="loading" class="text-[12px] text-neutral-400 text-center py-6 font-mono mt-[60px]">{{ t('common.loading') }}</div>

      <!-- Load error -->
      <div v-else-if="loadError" class="text-[12px] text-red-500 text-center py-6 font-mono mt-[60px]">{{ loadError }}</div>

      <!-- Content -->
      <template v-else>
        <!-- Pipeline -->
        <div class="px-[30px] pt-[18px] pb-[30px]">
          <table class="w-full text-[13px] border-collapse bg-surface-1 border border-neutral-200 rounded-10 overflow-hidden">
            <thead>
              <tr class="text-left bg-neutral-950">
                <th class="py-3 px-4 font-semibold font-mono text-[11px] tracking-[.06em] text-white">{{ t('tracking.columns.company') }}</th>
                <th class="py-3 px-4 font-semibold font-mono text-[11px] tracking-[.06em] text-white">{{ t('tracking.columns.role') }}</th>
                <th class="py-3 px-4 font-semibold font-mono text-[11px] tracking-[.06em] text-white">{{ t('tracking.columns.status') }}</th>
                <th class="py-3 px-4 font-semibold font-mono text-[11px] tracking-[.06em] text-white">{{ t('tracking.columns.applied') }}</th>
                <th class="py-3 px-4 font-semibold font-mono text-[11px] tracking-[.06em] text-white">{{ t('tracking.columns.inactivity') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="c in visibleCandidatures"
                :key="c.id"
                class="border-b border-neutral-100 last:border-0 cursor-pointer hover:bg-surface-3"
                @click="openRow(c)"
              >
                <td class="py-2.5 px-4">
                  <div class="flex items-center gap-[10px]">
                    <div class="w-[26px] h-[26px] rounded-7 flex-none flex items-center justify-center font-semibold text-[11px]" :class="avatarColor(c.id)">{{ initials(c.entreprise) }}</div>
                    <span class="font-semibold">{{ c.entreprise ?? '—' }}</span>
                  </div>
                </td>
                <td class="py-2.5 px-4">{{ c.poste }}</td>
                <td class="py-2.5 px-4">
                  <span class="inline-block font-medium text-[12px] rounded-5 px-[9px] py-1" :class="statusColor(c.status)">{{ statusLabel(c.status) }}</span>
                </td>
                <td class="py-2.5 px-4 font-mono text-neutral-500">{{ candidatedDate(c) }}</td>
                <td class="py-2.5 px-4 font-mono" :class="inactivityColor(inactivityDays(c))">{{ t('tracking.inactivity_days', { n: inactivityDays(c) }) }}</td>
              </tr>
            </tbody>
          </table>

          <div v-if="visibleCandidatures.length === 0" class="text-[12px] text-neutral-400 text-center py-6 font-mono">{{ t('tracking.empty') }}</div>
        </div>
      </template>
    </template>

    <!-- Onglet Planning -->
    <PlanningPanel v-else-if="activeTab === 'planning'" />

    <!-- Onglet Paramètres -->
    <SessionParametersForm v-else />
  </template>

  <!-- Popups -->
  <TrashPopup :list="trashedCandidatures" @close="closeTrashPopup" />
</template>
