<script setup lang="ts">
/**
 * Section « Formation » du profil : affichage, ajout, modification et
 * suppression des formations. Les changements sont tenus en local (via
 * useEditableList) jusqu'à l'appel de `save()` par le parent.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { useEditableList } from '@/composables/useEditableList'
import { buildYearRange, fmtYearRange } from './dateRangeFormat'
import type { FormationView, CreateFormationInput } from '@/types/index'

const props = defineProps<{
  items: FormationView[]
  editMode: boolean
}>()

interface FormDraft {
  titre: string
  ecole: string
  description: string
  debut_annee: number
  fin_annee: number
  en_cours: boolean
}

type FormDisplayItem = FormationView & { isPending?: true; pendingKind?: 'new' | 'edited' }

const yearRange = buildYearRange()

function toDateStr(y: number): string {
  return `${y}-01-01`
}

function defaultFormDraft(): FormDraft {
  return {
    titre: '',
    ecole: '',
    description: '',
    debut_annee: new Date().getFullYear(),
    fin_annee: new Date().getFullYear(),
    en_cours: false,
  }
}

function draftFromView(f: FormationView): FormDraft {
  const curYear = new Date().getFullYear()
  const parseYear = (d: string | null) => (d ? parseInt(d.split('-')[0] ?? String(curYear), 10) : curYear)
  return {
    titre: f.titre,
    ecole: f.ecole ?? '',
    description: f.description ?? '',
    debut_annee: parseYear(f.date_debut),
    fin_annee: parseYear(f.date_fin),
    en_cours: f.date_fin === null,
  }
}

function displayFromDraft(d: FormDraft, id: number, pendingKind: 'new' | 'edited'): FormDisplayItem {
  return {
    id,
    titre: d.titre,
    ecole: d.ecole || null,
    description: d.description || null,
    date_debut: toDateStr(d.debut_annee),
    date_fin: d.en_cours ? null : toDateStr(d.fin_annee),
    isPending: true,
    pendingKind,
  }
}

function draftToInput(d: FormDraft): CreateFormationInput {
  return {
    titre: d.titre.trim(),
    ecole: d.ecole.trim() || null,
    description: d.description.trim() || null,
    date_debut: toDateStr(d.debut_annee),
    date_fin: d.en_cours ? null : toDateStr(d.fin_annee),
  }
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
} = useEditableList<FormDisplayItem, FormDraft>({
  savedItems,
  defaultDraft: defaultFormDraft,
  draftFromView,
  displayFromDraft,
  isDraftValid: (d) => d.titre.trim() !== '',
})

/**
 * Sauvegarde les suppressions, modifications et ajouts en attente, puis
 * retourne la liste de formations à jour (à assigner par le parent).
 *
 * @returns Formations à jour après synchronisation avec le backend.
 */
async function save(): Promise<FormationView[]> {
  for (const id of deletedIds.value) {
    await invoke('delete_formation', { id })
  }
  let result = props.items.filter((f) => !deletedIds.value.includes(f.id))

  for (const [rawId, d] of Object.entries(edited.value)) {
    const id = Number(rawId)
    const updated = await invoke<FormationView>('update_formation', { id, input: draftToInput(d) })
    const idx = result.findIndex((f) => f.id === id)
    if (idx >= 0) result[idx] = updated
  }

  for (const d of [...pending.value].reverse()) {
    const created = await invoke<FormationView>('add_formation', { input: draftToInput(d) })
    result = [created, ...result]
  }

  reset()
  return result
}

defineExpose({ hasChanges, reset, save })
</script>

