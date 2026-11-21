/**
 * Filtres rapides et regroupement du tableau des candidatures : compteurs par
 * filtre, candidatures ouvertes affichées, groupe « Clôturées » repliable et
 * résumé de ses issues. Logique pure, sans Vue ni appel au backend.
 */

// Importation des modules internes
import { TRASHED_STATUSES } from '@/composables/candidature_display'

/** Filtre rapide du tableau. */
export type PipelineFilter = 'all' | 'todo' | 'active' | 'offer' | 'closed'

/** Ordre d'affichage des filtres rapides. */
export const PIPELINE_FILTERS: readonly PipelineFilter[] = ['all', 'todo', 'active', 'offer', 'closed']

/**
 * Issues d'une candidature clôturée, dans l'ordre du résumé du groupe. Ce
 * sont les statuts terminaux négatifs (`TRASHED_STATUSES`).
 */
export const CLOSED_OUTCOMES = ['refus', 'abandon', 'ghosting', 'offre_refusee'] as const

/** Statuts couverts par chaque filtre autre que « Toutes ». */
const GROUP_STATUSES: Record<Exclude<PipelineFilter, 'all'>, readonly string[]> = {
  todo: ['a_postuler'],
  active: ['postule', 'relance', 'entretien', 'entretien_passe'],
  offer: ['offre', 'accepte'],
  closed: TRASHED_STATUSES,
}

/** Forme minimale d'une candidature utile au regroupement. */
export interface PipelineItem {
  status: string
}

/** Contenu du tableau pour un filtre donné. */
export interface PipelineView<T extends PipelineItem> {
  /** Lignes affichées hors du groupe « Clôturées ». */
  open: T[]
  /** Candidatures clôturées (contenu du groupe). */
  closed: T[]
  /** Le groupe « Clôturées » est affiché. */
  showClosedGroup: boolean
  /** Le groupe est toujours déplié (filtre « Clôturées ») : pas de bascule. */
  closedAlwaysOpen: boolean
}

/**
 * Indique si une candidature est clôturée (issue négative définitive).
 *
 * @param status - Statut brut de la candidature.
 * @returns `true` pour un refus, un abandon, un ghosting ou une offre déclinée.
 */
export function isClosed(status: string): boolean {
  return TRASHED_STATUSES.includes(status)
}

/**
 * Indique si une candidature entre dans un filtre.
 *
 * @param filter - Filtre rapide.
 * @param status - Statut brut de la candidature.
 * @returns `true` si la candidature est comptée par ce filtre.
 */
export function matchesFilter(filter: PipelineFilter, status: string): boolean {
  return filter === 'all' || GROUP_STATUSES[filter].includes(status)
}

/**
 * Compte les candidatures de chaque filtre rapide.
 *
 * @param items - Candidatures de la session.
 * @returns Nombre de candidatures par filtre (« Toutes » = total).
 */
export function filterCounts(items: readonly PipelineItem[]): Record<PipelineFilter, number> {
  const counts = { all: 0, todo: 0, active: 0, offer: 0, closed: 0 }
  for (const filter of PIPELINE_FILTERS) {
    counts[filter] = items.filter((c) => matchesFilter(filter, c.status)).length
  }
  return counts
}

/**
 * Répartit les candidatures entre lignes ouvertes et groupe « Clôturées »
 * selon le filtre. « Toutes » montre les ouvertes puis le groupe replié ;
 * « Clôturées » montre seulement le groupe, déplié ; les autres filtres ne
 * montrent que leurs lignes.
 *
 * @param items - Candidatures de la session, dans l'ordre d'affichage.
 * @param filter - Filtre rapide actif.
 * @returns Lignes ouvertes, candidatures clôturées et état du groupe.
 */
export function pipelineView<T extends PipelineItem>(items: readonly T[], filter: PipelineFilter): PipelineView<T> {
  const closed = items.filter((c) => isClosed(c.status))
  const open = filter === 'all'
    ? items.filter((c) => !isClosed(c.status))
    : filter === 'closed' ? [] : items.filter((c) => matchesFilter(filter, c.status))
  return {
    open,
    closed,
    showClosedGroup: (filter === 'all' || filter === 'closed') && closed.length > 0,
    closedAlwaysOpen: filter === 'closed',
  }
}

/**
 * Résumé du groupe « Clôturées » : nombre de candidatures par issue, dans
 * l'ordre de `CLOSED_OUTCOMES`, issues absentes omises.
 *
 * @param closed - Candidatures clôturées.
 * @returns Paires issue / nombre, sans entrée à zéro.
 */
export function closedSummary(closed: readonly PipelineItem[]): { outcome: string; count: number }[] {
  return CLOSED_OUTCOMES
    .map((outcome) => ({ outcome, count: closed.filter((c) => c.status === outcome).length }))
    .filter((part) => part.count > 0)
}
