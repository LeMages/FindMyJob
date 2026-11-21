/**
 * Tests du regroupement par catégorie des événements proposés à l'ajout.
 */

// Importation des modules externes
import { describe, expect, it } from 'vitest'

// Importation des modules internes
import { groupEvents } from './event_categories'
import type { AvailableEvent } from '@/types'

/** Événement minimal d'un type donné. */
const ev = (event_type: string): AvailableEvent => ({ event_type, label: event_type, changes_status: false, target_status: null })

describe('groupEvents', () => {
  it('regroupe par catégorie dans l\'ordre d\'affichage, sans groupe vide', () => {
    const groups = groupEvents([ev('desistement'), ev('note'), ev('message_envoye'), ev('relance')])
    expect(groups.map((g) => g.category)).toEqual(['communication', 'follow_up', 'outcome'])
    expect(groups[0].items.map((e) => e.event_type)).toEqual(['message_envoye', 'relance'])
  })

  it('place un type inconnu dans un dernier groupe « other »', () => {
    const groups = groupEvents([ev('appel'), ev('nouveau_type')])
    expect(groups.map((g) => g.category)).toEqual(['communication', 'other'])
    expect(groups[1].items[0].event_type).toBe('nouveau_type')
  })

  it('renvoie une liste vide sans événement', () => {
    expect(groupEvents([])).toEqual([])
  })
})
