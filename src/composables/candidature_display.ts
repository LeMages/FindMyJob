/**
 * Libellés et regroupements d'affichage partagés entre le kanban de suivi et
 * la page de détail d'une candidature — dérivés du statut brut stocké en base.
 * Les libellés sont traduits à l'appel (langue de l'interface) ; les valeurs
 * stockées ne changent jamais.
 */

// Importation des modules internes
import { t, te } from '@/i18n'

// Statuts terminaux négatifs : la candidature n'évoluera plus et n'a pas
// débouché sur une acceptation — elle sort du pipeline principal et va en
// corbeille. `accepte` est aussi un statut terminal mais reste affiché sur le
// board puisque c'est une issue positive. `offre` n'est pas terminal : c'est
// une étape intermédiaire (l'offre peut encore être acceptée ou déclinée).
export const TRASHED_STATUSES = ['abandon', 'refus', 'ghosting', 'offre_refusee']

export interface SelectOption {
  value: string
  label: string
}

/**
 * Construit des options de sélection dont le libellé est traduit à l'appel :
 * appelée dans un rendu ou un `computed`, la liste se re-traduit au
 * changement de langue de l'interface.
 *
 * @param values - Valeurs stockées en base (jamais traduites).
 * @param keyPrefix - Préfixe des clés de traduction (`<prefix>.<valeur>`).
 * @returns Options `{ value, label }`.
 */
function translatedOptions(values: readonly string[], keyPrefix: string): SelectOption[] {
  return values.map((value) => ({ value, label: t(`${keyPrefix}.${value}`) }))
}

/**
 * Libellé traduit d'une valeur brute, avec repli sur la valeur elle-même si
 * aucune clé n'existe (valeur inconnue du frontend).
 */
function translatedLabel(keyPrefix: string, value: string): string {
  const key = `${keyPrefix}.${value}`
  return te(key) ? t(key) : value
}

/**
 * Types de contrat proposés à la création/modification d'une candidature.
 *
 * @returns Options traduites (valeurs inchangées).
 */
export function typeContratOptions(): SelectOption[] {
  return translatedOptions(['stage', 'alternance', 'interim', 'cdd', 'cdi'], 'contract')
}

/**
 * Types d'entretien proposés au formulaire de planification.
 *
 * @returns Options traduites (valeurs inchangées).
 */
export function interviewTypeOptions(): SelectOption[] {
  return translatedOptions(['rh', 'technique', 'manager', 'final', 'autre'], 'interview.type')
}

/**
 * Modalités d'entretien proposées au formulaire de planification.
 *
 * @returns Options traduites (valeurs inchangées).
 */
export function modaliteOptions(): SelectOption[] {
  return translatedOptions(['visio', 'presentiel', 'telephone'], 'interview.modalite')
}

// Les sources sont stockées en base sous leur libellé français complet
// (`candidatures.source`) : la valeur ne change jamais, seul l'affichage est
// traduit via un identifiant stable.
const SOURCE_VALUES: { value: string; key: string }[] = [
  { value: 'Plateforme en ligne (Indeed, LinkedIn, Welcome to the Jungle...)', key: 'online_platform' },
  { value: "Site carrière de l'entreprise (formulaire sur le site)", key: 'company_site' },
  { value: 'Mail (candidature directe par email)', key: 'email' },
  { value: 'Cooptation / Réseau (recommandation, contact perso)', key: 'referral' },
  { value: 'Forum / Salon emploi (rencontre physique)', key: 'job_fair' },
  { value: 'École (job board, offres des profs / asso)', key: 'school' },
  { value: 'Autre', key: 'other' },
]

/**
 * Plateformes d'origine de l'offre proposées à la création/modification
 * d'une candidature.
 *
 * @returns Options traduites (valeurs stockées en français, inchangées).
 */
export function sourceOptions(): SelectOption[] {
  return SOURCE_VALUES.map(({ value, key }) => ({ value, label: t(`source.${key}`) }))
}

