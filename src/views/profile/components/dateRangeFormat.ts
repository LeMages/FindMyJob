/**
 * Formatage des périodes de dates affichées dans les sections du profil
 * (expériences, formations, projets). Regroupé ici car utilisé par plusieurs
 * composants de `views/profile/components/`.
 */

export const MONTHS = ['Jan', 'Fév', 'Mar', 'Avr', 'Mai', 'Juin', 'Juil', 'Août', 'Sept', 'Oct', 'Nov', 'Déc']

/**
 * Construit la liste des années sélectionnables dans les formulaires du profil
 * (de deux ans dans le futur à trente ans dans le passé).
 *
 * @returns Années triées décroissantes.
 */
export function buildYearRange(): number[] {
  const cur = new Date().getFullYear()
  const years: number[] = []
  for (let y = cur + 2; y >= cur - 30; y--) years.push(y)
  return years
}

/**
 * Formate une date `YYYY-MM-DD` en « Mois Année » (ex. « Juin 2024 »).
 *
 * @param d - Date au format `YYYY-MM-DD`, ou `null`.
 * @returns Date formatée, ou `'?'` si absente.
 */
export function fmtDate(d: string | null): string {
  if (!d) return '?'
  const parts = d.split('-')
  if (parts.length < 2) return d
  return `${MONTHS[parseInt(parts[1], 10) - 1] ?? parts[1]} ${parts[0]}`
}

/**
 * Formate une période mois/année (ex. « Juin 2024 — Présent »).
 *
 * @param start - Date de début.
 * @param end - Date de fin, ou `null` si en cours.
 * @returns Période formatée.
 */
export function fmtRange(start: string | null, end: string | null): string {
  return `${fmtDate(start)} — ${end ? fmtDate(end) : 'Présent'}`
}

/**
 * Extrait l'année d'une date `YYYY-MM-DD` (utilisé pour formations et projets,
 * qui n'affichent que l'année).
 *
 * @param d - Date au format `YYYY-MM-DD`, ou `null`.
 * @returns Année, ou `'?'` si absente.
 */
export function fmtYear(d: string | null): string {
  if (!d) return '?'
  return d.split('-')[0] ?? d
}

/**
 * Formate une période en années seules (ex. « 2022 — Présent »).
 *
 * @param start - Date de début.
 * @param end - Date de fin, ou `null` si en cours.
 * @returns Période formatée.
 */
export function fmtYearRange(start: string | null, end: string | null): string {
  return `${fmtYear(start)} — ${end ? fmtYear(end) : 'Présent'}`
}
