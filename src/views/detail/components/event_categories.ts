/**
 * Regroupement par catégorie des événements proposés à l'ajout (popup
 * « Ajouter un événement »). Les catégories sont des identifiants stables :
 * le regroupement ne dépend pas de la langue, seul le titre affiché est
 * traduit (`event.category.<id>`).
 */

// Importation des modules internes
import type { AvailableEvent } from '@/types'

/** Catégorie d'événement ; `other` recueille un type pas encore classé. */
export type EventCategory = 'communication' | 'interview' | 'follow_up' | 'outcome' | 'other'

const EVENT_CATEGORIES: Record<string, Exclude<EventCategory, 'other'>> = {
  message_envoye: 'communication',
  message_recu: 'communication',
  appel: 'communication',
  relance: 'communication',
  entretien_planifie: 'interview',
  entretien_passe: 'interview',
  note: 'follow_up',
  refus_recu: 'outcome',
  ghosting: 'outcome',
  desistement: 'outcome',
  offre_recue: 'outcome',
  accepter_offre: 'outcome',
  decliner_offre: 'outcome',
}

const ORDER_CATEGORIES: Exclude<EventCategory, 'other'>[] = ['communication', 'interview', 'follow_up', 'outcome']

/** Groupe d'événements d'une même catégorie. */
export interface EventGroup {
  category: EventCategory
  items: AvailableEvent[]
}

/**
 * Regroupe les événements disponibles par catégorie, dans l'ordre
 * d'affichage, en omettant les catégories vides. Un type inconnu du
 * frontend tombe dans un dernier groupe `other` plutôt que de disparaître.
 *
 * @param events - Événements renvoyés par `get_available_events`.
 * @returns Groupes non vides, dans l'ordre d'affichage.
 */
export function groupEvents(events: AvailableEvent[]): EventGroup[] {
  const groups: EventGroup[] = ORDER_CATEGORIES
    .map((category) => ({ category, items: events.filter((ev) => EVENT_CATEGORIES[ev.event_type] === category) }))
    .filter((group) => group.items.length > 0)
  const unmapped = events.filter((ev) => !(ev.event_type in EVENT_CATEGORIES))
  if (unmapped.length > 0) groups.push({ category: 'other', items: unmapped })
  return groups
}
