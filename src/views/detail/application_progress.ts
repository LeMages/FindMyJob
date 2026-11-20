/**
 * Barre de progression en cinq étapes de la fiche candidature (À postuler,
 * Envoyée, Entretiens, Offre, Acceptée), dérivée du statut et, pour une
 * candidature clôturée, de son historique de suivi.
 */

// Importation des modules internes
import { TRASHED_STATUSES } from '@/composables/candidature_display'

/** Étapes de la barre, dans l'ordre. */
export const PROGRESS_STAGES = ['to_apply', 'sent', 'interviews', 'offer', 'accepted'] as const

/** Identifiant d'une étape de la barre. */
export type ProgressStage = (typeof PROGRESS_STAGES)[number]

/** État d'affichage d'une étape : franchie, courante, arrêt d'une candidature clôturée, à venir. */
export type ProgressStageState = 'done' | 'on' | 'stop' | 'todo'

/** Barre calculée : état de chaque étape et index de l'étape courante (ou d'arrêt). */
export interface ApplicationProgress {
  stages: { key: ProgressStage; state: ProgressStageState }[]
  /** La candidature est clôturée (refus, abandon, ghosting, offre déclinée). */
  closed: boolean
  /** Index de l'étape courante, ou de l'étape où une candidature clôturée s'est arrêtée. */
  currentIndex: number
}

/** Étape de chaque statut ouvert (les statuts clôturés sont déduits de l'historique). */
const STAGE_OF_STATUS: Record<string, number> = {
  a_postuler: 0,
  postule: 1,
  relance: 1,
  entretien: 2,
  entretien_passe: 2,
  offre: 3,
  accepte: 4,
}

/** Événements qui prouvent qu'une étape a été atteinte, de la plus avancée à la moins avancée. */
const STAGE_EVIDENCE: { index: number; events: string[] }[] = [
  { index: 3, events: ['offre_recue'] },
  { index: 2, events: ['entretien_planifie', 'entretien_passe'] },
  { index: 1, events: ['candidature_envoyee'] },
]

/**
 * Étape où une candidature clôturée s'est arrêtée : l'offre si elle a été
 * déclinée ou reçue, les entretiens si un entretien a été planifié, l'envoi
 * si la candidature a été envoyée, sinon « À postuler » (désistement avant
 * envoi).
 *
 * @param status - Statut clôturé.
 * @param eventTypes - Types des événements de l'historique.
 * @returns Index de l'étape d'arrêt.
 */
function closedStageIndex(status: string, eventTypes: string[]): number {
  if (status === 'offre_refusee') return 3
  const reached = STAGE_EVIDENCE.find(({ events }) => events.some((e) => eventTypes.includes(e)))
  return reached?.index ?? 0
}

/**
 * Calcule la barre de progression d'une candidature. Les étapes avant
 * l'étape courante sont franchies ; pour une candidature clôturée, l'étape
 * d'arrêt passe en `stop` (affichée en rose avec le libellé du statut final).
 *
 * @param status - Statut brut de la candidature.
 * @param history - Historique de suivi (seul `event_type` est lu).
 * @returns L'état de chaque étape.
 */
export function applicationProgress(status: string, history: { event_type: string }[]): ApplicationProgress {
  const closed = TRASHED_STATUSES.includes(status)
  const currentIndex = closed
    ? closedStageIndex(status, history.map((h) => h.event_type))
    : (STAGE_OF_STATUS[status] ?? 0)
  const stages = PROGRESS_STAGES.map((key, i) => {
    let state: ProgressStageState = 'todo'
    if (i < currentIndex) state = 'done'
    else if (i === currentIndex) state = closed ? 'stop' : 'on'
    return { key, state }
  })
  return { stages, closed, currentIndex }
}
