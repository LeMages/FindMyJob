/**
 * Tests de la mise en forme des statistiques.
 */

// Importation des modules externes
import { describe, expect, it } from 'vitest'

// Importation des modules internes
import { barWidth, percent, roundDays, shortLabel } from './stats_format'

describe('percent', () => {
  it('arrondit à l\'entier et vaut 0 sans total', () => {
    expect(percent(10, 24)).toBe(42)
    expect(percent(1, 3)).toBe(33)
    expect(percent(0, 0)).toBe(0)
  })
})

describe('roundDays', () => {
  it('arrondit les jours et garde l\'absence de mesure', () => {
    expect(roundDays(7.6)).toBe(8)
    expect(roundDays(null)).toBeNull()
  })
})

describe('barWidth', () => {
  it('rapporte à la plus grande valeur, bornée à 100', () => {
    expect(barWidth(2, 4)).toBe(50)
    expect(barWidth(5, 4)).toBe(100)
    expect(barWidth(3, 0)).toBe(0)
  })
})

describe('shortLabel', () => {
  it('retire la précision entre parenthèses', () => {
    expect(shortLabel('Plateforme en ligne (Indeed, LinkedIn...)')).toBe('Plateforme en ligne')
    expect(shortLabel('Autre')).toBe('Autre')
  })
})
