<script setup lang="ts">
/**
 * Section « Langues » du profil : affichage, ajout, modification et
 * suppression des langues. Les changements sont tenus en local (via
 * useEditableList) jusqu'à l'appel de `save()` par le parent.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { useEditableList } from '@/composables/useEditableList'
import type { LangView, CreateLangInput } from '@/types/index'

const props = defineProps<{
  items: LangView[]
  editMode: boolean
}>()

interface LangDraft {
  nom: string
  level: string
}

type LangDisplayItem = LangView & { isPending?: true; pendingKind?: 'new' | 'edited' }

const LANG_LEVELS = [
  'Langue maternelle',
  'Bilingue',
  'Courant (C2)',
  'Avancé (C1)',
  'Intermédiaire avancé (B2)',
  'Intermédiaire (B1)',
  'Élémentaire (A2)',
  'Débutant (A1)',
]

function defaultLangDraft(): LangDraft {
  return { nom: '', level: '' }
}

function draftFromView(l: LangView): LangDraft {
  return { nom: l.nom ?? '', level: l.level ?? '' }
}

function displayFromDraft(d: LangDraft, id: number, pendingKind: 'new' | 'edited'): LangDisplayItem {
  return { id, nom: d.nom || null, level: d.level || null, isPending: true, pendingKind }
}

function draftToInput(d: LangDraft): CreateLangInput {
  return { nom: d.nom.trim(), level: d.level.trim() || null }
}

const savedItems = computed(() => props.items)

const {
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
} = useEditableList<LangDisplayItem, LangDraft>({
  savedItems,
  defaultDraft: defaultLangDraft,
  draftFromView,
  displayFromDraft,
  isDraftValid: (d) => d.nom.trim() !== '',
  insertPendingAt: 'end',
})

/**
 * Sauvegarde les suppressions, modifications et ajouts en attente, puis
 * retourne la liste de langues à jour (à assigner par le parent).
 *
 * @returns Langues à jour après synchronisation avec le backend.
 */
async function save(): Promise<LangView[]> {
  for (const id of deletedIds.value) {
    await invoke('delete_lang', { id })
  }
  const result = props.items.filter((l) => !deletedIds.value.includes(l.id))

  for (const [rawId, d] of Object.entries(edited.value)) {
    const id = Number(rawId)
    const updated = await invoke<LangView>('update_lang', { id, input: draftToInput(d) })
    const idx = result.findIndex((l) => l.id === id)
    if (idx >= 0) result[idx] = updated
  }

  for (const d of pending.value) {
    const created = await invoke<LangView>('add_lang', { input: draftToInput(d) })
    result.push(created)
  }

  reset()
  return result
}

defineExpose({ hasChanges, reset, save })
</script>

<template>
  <div class="fieldset">
    <div :class="editMode ? 'fstitle-row' : ''">
      <div class="fstitle">LANGUES</div>
      <button v-if="editMode" class="btn exp-add-btn" type="button" @click="openAddModal">
        + Ajouter
      </button>
    </div>
    <template v-if="allItems.length">
      <div
        v-for="(lang, i) in allItems"
        :key="lang.id"
        class="setrow lang-row"
        :class="{ 'exp-pending': lang.isPending }"
        :style="i === allItems.length - 1 ? 'border-bottom:none' : ''"
      >
        <span class="setl">{{ lang.nom ?? '—' }}</span>
        <span class="cmeta" style="flex:1;text-align:right">{{ lang.level ?? '' }}</span>
        <span v-if="lang.isPending" class="tag exp-pending-tag" style="flex:none">
          {{ lang.pendingKind === 'edited' ? 'Modifié' : 'Non enregistré' }}
        </span>
        <div v-if="editMode" class="exp-actions" style="margin-left:8px">
          <button class="exp-btn exp-btn-edit" type="button" title="Modifier" @click="openEditModal(lang)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
          </button>
          <button class="exp-btn exp-btn-delete" type="button" title="Supprimer" @click="deleteItem(lang)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="3 6 5 6 21 6"/>
              <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>
              <path d="M10 11v6M14 11v6"/>
              <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"/>
            </svg>
          </button>
        </div>
      </div>
    </template>
    <div v-else class="cempty">Aucune langue renseignée</div>
  </div>

  <!-- Popup ajout/modification de langue -->
  <div v-if="modalOpen" class="popup-backdrop">
    <div class="popup-box" style="width:420px">
      <div class="popup-hd">
        <div class="popup-title">
          {{ editTarget.kind === 'new' ? 'Ajouter une langue' : 'Modifier une langue' }}
        </div>
        <button class="popup-close" type="button" @click="closeModal">×</button>
      </div>

      <div class="field">
        <label class="flabel">Langue *</label>
        <input v-model="draft.nom" class="input" placeholder="Anglais, Espagnol…" />
      </div>

      <div class="field" style="margin-bottom:0">
        <label class="flabel">Niveau</label>
        <select v-model="draft.level" class="input">
          <option value="">-- Sélectionner --</option>
          <option v-for="lvl in LANG_LEVELS" :key="lvl" :value="lvl">{{ lvl }}</option>
        </select>
      </div>

      <div class="popup-foot">
        <button class="btn" type="button" @click="closeModal">Annuler</button>
        <button
          class="btn edit-save"
          type="button"
          :class="{ disabled: !draft.nom.trim() }"
          :disabled="!draft.nom.trim()"
          @click="saveModal"
        >Valider</button>
      </div>
    </div>
  </div>
</template>
