/**
 * Tests de la règle « une seule carte du profil éditable à la fois ».
 */

// Importation des modules externes
import { describe, expect, it } from 'vitest'

// Importation des modules internes
import { cardMode } from './profile_sections'

describe('cardMode', () => {
  it('laisse toutes les cartes en lecture sans édition en cours', () => {
    expect(cardMode('info', null)).toBe('idle')
    expect(cardMode('projets', null)).toBe('idle')
  })

  it('édite la carte choisie et estompe les autres', () => {
    expect(cardMode('skills', 'skills')).toBe('editing')
    expect(cardMode('info', 'skills')).toBe('dimmed')
    expect(cardMode('experiences', 'skills')).toBe('dimmed')
  })
})
