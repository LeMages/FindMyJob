/**
 * Choix de la suite proposée quand l'analyse automatique d'un lien échoue :
 * l'option mise en avant (« Conseillé ») sur l'écran de choix.
 */

/** Option de l'écran de choix qui peut être mise en avant. */
export type RecommendedOption = 'capture' | 'paste'

/**
 * Option conseillée après un échec d'analyse : ouvrir la page dans
 * FindMyJob quand la capture peut aider, sinon coller le texte de l'offre
 * (la saisie manuelle n'est jamais conseillée).
 *
 * @param canCapture - La capture de la page peut-elle contourner l'échec ?
 * @returns L'option à mettre en avant.
 */
export function recommendedOption(canCapture: boolean): RecommendedOption {
  return canCapture ? 'capture' : 'paste'
}
