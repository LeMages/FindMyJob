/**
 * Tests du parseur d'affichage des descriptions : même table de cas que le
 * parseur Rust (`src-tauri/src/docgen/description_list.rs`), pour que l'app
 * et le CV découpent une description exactement de la même façon.
 */

// Importation des modules externes
import { describe, expect, it } from 'vitest'

// Importation des modules internes
import { parseDescriptionList } from '@/composables/parseDescriptionList'
import cases from '../../src-tauri/tests/fixtures/description_list_cases.json'

describe('parseDescriptionList', () => {
  it('a une table de cas commune non vide', () => {
    expect(cases.length).toBeGreaterThan(0)
  })

  it.each(cases)('$name', ({ input, intro, items }) => {
    expect(parseDescriptionList(input)).toEqual({ intro, items })
  })

  it('description absente = vide', () => {
    expect(parseDescriptionList(null)).toEqual({ intro: [], items: [] })
    expect(parseDescriptionList(undefined)).toEqual({ intro: [], items: [] })
  })
})
