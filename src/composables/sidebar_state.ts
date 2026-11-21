/**
 * Règles pures de l'état replié / déplié de la barre latérale, sans Vue ni
 * Tauri (testées par `sidebar_state.test.ts`) :
 * - en fenêtre large, l'état suit la préférence enregistrée de l'utilisateur ;
 * - en fenêtre étroite, la barre est repliée d'office, mais l'utilisateur peut
 *   la déplier : ce choix ne vaut que tant que la fenêtre reste étroite et ne
 *   modifie pas la préférence enregistrée.
 */

/** Largeur de fenêtre (px) en dessous de laquelle la barre se replie d'office. */
export const SIDEBAR_AUTO_COLLAPSE_WIDTH = 1100

/**
 * État de la barre latérale.
 *
 * Attributes:
 *   preferenceCollapsed - Préférence enregistrée (vaut en fenêtre large).
 *   narrow - `true` si la fenêtre est plus étroite que le seuil.
 *   narrowOverride - Choix explicite fait en fenêtre étroite (`null` si aucun).
 */
export interface SidebarState {
  preferenceCollapsed: boolean
  narrow: boolean
  narrowOverride: boolean | null
}

/**
 * Indique si une largeur de fenêtre est sous le seuil de repli automatique.
 *
 * @param width - Largeur de la fenêtre en pixels.
 * @returns `true` si la barre doit être repliée d'office.
 */
export function isNarrowWidth(width: number): boolean {
  return width < SIDEBAR_AUTO_COLLAPSE_WIDTH
}

/**
 * Calcule l'état affiché de la barre latérale.
 *
 * @param s - État courant.
 * @returns `true` si la barre est affichée repliée.
 */
export function isCollapsed(s: SidebarState): boolean {
  if (s.narrow) return s.narrowOverride ?? true
  return s.preferenceCollapsed
}

/**
 * Applique un changement de largeur de fenêtre. Repasser au-dessus du seuil
 * (ou en dessous) oublie le choix fait en fenêtre étroite.
 *
 * @param s - État courant.
 * @param narrow - Nouvelle valeur de « fenêtre étroite ».
 * @returns Nouvel état.
 */
export function withNarrow(s: SidebarState, narrow: boolean): SidebarState {
  if (s.narrow === narrow) return s
  return { ...s, narrow, narrowOverride: null }
}

/**
 * Bascule la barre latérale (bouton Réduire / Déplier).
 *
 * @param s - État courant.
 * @returns Nouvel état, et `persist` : `true` si la préférence enregistrée a
 *   changé et doit être sauvegardée (bascule en fenêtre large uniquement).
 */
export function toggled(s: SidebarState): { state: SidebarState; persist: boolean } {
  const next = !isCollapsed(s)
  if (s.narrow) return { state: { ...s, narrowOverride: next }, persist: false }
  return { state: { ...s, preferenceCollapsed: next }, persist: true }
}
