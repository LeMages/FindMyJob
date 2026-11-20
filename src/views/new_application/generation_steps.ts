/**
 * État de chaque étape de la génération des documents (popup de
 * génération), dérivé de la progression publiée par le backend.
 */

// Importation des modules internes
import type { GenerationProgress } from '@/types'

/** Nombre d'étapes avant la première réponse du backend (préparation + pipeline). */
export const DEFAULT_TOTAL_STEPS = 6

/** État d'une étape : faite, en cours, en échec ou à venir. */
export type GenerationStepState = 'done' | 'run' | 'fail' | 'todo'

/** États des étapes et remplissage de la barre de progression. */
export interface GenerationStepsView {
  states: GenerationStepState[]
  /** Remplissage de la barre, en pourcentage (0 à 100). */
  percent: number
}

/**
 * Calcule l'état de chaque étape : les étapes avant `current_step` sont
 * faites, l'étape courante est en cours (ou en échec), les suivantes à
 * venir. La barre avance d'une demi-étape sur l'étape en cours ; en échec,
 * elle s'arrête au début de l'étape fautive.
 *
 * @param progress - Dernière progression reçue, `null` avant la première.
 * @param failed - La génération (ou son suivi) a-t-elle échoué ?
 * @returns L'état de chaque étape et le remplissage de la barre.
 */
export function generationStepStates(progress: GenerationProgress | null, failed: boolean): GenerationStepsView {
  const total = progress && progress.total_steps > 0 ? progress.total_steps : DEFAULT_TOTAL_STEPS
  const current = Math.min(Math.max(progress?.current_step ?? 0, 0), total - 1)
  const states = Array.from({ length: total }, (_, i): GenerationStepState => {
    if (i < current) return 'done'
    if (i === current) return failed ? 'fail' : 'run'
    return 'todo'
  })
  const percent = ((failed ? current : current + 0.5) / total) * 100
  return { states, percent }
}
