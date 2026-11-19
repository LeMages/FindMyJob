/**
 * Tests des filtres rapides et du groupe « Clôturées » du tableau des
 * candidatures.
 */

// Importation des modules externes
import { describe, expect, it } from 'vitest'

// Importation des modules internes
import { closedSummary, filterCounts, isClosed, pipelineView } from './pipeline_groups'

const ITEMS = [
  { id: 1, status: 'a_postuler' },
  { id: 2, status: 'postule' },
  { id: 3, status: 'entretien' },
  { id: 4, status: 'offre' },
  { id: 5, status: 'refus' },
  { id: 6, status: 'ghosting' },
  { id: 7, status: 'refus' },
  { id: 8, status: 'accepte' },
]

const ids = (list: { id: number }[]) => list.map((c) => c.id)

describe('filterCounts', () => {
  it('compte chaque filtre, « Toutes » étant le total', () => {
    expect(filterCounts(ITEMS)).toEqual({ all: 8, todo: 1, active: 2, offer: 2, closed: 3 })
  })

  it('renvoie des zéros sans candidature', () => {
    expect(filterCounts([])).toEqual({ all: 0, todo: 0, active: 0, offer: 0, closed: 0 })
  })
})

describe('pipelineView', () => {
  it('« Toutes » : les ouvertes puis le groupe des clôturées, repliable', () => {
    const view = pipelineView(ITEMS, 'all')
    expect(ids(view.open)).toEqual([1, 2, 3, 4, 8])
    expect(ids(view.closed)).toEqual([5, 6, 7])
    expect(view.showClosedGroup).toBe(true)
    expect(view.closedAlwaysOpen).toBe(false)
  })

  it('« Clôturées » : seulement le groupe, toujours déplié', () => {
    const view = pipelineView(ITEMS, 'closed')
    expect(view.open).toEqual([])
    expect(view.showClosedGroup).toBe(true)
    expect(view.closedAlwaysOpen).toBe(true)
  })

  it('un autre filtre ne montre que ses lignes, sans groupe', () => {
    const view = pipelineView(ITEMS, 'offer')
    expect(ids(view.open)).toEqual([4, 8])
    expect(view.showClosedGroup).toBe(false)
  })

  it('pas de groupe quand aucune candidature n\'est clôturée', () => {
    expect(pipelineView(ITEMS.slice(0, 4), 'all').showClosedGroup).toBe(false)
  })
})

describe('closedSummary', () => {
  it('compte les issues dans l\'ordre fixe, sans les absentes', () => {
    expect(closedSummary(ITEMS.filter((c) => isClosed(c.status)))).toEqual([
      { outcome: 'refus', count: 2 },
      { outcome: 'ghosting', count: 1 },
    ])
  })
})
