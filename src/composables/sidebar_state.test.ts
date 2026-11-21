/**
 * Tests des règles pures de l'état de la barre latérale (`sidebar_state.ts`).
 */

// Importation des modules externes
import { describe, expect, it } from 'vitest'

// Importation des modules internes
import { isCollapsed, isNarrowWidth, toggled, withNarrow, type SidebarState } from './sidebar_state'

const wideExpanded: SidebarState = { preferenceCollapsed: false, narrow: false, narrowOverride: null }

describe('isNarrowWidth', () => {
  it('replie sous 1100 px seulement', () => {
    expect(isNarrowWidth(800)).toBe(true)
    expect(isNarrowWidth(1099)).toBe(true)
    expect(isNarrowWidth(1100)).toBe(false)
    expect(isNarrowWidth(1920)).toBe(false)
  })
})

describe('isCollapsed', () => {
  it('suit la préférence en fenêtre large', () => {
    expect(isCollapsed(wideExpanded)).toBe(false)
    expect(isCollapsed({ ...wideExpanded, preferenceCollapsed: true })).toBe(true)
  })

  it('replie d’office en fenêtre étroite, sauf choix explicite', () => {
    expect(isCollapsed({ ...wideExpanded, narrow: true })).toBe(true)
    expect(isCollapsed({ ...wideExpanded, narrow: true, narrowOverride: false })).toBe(false)
  })
})

describe('toggled', () => {
  it('en fenêtre large, change la préférence et demande sa sauvegarde', () => {
    const { state, persist } = toggled(wideExpanded)
    expect(persist).toBe(true)
    expect(state.preferenceCollapsed).toBe(true)
    expect(isCollapsed(state)).toBe(true)
  })

  it('en fenêtre étroite, permet de déplier sans toucher la préférence', () => {
    const narrow = withNarrow(wideExpanded, true)
    const { state, persist } = toggled(narrow)
    expect(persist).toBe(false)
    expect(isCollapsed(state)).toBe(false)
    expect(state.preferenceCollapsed).toBe(false)
  })
})

describe('withNarrow', () => {
  it('oublie le choix fait en fenêtre étroite quand la fenêtre s’élargit', () => {
    const expandedWhileNarrow = toggled(withNarrow({ ...wideExpanded, preferenceCollapsed: true }, true)).state
    expect(isCollapsed(expandedWhileNarrow)).toBe(false)
    const wide = withNarrow(expandedWhileNarrow, false)
    expect(wide.narrowOverride).toBeNull()
    expect(isCollapsed(wide)).toBe(true)
  })

  it('ne change rien si la largeur reste du même côté du seuil', () => {
    const s = { ...wideExpanded, narrow: true, narrowOverride: false }
    expect(withNarrow(s, true)).toBe(s)
  })
})
