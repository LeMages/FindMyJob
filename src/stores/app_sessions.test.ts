/**
 * Tests du rafraîchissement des sessions dans le store (`stores/app.ts`) :
 * après une mutation, la pastille (statut) et le compteur de la barre
 * latérale suivent le backend sans relancer l'app. Le pont Tauri est
 * remplacé par un faux `invoke` qui joue le rôle du backend.
 */

// Importation des modules externes
import { beforeEach, describe, expect, it, vi } from 'vitest'

// Importation des modules internes
import type { CandidatureDetail, Session } from '@/types'

// ---------------------------------------------------------------------------
// Faux backend : sessions renvoyées par `get_sessions`, modifiées par les
// commandes de mutation comme le ferait la base.
// ---------------------------------------------------------------------------

const backend = vi.hoisted(() => ({
  sessions: [] as Session[],
  calls: [] as string[],
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (command: string, args?: Record<string, unknown>) => {
    backend.calls.push(command)
    const session = backend.sessions[0]
    switch (command) {
      case 'get_sessions':
        // Copie : le store ne doit pas voir les changements avant de recharger.
        return backend.sessions.map(s => ({ ...s }))
      case 'add_timeline_event':
        // Accepter l'offre : la session passe à « termine » côté backend.
        if ((args?.event as { event_type: string }).event_type === 'accepter_offre') session.statut = 'termine'
        return null
      case 'update_session':
        session.statut = (args?.input as { statut: Session['statut'] }).statut
        return { ...session }
      case 'delete_candidature':
        session.candidature_count -= 1
        return null
      case 'postuler_sans_message':
        return null
      case 'get_candidature':
        return detail
      case 'get_contacts':
      case 'get_status_history':
        return []
      default:
        throw new Error(`commande inattendue : ${command}`)
    }
  }),
}))

const detail = { id: 7, session_id: 1, status: 'offre', cv_path: null, lettre_motivation_path: null } as CandidatureDetail

// Importé après le mock : le store capture le faux `invoke`.
const store = await import('@/stores/app')

function session(overrides: Partial<Session> = {}): Session {
  return {
    id: 1,
    nom: 'Stage M2',
    statut: 'active',
    ville: 'Lyon',
    zip_code: null,
    candidature_count: 3,
    created_at: '2026-11-19T08:00:00.000Z',
    ...overrides,
  }
}

beforeEach(async () => {
  backend.sessions = [session()]
  backend.calls = []
  await store.refreshSessions()
  store.state.sessionId = 1
  store.state.detailCandidature = { ...detail }
})

describe('refreshSessions', () => {
  it('reflète le statut et le compteur renvoyés par le backend', () => {
    expect(store.allSessions.value[0]).toMatchObject({ statut: 'active', count: 3 })
    expect(store.currentSessionLocked.value).toBe(false)
  })

  it('accepter une offre met à jour la pastille et verrouille la session', async () => {
    const error = await store.addTimelineEvent({
      event_type: 'accepter_offre',
      content: null,
      interview_data: null,
      interview_id: null,
      interview_issue: null,
    })
    expect(error).toBeNull()
    expect(store.allSessions.value[0].statut).toBe('termine')
    expect(store.currentSessionLocked.value).toBe(true)
    expect(store.isSessionLocked(1)).toBe(true)
  })

  it('changer le statut de la session recharge la liste', async () => {
    expect(await store.updateSession(1, { statut: 'pause' })).toBeNull()
    expect(store.allSessions.value[0].statut).toBe('pause')
    expect(backend.calls).toContain('get_sessions')
    expect(await store.updateSession(1, { statut: 'active' })).toBeNull()
    expect(store.currentSessionLocked.value).toBe(false)
  })

  it('supprimer une candidature met à jour le compteur', async () => {
    expect(await store.deleteCandidature()).toBeNull()
    await vi.waitFor(() => expect(store.allSessions.value[0].count).toBe(2))
  })

  it('postuler recharge les sessions', async () => {
    store.state.detailCandidature = { ...detail, status: 'a_postuler' }
    backend.calls = []
    expect(await store.applyWithoutMessage()).toBeNull()
    await vi.waitFor(() => expect(backend.calls).toContain('get_sessions'))
  })
})
