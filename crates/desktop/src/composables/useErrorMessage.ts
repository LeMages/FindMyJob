/**
 * Extraction uniforme d'un message d'erreur lisible depuis une valeur `unknown`
 * (les erreurs renvoyées par `invoke()` de Tauri sont sérialisées et n'ont pas
 * de type garanti côté frontend).
 */

/**
 * Extrait un message d'erreur affichable depuis une valeur d'erreur quelconque.
 *
 * @param e - Erreur interceptée (objet avec `message`, chaîne, ou autre).
 * @param fallback - Message utilisé si aucune information exploitable n'est trouvée.
 * @returns Message d'erreur à afficher à l'utilisateur.
 */
export function extractErrorMessage(e: unknown, fallback: string): string {
  if (e && typeof e === 'object' && 'message' in e) {
    return (e as { message: string }).message
  }
  if (typeof e === 'string') {
    return e
  }
  return fallback
}
