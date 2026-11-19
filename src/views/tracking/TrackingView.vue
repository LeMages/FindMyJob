<script setup lang="ts">
/**
 * Vue « Suivi » : en-tête de la session courante (compteurs ouvertes /
 * clôturées, nouvelle candidature) et onglets Candidatures, Planning et
 * Paramètres.
 */

// Importation des modules Vue / framework
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'

// Importation des composants
import CandidaturesTabs from '@/views/tracking/components/CandidaturesTabs.vue'
import PipelineTable from '@/views/tracking/components/PipelineTable.vue'
import PlanningPanel from '@/views/tracking/components/PlanningPanel.vue'
import SessionParametersForm from '@/views/tracking/components/SessionParametersForm.vue'

// Importation des composables
import { backendErrorMessage } from '@/composables/backend_error_message'
import { useDocumentsStatus } from '@/composables/useDocumentsStatus'

// Importation des modules internes
import { dateLocale } from '@/i18n'
import { currentSession, navigate, openCard, addSession } from '@/stores/app'
import { isClosed } from '@/views/tracking/components/pipeline_groups'
import type { CandidatureView } from '@/types'

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

/** « N candidatures · X en cours · Y clôturées ». */
const summary = computed(() => {
  const total = candidatures.value.length
  const closed = candidatures.value.filter(c => isClosed(c.status)).length
  return [
    t('tracking.summary.total', { n: total }, total),
    t('tracking.summary.open', { n: total - closed }),
    t('tracking.summary.closed', { n: closed }, closed),
  ].join(' · ')
})

/**
 * Ouvre le détail d'une candidature depuis une ligne du tableau.
 *
 * @param c - Candidature dont la ligne a été cliquée.
 */
function openRow(c: CandidatureView) {
  openCard({
    id: c.id,
    company: c.entreprise ?? '—',
    role: c.poste,
    location: c.localisation ?? '',
    date: new Date(c.created_at).toLocaleDateString(dateLocale(), { day: 'numeric', month: 'long', year: 'numeric' }),
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
    <div class="px-5 sm:px-[30px] pt-[26px] pb-4">
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div class="min-w-0">
          <h1 class="text-2xl font-semibold leading-tight">{{ currentSession.nom }}</h1>
          <div class="text-[13px] text-neutral-500 mt-[5px]">{{ summary }}</div>
        </div>
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

    <!-- Onglets -->
    <CandidaturesTabs :active="activeTab" @select="tab => (activeTab = tab)" />

    <!-- Onglet Candidatures -->
    <template v-if="activeTab === 'candidatures'">
      <!-- Loading -->
      <div v-if="loading" class="text-[12px] text-neutral-400 text-center py-6 font-mono mt-[60px]">{{ t('common.loading') }}</div>

      <!-- Load error -->
      <div v-else-if="loadError" class="text-[12px] text-red-500 text-center py-6 font-mono mt-[60px]">{{ loadError }}</div>

      <PipelineTable v-else :candidatures="candidatures" @open="openRow" />
    </template>

    <!-- Onglet Planning -->
    <PlanningPanel v-else-if="activeTab === 'planning'" />

    <!-- Onglet Paramètres -->
    <SessionParametersForm v-else />
  </template>

</template>
