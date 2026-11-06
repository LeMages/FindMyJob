/**
 * Instance `vue-i18n` de l'application (mode Composition) : langue de
 * l'interface uniquement. Elle n'a aucun lien avec la langue des documents
 * générés, qui suit toujours celle de l'offre côté backend.
 *
 * Les modules hors composants (stores, composables, formateurs) traduisent
 * via `t` exporté ici ; les composants utilisent `useI18n()`.
 */

// Importation des modules externes
import { createI18n } from 'vue-i18n'

// Importation des modules internes
import { en } from '@/locales/en'
import { fr } from '@/locales/fr'
import type { UiLanguage } from '@/types'

/** Locale `Intl` utilisée pour les dates, par langue d'interface. */
const DATE_LOCALES: Record<UiLanguage, string> = {
  fr: 'fr-FR',
  en: 'en-GB',
}

export const i18n = createI18n({
  legacy: false,
  locale: 'fr',
  fallbackLocale: 'fr',
  messages: { fr, en },
  // Avertissements de clé manquante utiles en développement, bruit en production.
  missingWarn: import.meta.env.DEV,
  fallbackWarn: import.meta.env.DEV,
})

/** Traduction hors composant (réactive si appelée dans un rendu ou un `computed`). */
export const t = i18n.global.t

/** Teste l'existence d'une clé de traduction dans la langue courante. */
export const te = i18n.global.te

/**
 * Langue d'interface courante.
 *
 * @returns `'fr'` ou `'en'` (lecture réactive de la locale `vue-i18n`).
 */
export function currentUiLanguage(): UiLanguage {
  return i18n.global.locale.value === 'en' ? 'en' : 'fr'
}

/**
 * Locale `Intl` à passer aux `toLocaleDateString` / `toLocaleTimeString`.
 *
 * @returns `'fr-FR'` en français, `'en-GB'` en anglais (lecture réactive).
 */
export function dateLocale(): string {
  return DATE_LOCALES[currentUiLanguage()]
}
