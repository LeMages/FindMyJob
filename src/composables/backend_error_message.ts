/**
 * Message affiché pour une erreur renvoyée par le backend, selon la langue
 * de l'interface. Le backend n'est pas traduit : ses messages libres sont en
 * français. En français, ils sont affichés tels quels ; en anglais, on
 * affiche un message générique selon `AppError.kind`, suivi du message
 * d'origine entre parenthèses.
 */

// Importation des modules internes
import { extractErrorMessage } from '@/composables/useErrorMessage'
import { currentUiLanguage, t, te } from '@/i18n'

/**
 * Lit le `kind` d'une erreur `AppError` sérialisée (`{ kind, message }`).
 *
 * @param e - Erreur interceptée autour d'un `invoke()`.
 * @returns Le `kind`, ou `null` si l'erreur n'a pas cette forme.
 */
function appErrorKind(e: unknown): string | null {
  if (!e || typeof e !== 'object' || !('kind' in e)) return null
  const { kind } = e as { kind: unknown }
  return typeof kind === 'string' ? kind : null
}

/**
 * Message affichable d'une erreur backend dans la langue de l'interface.
 *
 * @param e - Erreur interceptée (objet `AppError`, chaîne, ou autre).
 * @param fallback - Message (déjà traduit) si l'erreur n'est pas exploitable.
 * @returns En FR : le message d'origine (inchangé). En EN : le message
 *   générique du `kind` suivi du message d'origine, ou le message d'origine
 *   seul si le `kind` est inconnu.
 */
export function backendErrorMessage(e: unknown, fallback: string): string {
  const original = extractErrorMessage(e, fallback)
  if (currentUiLanguage() === 'fr') return original

  const kind = appErrorKind(e)
  if (!kind) return original
  const key = `error.kind.${kind}`
  return te(key) ? `${t(key)} (${original})` : original
}

/**
 * Message affichable pour un texte d'erreur libre renvoyé dans un DTO du
 * backend (pas une `AppError` : aucun `kind` à exploiter).
 *
 * @param original - Texte d'origine du backend (en français).
 * @param generic - Message générique déjà traduit, utilisé en anglais.
 * @returns En FR : le texte d'origine (inchangé). En EN : le message
 *   générique suivi du texte d'origine entre parenthèses.
 */
export function backendTextMessage(original: string, generic: string): string {
  if (currentUiLanguage() === 'fr') return original
  return `${generic} (${original})`
}
