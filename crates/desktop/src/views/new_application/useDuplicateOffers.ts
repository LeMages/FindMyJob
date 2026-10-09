/**
 * Avertissement non bloquant de doublon : cherche, via l'agent, les
 * candidatures existantes (toutes sessions) dont l'URL d'offre correspond à
 * celle saisie. La comparaison normalisée est faite par l'agent.
 */

// Importation des modules Vue / framework
import { ref } from 'vue'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { extractErrorMessage } from '@/composables/useErrorMessage'
import type { DuplicateCandidatureView } from '@/types'

/**
 * État et action de la vérification de doublon.
 *
 * @returns Les doublons trouvés, l'erreur éventuelle de vérification, et
 *   les actions `check` / `clear`.
 */
export function useDuplicateOffers() {
  const duplicates = ref<DuplicateCandidatureView[]>([])
  const checkError = ref<string | null>(null)
  // Numéro de la dernière vérification : une réponse plus ancienne, arrivée
  // après une saisie plus récente, est ignorée.
  let lastCheck = 0

  /**
   * Vérifie l'URL donnée ; vide la liste si l'URL est vide.
   *
   * @param url - URL de l'offre saisie.
   */
  async function check(url: string) {
    const trimmed = url.trim()
    const current = ++lastCheck
    checkError.value = null
    if (!trimmed) {
      duplicates.value = []
      return
    }
    try {
      const found = await invoke<DuplicateCandidatureView[]>('find_candidatures_by_url', { url: trimmed })
      if (current === lastCheck) duplicates.value = found
    } catch (e: unknown) {
      if (current === lastCheck) {
        duplicates.value = []
        checkError.value = extractErrorMessage(e, 'Vérification des doublons impossible.')
      }
    }
  }

  /** Efface l'avertissement. */
  function clear() {
    lastCheck++
    duplicates.value = []
    checkError.value = null
  }

  return { duplicates, checkError, check, clear }
}
