/**
 * Parcours d'import d'une offre dans l'écran « Nouvelle candidature » :
 * lien → (écran de choix → capture dans FindMyJob | texte collé) →
 * formulaire. Le composable ne fait qu'appeler les commandes Tauri et
 * piloter les étapes ; l'extraction reste côté Rust.
 */

// Importation des modules Vue / framework
import { ref, type Ref } from 'vue'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { backendErrorMessage } from '@/composables/backend_error_message'
import { t } from '@/i18n'
import type { OfferExtraction } from '@/types'
import { classifyExtractionError, extractionErrorMessage } from './extraction_error'

/** Étapes de l'écran de création. */
export type ImportStep = 'url' | 'choice' | 'capture' | 'paste' | 'form'

/** Contenu de l'écran de choix proposé après un échec de l'analyse du lien. */
export interface ExtractionChoice {
  message: string
  canCapture: boolean
}

/**
 * Pilote les étapes d'import d'une offre.
 *
 * @param url - URL de l'offre saisie (partagée avec le formulaire).
 * @param onExtracted - Appelé avec les champs extraits avant le passage au formulaire.
 * @returns L'état des étapes et les actions de l'écran.
 */
export function useOfferImport(url: Ref<string>, onExtracted: (extraction: OfferExtraction) => void) {
  const step = ref<ImportStep>('url')
  const busy = ref(false)
  const urlError = ref<string | null>(null)
  const urlUnavailable = ref<string | null>(null)
  const choice = ref<ExtractionChoice | null>(null)
  const captureError = ref<string | null>(null)
  const pasteError = ref<string | null>(null)
  const pasteText = ref('')

  /** Passe au formulaire avec les champs extraits. */
  function finish(extraction: OfferExtraction) {
    onExtracted(extraction)
    step.value = 'form'
  }

  /** Analyse le lien saisi : formulaire pré-rempli, écran de choix ou message d'erreur. */
  async function analyzeUrl() {
    busy.value = true
    urlError.value = null
    urlUnavailable.value = null
    try {
      finish(await invoke<OfferExtraction>('scrape_offer', { url: url.value.trim() }))
    } catch (e: unknown) {
      const outcome = classifyExtractionError(e)
      if (outcome.kind === 'choice') {
        choice.value = { message: outcome.message, canCapture: outcome.canCapture }
        step.value = 'choice'
      } else if (outcome.kind === 'unavailable') {
        urlUnavailable.value = outcome.message
      } else {
        urlError.value = outcome.message
      }
    } finally {
      busy.value = false
    }
  }

  /** Ouvre (ou rouvre) la page de l'offre dans la fenêtre de capture. */
  async function openCapture() {
    busy.value = true
    captureError.value = null
    try {
      await invoke('open_capture_window', { url: url.value.trim() })
      step.value = 'capture'
    } catch (e: unknown) {
      captureError.value = extractionErrorMessage(e, t('error.fallback.open_offer_page'))
      step.value = 'capture'
    } finally {
      busy.value = false
    }
  }

  /** Capture la page affichée dans la fenêtre et en extrait l'offre. */
  async function capturePage() {
    busy.value = true
    captureError.value = null
    try {
      finish(await invoke<OfferExtraction>('capture_offer_page'))
    } catch (e: unknown) {
      captureError.value = extractionErrorMessage(e, t('error.fallback.capture_failed'))
    } finally {
      busy.value = false
    }
  }

  /** Analyse le texte collé par l'utilisateur. */
  async function analyzePaste() {
    busy.value = true
    pasteError.value = null
    try {
      finish(await invoke<OfferExtraction>('extract_offer_from_text', { text: pasteText.value.trim() }))
    } catch (e: unknown) {
      pasteError.value = extractionErrorMessage(e, t('error.fallback.extract_from_text'))
    } finally {
      busy.value = false
    }
  }

  /**
   * Ferme la fenêtre de capture. Une erreur est signalée sur l'écran
   * courant sans bloquer la navigation entre étapes.
   */
  async function closeCapture() {
    try {
      await invoke('close_capture_window')
    } catch (e: unknown) {
      captureError.value = backendErrorMessage(e, t('error.fallback.close_capture_window'))
    }
  }

  /** Passe au texte collé (la fenêtre de capture reste ouverte pour pouvoir y copier le texte). */
  function goToPaste() {
    pasteError.value = null
    step.value = 'paste'
  }

  /** Passe à la saisie manuelle. */
  async function goToManual() {
    await closeCapture()
    step.value = 'form'
  }

  /** Revient à l'étape du lien. */
  async function backToUrl() {
    await closeCapture()
    choice.value = null
    captureError.value = null
    pasteError.value = null
    step.value = 'url'
  }

  /** Réinitialise tout le parcours (après une création réussie). */
  function reset() {
    step.value = 'url'
    urlError.value = null
    urlUnavailable.value = null
    choice.value = null
    captureError.value = null
    pasteError.value = null
    pasteText.value = ''
  }

  return {
    step,
    busy,
    urlError,
    urlUnavailable,
    choice,
    captureError,
    pasteError,
    pasteText,
    analyzeUrl,
    openCapture,
    capturePage,
    analyzePaste,
    closeCapture,
    goToPaste,
    goToManual,
    backToUrl,
    reset,
  }
}
