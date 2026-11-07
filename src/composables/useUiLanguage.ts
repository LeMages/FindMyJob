/**
 * Gestion de la langue de l'interface (FR / EN). État porté par l'instance
 * `vue-i18n` (`src/i18n.ts`) : une seule source de vérité partagée par
 * `main.ts` (application initiale, avant le premier rendu), les Paramètres
 * et l'onboarding. Sans lien avec la langue des documents générés.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { backendErrorMessage } from '@/composables/backend_error_message'
import { currentUiLanguage, i18n, t } from '@/i18n'
import type { UiLanguage } from '@/types'

let initialized = false

/**
 * Langues proposées. Les noms sont écrits dans leur propre langue et jamais
 * traduits : l'utilisateur doit reconnaître la sienne quelle que soit la
 * langue affichée.
 */
export const UI_LANGUAGE_OPTIONS: { value: UiLanguage; label: string }[] = [
  { value: 'fr', label: 'Français' },
  { value: 'en', label: 'English' },
]

/**
 * Applique une langue à l'interface sans la persister : locale `vue-i18n`
 * et attribut `lang` de `<html>`.
 *
 * @param lang - Langue à appliquer.
 */
export function applyUiLanguage(lang: UiLanguage): void {
  i18n.global.locale.value = lang
  document.documentElement.lang = lang
}

/**
 * Initialise la langue au démarrage : charge le réglage persisté et
 * l'applique avant le montage (pas de flash de langue). Idempotent — un seul
 * appel a un effet (voir `main.ts`).
 */
export async function initUiLanguage(): Promise<void> {
  if (initialized) return
  initialized = true

  try {
    const stored = await invoke<string>('get_ui_language')
    applyUiLanguage(stored === 'en' ? 'en' : 'fr')
  } catch (e: unknown) {
    console.error("Impossible de charger la langue de l'interface.", e)
    applyUiLanguage('fr')
  }
}

/**
 * Change la langue de l'interface : appliquée immédiatement, puis persistée.
 *
 * @param lang - Langue à appliquer (`'fr'` ou `'en'`).
 * @returns `null` si la persistance a réussi, sinon un message d'erreur à
 *   afficher (la langue reste appliquée même si la sauvegarde échoue).
 */
export async function setUiLanguage(lang: UiLanguage): Promise<string | null> {
  applyUiLanguage(lang)
  try {
    await invoke('set_ui_language', { lang })
    return null
  } catch (e: unknown) {
    return backendErrorMessage(e, t('settings.language.save_error'))
  }
}

/** Expose la langue courante (réactive) et la fonction pour la modifier. */
export function useUiLanguage() {
  const uiLanguage = computed<UiLanguage>(() => currentUiLanguage())
  return { uiLanguage, setUiLanguage }
}
