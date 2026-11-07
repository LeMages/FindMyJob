/**
 * Formatage lisible d'une taille de fichier en octets, unités dans la langue
 * de l'interface (o/Ko/Mo en français, B/KB/MB en anglais).
 */

// Importation des modules internes
import { t } from '@/i18n'

/**
 * Formate une taille de fichier en octets vers une chaîne lisible (o/Ko/Mo ou B/KB/MB).
 *
 * @param bytes - Taille en octets.
 * @returns Chaîne formatée, ex. "184 Ko".
 */
export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} ${t('common.file_size.b')}`
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} ${t('common.file_size.kb')}`
  return `${(bytes / (1024 * 1024)).toFixed(1)} ${t('common.file_size.mb')}`
}
