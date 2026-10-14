/**
 * Généralise le pattern d'édition « brouillon local avant sauvegarde » utilisé
 * par les listes éditables du profil (expériences, formations, projets,
 * compétences, langues) : ajouts en attente, suppressions marquées, éditions
 * en cours, le tout fusionné avec les éléments déjà sauvegardés pour l'affichage.
 */

// Importation des modules Vue / framework
import { computed, ref, type ComputedRef, type Ref } from 'vue'

/** Cible d'édition courante de la modale d'ajout/modification. */
export type EditTarget =
  | { kind: 'new' }
  | { kind: 'saved'; id: number }
  | { kind: 'pending'; index: number }

/** Où insérer un nouvel élément en attente dans la liste affichée. */
export type PendingInsertPosition = 'start' | 'end'

export interface UseEditableListOptions<TView extends { id: number }, TDraft> {
  /** Éléments déjà sauvegardés (typiquement `computed(() => profile.value?.experiences ?? [])`). */
  savedItems: Ref<TView[]> | ComputedRef<TView[]>
  /** Brouillon vide utilisé pour l'ajout d'un nouvel élément. */
  defaultDraft: () => TDraft
  /** Convertit un élément sauvegardé en brouillon éditable. */
  draftFromView: (view: TView) => TDraft
  /** Convertit un brouillon en élément affichable (avec l'id réel ou négatif pour un élément en attente). */
  displayFromDraft: (draft: TDraft, id: number, pendingKind: 'new' | 'edited') => TView
  /** Valide qu'un brouillon peut être sauvegardé (ex. champ requis non vide). */
  isDraftValid: (draft: TDraft) => boolean
  /** Position d'insertion des nouveaux éléments en attente (défaut : 'start'). */
  insertPendingAt?: PendingInsertPosition
}

/**
 * Gère l'état d'édition local (ajouts, suppressions, modifications en attente)
 * d'une liste d'entités du profil, avant leur sauvegarde effective en base.
 *
 * @param options - Convertisseurs et validateur propres à l'entité éditée.
 * @returns État et actions de la liste éditable (voir chaque propriété retournée).
 */
export function useEditableList<TView extends { id: number }, TDraft>(
  options: UseEditableListOptions<TView, TDraft>,
) {
  const { savedItems, defaultDraft, draftFromView, displayFromDraft, isDraftValid } = options
  const insertPendingAt = options.insertPendingAt ?? 'start'

  const pending = ref<TDraft[]>([]) as Ref<TDraft[]>
  const deletedIds = ref<number[]>([])
  const edited = ref<Record<number, TDraft>>({}) as Ref<Record<number, TDraft>>

  const modalOpen = ref(false)
  const draft = ref<TDraft>(defaultDraft()) as Ref<TDraft>
  const editTarget = ref<EditTarget>({ kind: 'new' })

  /** Fusionne éléments en attente et éléments sauvegardés (filtrés/édités) pour l'affichage. */
  const allItems = computed<TView[]>(() => {
    const pendingItems = pending.value.map((d, i) => displayFromDraft(d, -(i + 1), 'new'))

    const savedDisplay = savedItems.value
      .filter((item) => !deletedIds.value.includes(item.id))
      .map((item) => {
        const editDraft = edited.value[item.id]
        return editDraft ? displayFromDraft(editDraft, item.id, 'edited') : item
      })

    return [...pendingItems, ...savedDisplay]
  })

  /** `true` si des ajouts, suppressions ou modifications sont en attente de sauvegarde. */
  const hasChanges = computed(
    () => pending.value.length > 0 || deletedIds.value.length > 0 || Object.keys(edited.value).length > 0,
  )

  function openAddModal() {
    editTarget.value = { kind: 'new' }
    draft.value = defaultDraft()
    modalOpen.value = true
  }

  function openEditModal(item: TView) {
    if (item.id < 0) {
      const index = -item.id - 1
      editTarget.value = { kind: 'pending', index }
      draft.value = { ...pending.value[index] }
    } else {
      editTarget.value = { kind: 'saved', id: item.id }
      const existingEdit = edited.value[item.id]
      if (existingEdit) {
        draft.value = { ...existingEdit }
      } else {
        const saved = savedItems.value.find((v) => v.id === item.id)
        draft.value = saved ? draftFromView(saved) : defaultDraft()
      }
    }
    modalOpen.value = true
  }

  function closeModal() {
    modalOpen.value = false
  }

  function saveModal() {
    if (!isDraftValid(draft.value)) return
    const target = editTarget.value
    if (target.kind === 'new') {
      if (insertPendingAt === 'end') {
        pending.value.push({ ...draft.value })
      } else {
        pending.value.unshift({ ...draft.value })
      }
    } else if (target.kind === 'pending') {
      pending.value[target.index] = { ...draft.value }
    } else {
      edited.value = { ...edited.value, [target.id]: { ...draft.value } }
    }
    modalOpen.value = false
  }

  function deleteItem(item: TView) {
    if (item.id < 0) {
      const index = -item.id - 1
      pending.value.splice(index, 1)
    } else {
      deletedIds.value = [...deletedIds.value, item.id]
      const updated = { ...edited.value }
      delete updated[item.id]
      edited.value = updated
    }
  }

  /** Efface tout l'état d'édition local (utilisé quand l'utilisateur annule). */
  function reset() {
    pending.value = []
    deletedIds.value = []
    edited.value = {}
  }

  return {
    pending,
    deletedIds,
    edited,
    modalOpen,
    draft,
    editTarget,
    allItems,
    hasChanges,
    openAddModal,
    openEditModal,
    closeModal,
    saveModal,
    deleteItem,
    reset,
  }
}
