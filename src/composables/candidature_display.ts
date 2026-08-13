/**
 * Libellés et regroupements d'affichage partagés entre le kanban de suivi et
 * la page de détail d'une candidature — dérivés du statut brut stocké en base.
 */

/** Colonnes du kanban, chacune regroupant un ou plusieurs statuts bruts. */
export const KANBAN_COLUMNS: { title: string; statuses: string[] }[] = [
  { title: 'À postuler',          statuses: ['a_postuler'] },
  { title: 'Candidature envoyée', statuses: ['postule'] },
  { title: 'Discussion',          statuses: ['relance'] },
  { title: 'Entretien',           statuses: ['entretien', 'entretien_passe'] },
  { title: 'Accepté',             statuses: ['offre', 'accepte'] },
]

// Statuts terminaux négatifs : la candidature n'évoluera plus et n'a pas
// débouché sur une acceptation — elle sort du pipeline principal et va en
// corbeille. `accepte` est aussi un statut terminal mais reste affiché sur le
// board puisque c'est une issue positive. `offre` n'est pas terminal : c'est
// une étape intermédiaire (l'offre peut encore être acceptée ou déclinée).
export const TRASHED_STATUSES = ['abandon', 'refus', 'ghosting', 'offre_refusee']

/** Libellés d'affichage des types de contrat. */
export const TYPE_CONTRAT_LABELS: Record<string, string> = {
  stage: 'Stage',
  alternance: 'Alternance',
  cdi: 'CDI',
  cdd: 'CDD',
  interim: 'Intérim',
  freelance: 'Freelance',
}

export interface SelectOption {
  value: string
  label: string
}

/** Types de contrat proposés à la création/modification d'une candidature. */
export const TYPE_CONTRAT_OPTIONS: SelectOption[] = [
  { value: 'stage', label: 'Stage' },
  { value: 'alternance', label: 'Alternance' },
  { value: 'interim', label: 'Intérim' },
  { value: 'cdd', label: 'CDD' },
  { value: 'cdi', label: 'CDI' },
]

/** Types d'entretien proposés au formulaire de planification. */
export const INTERVIEW_TYPE_OPTIONS: SelectOption[] = [
  { value: 'rh', label: 'RH' },
  { value: 'technique', label: 'Technique' },
  { value: 'manager', label: 'Manager' },
  { value: 'final', label: 'Final' },
  { value: 'autre', label: 'Autre' },
]

/** Modalités d'entretien proposées au formulaire de planification. */
export const MODALITE_OPTIONS: SelectOption[] = [
  { value: 'visio', label: 'Visio' },
  { value: 'presentiel', label: 'Présentiel' },
  { value: 'telephone', label: 'Téléphone' },
]

/** Plateformes d'origine de l'offre proposées à la création/modification d'une candidature. */
export const SOURCE_OPTIONS: SelectOption[] = [
  { value: 'Plateforme en ligne (Indeed, LinkedIn, Welcome to the Jungle...)', label: 'Plateforme en ligne (Indeed, LinkedIn, Welcome to the Jungle...)' },
  { value: "Site carrière de l'entreprise (formulaire sur le site)", label: "Site carrière de l'entreprise (formulaire sur le site)" },
  { value: 'Mail (candidature directe par email)', label: 'Mail (candidature directe par email)' },
  { value: 'Cooptation / Réseau (recommandation, contact perso)', label: 'Cooptation / Réseau (recommandation, contact perso)' },
  { value: 'Forum / Salon emploi (rencontre physique)', label: 'Forum / Salon emploi (rencontre physique)' },
  { value: 'École (job board, offres des profs / asso)', label: 'École (job board, offres des profs / asso)' },
  { value: 'Autre', label: 'Autre' },
]

/** Libellés d'affichage des statuts bruts de candidature. */
const STATUS_LABELS: Record<string, string> = {
  a_postuler: 'À postuler',
  postule: 'Candidature envoyée',
  relance: 'Discussion',
  entretien: 'Entretien',
  entretien_passe: 'En attente de réponse',
  offre: 'Offre',
  refus: 'Refus',
  abandon: 'Abandon',
  ghosting: 'Ghosté',
  accepte: 'Accepté',
  offre_refusee: 'Offre déclinée',
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

/** Libellés d'affichage des types d'événement de la timeline de suivi. */
const EVENT_LABELS: Record<string, string> = {
  creation: 'Candidature créée',
  candidature_envoyee: 'Candidature envoyée',
  relance: 'Relance envoyée',
  entretien_planifie: 'Entretien planifié',
  entretien_passe: 'Entretien passé',
  offre_recue: 'Offre reçue',
  refus_recu: 'Refus reçu',
  ghosting: 'Ghosting',
  desistement: 'Désistement',
  accepter_offre: 'Offre acceptée',
  decliner_offre: 'Offre déclinée',
  message_envoye: 'Message envoyé',
  message_recu: 'Message reçu',
  appel: 'Appel',
  note: 'Note',
}

/** Libellés d'affichage des types d'entretien. */
const INTERVIEW_TYPE_LABELS: Record<string, string> = {
  rh: 'RH',
  technique: 'Technique',
  manager: 'Manager',
  final: 'Final',
  autre: 'Autre',
}

/** Libellés d'affichage des modalités d'entretien. */
const MODALITE_LABELS: Record<string, string> = {
  visio: 'Visio',
  presentiel: 'Présentiel',
  telephone: 'Téléphone',
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
 * Trouve le titre de colonne kanban correspondant à un statut brut.
 *
 * @param status - Statut brut de la candidature (ex. `'entretien'`).
 * @returns Titre de la colonne kanban, ou chaîne vide si le statut est inconnu.
 */
export function columnTitleForStatus(status: string): string {
  return KANBAN_COLUMNS.find(col => col.statuses.includes(status))?.title ?? ''
}

/**
 * Libellé d'affichage d'un statut brut de candidature.
 *
 * @param status - Statut brut de la candidature.
 * @returns Libellé lisible, ou le statut brut si inconnu.
 */
export function statusLabel(status: string): string {
  return STATUS_LABELS[status] ?? status
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
  return EVENT_LABELS[eventType] ?? eventType
}

/**
 * Libellé d'affichage d'un type d'entretien.
 *
 * @param type - Type d'entretien brut, `null` si non renseigné.
 * @returns Libellé lisible, le type brut si inconnu, ou « Entretien » si absent.
 */
export function interviewTypeLabel(type: string | null): string {
  if (!type) return 'Entretien'
  return INTERVIEW_TYPE_LABELS[type] ?? type
}

/**
 * Libellé d'affichage d'une modalité d'entretien.
 *
 * @param modalite - Modalité brute, `null` si non renseignée.
 * @returns Libellé lisible, ou la modalité brute si inconnue.
 */
export function modaliteLabel(modalite: string): string {
  return MODALITE_LABELS[modalite] ?? modalite
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
