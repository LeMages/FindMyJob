/**
 * Mise en forme des chiffres de l'onglet Statistiques (pourcentages, jours,
 * largeurs de barres). Les calculs eux-mêmes sont faits côté Rust ; ce
 * module ne fait qu'arrondir pour l'affichage.
 */

/**
 * Pourcentage arrondi d'une part sur un total.
 *
 * @param part - Effectif de la part.
 * @param total - Effectif total.
 * @returns Pourcentage entier, 0 pour un total nul.
 */
export function percent(part: number, total: number): number {
  return total > 0 ? Math.round((part / total) * 100) : 0
}

/**
 * Nombre de jours arrondi à l'entier, pour un délai ou une durée.
 *
 * @param days - Durée en jours (fractionnaire), `null` si non mesurable.
 * @returns Jours entiers, `null` si la durée est absente.
 */
export function roundDays(days: number | null): number | null {
  return days === null ? null : Math.round(days)
}

/**
 * Largeur relative d'une barre, en pourcentage de la plus grande valeur.
 *
 * @param value - Valeur de la barre.
 * @param max - Plus grande valeur de la série.
 * @returns Largeur entre 0 et 100.
 */
export function barWidth(value: number, max: number): number {
  return max > 0 ? Math.min(100, (value / max) * 100) : 0
}

/**
 * Libellé court d'une valeur longue de type « Nom (précision) » : la partie
 * avant la parenthèse, pour tenir dans une colonne étroite.
 *
 * @param label - Libellé complet.
 * @returns Libellé sans la précision entre parenthèses.
 */
export function shortLabel(label: string): string {
  const cut = label.indexOf(' (')
  return cut > 0 ? label.slice(0, cut) : label
}
