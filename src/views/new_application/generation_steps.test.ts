/**
 * Tests de l'état des étapes de la popup de génération.
 */

// Importation des modules externes
import { describe, expect, it } from 'vitest'

// Importation des modules internes
import { generationStepStates } from './generation_steps'
import type { GenerationProgress } from '@/types'

/** Progression à l'étape `current_step` sur 6. */
const at = (current_step: number, status: GenerationProgress['status'] = 'in_progress'): GenerationProgress => ({
  candidature_id: 1, current_step, total_steps: 6, step_label: '', status, error: null,
})

describe('generationStepStates', () => {
  it('démarre sur la première étape avant toute réponse', () => {
    const view = generationStepStates(null, false)
    expect(view.states).toEqual(['run', 'todo', 'todo', 'todo', 'todo', 'todo'])
    expect(view.percent).toBeCloseTo((0.5 / 6) * 100)
  })

  it('marque les étapes passées comme faites', () => {
    expect(generationStepStates(at(3), false).states).toEqual(['done', 'done', 'done', 'run', 'todo', 'todo'])
  })

  it("passe l'étape courante en échec et arrête la barre à son début", () => {
    const view = generationStepStates(at(2, 'failed'), true)
    expect(view.states).toEqual(['done', 'done', 'fail', 'todo', 'todo', 'todo'])
    expect(view.percent).toBeCloseTo((2 / 6) * 100)
  })

  it('borne une étape hors limites', () => {
    const { states } = generationStepStates(at(9), false)
    expect(states[states.length - 1]).toBe('run')
  })
})
