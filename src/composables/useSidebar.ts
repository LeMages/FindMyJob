/**
 * État replié / déplié de la barre latérale. Singleton au niveau module,
 * comme `useTheme.ts` : initialisé par `main.ts` avant le premier rendu
 * (préférence enregistrée côté Rust, setting `sidebar_collapsed`), puis
 * suivi de la largeur de la fenêtre. Les règles sont dans `sidebar_state.ts`.
 */

// Importation des modules Vue / framework
import { computed, ref } from 'vue'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { backendErrorMessage } from '@/composables/backend_error_message'
import { isCollapsed, isNarrowWidth, toggled, withNarrow, type SidebarState } from '@/composables/sidebar_state'
import { t } from '@/i18n'
import { showToast } from '@/stores/app'

const state = ref<SidebarState>({ preferenceCollapsed: false, narrow: false, narrowOverride: null })
let initialized = false

/** Met à jour l'état « fenêtre étroite » d'après la largeur courante. */
function handleResize() {
  state.value = withNarrow(state.value, isNarrowWidth(window.innerWidth))
}

/**
 * Initialise l'état de la barre latérale : charge la préférence enregistrée
 * et écoute les redimensionnements. Idempotent (voir `main.ts`).
 */
export async function initSidebar(): Promise<void> {
  if (initialized) return
  initialized = true

  try {
    state.value.preferenceCollapsed = await invoke<boolean>('get_sidebar_collapsed')
  } catch (e: unknown) {
    console.error("Impossible de charger l'état de la barre latérale.", e)
  }
  handleResize()
  window.addEventListener('resize', handleResize)
}

/**
 * Bascule la barre latérale. En fenêtre large, la préférence est enregistrée ;
 * un échec d'enregistrement est signalé sans annuler l'affichage.
 */
async function toggleSidebar(): Promise<void> {
  const { state: next, persist } = toggled(state.value)
  state.value = next
  if (!persist) return
  try {
    await invoke('set_sidebar_collapsed', { collapsed: next.preferenceCollapsed })
  } catch (e: unknown) {
    showToast(backendErrorMessage(e, t('sidebar.save_error')), 'error')
  }
}

/** Expose l'état affiché (réactif) et la bascule. */
export function useSidebar() {
  const collapsed = computed(() => isCollapsed(state.value))
  return { collapsed, toggleSidebar }
}
