/**
 * Suivi par polling de la progression de génération des documents (CV +
 * lettre de motivation) d'une candidature, après sa création. État local à
 * l'appelant (pas un singleton de module comme `useDocumentsStatus` : la
 * progression est scopée à une soumission en cours, pas partagée entre vues).
 */

// Importation des modules Vue / framework
import { ref, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { backendErrorMessage } from '@/composables/backend_error_message'
import { t } from '@/i18n'
import type { GenerationProgress } from '@/types'

const POLL_INTERVAL_MS = 2000

export function useGenerationProgress() {
  const progress = ref<GenerationProgress | null>(null)
  const pollError = ref<string | null>(null)
  let intervalId: ReturnType<typeof setInterval> | null = null

  /** Arrête le polling en cours, sans effet s'il n'y en a pas. */
  function stop() {
    if (intervalId !== null) clearInterval(intervalId)
    intervalId = null
  }

  /**
   * Démarre le polling de la progression d'une candidature — interroge
   * `get_generation_progress` toutes les 2 secondes jusqu'à ce que le statut
   * ne soit plus `in_progress`, ou jusqu'à `stop()`.
   *
   * @param candidatureId - Identifiant de la candidature dont on suit la génération.
   */
  function start(candidatureId: number) {
    stop()
    progress.value = null
    pollError.value = null

    intervalId = setInterval(async () => {
      try {
        const next = await invoke<GenerationProgress>('get_generation_progress', { candidatureId })
        progress.value = next
        if (next.status !== 'in_progress') stop()
      } catch (e: unknown) {
        pollError.value = backendErrorMessage(e, t('error.fallback.generation_tracking'))
        stop()
      }
    }, POLL_INTERVAL_MS)
  }

  onUnmounted(stop)

  return { progress, pollError, start, stop }
}
