/**
 * Gestion du thème d'affichage (clair / sombre / système). État singleton
 * au niveau module : une seule source de vérité partagée par `main.ts`
 * (application initiale, avant le premier rendu) et `SettingsView.vue`
 * (sélecteur utilisateur).
 */

// Importation des modules Vue / framework
import { ref } from 'vue'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { extractErrorMessage } from '@/composables/useErrorMessage'
import type { ThemeSetting } from '@/types'

const theme = ref<ThemeSetting>('systeme')
let initialized = false
let mediaQuery: MediaQueryList | null = null

/** Résout la préférence système actuelle (`prefers-color-scheme`). */
function resolveSystemTheme(): 'clair' | 'sombre' {
  return mediaQuery?.matches ? 'sombre' : 'clair'
}

/** Pose ou retire la classe `dark` sur `<html>` selon le thème résolu. */
function applyResolvedTheme(value: ThemeSetting) {
  const resolved = value === 'systeme' ? resolveSystemTheme() : value
  document.documentElement.classList.toggle('dark', resolved === 'sombre')
}

/** Ré-applique le thème si l'utilisateur est en mode "système" et que l'OS a changé de préférence. */
function handleSystemPreferenceChange() {
  if (theme.value === 'systeme') {
    applyResolvedTheme(theme.value)
  }
}

/**
 * Initialise le thème au démarrage de l'application : charge le réglage
 * persisté, l'applique immédiatement, et écoute les changements de
 * préférence système. Idempotent — un seul appel a un effet (voir `main.ts`).
 */
export async function initTheme(): Promise<void> {
  if (initialized) return
  initialized = true

  mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
  mediaQuery.addEventListener('change', handleSystemPreferenceChange)

  try {
    theme.value = await invoke<ThemeSetting>('get_theme')
  } catch (e: unknown) {
    console.error('Impossible de charger le thème configuré.', e)
  }
  applyResolvedTheme(theme.value)
}

/**
 * Change le thème d'affichage : appliqué immédiatement, puis persisté.
 *
 * @param value - Thème à appliquer (`'clair'`, `'sombre'` ou `'systeme'`).
 * @returns `null` si la persistance a réussi, sinon un message d'erreur à
 *   afficher à l'utilisateur (l'affichage reste appliqué même en cas
 *   d'échec de la sauvegarde).
 */
export async function setTheme(value: ThemeSetting): Promise<string | null> {
  theme.value = value
  applyResolvedTheme(value)
  try {
    await invoke('set_theme', { theme: value })
    return null
  } catch (e: unknown) {
    return extractErrorMessage(e, "Impossible d'enregistrer le thème.")
  }
}

/** Expose le thème courant (réactif) et la fonction pour le modifier. */
export function useTheme() {
  return { theme, setTheme }
}
