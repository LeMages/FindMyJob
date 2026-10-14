/**
 * Interprétation, pour l'écran « Nouvelle candidature », des erreurs
 * renvoyées par l'extraction d'une offre. Le backend renvoie un code
 * (`{ kind: 'Scraping', message: 'access_denied' }`) : c'est ici que l'on
 * choisit le texte affiché et la suite proposée à l'utilisateur.
 */

// Importation des modules internes
import { extractErrorMessage } from '@/composables/useErrorMessage'
import type { ScrapingFailure } from '@/types'

/**
 * Suite à donner à un échec d'extraction :
 * - `choice` : écran de choix (capture dans FindMyJob, texte collé, saisie manuelle) ;
 * - `unavailable` : offre introuvable ou expirée, sans proposition de capture ;
 * - `error` : erreur réelle à afficher telle quelle (LLM, clé API…).
 */
export type ExtractionOutcome =
  | { kind: 'choice'; message: string; canCapture: boolean }
  | { kind: 'unavailable'; message: string }
  | { kind: 'error'; message: string }

/** Texte affiché pour chaque raison d'échec. */
const FAILURE_MESSAGES: Record<ScrapingFailure, string> = {
  invalid_url: "Ce lien n'est pas valide : seules les adresses web publiques en http ou https sont acceptées.",
  offer_unavailable: "Cette offre est introuvable ou a expiré : la page n'existe plus sur le site.",
  access_denied: 'Ce site refuse la lecture automatique de ses pages.',
  timeout: "Le site n'a pas répondu à temps.",
  network: "Le site n'a pas pu être joint.",
  not_html: "Ce lien ne mène pas à une page web (fichier PDF ou autre document).",
  empty_page: 'La page récupérée est vide.',
  too_large: 'La page est trop volumineuse pour être analysée (plus de 5 Mo).',
  insufficient_content:
    "La page récupérée ne contient pas le texte de l'offre (contenu chargé en JavaScript, vérification anti-robot ou bandeau cookies).",
}

/** Raisons pour lesquelles ouvrir la page dans FindMyJob peut aider. */
const CAPTURABLE_FAILURES: ScrapingFailure[] = [
  'access_denied',
  'insufficient_content',
  'timeout',
  'network',
  'empty_page',
]

/** Raisons qui mènent à l'écran de choix (les autres sont affichées sur l'étape du lien). */
const CHOICE_FAILURES: ScrapingFailure[] = [...CAPTURABLE_FAILURES, 'not_html', 'too_large']

/**
 * Lit le code d'échec d'une erreur d'extraction, s'il s'agit d'une erreur
 * `Scraping` du backend.
 *
 * @param e - Erreur interceptée autour d'un `invoke()`.
 * @returns Le code d'échec, ou `null` pour toute autre erreur.
 */
export function scrapingFailureOf(e: unknown): ScrapingFailure | null {
  if (!e || typeof e !== 'object' || !('kind' in e) || !('message' in e)) return null
  const { kind, message } = e as { kind: unknown; message: unknown }
  if (kind !== 'Scraping' || typeof message !== 'string') return null
  return message in FAILURE_MESSAGES ? (message as ScrapingFailure) : null
}

/**
 * Texte lisible d'une erreur d'extraction : message dédié pour un code
 * d'échec connu, message réel de l'erreur sinon.
 *
 * @param e - Erreur interceptée autour d'un `invoke()`.
 * @param fallback - Message si l'erreur n'est pas exploitable.
 * @returns Le message à afficher.
 */
export function extractionErrorMessage(e: unknown, fallback: string): string {
  const failure = scrapingFailureOf(e)
  return failure ? FAILURE_MESSAGES[failure] : extractErrorMessage(e, fallback)
}

/**
 * Détermine la suite à donner à un échec de l'analyse d'un lien.
 *
 * @param e - Erreur interceptée autour de `invoke('scrape_offer')`.
 * @returns L'issue à afficher (écran de choix, offre expirée ou erreur réelle).
 */
export function classifyExtractionError(e: unknown): ExtractionOutcome {
  const failure = scrapingFailureOf(e)
  if (failure === 'offer_unavailable') {
    return { kind: 'unavailable', message: FAILURE_MESSAGES[failure] }
  }
  if (failure && CHOICE_FAILURES.includes(failure)) {
    return { kind: 'choice', message: FAILURE_MESSAGES[failure], canCapture: CAPTURABLE_FAILURES.includes(failure) }
  }
  return { kind: 'error', message: extractionErrorMessage(e, "L'analyse de l'offre a échoué.") }
}
