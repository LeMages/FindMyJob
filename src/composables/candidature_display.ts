/**
 * Libellés et regroupements d'affichage partagés entre le kanban de suivi et
 * la page de détail d'une candidature — dérivés du statut brut stocké en base.
 */

/** Colonnes du kanban, chacune regroupant un ou plusieurs statuts bruts. */
export const KANBAN_COLUMNS: { title: string; statuses: string[] }[] = [
  { title: 'À postuler',          statuses: ['a_postuler'] },
  { title: 'Candidature envoyée', statuses: ['postule'] },
  { title: 'Discussion',          statuses: ['relance'] },
  { title: 'Entretien',           statuses: ['entretien'] },
  { title: 'Accepté',             statuses: ['offre', 'refus', 'abandon', 'ghosting'] },
]

// Statuts terminaux négatifs : la candidature n'évoluera plus et n'a pas
// débouché sur une offre — elle sort du pipeline principal et va en corbeille.
// `offre` est aussi un statut terminal mais reste affiché sur le board
// puisque c'est une issue positive.
export const TRASHED_STATUSES = ['abandon', 'refus', 'ghosting']

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
  offre: 'Offre',
  refus: 'Refus',
  abandon: 'Abandon',
  ghosting: 'Ghosté',
}

/** Classes Tailwind (fond + texte) associées à chaque statut brut de candidature. */
const STATUS_COLORS: Record<string, string> = {
  a_postuler: 'bg-neutral-100 text-neutral-600',
  postule: 'bg-blue-100 text-blue-700',
  relance: 'bg-amber-100 text-amber-700',
  entretien: 'bg-purple-100 text-purple-700',
  offre: 'bg-emerald-100 text-emerald-700',
  refus: 'bg-rose-100 text-rose-700',
  abandon: 'bg-neutral-200 text-neutral-500',
  ghosting: 'bg-neutral-200 text-neutral-500',
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
  message_envoye: 'Message envoyé',
  message_recu: 'Message reçu',
  appel: 'Appel',
  note: 'Note',
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
