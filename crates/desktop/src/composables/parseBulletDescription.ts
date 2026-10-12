/**
 * Découpage d'une description à puces (« • ») en liste de lignes affichables.
 */

/**
 * Découpe une description saisie avec des puces (« • ») en tableau de lignes.
 *
 * @param text - Texte brut contenant des puces.
 * @returns Lignes non vides, trimées.
 */
export function parseBulletDescription(text: string): string[] {
  return text
    .split('•')
    .map(s => s.trim())
    .filter(s => s.length > 0)
}
