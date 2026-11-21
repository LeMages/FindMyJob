/**
 * Tests de la liste de vérification « Prêt à générer ? ».
 */

// Importation des modules externes
import { describe, expect, it } from 'vitest'

// Importation des modules internes
import { buildGenerationChecklist, type ChecklistInput } from './generation_checklist'

const READY: ChecklistInput = { fieldsValid: true, documentsReady: true, sessionCity: 'Lyon', sessionLocked: false }

/** État `ok` du point « ville ». */
const cityOk = (input: ChecklistInput) =>
  buildGenerationChecklist(input).items.find((i) => i.key === 'city')?.ok

describe('buildGenerationChecklist', () => {
  it('autorise la génération quand tout est prêt', () => {
    const result = buildGenerationChecklist(READY)
    expect(result.canGenerate).toBe(true)
    expect(result.items.map((i) => i.key)).toEqual(['fields', 'documents', 'city'])
  })

  it('bloque sur des champs obligatoires manquants, sans lien vers les paramètres', () => {
    const result = buildGenerationChecklist({ ...READY, fieldsValid: false })
    expect(result.canGenerate).toBe(false)
    expect(result.items[0]).toEqual({ key: 'fields', ok: false, fixInSessionSettings: false })
  })

  it('bloque sur des modèles manquants, avec lien vers les paramètres', () => {
    const result = buildGenerationChecklist({ ...READY, documentsReady: false })
    expect(result.canGenerate).toBe(false)
    expect(result.items[1]).toEqual({ key: 'documents', ok: false, fixInSessionSettings: true })
  })

  it('bloque sur une ville absente, vide ou faite d\'espaces', () => {
    expect(cityOk({ ...READY, sessionCity: null })).toBe(false)
    expect(cityOk({ ...READY, sessionCity: '   ' })).toBe(false)
    expect(buildGenerationChecklist({ ...READY, sessionCity: '' }).canGenerate).toBe(false)
  })

  it('ajoute un point bloquant pour une session en lecture seule', () => {
    const result = buildGenerationChecklist({ ...READY, sessionLocked: true })
    expect(result.canGenerate).toBe(false)
    expect(result.items[result.items.length - 1]).toEqual({ key: 'session', ok: false, fixInSessionSettings: true })
  })
})
