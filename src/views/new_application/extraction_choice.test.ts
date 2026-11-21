/**
 * Tests du choix de l'option « Conseillé » après un échec d'analyse.
 */

// Importation des modules externes
import { describe, expect, it } from 'vitest'

// Importation des modules internes
import { recommendedOption } from './extraction_choice'

describe('recommendedOption', () => {
  it('conseille la capture quand elle est possible', () => {
    expect(recommendedOption(true)).toBe('capture')
  })

  it('conseille le collage quand la capture ne peut pas aider', () => {
    expect(recommendedOption(false)).toBe('paste')
  })
})