<template>
  <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
    <div :class="editMode ? 'flex items-center justify-between mb-[14px]' : ''">
      <div class="text-[11px] font-semibold font-mono tracking-[.06em] text-neutral-500" :class="editMode ? '' : 'mb-[14px]'">FORMATION</div>
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
        v-for="(form, i) in allItems"
        :key="form.id"
        class="flex gap-[14px] py-[14px] border-b border-neutral-100"
        :class="[form.isPending ? 'opacity-[0.72]' : '', i === allItems.length - 1 ? 'border-b-0' : '']"
      >
        <div class="w-[42px] h-[42px] rounded-lg flex-none bg-neutral-100 bg-[repeating-linear-gradient(45deg,var(--color-neutral-200)_0,var(--color-neutral-200)_1px,transparent_1px,transparent_7px)] border border-neutral-200" />
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2 flex-wrap">
            <div class="text-[14px] font-semibold">{{ form.titre }}</div>
            <span v-if="form.isPending" class="inline-block font-mono text-[10.5px] rounded-5 px-[7px] py-0.5 border border-amber-300 text-amber-700 bg-amber-50">
              {{ form.pendingKind === 'edited' ? 'Modifié' : 'Non enregistré' }}
            </span>
          </div>
          <div v-if="form.ecole" class="text-[13px] text-neutral-500 mt-px">{{ form.ecole }}</div>
          <div v-if="form.description" class="text-[13px] text-neutral-700 mt-[5px] leading-[1.55] whitespace-pre-line">{{ form.description }}</div>
          <div class="font-mono text-[11.5px] text-neutral-400 mt-[3px]">{{ fmtYearRange(form.date_debut, form.date_fin) }}</div>
        </div>
        <div v-if="editMode" class="flex flex-row gap-[5px] flex-none self-center ml-[6px]">
          <button
            class="w-[30px] h-[30px] rounded-7 border border-neutral-200 bg-surface-2 flex items-center justify-center cursor-pointer flex-none text-neutral-600 hover:bg-surface-3 hover:text-neutral-950 hover:border-neutral-300"
            type="button"
            title="Modifier"
            @click="openEditModal(form)"
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
            @click="deleteItem(form)"
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
    <div v-else class="text-[12px] text-neutral-400 text-center py-6 font-mono">Aucune formation renseignée</div>
  </div>

  <!-- Popup ajout/modification de formation -->
  <div v-if="modalOpen" class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100]">
    <div class="bg-surface-3 rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[580px] max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6">
      <div class="flex items-center justify-between mb-5">
        <div class="text-[17px] font-semibold">
          {{ editTarget.kind === 'new' ? 'Ajouter une formation' : 'Modifier une formation' }}
        </div>
        <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" type="button" @click="closeModal">×</button>
      </div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Titre / Diplôme *</label>
        <input v-model="draft.titre" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Master Informatique" />
      </div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">École / Établissement</label>
        <input v-model="draft.ecole" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="EFREI Paris" />
      </div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Détails (un par ligne)</label>
        <textarea v-model="draft.description" rows="3" class="w-full border border-neutral-300 rounded-lg px-3 py-2 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 resize-y" placeholder="Spécialité, cours clés, échange universitaire..."></textarea>
      </div>

      <div class="flex gap-[14px]">
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Année de début</label>
          <select
            v-model="draft.debut_annee"
            class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 appearance-none bg-no-repeat bg-[right_12px_center] pr-8 cursor-pointer bg-[url('data:image/svg+xml,%3Csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2712%27%20height=%278%27%20viewBox=%270%200%2012%208%27%3E%3Cpath%20d=%27M1%201l5%205%205-5%27%20stroke=%27%239b9b95%27%20stroke-width=%271.5%27%20fill=%27none%27%20stroke-linecap=%27round%27/%3E%3C/svg%3E')]"
          >
            <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
          </select>
        </div>
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Année de fin</label>
          <div class="flex gap-2 items-center">
            <select
              v-if="!draft.en_cours"
              v-model="draft.fin_annee"
              class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 appearance-none bg-no-repeat bg-[right_12px_center] pr-8 cursor-pointer flex-1 bg-[url('data:image/svg+xml,%3Csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2712%27%20height=%278%27%20viewBox=%270%200%2012%208%27%3E%3Cpath%20d=%27M1%201l5%205%205-5%27%20stroke=%27%239b9b95%27%20stroke-width=%271.5%27%20fill=%27none%27%20stroke-linecap=%27round%27/%3E%3C/svg%3E')]"
            >
              <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
            </select>
            <label class="flex items-center gap-[6px] text-[13px] cursor-pointer whitespace-nowrap select-none" :class="draft.en_cours ? '' : 'flex-none'">
              <input type="checkbox" v-model="draft.en_cours" />
              En cours
            </label>
          </div>
        </div>
      </div>

      <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
          type="button"
          @click="closeModal"
        >Annuler</button>
        <button
          class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border bg-green-500 text-white border-green-500 hover:bg-green-600 hover:border-green-600"
          :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': !draft.titre.trim() }"
          type="button"
          :disabled="!draft.titre.trim()"
          @click="saveModal"
        >Valider</button>
      </div>
    </div>
  </div>
</template>
