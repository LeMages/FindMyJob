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
  <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
    <div :class="editMode ? 'flex items-center justify-between mb-[14px]' : ''">
      <div class="text-[11px] font-semibold font-mono tracking-[.06em] text-neutral-500" :class="editMode ? '' : 'mb-[14px]'">LANGUES</div>
      <button
        v-if="editMode"
        class="h-[30px] px-3 text-[12px] rounded-lg border border-neutral-300 bg-surface-2 font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        type="button"
        @click="openAddModal"
      >
        + Ajouter
      </button>
    </div>
    <template v-if="allItems.length">
      <div
        v-for="(lang, i) in allItems"
        :key="lang.id"
        class="flex items-center justify-between gap-5 py-[14px] border-b border-neutral-100"
        :class="[lang.isPending ? 'opacity-[0.72]' : '', i === allItems.length - 1 ? 'border-b-0' : '']"
      >
        <span class="text-[14px] font-medium">{{ lang.nom ?? '—' }}</span>
        <span class="font-mono text-[11px] text-neutral-500 flex-1 text-right">{{ lang.level ?? '' }}</span>
        <span v-if="lang.isPending" class="inline-block font-mono text-[10.5px] rounded-5 px-[7px] py-0.5 border border-amber-300 text-amber-700 bg-amber-50 flex-none">
          {{ lang.pendingKind === 'edited' ? 'Modifié' : 'Non enregistré' }}
        </span>
        <div v-if="editMode" class="flex flex-row gap-[5px] flex-none self-center ml-2">
          <button
            class="w-[30px] h-[30px] rounded-7 border border-neutral-200 bg-surface-2 flex items-center justify-center cursor-pointer flex-none text-neutral-600 hover:bg-surface-3 hover:text-neutral-950 hover:border-neutral-300"
            type="button"
            title="Modifier"
            @click="openEditModal(lang)"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
          </button>
          <button
            class="w-[30px] h-[30px] rounded-7 flex items-center justify-center cursor-pointer flex-none text-red-600 border border-red-200 bg-red-50 hover:bg-red-100 hover:border-red-300"
            type="button"
            title="Supprimer"
            @click="deleteItem(lang)"
          >
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
    <div v-else class="text-[12px] text-neutral-400 text-center py-6 font-mono">Aucune langue renseignée</div>
  </div>

  <!-- Popup ajout/modification de langue -->
  <div v-if="modalOpen" class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100]">
    <div class="bg-surface-3 rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[420px] max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6">
      <div class="flex items-center justify-between mb-5">
        <div class="text-[17px] font-semibold">
          {{ editTarget.kind === 'new' ? 'Ajouter une langue' : 'Modifier une langue' }}
        </div>
        <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" type="button" @click="closeModal">×</button>
      </div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Langue *</label>
        <input v-model="draft.nom" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Anglais, Espagnol…" />
      </div>

      <div class="mb-0">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Niveau</label>
        <select
          v-model="draft.level"
          class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 appearance-none bg-no-repeat bg-[right_12px_center] pr-8 cursor-pointer bg-[url('data:image/svg+xml,%3Csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2712%27%20height=%278%27%20viewBox=%270%200%2012%208%27%3E%3Cpath%20d=%27M1%201l5%205%205-5%27%20stroke=%27%239b9b95%27%20stroke-width=%271.5%27%20fill=%27none%27%20stroke-linecap=%27round%27/%3E%3C/svg%3E')]"
        >
          <option value="">-- Sélectionner --</option>
          <option v-for="lvl in LANG_LEVELS" :key="lvl" :value="lvl">{{ lvl }}</option>
        </select>
      </div>

      <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
          type="button"
          @click="closeModal"
        >Annuler</button>
        <button
          class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border bg-green-500 text-white border-green-500 hover:bg-green-600 hover:border-green-600"
          :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': !draft.nom.trim() }"
          type="button"
          :disabled="!draft.nom.trim()"
          @click="saveModal"
        >Valider</button>
      </div>
    </div>
  </div>
</template>
