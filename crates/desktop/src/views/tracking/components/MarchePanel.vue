<script setup lang="ts">
/**
 * Onglet « Marché » de la page Candidatures : offres scorées de la session
 * courante, triées par score décroissant.
 */

// Importation des modules Vue / framework
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { currentSession, navigate, startApplicationFromOffer } from '@/stores/app'
import { extractErrorMessage } from '@/composables/useErrorMessage'
import { useDocumentsStatus } from '@/composables/useDocumentsStatus'
import type { OfferView } from '@/types'

const { documentsBlocking, missingDocumentsMessage, loadDocumentsStatus } = useDocumentsStatus()

const offers = ref<OfferView[]>([])
const loading = ref(true)
const loadError = ref<string | null>(null)
const linkError = ref<string | null>(null)

const sessionId = computed(() => currentSession.value?.id ?? null)
const minScore = computed(() => currentSession.value?.min_score ?? null)

/**
 * Classes Tailwind du badge de score : vert dès 0,8, orange dès 0,65, neutre en dessous.
 *
 * @param score - Score de l'offre (0 à 1), `null` si non scorée.
 * @returns Classes de fond et de texte du badge.
 */
function scoreColor(score: number | null): string {
  if (score === null) return 'bg-neutral-100 text-neutral-500'
  if (score >= 0.8) return 'bg-green-100 text-green-700'
  if (score >= 0.65) return 'bg-orange-100 text-orange-700'
  return 'bg-neutral-100 text-neutral-600'
}

/**
 * Formate un score sur une échelle de 0 à 100.
 *
 * @param score - Score de l'offre (0 à 1), `null` si non scorée.
 * @returns Entier en chaîne, ou « — » si absent.
 */
function scoreLabel(score: number | null): string {
  return score === null ? '—' : String(Math.round(score * 100))
}

/**
 * Formate une date ISO pour l'affichage.
 *
 * @param iso - Date ISO 8601.
 * @returns Date au format « 12 juin 2026 ».
 */
function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString('fr-FR', { day: 'numeric', month: 'long', year: 'numeric' })
}

/**
 * Ouvre l'URL d'une offre dans le navigateur par défaut, via la commande
 * Tauri `open_url` — un `<a target="_blank">` ne fonctionne pas dans la webview.
 *
 * @param url - URL de l'offre.
 */
async function openOffer(url: string): Promise<void> {
  linkError.value = null
  try {
    await invoke('open_url', { url })
  } catch (e: unknown) {
    linkError.value = extractErrorMessage(e, "Impossible d'ouvrir ce lien.")
  }
}

/** Charge les offres scorées de la session courante depuis le backend. */
async function loadOffers() {
  const sid = currentSession.value?.id
  const threshold = currentSession.value?.min_score
  if (!sid || threshold === undefined) {
    offers.value = []
    loading.value = false
    return
  }
  loading.value = true
  loadError.value = null
  try {
    offers.value = await invoke<OfferView[]>('get_session_offers', { sessionId: sid, minScore: threshold })
  } catch (e: unknown) {
    loadError.value = extractErrorMessage(e, 'Impossible de charger les offres.')
  } finally {
    loading.value = false
  }
}

onMounted(loadOffers)
watch([sessionId, minScore], loadOffers)

onMounted(loadDocumentsStatus)
watch(sessionId, loadDocumentsStatus)
</script>

<template>
  <div class="px-[30px] pt-[18px] pb-[30px]">
    <!-- Loading -->
    <div v-if="loading" class="text-[12px] text-neutral-400 text-center py-6 font-mono mt-[60px]">Chargement…</div>

    <!-- Load error -->
    <div v-else-if="loadError" class="text-[12px] text-red-500 text-center py-6 font-mono mt-[60px]">{{ loadError }}</div>

    <!-- Empty -->
    <div v-else-if="offers.length === 0" class="text-center py-6 mt-[60px]">
      <div class="text-[13px] text-neutral-600 font-medium">Aucune offre trouvée pour cette session</div>
      <div class="text-[12px] text-neutral-400 font-mono mt-1">Lancez une recherche depuis l'onglet Paramètres pour alimenter le marché.</div>
    </div>

    <!-- Content -->
    <template v-else>
      <div class="text-[13px] text-neutral-500 font-mono mb-4">
        {{ offers.length }} offre{{ offers.length > 1 ? 's' : '' }} · score &gt; {{ Math.round((minScore ?? 0) * 100) }}
      </div>
      <div v-if="linkError" class="text-[12px] text-red-500 font-mono mb-3">{{ linkError }}</div>
      <div v-if="documentsBlocking" class="bg-amber-50 border border-amber-200 rounded-lg px-4 py-3 mb-3 text-[13px] text-amber-700 flex items-center justify-between gap-3">
        <span>{{ missingDocumentsMessage }}</span>
        <button class="underline font-medium whitespace-nowrap" @click="navigate('settings')">Aller aux paramètres</button>
      </div>
      <table class="w-full text-[13px] border-collapse bg-surface-1 border border-neutral-200 rounded-10 overflow-hidden">
        <thead>
          <tr class="text-left bg-neutral-950">
            <th class="py-3 px-4 font-semibold font-mono text-[11px] tracking-[.06em] text-white">SCORE</th>
            <th class="py-3 px-4 font-semibold font-mono text-[11px] tracking-[.06em] text-white">TITRE</th>
            <th class="py-3 px-4 font-semibold font-mono text-[11px] tracking-[.06em] text-white">ENTREPRISE</th>
            <th class="py-3 px-4 font-semibold font-mono text-[11px] tracking-[.06em] text-white">LOCALISATION</th>
            <th class="py-3 px-4 font-semibold font-mono text-[11px] tracking-[.06em] text-white">CONTRAT</th>
            <th class="py-3 px-4 font-semibold font-mono text-[11px] tracking-[.06em] text-white">SCRAPÉE LE</th>
            <th class="py-3 px-4 font-semibold font-mono text-[11px] tracking-[.06em] text-white">ACTIONS</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="o in offers" :key="o.id" class="border-b border-neutral-100 last:border-0 hover:bg-surface-3">
            <td class="py-2.5 px-4">
              <span class="inline-block font-mono font-medium text-[12px] rounded-5 px-[9px] py-1" :class="scoreColor(o.score)">{{ scoreLabel(o.score) }}</span>
            </td>
            <td class="py-2.5 px-4">
              <a :href="o.url" class="font-semibold text-neutral-950 hover:underline" @click.prevent="openOffer(o.url)">{{ o.title }}</a>
            </td>
            <td class="py-2.5 px-4">{{ o.company }}</td>
            <td class="py-2.5 px-4 text-neutral-600">{{ o.location }}</td>
            <td class="py-2.5 px-4 text-neutral-600">{{ o.contract_type ?? '—' }}</td>
            <td class="py-2.5 px-4 font-mono text-neutral-500">{{ formatDate(o.created_at) }}</td>
            <td class="py-2.5 px-4">
              <button
                class="h-8 px-3 rounded-lg border border-neutral-950 bg-neutral-950 text-white text-[12px] font-medium cursor-pointer whitespace-nowrap hover:bg-neutral-800"
                :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': o.has_candidature || documentsBlocking }"
                :disabled="o.has_candidature || documentsBlocking"
                @click="startApplicationFromOffer(o)"
              >{{ o.has_candidature ? 'Déjà postulé' : 'Postuler' }}</button>
            </td>
          </tr>
        </tbody>
      </table>
    </template>
  </div>
</template>
