/**
 * Utilitaires de dates pour l'onglet planning (semaine, mois, formatage dans
 * la langue de l'interface). Regroupé ici car utilisé par plusieurs
 * composants de `views/tracking/components/`.
 */

// Importation des modules internes
import { dateLocale, t } from '@/i18n'

/**
 * Parse la date d'un entretien en objet `Date`, en heure locale.
 *
 * Tolère le format `datetime-local` du formulaire de saisie
 * (`"YYYY-MM-DDTHH:mm"`) ainsi que le format renvoyé par SQLite
 * (`"YYYY-MM-DD HH:MM:SS"`), dont l'espace est remplacé par un `T` pour
 * obtenir une chaîne ISO 8601 sans fuseau — interprétée en heure locale par
 * le constructeur `Date`.
 *
 * @param value - Date brute de l'entretien, `null` si non renseignée.
 * @returns Date valide, ou `null` si absente ou invalide.
 */
export function parseInterviewDate(value: string | null): Date | null {
  if (!value) return null
  const isoLike = value.includes(' ') ? value.replace(' ', 'T') : value
  const date = new Date(isoLike)
  return Number.isNaN(date.getTime()) ? null : date
}

/**
 * Renvoie le lundi de la semaine contenant la date donnée, à minuit.
 *
 * @param d - Date de référence.
 * @returns Nouvelle date, lundi 00:00 de la semaine de `d`.
 */
export function startOfWeek(d: Date): Date {
  const result = new Date(d)
  result.setHours(0, 0, 0, 0)
  // getDay() renvoie 0 pour dimanche : on le traite comme le 7e jour pour
  // que la semaine commence toujours un lundi.
  const dayIndex = (result.getDay() + 6) % 7
  result.setDate(result.getDate() - dayIndex)
  return result
}

/**
 * Ajoute (ou retire) un nombre de jours à une date.
 *
 * @param d - Date de départ.
 * @param n - Nombre de jours à ajouter (négatif pour reculer).
 * @returns Nouvelle date.
 */
export function addDays(d: Date, n: number): Date {
  const result = new Date(d)
  result.setDate(result.getDate() + n)
  return result
}

/**
 * Ajoute (ou retire) un nombre de mois à une date.
 *
 * @param d - Date de départ.
 * @param n - Nombre de mois à ajouter (négatif pour reculer).
 * @returns Nouvelle date.
 */
export function addMonths(d: Date, n: number): Date {
  const result = new Date(d)
  result.setMonth(result.getMonth() + n)
  return result
}

/**
 * Renvoie le premier jour du mois contenant la date donnée, à minuit.
 *
 * @param d - Date de référence.
 * @returns Nouvelle date, 1er du mois de `d`.
 */
export function startOfMonth(d: Date): Date {
  return new Date(d.getFullYear(), d.getMonth(), 1)
}

/**
 * Construit la grille de 42 jours (6 semaines) affichée en vue mois, en
 * débordant sur les mois voisins pour toujours commencer un lundi.
 *
 * @param d - Date de référence (n'importe quel jour du mois affiché).
 * @returns 42 dates consécutives couvrant la grille du mois.
 */
export function monthGridDays(d: Date): Date[] {
  const firstDay = startOfWeek(startOfMonth(d))
  return Array.from({ length: 42 }, (_, i) => addDays(firstDay, i))
}

/**
 * Compare deux dates au jour près (ignore l'heure).
 *
 * @param a - Première date.
 * @param b - Seconde date.
 * @returns `true` si `a` et `b` tombent le même jour.
 */
export function isSameDay(a: Date, b: Date): boolean {
  return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate()
}

/**
 * Indique si une date correspond au jour courant.
 *
 * @param d - Date à tester.
 * @returns `true` si `d` est aujourd'hui.
 */
export function isToday(d: Date): boolean {
  return isSameDay(d, new Date())
}

/**
 * Indique si une date est déjà passée par rapport à maintenant.
 *
 * @param d - Date à tester.
 * @returns `true` si `d` est dans le passé.
 */
export function isPast(d: Date): boolean {
  return d.getTime() < Date.now()
}

/** Statut visuel d'un entretien dans le planning : passé confirmé, en retard, ou rien à signaler. */
export type InterviewStatusDot = 'passed' | 'overdue' | null

/**
 * Détermine la pastille de statut à afficher sur un entretien dans le planning.
 *
 * @param issue - Issue de l'entretien (`en_attente`, `positif`, `negatif`).
 * @param date - Date de l'entretien.
 * @returns `'passed'` si l'entretien a été marqué comme passé, `'overdue'` s'il est en attente et sa date est dépassée, `null` sinon.
 */
export function interviewStatusDot(issue: string, date: Date): InterviewStatusDot {
  if (issue !== 'en_attente') return 'passed'
  return isPast(date) ? 'overdue' : null
}

/**
 * Formate une heure entière au format `"HH:00"`.
 *
 * @param hour - Heure (0-23).
 * @returns Chaîne formatée, ex. `"14:00"`.
 */
export function formatHour(hour: number): string {
  return `${String(hour).padStart(2, '0')}:00`
}

/**
 * Formate l'heure d'une date au format `"HH:mm"`.
 *
 * @param d - Date à formater.
 * @returns Chaîne formatée, ex. `"14:30"`.
 */
export function formatTime(d: Date): string {
  return d.toLocaleTimeString(dateLocale(), { hour: '2-digit', minute: '2-digit' })
}

/**
 * Formate une date au format court « jour de semaine + numéro », pour l'en-tête
 * des colonnes de la vue semaine.
 *
 * @param d - Date à formater.
 * @returns Chaîne formatée, ex. `"lun. 12"`.
 */
export function formatDayLabel(d: Date): string {
  return d.toLocaleDateString(dateLocale(), { weekday: 'short', day: 'numeric' })
}

/**
 * Formate une date au format « mois année », pour l'en-tête de la vue mois.
 *
 * @param d - Date à formater.
 * @returns Chaîne formatée, ex. `"janvier 2026"`.
 */
export function formatMonthLabel(d: Date): string {
  return d.toLocaleDateString(dateLocale(), { month: 'long', year: 'numeric' })
}

/**
 * Formate une date complète avec l'heure, pour la popup de détail d'un entretien.
 *
 * @param d - Date à formater.
 * @returns Chaîne formatée, ex. `"lundi 12 janvier 2026 à 14:30"`.
 */
export function formatFullDate(d: Date): string {
  const datePart = d.toLocaleDateString(dateLocale(), { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric' })
  return `${datePart} ${t('common.date.at')} ${formatTime(d)}`
}
