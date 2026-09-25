/**
 * Téléchargement manuel d'un document de la candidature ouverte (boutons de
 * `DetailView` et de sa modale d'aperçu), avec état de chargement et toast
 * de confirmation/erreur.
 */

// Importation des modules Vue / framework
import { ref } from 'vue'

// Importation des modules internes
import { state, showToast } from '@/stores/app'
import { downloadCandidatureDocument } from '@/composables/document_download'
import { extractErrorMessage } from '@/composables/useErrorMessage'
import type { DownloadDocType } from '@/composables/document_file_name'

/**
 * Expose le téléchargement d'un document de la candidature ouverte vers le
 * dossier de téléchargements.
 *
 * @returns `downloading` (type en cours, ou `null`) et `download(docType, path)`.
 */
export function useDocumentDownload() {
  const downloading = ref<DownloadDocType | null>(null)

  /**
   * Télécharge un document de `state.detailCandidature` et affiche le chemin
   * sauvegardé (ou l'erreur) dans un toast. Ignoré si un téléchargement est
   * déjà en cours.
   *
   * @param docType - `'cv'` ou `'lm'`.
   * @param path - Chemin du document côté agent.
   */
  async function download(docType: DownloadDocType, path: string) {
    const candidature = state.detailCandidature
    if (!candidature || downloading.value) return
    downloading.value = docType
    try {
      const saved = await downloadCandidatureDocument(docType, path, candidature, {
        nom: state.userNom,
        prenom: state.userPrenom,
      })
      showToast(`Document téléchargé : ${saved}`)
    } catch (e: unknown) {
      showToast(extractErrorMessage(e, 'Impossible de télécharger le document.'), 'error')
    } finally {
      downloading.value = null
    }
  }

  return { downloading, download }
}