/** Classes Tailwind (fond + texte) associées à chaque statut brut de candidature. */
const STATUS_COLORS: Record<string, string> = {
  a_postuler: 'bg-neutral-100 text-neutral-600',
  postule: 'bg-blue-100 text-blue-700',
  relance: 'bg-amber-100 text-amber-700',
  entretien: 'bg-purple-100 text-purple-700',
  entretien_passe: 'bg-violet-100 text-violet-700',
  offre: 'bg-emerald-100 text-emerald-700',
  refus: 'bg-rose-100 text-rose-700',
  abandon: 'bg-neutral-200 text-neutral-500',
  ghosting: 'bg-neutral-200 text-neutral-500',
  accepte: 'bg-emerald-100 text-emerald-700',
  offre_refusee: 'bg-rose-100 text-rose-700',
}

/** Classes Tailwind (fond + bordure + texte) associées à chaque type d'entretien, utilisées dans le planning. */
const INTERVIEW_TYPE_COLORS: Record<string, string> = {
  rh: 'bg-blue-50 border-blue-200 text-blue-700',
  technique: 'bg-violet-50 border-violet-200 text-violet-700',
  manager: 'bg-amber-50 border-amber-300 text-amber-700',
  final: 'bg-red-50 border-red-200 text-red-700',
  autre: 'bg-neutral-100 border-neutral-300 text-neutral-700',
}

/** Classes Tailwind de fond de la pastille de statut d'un entretien dans le planning. */
const INTERVIEW_STATUS_DOT_COLORS: Record<'passed' | 'overdue', string> = {
  passed: 'bg-green-500',
  overdue: 'bg-red-500',
}

/**
 * Libellé d'affichage d'un statut brut de candidature.
 *
 * @param status - Statut brut de la candidature.
 * @returns Libellé lisible, ou le statut brut si inconnu.
 */
export function statusLabel(status: string): string {
  return translatedLabel('status', status)
}

/**
 * Couleur d'affichage d'un statut brut de candidature.
 *
 * @param status - Statut brut de la candidature.
 * @returns Classes Tailwind de fond et de texte, grises par défaut si le statut est inconnu.
 */
export function statusColor(status: string): string {
  return STATUS_COLORS[status] ?? 'bg-neutral-100 text-neutral-600'
}

/**
 * Libellé d'affichage d'un type d'événement de la timeline de suivi.
 *
 * @param eventType - Type d'événement brut (`entry.event_type`).
 * @returns Libellé lisible, ou le type brut si inconnu.
 */
export function eventLabel(eventType: string): string {
  return translatedLabel('event.history', eventType)
}

/**
 * Libellé d'action d'un événement proposable (popup d'ajout d'événement).
 *
 * @param eventType - Type d'événement brut (`AvailableEvent.event_type`).
 * @param backendLabel - Libellé renvoyé par le backend, utilisé si le type est inconnu.
 * @returns Libellé traduit, ou le libellé du backend.
 */
export function eventActionLabel(eventType: string, backendLabel: string): string {
  const key = `event.action.${eventType}`
  return te(key) ? t(key) : backendLabel
}

/**
 * Libellé d'affichage d'un type d'entretien.
 *
 * @param type - Type d'entretien brut, `null` si non renseigné.
 * @returns Libellé lisible, le type brut si inconnu, ou « Entretien » si absent.
 */
export function interviewTypeLabel(type: string | null): string {
  if (!type) return t('interview.type_default')
  return translatedLabel('interview.type', type)
}

/**
 * Libellé d'affichage d'une modalité d'entretien.
 *
 * @param modalite - Modalité brute, `null` si non renseignée.
 * @returns Libellé lisible, ou la modalité brute si inconnue.
 */
export function modaliteLabel(modalite: string): string {
  return translatedLabel('interview.modalite', modalite)
}

/**
 * Couleur d'affichage d'un type d'entretien, utilisée pour les blocs et chips
 * de la vue planning.
 *
 * @param type - Type d'entretien brut, `null` si non renseigné.
 * @returns Classes Tailwind de fond, bordure et texte, grises par défaut si le type est inconnu.
 */
export function interviewTypeColor(type: string | null): string {
  if (!type) return INTERVIEW_TYPE_COLORS.autre
  return INTERVIEW_TYPE_COLORS[type] ?? INTERVIEW_TYPE_COLORS.autre
}

/**
 * Couleur de la pastille de statut d'un entretien dans le planning.
 *
 * @param status - Statut de pastille (`'passed'` ou `'overdue'`).
 * @returns Classe Tailwind de fond de la pastille.
 */
export function interviewStatusDotColor(status: 'passed' | 'overdue'): string {
  return INTERVIEW_STATUS_DOT_COLORS[status]
}
