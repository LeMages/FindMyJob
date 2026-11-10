/**
 * Formatage d'une date ISO 8601 en date et heure, au format de la langue de
 * l'interface.
 */

// Importation des modules internes
import { dateLocale } from '@/i18n'

/**
 * Formate une date ISO 8601 en "JJ/MM/AAAA HH:mm" (ordre jour/mois en FR comme en EN, `en-GB`).
 *
 * @param iso - Date ISO 8601 (ex. `created_at` renvoyé par le backend).
 * @returns Date et heure (ex. "24/09/2026 14:02"), ou `'?'` si la date est invalide.
 */
export function formatDateTime(iso: string): string {
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return '?'
  return `${d.toLocaleDateString(dateLocale())} ${d.toLocaleTimeString(dateLocale(), { hour: '2-digit', minute: '2-digit' })}`
}
