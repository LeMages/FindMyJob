/**
 * Formatage lisible d'une taille de fichier en octets.
 */

/**
 * Formate une taille de fichier en octets vers une chaîne lisible (o/Ko/Mo).
 *
 * @param bytes - Taille en octets.
 * @returns Chaîne formatée, ex. "184 Ko".
 */
export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} o`
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} Ko`
  return `${(bytes / (1024 * 1024)).toFixed(1)} Mo`
}
