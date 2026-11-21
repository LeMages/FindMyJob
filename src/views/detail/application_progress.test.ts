/**
 * Tests de la correspondance statut / étape de la barre de progression de la
 * fiche candidature, candidatures clôturées comprises.
 */

// Importation des modules externes
import { describe, expect, it } from 'vitest'

// Importation des modules internes
import { applicationProgress } from './application_progress'

/** Historique réduit aux types d'événements. */
const history = (...types: string[]) => types.map((event_type) => ({ event_type }))

/** États des cinq étapes, dans l'ordre. */
const states = (status: string, events: string[] = []) =>
  applicationProgress(status, history(...events)).stages.map((s) => s.state)

describe('applicationProgress — candidatures ouvertes', () => {
  it.each([
    ['a_postuler', 0],
    ['postule', 1],
    ['relance', 1],
    ['entretien', 2],
    ['entretien_passe', 2],
    ['offre', 3],
    ['accepte', 4],
  ])('place %s sur l\'étape %i', (status, index) => {
    const progress = applicationProgress(status, [])
    expect(progress.closed).toBe(false)
    expect(progress.currentIndex).toBe(index)
    expect(progress.stages[index].state).toBe('on')
  })

  it('marque les étapes précédentes comme franchies et les suivantes à venir', () => {
    expect(states('entretien')).toEqual(['done', 'done', 'on', 'todo', 'todo'])
    expect(states('accepte')).toEqual(['done', 'done', 'done', 'done', 'on'])
  })
})

describe('applicationProgress — candidatures clôturées', () => {
  it('arrête un refus après le seul envoi sur « Envoyée »', () => {
    expect(states('refus', ['creation', 'candidature_envoyee', 'refus_recu'])).toEqual(['done', 'stop', 'todo', 'todo', 'todo'])
  })

  it('arrête un refus après un entretien sur « Entretiens »', () => {
    const events = ['creation', 'candidature_envoyee', 'entretien_planifie', 'entretien_passe', 'refus_recu']
    expect(states('refus', events)).toEqual(['done', 'done', 'stop', 'todo', 'todo'])
  })

  it('arrête un ghosting après une relance sur « Envoyée »', () => {
    expect(states('ghosting', ['creation', 'candidature_envoyee', 'relance', 'ghosting'])).toEqual(['done', 'stop', 'todo', 'todo', 'todo'])
  })

  it('arrête un désistement avant envoi sur « À postuler »', () => {
    expect(states('abandon', ['creation', 'desistement'])).toEqual(['stop', 'todo', 'todo', 'todo', 'todo'])
  })

  it('arrête un désistement après une offre reçue sur « Offre »', () => {
    const events = ['creation', 'candidature_envoyee', 'entretien_planifie', 'offre_recue', 'desistement']
    expect(states('abandon', events)).toEqual(['done', 'done', 'done', 'stop', 'todo'])
  })

  it('arrête une offre déclinée sur « Offre », même sans historique', () => {
    expect(states('offre_refusee')).toEqual(['done', 'done', 'done', 'stop', 'todo'])
  })

  it('signale la clôture', () => {
    expect(applicationProgress('refus', []).closed).toBe(true)
  })
})
